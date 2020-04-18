use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use super::karx::Karx;

/// A handle that awaits the result of a Karx.
///
/// Dropping a [`JoinHandle`] will detach the Karx, meaning that there is no longer
/// a handle to the Karx and no way to `join` on it.
///
/// Created when a Karx is [spawned].
///
/// [spawned]: fn.spawn.html
#[derive(Debug)]
pub struct JoinHandle<T>(crate::karx::kernel::JoinHandle<T, Karx>);

impl<T> JoinHandle<T> {
    /// Creates a new `JoinHandle`.
    pub(crate) fn new(inner: crate::karx::kernel::JoinHandle<T, Karx>) -> JoinHandle<T> {
        JoinHandle(inner)
    }

    /// Returns a handle to the underlying karx.
    ///
    /// # Examples
    ///
    /// ```
    /// # async_std::task::block_on(async {
    /// #
    /// use async_std::task;
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

impl<T> Future for JoinHandle<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.0).poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => panic!("cannot await the result of a panicked Karx"),
            Poll::Ready(Some(val)) => Poll::Ready(val),
        }
    }
}
