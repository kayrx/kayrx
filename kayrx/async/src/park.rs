//! Abstraction over blocking and unblocking the current thread.
//!
//! Provides an abstraction over blocking the current thread. This is similar to
//! the park / unpark constructs provided by [`std`] but made generic. This
//! allows embedding custom functionality to perform when the thread is blocked.
//!
//! A blocked [`Park`][p] instance is unblocked by calling [`unpark`] on its
//! [`Unpark`][up] handle.
//!
//! The [`ParkThread`] struct implements [`Park`][p] using
//! [`thread::park`][`std`] to put the thread to sleep. The kayrx reactor also
//! implements park, but uses [`mio::Poll`][mio] to block the thread instead.
//!
//! The [`Park`][p] trait is composable. A timer implementation might decorate a
//! [`Park`][p] implementation by checking if any timeouts have elapsed after
//! the inner [`Park`][p] implementation unblocks.
//!
//! # Model
//!
//! Conceptually, each [`Park`][p] instance has an associated token, which is
//! initially not present:
//!
//! * The [`park`] method blocks the current thread unless or until the token
//!   is available, at which point it atomically consumes the token.
//! * The [`unpark`] method atomically makes the token available if it wasn't
//!   already.
//!
//! Some things to note:
//!
//! * If [`unpark`] is called before [`park`], the next call to [`park`] will
//!   **not** block the thread.
//! * **Spurious** wakeups are permitted, i.e., the [`park`] method may unblock
//!   even if [`unpark`] was not called.
//! * [`park_timeout`] does the same as [`park`] but allows specifying a maximum
//!   time to block the thread for.

use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Condvar, Mutex};
use std::sync::atomic::Ordering::SeqCst;
use std::time::Duration;
use std::marker::PhantomData;
use std::rc::Rc;
use std::mem;
use std::task::{RawWaker, RawWakerVTable, Waker};


/// Block the current thread.
///
pub(crate)  trait Park {
    /// Unpark handle type for the `Park` implementation.
    type Unpark: Unpark;

    /// Error returned by `park`
    type Error;

    /// Get a new `Unpark` handle associated with this `Park` instance.
    fn unpark(&self) -> Self::Unpark;

    /// Block the current thread unless or until the token is available.
    ///
    /// A call to `park` does not guarantee that the thread will remain blocked
    /// forever, and callers should be prepared for this possibility. This
    /// function may wakeup spuriously for any reason.
    ///
    /// See [module documentation][mod] for more details.
    ///
    /// # Panics
    ///
    /// This function **should** not panic, but ultimately, panics are left as
    /// an implementation detail. Refer to the documentation for the specific
    /// `Park` implementation
    ///
    /// [mod]: ../index.html
    fn park(&mut self) -> Result<(), Self::Error>;

    /// Park the current thread for at most `duration`.
    ///
    /// This function is the same as `park` but allows specifying a maximum time
    /// to block the thread for.
    ///
    /// Same as `park`, there is no guarantee that the thread will remain
    /// blocked for any amount of time. Spurious wakeups are permitted for any
    /// reason.
    ///
    /// See [module documentation][mod] for more details.
    ///
    /// # Panics
    ///
    /// This function **should** not panic, but ultimately, panics are left as
    /// an implementation detail. Refer to the documentation for the specific
    /// `Park` implementation
    ///
    /// [mod]: ../index.html
    fn park_timeout(&mut self, duration: Duration) -> Result<(), Self::Error>;
}

/// Unblock a thread blocked by the associated [`Park`] instance.
///
/// See [module documentation][mod] for more details.
///
/// [mod]: ../index.html
/// [`Park`]: trait.Park.html
pub(crate)  trait Unpark: Sync + Send + 'static {
    /// Unblock a thread that is blocked by the associated `Park` handle.
    ///
    /// Calling `unpark` atomically makes available the unpark token, if it is
    /// not already available.
    ///
    /// See [module documentation][mod] for more details.
    ///
    /// # Panics
    ///
    /// This function **should** not panic, but ultimately, panics are left as
    /// an implementation detail. Refer to the documentation for the specific
    /// `Unpark` implementation
    ///
    /// [mod]: ../index.html
    fn unpark(&self);
}

impl Unpark for Box<dyn Unpark> {
    fn unpark(&self) {
        (**self).unpark()
    }
}

impl Unpark for Arc<dyn Unpark> {
    fn unpark(&self) {
        (**self).unpark()
    }
}

/// Error returned by [`ParkThread`]
///
/// This currently is never returned, but might at some point in the future.
///
/// [`ParkThread`]: struct.ParkThread.html
#[derive(Debug)]
pub(crate)  struct ParkError {
    _p: (),
}

#[derive(Debug)]
pub(crate)  struct ParkThread {
    inner: Arc<Inner>,
}

/// Unblocks a thread that was blocked by `ParkThread`.
#[derive(Clone, Debug)]
pub(crate)  struct UnparkThread {
    inner: Arc<Inner>,
}

#[derive(Debug)]
struct Inner {
    state: AtomicUsize,
    mutex: Mutex<()>,
    condvar: Condvar,
}

const EMPTY: usize = 0;
const PARKED: usize = 1;
const NOTIFIED: usize = 2;

thread_local! {
    static CURRENT_PARKER: ParkThread = ParkThread::new();
}

// ==== impl ParkThread ====

impl ParkThread {
    pub(crate)  fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                state: AtomicUsize::new(EMPTY),
                mutex: Mutex::new(()),
                condvar: Condvar::new(),
            }),
        }
    }
}

impl Park for ParkThread {
    type Unpark = UnparkThread;
    type Error = ParkError;

    fn unpark(&self) -> Self::Unpark {
        let inner = self.inner.clone();
        UnparkThread { inner }
    }

    fn park(&mut self) -> Result<(), Self::Error> {
        self.inner.park();
        Ok(())
    }

    fn park_timeout(&mut self, duration: Duration) -> Result<(), Self::Error> {
        self.inner.park_timeout(duration);
        Ok(())
    }
}

// ==== impl Inner ====

impl Inner {
    /// Park the current thread for at most `dur`.
    fn park(&self) {
        // If we were previously notified then we consume this notification and
        // return quickly.
        if self
            .state
            .compare_exchange(NOTIFIED, EMPTY, SeqCst, SeqCst)
            .is_ok()
        {
            return;
        }

        // Otherwise we need to coordinate going to sleep
        let mut m = self.mutex.lock().unwrap();

        match self.state.compare_exchange(EMPTY, PARKED, SeqCst, SeqCst) {
            Ok(_) => {}
            Err(NOTIFIED) => {
                // We must read here, even though we know it will be `NOTIFIED`.
                // This is because `unpark` may have been called again since we read
                // `NOTIFIED` in the `compare_exchange` above. We must perform an
                // acquire operation that synchronizes with that `unpark` to observe
                // any writes it made before the call to unpark. To do that we must
                // read from the write it made to `state`.
                let old = self.state.swap(EMPTY, SeqCst);
                debug_assert_eq!(old, NOTIFIED, "park state changed unexpectedly");

                return;
            }
            Err(actual) => panic!("inconsistent park state; actual = {}", actual),
        }

        loop {
            m = self.condvar.wait(m).unwrap();

            if self
                .state
                .compare_exchange(NOTIFIED, EMPTY, SeqCst, SeqCst)
                .is_ok()
            {
                // got a notification
                return;
            }

            // spurious wakeup, go back to sleep
        }
    }

    fn park_timeout(&self, dur: Duration) {
        // Like `park` above we have a fast path for an already-notified thread,
        // and afterwards we start coordinating for a sleep. Return quickly.
        if self
            .state
            .compare_exchange(NOTIFIED, EMPTY, SeqCst, SeqCst)
            .is_ok()
        {
            return;
        }

        let m = self.mutex.lock().unwrap();

        match self.state.compare_exchange(EMPTY, PARKED, SeqCst, SeqCst) {
            Ok(_) => {}
            Err(NOTIFIED) => {
                // We must read again here, see `park`.
                let old = self.state.swap(EMPTY, SeqCst);
                debug_assert_eq!(old, NOTIFIED, "park state changed unexpectedly");

                return;
            }
            Err(actual) => panic!("inconsistent park_timeout state; actual = {}", actual),
        }

        // Wait with a timeout, and if we spuriously wake up or otherwise wake up
        // from a notification, we just want to unconditionally set the state back to
        // empty, either consuming a notification or un-flagging ourselves as
        // parked.
        let (_m, _result) = self.condvar.wait_timeout(m, dur).unwrap();

        match self.state.swap(EMPTY, SeqCst) {
            NOTIFIED => {} // got a notification, hurray!
            PARKED => {}   // no notification, alas
            n => panic!("inconsistent park_timeout state: {}", n),
        }
    }

    fn unpark(&self) {
        // To ensure the unparked thread will observe any writes we made before
        // this call, we must perform a release operation that `park` can
        // synchronize with. To do that we must write `NOTIFIED` even if `state`
        // is already `NOTIFIED`. That is why this must be a swap rather than a
        // compare-and-swap that returns if it reads `NOTIFIED` on failure.
        match self.state.swap(NOTIFIED, SeqCst) {
            EMPTY => return,    // no one was waiting
            NOTIFIED => return, // already unparked
            PARKED => {}        // gotta go wake someone up
            _ => panic!("inconsistent state in unpark"),
        }

        // There is a period between when the parked thread sets `state` to
        // `PARKED` (or last checked `state` in the case of a spurious wake
        // up) and when it actually waits on `cvar`. If we were to notify
        // during this period it would be ignored and then when the parked
        // thread went to sleep it would never wake up. Fortunately, it has
        // `lock` locked at this stage so we can acquire `lock` to wait until
        // it is ready to receive the notification.
        //
        // Releasing `lock` before the call to `notify_one` means that when the
        // parked thread wakes it doesn't get woken only to have to wait for us
        // to release `lock`.
        drop(self.mutex.lock().unwrap());

        self.condvar.notify_one()
    }
}

impl Default for ParkThread {
    fn default() -> Self {
        Self::new()
    }
}

// ===== impl UnparkThread =====

impl Unpark for UnparkThread {
    fn unpark(&self) {
        self.inner.unpark();
    }
}



// ============blocking_impl==================

/// Blocks the current thread using a condition variable.
#[derive(Debug)]
pub(crate)  struct CachedParkThread {
        _anchor: PhantomData<Rc<()>>,
}


impl CachedParkThread {
        /// Create a new `ParkThread` handle for the current thread.
        ///
        /// This type cannot be moved to other threads, so it should be created on
        /// the thread that the caller intends to park.
        pub(crate)  fn new() -> CachedParkThread {
            CachedParkThread {
                _anchor: PhantomData,
            }
        }

        pub(crate) fn get_unpark(&self) -> Result<UnparkThread, ParkError> {
            self.with_current(|park_thread| park_thread.unpark())
        }

        /// Get a reference to the `ParkThread` handle for this thread.
        fn with_current<F, R>(&self, f: F) ->  Result<R, ParkError>
        where
            F: FnOnce(&ParkThread) -> R,
        {
            // CURRENT_PARKER.with(|inner| f(inner))
            CURRENT_PARKER.try_with(|inner| f(inner))
                .map_err(|_| ParkError { _p: () })
        }
}

impl Park for CachedParkThread {
        type Unpark = UnparkThread;
        type Error = ParkError;

        fn unpark(&self) -> Self::Unpark {
            self.get_unpark().unwrap()
        }

        fn park(&mut self) -> Result<(), Self::Error> {
            self.with_current(|park_thread| park_thread.inner.park());
            Ok(())
        }

        fn park_timeout(&mut self, duration: Duration) -> Result<(), Self::Error> {
            self.with_current(|park_thread| park_thread.inner.park_timeout(duration));
            Ok(())
        }
}

impl UnparkThread {
    pub(crate)  fn into_waker(self) -> Waker {
            unsafe {
                let raw = unparker_to_raw_waker(self.inner);
                Waker::from_raw(raw)
            }
    }
}

impl Inner {

    #[allow(clippy::wrong_self_convention)]
    fn into_raw(this: Arc<Inner>) -> *const () {
        Arc::into_raw(this) as *const ()
    }

    unsafe fn from_raw(ptr: *const ()) -> Arc<Inner> {
        Arc::from_raw(ptr as *const Inner)
    }
}


unsafe fn unparker_to_raw_waker(unparker: Arc<Inner>) -> RawWaker {
        RawWaker::new(
            Inner::into_raw(unparker),
            &RawWakerVTable::new(clone, wake, wake_by_ref, drop_waker),
        )
}

unsafe fn clone(raw: *const ()) -> RawWaker {
        let unparker = Inner::from_raw(raw);

        // Increment the ref count
        mem::forget(unparker.clone());

        unparker_to_raw_waker(unparker)
}

unsafe fn drop_waker(raw: *const ()) {
        let _ = Inner::from_raw(raw);
}

unsafe fn wake(raw: *const ()) {
        let unparker = Inner::from_raw(raw);
        unparker.unpark();
}

unsafe fn wake_by_ref(raw: *const ()) {
        let unparker = Inner::from_raw(raw);
        unparker.unpark();

        // We don't actually own a reference to the unparker
        mem::forget(unparker);
}