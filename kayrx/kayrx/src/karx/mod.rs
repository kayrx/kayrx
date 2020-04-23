//! Karx Async Runtime

pub mod futures;
mod karx;
mod reactor;
mod runtime;
pub(in crate::karx) mod utils;

pub use self::karx::*;
pub use self::reactor::{Reactor, Watcher};
pub use self::runtime::Runtime;

use once_cell::sync::Lazy;
use std::thread;

use self::karx::utils::abort_on_panic;

/// The global runtime.
pub static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    thread::Builder::new()
        .name("karx/runtime".to_string())
        .spawn(|| abort_on_panic(|| RUNTIME.run()))
        .expect("cannot start a runtime thread");

    Runtime::new()
});
