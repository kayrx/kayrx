use crate::timer::driver::future::poll_fn;
use crate::timer::{delay_until, Delay, Duration, Instant};

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

macro_rules! ready {
    ($e:expr $(,)?) => {
        match $e {
            std::task::Poll::Ready(t) => t,
            std::task::Poll::Pending => return std::task::Poll::Pending,
        }
    };
}

/// Creates new `Interval` that yields with interval of `duration`. The first
/// tick completes immediately.
///
/// An interval will tick indefinitely. At any time, the `Interval` value can be
/// dropped. This cancels the interval.
///
/// This function is equivalent to `interval_at(Instant::now(), period)`.
///
/// # Panics
///
/// This function panics if `period` is zero.
///
/// # Examples
///
/// ```rust
/// use kayrx::timer::{self, Duration};
/// use kayrx::karx;
///
/// fn main() {
///          kayrx::karx::exec(async {
///              let mut interval = timer::interval(Duration::from_millis(10));
///         
///              interval.tick().await;
///              interval.tick().await;
///              interval.tick().await;
///         
///              // approximately 20ms have elapsed.
///          });
/// }
/// ```
pub fn interval(period: Duration) -> Interval {
    assert!(period > Duration::new(0, 0), "`period` must be non-zero.");

    interval_at(Instant::now(), period)
}

/// Creates new `Interval` that yields with interval of `period` with the
/// first tick completing at `at`.
///
/// An interval will tick indefinitely. At any time, the `Interval` value can be
/// dropped. This cancels the interval.
///
/// # Panics
///
/// This function panics if `period` is zero.
///
/// # Examples
///
/// ```
/// use kayrx::timer::{interval_at, Duration, Instant};
/// use kayrx::karx;
///
/// fn main() {
///          kayrx::karx::exec(async {
///              let start = Instant::now() + Duration::from_millis(50);
///              let mut interval = interval_at(start, Duration::from_millis(10));
///         
///              interval.tick().await;
///              interval.tick().await;
///              interval.tick().await;
///         
///              // approximately 70ms have elapsed.
///          });
/// }
/// ```
pub fn interval_at(start: Instant, period: Duration) -> Interval {
    assert!(period > Duration::new(0, 0), "`period` must be non-zero.");

    Interval {
        delay: delay_until(start),
        period,
    }
}

/// Stream returned by [`interval`](interval) and [`interval_at`](interval_at).
#[derive(Debug)]
pub struct Interval {
    /// Future that completes the next time the `Interval` yields a value.
    delay: Delay,

    /// The duration between values yielded by `Interval`.
    period: Duration,
}

impl Interval {
    #[doc(hidden)] // TODO: document
    pub fn poll_tick(&mut self, cx: &mut Context<'_>) -> Poll<Instant> {
        // Wait for the delay to be done
        ready!(Pin::new(&mut self.delay).poll(cx));

        // Get the `now` by looking at the `delay` deadline
        let now = self.delay.deadline();

        // The next interval value is `duration` after the one that just
        // yielded.
        let next = now + self.period;
        self.delay.reset(next);

        // Return the current instant
        Poll::Ready(now)
    }

    /// Completes when the next instant in the interval has been reached.
    ///
    /// # Examples
    ///
    /// ```
    /// use kayrx::timer;
    /// use std::time::Duration;
    /// use kayrx::karx;
    ///
    /// fn main() {
    ///          kayrx::karx::exec(async {
    ///              let mut interval = timer::interval(Duration::from_millis(10));
    ///         
    ///              interval.tick().await;
    ///              interval.tick().await;
    ///              interval.tick().await;
    ///         
    ///              // approximately 20ms have elapsed.
    ///          });
    /// }
    /// ```
    #[allow(clippy::should_implement_trait)] 
    pub async fn tick(&mut self) -> Instant {
        poll_fn(|cx| self.poll_tick(cx)).await
    }
}

impl futures_core::stream::Stream for Interval {
    type Item = Instant;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Instant>> {
        Poll::Ready(Some(ready!(self.poll_tick(cx))))
    }
}