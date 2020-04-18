//! Synchronization primitives.

mod rwlock;
pub(crate) mod spin_lock;
pub(crate) mod waker_set;

pub(crate) use spin_lock::Spinlock;
pub(crate) use waker_set::WakerSet;
pub use rwlock::{RwLock, RwLockReadGuard, RwLockWriteGuard};

    


