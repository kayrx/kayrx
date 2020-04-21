use std::cell::Cell;
use std::future::Future;
use std::mem::{self, ManuallyDrop};
use std::sync::Arc;
use std::task::{RawWaker, RawWakerVTable};
use std::task::{Context, Poll , Waker};

use crossbeam_utils::sync::Parker;

use super::Karx;

/// Spawns a karx and blocks the current thread on its result.
///
/// Calling this function is similar to [spawning] a thread and immediately [joining] it, except an
/// asynchronous karx will be spawned.
///
/// See also: [`karx::block`].
///
/// [`karx::block`]: fn.block.html
///
/// [spawning]: https://doc.rust-lang.org/std/thread/fn.spawn.html
/// [joining]: https://doc.rust-lang.org/std/thread/struct.JoinHandle.html#method.join
///
/// # Examples
///
/// ```no_run
/// use kayrx::karx;
///
/// fn main() {
///     karx::exec(async {
///         println!("Hello, world!");
///     })
/// }
/// ```
pub fn exec<F, T>(future: F) -> T
where
    F: Future<Output = T>,
{
    // Create a new karx handle.
    let task = Karx::new(None);


    let future = async move {
        // Drop task-locals on exit.
        defer! {
            Karx::get_current(|t| unsafe { t.drop_locals() });
        }

        future.await
    };

    // Run the future as a karx.
    unsafe { Karx::set_current(&task, || run(future)) }
}

/// Blocks the current thread on a future's result.
fn run<F, T>(future: F) -> T
where
    F: Future<Output = T>,
{
    thread_local! {
        // May hold a pre-allocated parker that can be reused for efficiency.
        //
        // Note that each invocation of `block` needs its own parker. In particular, if `block`
        // recursively calls itself, we must make sure that each recursive call uses a distinct
        // parker instance.
        static CACHE: Cell<Option<Arc<Parker>>> = Cell::new(None);
    }

    // Virtual table for wakers based on `Arc<Parker>`.
    static VTABLE: RawWakerVTable = {
        unsafe fn clone_raw(ptr: *const ()) -> RawWaker {
            let arc = ManuallyDrop::new(Arc::from_raw(ptr as *const Parker));
            #[allow(clippy::redundant_clone)]
            mem::forget(arc.clone());
            RawWaker::new(ptr, &VTABLE)
        }

        unsafe fn wake_raw(ptr: *const ()) {
            let arc = Arc::from_raw(ptr as *const Parker);
            arc.unparker().unpark();
        }

        unsafe fn wake_by_ref_raw(ptr: *const ()) {
            let arc = ManuallyDrop::new(Arc::from_raw(ptr as *const Parker));
            arc.unparker().unpark();
        }

        unsafe fn drop_raw(ptr: *const ()) {
            drop(Arc::from_raw(ptr as *const Parker))
        }

        RawWakerVTable::new(clone_raw, wake_raw, wake_by_ref_raw, drop_raw)
    };

    // Pin the future on the stack.
    pin_utils::pin_mut!(future);

    CACHE.with(|cache| {
        // Reuse a cached parker or create a new one for this invocation of `block`.
        let arc_parker: Arc<Parker> = cache.take().unwrap_or_else(|| Arc::new(Parker::new()));
        let ptr = (&*arc_parker as *const Parker) as *const ();

        // Create a waker and karx context.
        let waker = unsafe { ManuallyDrop::new(Waker::from_raw(RawWaker::new(ptr, &VTABLE))) };
        let cx = &mut Context::from_waker(&waker);

        loop {
            if let Poll::Ready(t) = future.as_mut().poll(cx) {
                // Save the parker for the next invocation of `block`.
                cache.set(Some(arc_parker));
                return t;
            }

            arc_parker.park();
        }
    })
}
