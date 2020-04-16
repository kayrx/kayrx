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

extern crate alloc;

mod kalloc;
mod atomic_waker;
mod cell;
mod future;
mod karx;
mod park;
mod timer;

pub(crate) use park::{Park, Unpark, ParkThread, UnparkThread, CachedParkThread, ParkError};
pub(crate) use self::atomic_waker::AtomicWaker;
use futures_core::future::{BoxFuture, Future, LocalBoxFuture};
use futures_executor::{LocalPool, LocalSpawner};
use futures_util::task::{LocalSpawn as _, Spawn as _};

/// The Engine for driving the test application.
pub trait Engine {
    /// The value for spawning test cases.
    type Spawner: Spawner;

    /// Create the instance of `Spawner`.
    fn spawner(&self) -> Self::Spawner;

    /// Run a future and wait for its result.
    fn block_on<Fut>(&mut self, fut: Fut) -> Fut::Output
    where
        Fut: Future;
}

impl<T: ?Sized> Engine for &mut T
where
    T: Engine,
{
    type Spawner = T::Spawner;

    #[inline]
    fn spawner(&self) -> Self::Spawner {
        (**self).spawner()
    }

    #[inline]
    fn block_on<Fut>(&mut self, fut: Fut) -> Fut::Output
    where
        Fut: Future,
    {
        (**self).block_on(fut)
    }
}

impl<T: ?Sized> Engine for Box<T>
where
    T: Engine,
{
    type Spawner = T::Spawner;

    #[inline]
    fn spawner(&self) -> Self::Spawner {
        (**self).spawner()
    }

    #[inline]
    fn block_on<Fut>(&mut self, fut: Fut) -> Fut::Output
    where
        Fut: Future,
    {
        (**self).block_on(fut)
    }
}

/// The value for spawning test cases.
pub trait Spawner {
    /// Spawn a task to execute a test case.
    fn spawn(&mut self, fut: BoxFuture<'static, ()>) -> anyhow::Result<()>;

    /// Spawn a task to execute a test case onto the current thread.
    fn spawn_local(&mut self, fut: LocalBoxFuture<'static, ()>) -> anyhow::Result<()>;

    /// Spawn a task to execute a test case which may block the running thread.
    fn spawn_blocking(&mut self, f: Box<dyn FnOnce() + Send + 'static>) -> anyhow::Result<()>;
}

impl<T: ?Sized> Spawner for &mut T
where
    T: Spawner,
{
    #[inline]
    fn spawn(&mut self, fut: BoxFuture<'static, ()>) -> anyhow::Result<()> {
        (**self).spawn(fut)
    }

    #[inline]
    fn spawn_local(&mut self, fut: LocalBoxFuture<'static, ()>) -> anyhow::Result<()> {
        (**self).spawn_local(fut)
    }

    #[inline]
    fn spawn_blocking(&mut self, f: Box<dyn FnOnce() + Send + 'static>) -> anyhow::Result<()> {
        (**self).spawn_blocking(f)
    }
}

impl<T: ?Sized> Spawner for Box<T>
where
    T: Spawner,
{
    #[inline]
    fn spawn(&mut self, fut: BoxFuture<'static, ()>) -> anyhow::Result<()> {
        (**self).spawn(fut)
    }

    #[inline]
    fn spawn_local(&mut self, fut: LocalBoxFuture<'static, ()>) -> anyhow::Result<()> {
        (**self).spawn_local(fut)
    }

    #[inline]
    fn spawn_blocking(&mut self, f: Box<dyn FnOnce() + Send + 'static>) -> anyhow::Result<()> {
        (**self).spawn_blocking(f)
    }
}

/// Create an instance of `Engine` used by the default test harness.
pub fn default_engine() -> impl Engine {
    DefaultEngine {
        pool: LocalPool::new(),
    }
}

struct DefaultEngine {
    pool: LocalPool,
}

impl Engine for DefaultEngine {
    type Spawner = DefaultSpawner;

    #[inline]
    fn spawner(&self) -> Self::Spawner {
        DefaultSpawner {
            spawner: self.pool.spawner(),
        }
    }

    #[inline]
    fn block_on<Fut>(&mut self, fut: Fut) -> Fut::Output
    where
        Fut: Future,
    {
        self.pool.run_until(fut)
    }
}

struct DefaultSpawner {
    spawner: LocalSpawner,
}

impl Spawner for DefaultSpawner {
    fn spawn(&mut self, fut: BoxFuture<'static, ()>) -> anyhow::Result<()> {
        self.spawner.spawn_obj(fut.into()).map_err(Into::into)
    }

    fn spawn_local(&mut self, fut: LocalBoxFuture<'static, ()>) -> anyhow::Result<()> {
        self.spawner.spawn_local_obj(fut.into()).map_err(Into::into)
    }

    fn spawn_blocking(&mut self, f: Box<dyn FnOnce() + Send + 'static>) -> anyhow::Result<()> {
        self.spawn_local(Box::pin(async move { f() }))
    }
}