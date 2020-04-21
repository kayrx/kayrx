//! Futures Async Execute Engine

use futures_core::future::{BoxFuture, Future, LocalBoxFuture};
use futures_executor::{LocalPool, LocalSpawner};
use futures_util::task::{LocalSpawn as _, Spawn as _};

/// The Runtime for driving the  application.
pub trait Runtime {
    /// The value for spawning  cases.
    type Spawner: Spawner;

    /// Create the instance of `Spawner`.
    fn spawner(&self) -> Self::Spawner;

    /// Run a future and wait for its result.
    fn exec<Fut>(&mut self, fut: Fut) -> Fut::Output
    where
        Fut: Future;
}

impl<T: ?Sized> Runtime for &mut T
where
    T: Runtime,
{
    type Spawner = T::Spawner;

    #[inline]
    fn spawner(&self) -> Self::Spawner {
        (**self).spawner()
    }

    #[inline]
    fn exec<Fut>(&mut self, fut: Fut) -> Fut::Output
    where
        Fut: Future,
    {
        (**self).exec(fut)
    }
}

impl<T: ?Sized> Runtime for Box<T>
where
    T: Runtime,
{
    type Spawner = T::Spawner;

    #[inline]
    fn spawner(&self) -> Self::Spawner {
        (**self).spawner()
    }

    #[inline]
    fn exec<Fut>(&mut self, fut: Fut) -> Fut::Output
    where
        Fut: Future,
    {
        (**self).exec(fut)
    }
}

/// The value for spawning  cases.
pub trait Spawner {
    /// Spawn a task to execute a  case.
    fn spawn(&mut self, fut: BoxFuture<'static, ()>) -> anyhow::Result<()>;

    /// Spawn a task to execute a  case onto the current thread.
    fn spawn_local(&mut self, fut: LocalBoxFuture<'static, ()>) -> anyhow::Result<()>;

    /// Spawn a task to execute a  case which may block the running thread.
    fn block(&mut self, f: Box<dyn FnOnce() + Send + 'static>) -> anyhow::Result<()>;
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
    fn block(&mut self, f: Box<dyn FnOnce() + Send + 'static>) -> anyhow::Result<()> {
        (**self).block(f)
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
    fn block(&mut self, f: Box<dyn FnOnce() + Send + 'static>) -> anyhow::Result<()> {
        (**self).block(f)
    }
}

/// Create an instance of `Runtime` used by the default  harness.
pub fn default() -> impl Runtime {
    DefaultRuntime {
        pool: LocalPool::new(),
    }
}

struct DefaultRuntime {
    pool: LocalPool,
}

struct DefaultSpawner {
    spawner: LocalSpawner,
}

impl Runtime for DefaultRuntime {
    type Spawner = DefaultSpawner;

    #[inline]
    fn spawner(&self) -> Self::Spawner {
        DefaultSpawner {
            spawner: self.pool.spawner(),
        }
    }

    #[inline]
    fn exec<Fut>(&mut self, fut: Fut) -> Fut::Output
    where
        Fut: Future,
    {
        self.pool.run_until(fut)
    }
}

impl Spawner for DefaultSpawner {
    fn spawn(&mut self, fut: BoxFuture<'static, ()>) -> anyhow::Result<()> {
        self.spawner.spawn_obj(fut.into()).map_err(Into::into)
    }

    fn spawn_local(&mut self, fut: LocalBoxFuture<'static, ()>) -> anyhow::Result<()> {
        self.spawner.spawn_local_obj(fut.into()).map_err(Into::into)
    }

    fn block(&mut self, f: Box<dyn FnOnce() + Send + 'static>) -> anyhow::Result<()> {
        self.spawn_local(Box::pin(async move { f() }))
    }
}