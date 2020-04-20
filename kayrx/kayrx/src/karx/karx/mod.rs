//! Karx async execute engine

#[macro_use]
pub(crate) mod utils;
pub mod task {
    pub use super::kernel::*;
}

mod block_on;
mod builder;
mod current;
mod kernel;
mod join_karx;
mod karx;
mod local;
mod spawn;
mod spawn_blocking;
mod yield_now;

pub(crate) use self::builder::Runnable;
pub(crate) use self::local::LocalsMap;

pub use self::block_on::block_on;
pub use self::builder::Builder;
pub use self::current::current;
pub use self::karx::{Karx, KarxId};
pub use self::join_karx::JoinKarx;
pub use self::local::{AccessError, LocalKey};
pub use self::spawn::spawn;
pub use self::spawn_blocking::spawn_blocking;
pub use self::yield_now::yield_now;

