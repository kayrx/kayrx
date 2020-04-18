use std::future::Future;

use super::Builder;
use crate::karx::JoinKarx;

/// Spawns a Karx.
///
/// This function is similar to [`std::thread::spawn`], except it spawns an asynchronous Karx.
///
/// [`std::thread`]: https://doc.rust-lang.org/std/thread/fn.spawn.html
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
///
/// assert_eq!(handle.await, 3);
/// #
/// # })
/// ```
pub fn spawn<F, T>(future: F) -> JoinKarx<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    Builder::new().spawn(future).expect("cannot spawn Karx")
}
