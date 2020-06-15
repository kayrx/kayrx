use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use super::Karx;

/// A handle that awaits the result of a Karx.
///
/// Dropping a [`JoinKarx`] will detach the Karx, meaning that there is no longer
/// a handle to the Karx and no way to `join` on it.
///
/// Created when a Karx is [spawned].
///
/// [spawned]: fn.spawn.html
#[derive(Debug)]
pub struct JoinKarx<T>(super::kernel::JoinHandle<T, Karx>);

impl<T> JoinKarx<T> {
    /// Creates a new `JoinKarx`.
    pub(crate) fn new(inner: super::kernel::JoinHandle<T, Karx>) -> JoinKarx<T> {
        JoinKarx(inner)
    }

    /// Returns a handle to the underlying karx.
    ///
    /// # Examples
    ///
    /// ```
    /// # kayrx::karx::exec(async {
    /// #
    /// use kayrx::karx::task;
    ///
    /// let handle = task::spawn(async {
    ///     1 + 2
    /// });
    /// println!("id = {}", handle.task().id());
    /// #
    /// # })
    pub fn task(&self) -> &Karx {
        self.0.tag()
    }
}

impl<T> Future for JoinKarx<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.0).poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => panic!("cannot await the result of a panicked Karx"),
            Poll::Ready(Some(val)) => Poll::Ready(val),
        }
    }
}
