use core::pin::Pin;

use crate::{BlockingSpawner, Executor, HasBlockingSpawner, HasSpawner, Spawner, Task};

/// A handle to a `smol` task.
///
/// Implements [`Task`] by detaching the underlying [`smol::Task`].
#[derive(Debug)]
pub struct SmolTask(pub smol::Task<()>);

impl Task for SmolTask {
    fn detach(self) {
        self.0.detach();
    }
}

/// Executor adapter that spawns tasks on the `smol` runtime.
///
/// Tasks are spawned with [`smol::spawn`] and run on the global
/// `smol` executor. Futures must be `Send` and `'static`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SmolExecutor;

impl<T> Executor<T> for SmolExecutor
where
    T: core::future::Future + Send + 'static,
    T::Output: Send + 'static,
{
    type Task = SmolTask;

    fn spawn(&self, work: T) -> Self::Task {
        SmolTask(smol::spawn(async move {
            let _ = work.await;
        }))
    }
}

#[cfg(feature = "hyper")]
impl<T> hyper::rt::Executor<T> for SmolExecutor
where
    T: core::future::Future + Send + 'static,
    T::Output: Send + 'static,
{
    fn execute(&self, work: T) {
        smol::spawn(work).detach();
    }
}

impl Spawner<'static> for SmolExecutor {
    type Task = SmolTask;

    fn spawn<T>(&self, work: T) -> Self::Task
    where
        T: core::future::Future<Output = ()> + Send + 'static,
    {
        SmolTask(smol::spawn(work))
    }
}

impl BlockingSpawner for SmolExecutor {
    type Error = core::convert::Infallible;
    type Future<R> = SmolBlockingFuture<R>;
    fn spawn_blocking<T, R>(&self, work: T) -> Self::Future<R>
    where
        R: Send + 'static,
        T: FnOnce() -> R + Send + 'static,
    {
        SmolBlockingFuture(smol::unblock(work))
    }
}

/// Future returned by [`SmolExecutor::spawn_blocking`].
///
/// Wraps a [`smol::Task`] so that it resolves to `Result<R, Infallible>`,
/// matching the [`BlockingSpawner`] contract.
#[derive(Debug)]
pub struct SmolBlockingFuture<R>(smol::Task<R>);

impl<R> core::future::Future for SmolBlockingFuture<R> {
    type Output = Result<R, core::convert::Infallible>;
    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        Pin::new(&mut self.0).poll(cx).map(Ok)
    }
}

impl HasBlockingSpawner for SmolExecutor {
    type Spawner = Self;
    fn blocking_spawner(&self) -> &Self::Spawner {
        self
    }
}

impl HasSpawner<'static> for SmolExecutor {
    type Spawner = Self;
    fn spawner(&self) -> &Self::Spawner {
        self
    }
}
