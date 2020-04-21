//! Karx async execute engine

#[macro_use]
pub(crate) mod utils;
pub mod task {
    pub use super::kernel::*;
}

mod builder;
mod current;
mod exec;
mod kernel;
mod join_karx;
mod karx;
mod local;
mod spawn;
mod block;
mod yield_now;

pub(crate) use self::builder::Runnable;
pub(crate) use self::local::LocalsMap;

pub use self::exec::exec;
pub use self::builder::Builder;
pub use self::current::current;
pub use self::karx::{Karx, KarxId};
pub use self::join_karx::JoinKarx;
pub use self::local::{AccessError, LocalKey};
pub use self::spawn::spawn;
pub use self::block::block;
pub use self::yield_now::yield_now;

