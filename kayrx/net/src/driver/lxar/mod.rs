#![allow(missing_docsbare_trait_objects, deprecated, unknown_lints)]
#![deny(missing_debug_implementations)]

//! Linux Api for Rust
//!
//! # Features
//!
//! * Non-blocking TCP, UDP
//! * I/O event notification queue backed by epoll
//! * Zero allocations at runtime
//! * Platform specific extensions
//!
//! handle interfacing with each of the event notification systems of the aforementioned platforms. The details of
//! their implementation are further discussed in [`Poll`].
//!
//! # Usage
//!
//! Creating a [`Poll`], which reads events from the OS and
//! put them into [`Events`]. You can handle IO events from the OS with it.
//!
//! For more detail, see [`Poll`].
//!
//! [`Poll`]: struct.Poll.html
//! [`Events`]: struct.Events.html

pub mod event;
pub mod net;

mod poll;
mod sys;
mod token;

pub use self::poll::{Poll, Registration, SetReadiness};
pub use self::sys::UnixReady;
pub use self::token::Token;
