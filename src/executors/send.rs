use alloc::{boxed::Box, sync::Arc};

use crate::{BoxFuture, Executor, Spawner, Task};

/// A task spawned by a [`SendExecutor`].
///
/// Wraps the task returned by the inner executor and forwards [`Task::detach`].
#[derive(Debug)]
pub struct SendExecutorTask<T>(T);

impl<T> Task for SendExecutorTask<T>
where
    T: Task,
{
    fn detach(self) {
        self.0.detach();
    }
}

/// A thread-safe, reference-counted executor for sendable futures.
///
/// `SendExecutor` wraps any [`Executor`] that accepts [`BoxFuture`]s and clones
/// via `Arc`, allowing it to be shared across threads.
pub struct SendExecutor<'js, E> {
    pub(super) inner: Arc<E>,
    _marker: core::marker::PhantomData<&'js ()>,
}

impl<'js, E> Clone for SendExecutor<'js, E> {
    fn clone(&self) -> Self {
        SendExecutor {
            inner: self.inner.clone(),
            _marker: core::marker::PhantomData,
        }
    }
}

impl<'js, E> SendExecutor<'js, E> {
    /// Creates a new `SendExecutor` from a thread-safe [`Executor`] implementation.
    pub fn new(inner: E) -> Self
    where
        E: Send + Sync,
    {
        SendExecutor {
            inner: Arc::new(inner),
            _marker: core::marker::PhantomData,
        }
    }

    /// Spawns a sendable future onto the executor.
    pub fn spawn<T>(&self, work: T) -> E::Task
    where
        E: Executor<BoxFuture<'js, T::Output>>,
        T: core::future::Future + Send + 'js,
    {
        self.inner.spawn(Box::pin(work))
    }
}

impl<'js, E, T> Executor<T> for SendExecutor<'js, E>
where
    E: Executor<BoxFuture<'js, T::Output>>,
    T: core::future::Future + Send + 'js,
{
    type Task = E::Task;

    fn spawn(&self, work: T) -> Self::Task {
        self.inner.spawn(Box::pin(work))
    }
}

impl<'js, E> Spawner<'js> for SendExecutor<'js, E>
where
    E: Executor<BoxFuture<'js, ()>>,
{
    type Task = SendExecutorTask<E::Task>;

    fn spawn<T>(&self, work: T) -> Self::Task
    where
        T: core::future::Future<Output = ()> + Send + 'js,
    {
        SendExecutorTask(self.inner.spawn(Box::pin(work)))
    }
}

#[cfg(feature = "hyper")]
impl<'js, E, T> hyper::rt::Executor<T> for SendExecutor<'js, E>
where
    E: Executor<BoxFuture<'js, T::Output>>,
    T: core::future::Future + Send + 'js,
{
    fn execute(&self, work: T) {
        self.inner.spawn(Box::pin(work)).detach();
    }
}
