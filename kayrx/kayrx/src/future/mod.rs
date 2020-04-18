//! Utility with Futures

use std::future::Future;
use std::error::Error;
use std::fmt;
use std::pin::Pin;
use std::time::Duration;
use std::task::{Context, Poll};
use std::io;

use futures_util::future;
use futures_timer::Delay;
use pin_project_lite::pin_project;

/// Convert a type into a `Future`.
///
/// # Examples
///
/// ```
/// use async_std::future::{Future, IntoFuture};
/// use async_std::io;
/// use async_std::pin::Pin;
///
/// struct Client;
///
/// impl Client {
///     pub async fn send(self) -> io::Result<()> {
///         // Send a request
///         Ok(())
///     }
/// }
///
/// impl IntoFuture for Client {
///     type Output = io::Result<()>;
///
///     type Future = Pin<Box<dyn Future<Output = Self::Output>>>;
///
///     fn into_future(self) -> Self::Future {
///         Box::pin(async {
///             self.send().await
///         })
///     }
/// }
/// ```
pub trait IntoFuture {
    /// The type of value produced on completion.
    type Output;

    /// Which kind of future are we turning this into?
    type Future: Future<Output = Self::Output>;

    /// Create a future from a value
    fn into_future(self) -> Self::Future;
}

impl<T: Future> IntoFuture for T {
    type Output = T::Output;
    type Future = T;

    fn into_future(self) -> Self::Future {
        self
    }
}

/// Sleeps for the specified amount of time.
///
/// This function might sleep for slightly longer than the specified duration but never less.
///
/// This function is an async version of [`std::thread::sleep`].
///
/// [`std::thread::sleep`]: https://doc.rust-lang.org/std/thread/fn.sleep.html
///
/// See also: [`stream::interval`].
///
/// [`stream::interval`]: ../stream/fn.interval.html
///
/// # Examples
///
/// ```
/// # async_std::task::block_on(async {
/// #
/// use std::time::Duration;
///
/// use async_std::task;
///
/// task::sleep(Duration::from_secs(1)).await;
/// #
/// # })
/// ```
pub async fn sleep(dur: Duration) {
    let _: io::Result<()> = crate::future::io_timeout(dur, future::pending()).await;
}


/// Awaits a future or times out after a duration of time.
///
/// If you want to await an I/O future consider using
/// [`io::timeout`](../io/fn.timeout.html) instead.
///
/// # Examples
///
/// ```
/// # fn main() -> std::io::Result<()> { async_std::task::block_on(async {
/// #
/// use std::time::Duration;
///
/// use async_std::future;
///
/// let never = future::pending::<()>();
/// let dur = Duration::from_millis(5);
/// assert!(future::timeout(dur, never).await.is_err());
/// #
/// # Ok(()) }) }
/// ```
pub async fn timeout<F, T>(dur: Duration, f: F) -> Result<T, TimeoutError>
where
    F: Future<Output = T>,
{
    let f = TimeoutFuture {
        future: f,
        delay: Delay::new(dur),
    };
    f.await
}

pin_project! {
    /// A future that times out after a duration of time.
    pub struct TimeoutFuture<F> {
        #[pin]
        future: F,
        #[pin]
        delay: Delay,
    }
}

impl<F> TimeoutFuture<F> {
    #[allow(dead_code)]
    pub(super) fn new(future: F, dur: Duration) -> TimeoutFuture<F> {
        TimeoutFuture { future: future, delay: Delay::new(dur) }
    }
}

impl<F: Future> Future for TimeoutFuture<F> {
    type Output = Result<F::Output, TimeoutError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.future.poll(cx) {
            Poll::Ready(v) => Poll::Ready(Ok(v)),
            Poll::Pending => match this.delay.poll(cx) {
                Poll::Ready(_) => Poll::Ready(Err(TimeoutError { _private: () })),
                Poll::Pending => Poll::Pending,
            },
        }
    }
}

/// An error returned when a future times out.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimeoutError {
    _private: (),
}

impl Error for TimeoutError {}

impl fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "future has timed out".fmt(f)
    }
}



/// Awaits an I/O future or times out after a duration of time.
///
/// If you want to await a non I/O future consider using
/// [`future::timeout`](../future/fn.timeout.html) instead.
///
/// # Examples
///
/// ```no_run
/// # fn main() -> std::io::Result<()> { async_std::task::block_on(async {
/// #
/// use std::time::Duration;
///
/// use async_std::io;
///
/// io::timeout(Duration::from_secs(5), async {
///     let stdin = io::stdin();
///     let mut line = String::new();
///     let n = stdin.read_line(&mut line).await?;
///     Ok(())
/// })
/// .await?;
/// #
/// # Ok(()) }) }
/// ```
pub async fn io_timeout<F, T>(dur: Duration, f: F) -> io::Result<T>
where
    F: Future<Output = io::Result<T>>,
{
    IoTimeout {
        timeout: Delay::new(dur),
        future: f,
    }
    .await
}

pin_project! {
    /// Future returned by the `FutureExt::timeout` method.
    #[derive(Debug)]
    pub struct IoTimeout<F, T>
    where
        F: Future<Output = io::Result<T>>,
    {
        #[pin]
        future: F,
        #[pin]
        timeout: Delay,
    }
}

impl<F, T> Future for IoTimeout<F, T>
where
    F: Future<Output = io::Result<T>>,
{
    type Output = io::Result<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.future.poll(cx) {
            Poll::Pending => {}
            other => return other,
        }

        if this.timeout.poll(cx).is_ready() {
            let err = Err(io::Error::new(io::ErrorKind::TimedOut, "future timed out"));
            Poll::Ready(err)
        } else {
            Poll::Pending
        }
    }
}
