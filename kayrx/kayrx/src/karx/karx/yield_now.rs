use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Cooperatively gives up a timeslice to the karx scheduler.
///
/// Calling this function will move the currently executing future to the back
/// of the execution queue, making room for other futures to execute. This is
/// especially useful after running CPU-intensive operations inside a future.
///
/// See also [`karx::block`].
///
/// [`karx::block`]: fn.block.html
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// # kayrx::karx::exec(async {
/// #
/// use kayrx::karx;
///
/// karx::yield_now().await;
/// #
/// # })
/// ```
#[inline]
pub async fn yield_now() {
    YieldNow(false).await
}

struct YieldNow(bool);

impl Future for YieldNow {
    type Output = ();

    // The futures executor is implemented as a FIFO queue, so all this future
    // does is re-schedule the future back to the end of the queue, giving room
    // for other futures to progress.
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.0 {
            self.0 = true;
            cx.waker().wake_by_ref();

            crate::karx::RUNTIME.yield_now();

            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}
