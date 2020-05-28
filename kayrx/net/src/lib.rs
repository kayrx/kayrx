#![warn(
    rust_2018_idioms,
    unreachable_pub,
    // missing_debug_implementations,
    // missing_docs,
)]
#![allow(
    type_alias_bounds,
    clippy::type_complexity,
    clippy::borrow_interior_mutable_const,
    clippy::needless_doctest_main,
    clippy::too_many_arguments,
    clippy::new_without_default
)]

#[macro_use]
extern crate log;

// pub use ntex_rt_macros::{main, test};

pub(crate) mod task;
pub(crate) mod testing;
pub(crate) mod util;

pub mod server;
pub use ntex_rt::net::*;

pub(crate) mod rt {
    pub use ntex_rt::*;
}

pub(crate) mod service {
    pub use ntex_service::*;
}

pub(crate) mod codec {
    pub use ntex_codec::*;
}