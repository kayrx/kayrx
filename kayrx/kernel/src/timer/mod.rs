#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]
#![deny(intra_doc_link_resolution_failure)]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]

//! Utilities for tracking time.
//!
//! This crate provides a number of utilities for working with periods of time:
//!
//! * [`Delay`]: A future that completes at a specified instant in time.
//!
//! * [`Interval`] A stream that yields at fixed time intervals.
//!
//! * [`Throttle`]: Throttle down a stream by enforcing a fixed delay between items.
//!
//! * [`Timeout`]: Wraps a future or stream, setting an upper bound to the
//!   amount of time it is allowed to execute. If the future or stream does not
//!   complete in time, then it is canceled and an error is returned.
//!
//! * [`DelayQueue`]: A queue where items are returned once the requested delay
//!   has expired.
//!
//! These three types are backed by a [`Timer`] instance. In order for
//! [`Delay`], [`Interval`], and [`Timeout`] to function, the associated
//! [`Timer`] instance must be running on some thread.
//!
//! [`Delay`]: struct.Delay.html
//! [`DelayQueue`]: struct.DelayQueue.html
//! [`Throttle`]: throttle::Throttle.html
//! [`Timeout`]: struct.Timeout.html
//! [`Interval`]: struct.Interval.html
//! [`Timer`]: timer::Timer.html

pub mod delay_queue;
pub(crate) mod utils;

mod atomic;
mod clock;
mod delay;
mod driver;
mod error;
mod interval;
mod throttle;
mod timeout;
mod wheel;

pub use self::clock::{now, with_default, Clock, Now};
pub use self::driver::*;
pub use self::throttle::Throttle;
pub use delay::Delay;
#[doc(inline)]
pub use delay_queue::DelayQueue;
pub use driver::{set_default, Timer};
pub use error::Error;
pub use interval::Interval;
#[doc(inline)]
pub use timeout::{Elapsed, Timeout};

use std::time::{Duration, Instant};

/// Create a Future that completes at `deadline`.
pub fn delay(deadline: Instant) -> Delay {
    Delay::new(deadline)
}

/// Create a Future that completes in `duration` from now.
///
/// Equivalent to `delay(kayrx::timer::clock::now() + duration)`. Analogous to `std::thread::sleep`.
pub fn delay_for(duration: Duration) -> Delay {
    delay(clock::now() + duration)
}

// ===== Internal utils =====

enum Round {
    Up,
    Down,
}

/// Convert a `Duration` to milliseconds, rounding up and saturating at
/// `u64::MAX`.
///
/// The saturating is fine because `u64::MAX` milliseconds are still many
/// million years.
#[inline]
fn ms(duration: Duration, round: Round) -> u64 {
    const NANOS_PER_MILLI: u32 = 1_000_000;
    const MILLIS_PER_SEC: u64 = 1_000;

    // Round up.
    let millis = match round {
        Round::Up => (duration.subsec_nanos() + NANOS_PER_MILLI - 1) / NANOS_PER_MILLI,
        Round::Down => duration.subsec_millis(),
    };

    duration
        .as_secs()
        .saturating_mul(MILLIS_PER_SEC)
        .saturating_add(u64::from(millis))
}
