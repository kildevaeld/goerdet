use core::{fmt, pin::Pin};

use alloc::string::{String, ToString};

use crate::{
    BlockingSpawner, Executor, HasBlockingSpawner, HasLocalSpawner, HasSpawner, LocalSpawner,
    Spawner, Task,
};

/// A handle to a `compio` task.
///
/// Implements [`Task`] by detaching the underlying [`compio::runtime::JoinHandle`].
#[derive(Debug)]
pub struct CompioTask(pub compio::runtime::JoinHandle<()>);

impl Task for CompioTask {
    fn detach(self) {
        self.0.detach();
    }
}

/// Executor adapter that spawns tasks on the `compio` runtime.
///
/// Tasks are spawned with [`compio::runtime::spawn`]. Both sendable
/// and local futures are supported.
#[derive(Debug, Clone, Copy, Default)]
pub struct CompioExecutor;

impl<T> Executor<T> for CompioExecutor
where
    T: core::future::Future + 'static,
    T::Output: 'static,
{
    type Task = CompioTask;

    fn spawn(&self, work: T) -> Self::Task {
        CompioTask(compio::runtime::spawn(async move {
            let _ = work.await;
        }))
    }
}

#[cfg(feature = "hyper")]
impl<T> hyper::rt::Executor<T> for CompioExecutor
where
    T: core::future::Future + 'static,
    T::Output: 'static,
{
    fn execute(&self, work: T) {
        compio::runtime::spawn(work).detach();
    }
}

impl Spawner<'static> for CompioExecutor {
    type Task = CompioTask;

    fn spawn<T>(&self, work: T) -> Self::Task
    where
        T: core::future::Future<Output = ()> + Send + 'static,
    {
        CompioTask(compio::runtime::spawn(work))
    }
}

impl LocalSpawner<'static> for CompioExecutor {
    type Task = CompioTask;

    fn spawn<T>(&self, work: T) -> Self::Task
    where
        T: core::future::Future<Output = ()> + 'static,
    {
        CompioTask(compio::runtime::spawn(work))
    }
}

impl LocalSpawner<'static> for compio::runtime::Runtime {
    type Task = CompioTask;

    fn spawn<T>(&self, work: T) -> Self::Task
    where
        T: core::future::Future<Output = ()> + 'static,
    {
        CompioTask(compio::runtime::spawn(work))
    }
}

impl BlockingSpawner for CompioExecutor {
    type Error = CompioJoinError;
    type Future<R> = CompioBlockingFuture<R>;
    fn spawn_blocking<T, R>(&self, work: T) -> Self::Future<R>
    where
        R: Send + 'static,
        T: FnOnce() -> R + Send + 'static,
    {
        CompioBlockingFuture(compio::runtime::spawn_blocking(work))
    }
}

#[derive(Debug)]
pub struct CompioJoinError(String);

impl fmt::Display for CompioJoinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CompioJoinError: {}", self.0)
    }
}

impl core::error::Error for CompioJoinError {}

/// Future returned by [`SmolExecutor::spawn_blocking`].
///
/// Wraps a [`smol::Task`] so that it resolves to `Result<R, Infallible>`,
/// matching the [`BlockingSpawner`] contract.
#[derive(Debug)]
pub struct CompioBlockingFuture<R>(compio_executor::JoinHandle<R>);

impl<R> core::future::Future for CompioBlockingFuture<R> {
    type Output = Result<R, CompioJoinError>;
    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        Pin::new(&mut self.0)
            .poll(cx)
            .map_err(|m| CompioJoinError(m.to_string()))
    }
}

impl HasBlockingSpawner for CompioExecutor {
    type Spawner = Self;
    fn blocking_spawner(&self) -> &Self::Spawner {
        self
    }
}

impl HasLocalSpawner<'static> for CompioExecutor {
    type Spawner = Self;
    fn local_spawner(&self) -> &Self::Spawner {
        self
    }
}

impl HasSpawner<'static> for CompioExecutor {
    type Spawner = Self;
    fn spawner(&self) -> &Self::Spawner {
        self
    }
}
