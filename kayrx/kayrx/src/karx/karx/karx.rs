use std::cell::Cell;
use std::fmt;
use std::mem::ManuallyDrop;
use std::ptr;
use std::sync::atomic::{AtomicPtr, AtomicU64, Ordering};
use std::sync::Arc;

use super::LocalsMap;
use super::utils::abort_on_panic;

thread_local! {
    /// A pointer to the currently running Karx.
    static CURRENT: Cell<*const Karx> = Cell::new(ptr::null_mut());
}

/// The inner representation of a Karx handle.
struct Inner {
    /// The Karx ID.
    id: KarxId,

    /// The optional Karx name.
    name: Option<Box<str>>,

    /// The map holding Karx-local values.
    locals: LocalsMap,
}

impl Inner {
    #[inline]
    fn new(name: Option<String>) -> Inner {
        Inner {
            id: KarxId::generate(),
            name: name.map(String::into_boxed_str),
            locals: LocalsMap::new(),
        }
    }
}

/// A handle to a Karx.
pub struct Karx {
    /// The inner representation.
    ///
    /// This pointer is lazily initialized on first use. In most cases, the inner representation is
    /// never touched and therefore we don't allocate it unless it's really needed.
    inner: AtomicPtr<Inner>,
}

unsafe impl Send for Karx {}
unsafe impl Sync for Karx {}

impl Karx {
    /// Creates a new Karx handle.
    ///
    /// If the Karx is unnamed, the inner representation of the Karx will be lazily allocated on
    /// demand.
    #[inline]
    pub(crate) fn new(name: Option<String>) -> Karx {
        let inner = match name {
            None => AtomicPtr::default(),
            Some(name) => {
                let raw = Arc::into_raw(Arc::new(Inner::new(Some(name))));
                AtomicPtr::new(raw as *mut Inner)
            }
        };
        Karx { inner }
    }

    /// Gets the Karx's unique identifier.
    #[inline]
    pub fn id(&self) -> KarxId {
        self.inner().id
    }

    /// Returns the name of this Karx.
    ///
    /// The name is configured by [`Builder::name`] before spawning.
    ///
    /// [`Builder::name`]: struct.Builder.html#method.name
    pub fn name(&self) -> Option<&str> {
        self.inner().name.as_ref().map(|s| &**s)
    }

    /// Returns the map holding Karx-local values.
    pub(crate) fn locals(&self) -> &LocalsMap {
        &self.inner().locals
    }

    /// Drops all Karx-local values.
    ///
    /// This method is only safe to call at the end of the Karx.
    #[inline]
    pub(crate) unsafe fn drop_locals(&self) {
        let raw = self.inner.load(Ordering::Acquire);
        if let Some(inner) = raw.as_mut() {
            // Abort the process if dropping Karx-locals panics.
            abort_on_panic(|| {
                inner.locals.clear();
            });
        }
    }

    /// Returns the inner representation, initializing it on first use.
    fn inner(&self) -> &Inner {
        loop {
            let raw = self.inner.load(Ordering::Acquire);
            if !raw.is_null() {
                return unsafe { &*raw };
            }

            let new = Arc::into_raw(Arc::new(Inner::new(None))) as *mut Inner;
            if self.inner.compare_and_swap(raw, new, Ordering::AcqRel) != raw {
                unsafe {
                    drop(Arc::from_raw(new));
                }
            }
        }
    }

    /// Set a reference to the current Karx.
    pub(crate) unsafe fn set_current<F, R>(karx: *const Karx, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        CURRENT.with(|current| {
            let old_karx = current.replace(karx);
            defer! {
                current.set(old_karx);
            }
            f()
        })
    }

    /// Gets a reference to the current Karx.
    pub(crate) fn get_current<F, R>(f: F) -> Option<R>
    where
        F: FnOnce(&Karx) -> R,
    {
        let res = CURRENT.try_with(|current| unsafe { current.get().as_ref().map(f) });
        match res {
            Ok(Some(val)) => Some(val),
            Ok(None) | Err(_) => None,
        }
    }
}

impl Drop for Karx {
    fn drop(&mut self) {
        // Deallocate the inner representation if it was initialized.
        let raw = *self.inner.get_mut();
        if !raw.is_null() {
            unsafe {
                drop(Arc::from_raw(raw));
            }
        }
    }
}

impl Clone for Karx {
    fn clone(&self) -> Karx {
        // We need to make sure the inner representation is initialized now so that this instance
        // and the clone have raw pointers that point to the same `Arc<Inner>`.
        let arc = unsafe { ManuallyDrop::new(Arc::from_raw(self.inner())) };
        let raw = Arc::into_raw(Arc::clone(&arc));
        Karx {
            inner: AtomicPtr::new(raw as *mut Inner),
        }
    }
}

impl fmt::Debug for Karx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Karx")
            .field("id", &self.id())
            .field("name", &self.name())
            .finish()
    }
}


/// A unique identifier for a karx.
///
/// # Examples
///
/// ```
/// use kayrx::karx;
///
/// karx::block_on(async {
///     println!("id = {:?}", karx::current().id());
/// })
/// ```
#[derive(Eq, PartialEq, Clone, Copy, Hash, Debug)]
pub struct KarxId(pub(crate) u64);

impl KarxId {
    /// Generates a new `KarxId`.
    pub(crate) fn generate() -> KarxId {
        static COUNTER: AtomicU64 = AtomicU64::new(1);

        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        if id > u64::max_value() / 2 {
            std::process::abort();
        }
        KarxId(id)
    }
}

impl fmt::Display for KarxId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

