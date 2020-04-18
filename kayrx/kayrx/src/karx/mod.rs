//! Karx async execute engine

#[macro_use]
mod utils;

mod executor;
mod join_karx;
mod karx;
mod kernel;
mod reactor;
mod runtime;
mod local;
mod yield_now;

pub mod futures;
pub(crate) use self::local::LocalsMap;

pub use self::executor::*;
pub use self::kernel::*;
pub use self::reactor::{Reactor, Watcher};
pub use self::runtime::Runtime;
pub use self::karx::{Karx, KarxId};
pub use self::join_karx::JoinHandle as JoinKarx;
pub use self::local::{AccessError, LocalKey};
pub use self::yield_now::yield_now;

use std::thread;
use once_cell::sync::Lazy;

use self::utils::abort_on_panic;

/// The global runtime.
pub static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    thread::Builder::new()
        .name("async-std/runtime".to_string())
        .spawn(|| abort_on_panic(|| RUNTIME.run()))
        .expect("cannot start a runtime thread");

    Runtime::new()
});
