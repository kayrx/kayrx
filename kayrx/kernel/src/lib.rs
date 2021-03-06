#![warn(
    rust_2018_idioms,
    unreachable_pub,
    // missing_debug_implementations,
    // missing_docs,
)]
#![allow(
    warnings,
    missing_docs,
    type_alias_bounds,
    clippy::type_complexity,
    clippy::borrow_interior_mutable_const,
    clippy::needless_doctest_main,
    clippy::too_many_arguments,
    clippy::new_without_default
)]

//! The Kayrx Framework.

extern crate alloc;

#[cfg(feature = "macro")]
#[doc(inline)]
pub use kayrx_macro::{futures, main, test};

pub mod karx;
pub mod timer;
