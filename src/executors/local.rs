use alloc::{boxed::Box, rc::Rc};

use crate::{Executor, LocalBoxFuture, LocalSpawner, Spawner, Task};

/// A task spawned by a [`LocalExecutor`].
///
/// Wraps the task returned by the inner executor and forwards [`Task::detach`].
#[derive(Debug)]
pub struct LocalExecutorTask<T>(T);

impl<T> Task for LocalExecutorTask<T>
where
    T: Task,
{
    fn detach(self) {
        self.0.detach();
    }
}

/// A reference-counted executor for local (non-sendable) futures.
///
/// `LocalExecutor` wraps any [`Executor`] that accepts [`LocalBoxFuture`]s and
/// clones via `Rc`, making it suitable for single-threaded contexts.
pub struct LocalExecutor<'js, E> {
    pub(super) inner: Rc<E>,
    _marker: core::marker::PhantomData<&'js ()>,
}

impl<'js, E> Clone for LocalExecutor<'js, E> {
    fn clone(&self) -> Self {
        LocalExecutor {
            inner: self.inner.clone(),
            _marker: core::marker::PhantomData,
        }
    }
}

impl<'js, E> LocalExecutor<'js, E> {
    /// Creates a new `LocalExecutor` from an [`Executor`] implementation.
    pub fn new(inner: E) -> Self {
        LocalExecutor {
            inner: Rc::new(inner),
            _marker: core::marker::PhantomData,
        }
    }

    /// Spawns a future onto the executor.
    pub fn spawn<T>(&self, work: T) -> E::Task
    where
        E: Executor<LocalBoxFuture<'js, T::Output>>,
        T: core::future::Future + 'js,
    {
        self.inner.spawn(Box::pin(work))
    }
}

impl<'js, E, T> Executor<T> for LocalExecutor<'js, E>
where
    E: Executor<LocalBoxFuture<'js, T::Output>>,
    T: core::future::Future + 'js,
{
    type Task = E::Task;

    fn spawn(&self, work: T) -> Self::Task {
        self.inner.spawn(Box::pin(work))
    }
}

impl<'js, E> LocalSpawner<'js> for LocalExecutor<'js, E>
where
    E: Executor<LocalBoxFuture<'js, ()>>,
{
    type Task = LocalExecutorTask<E::Task>;

    fn spawn<T>(&self, work: T) -> Self::Task
    where
        T: core::future::Future<Output = ()> + 'js,
    {
        LocalExecutorTask(self.inner.spawn(Box::pin(work)))
    }
}

impl<'js, E> Spawner<'js> for LocalExecutor<'js, E>
where
    E: Executor<LocalBoxFuture<'js, ()>>,
{
    type Task = LocalExecutorTask<E::Task>;

    fn spawn<T>(&self, work: T) -> Self::Task
    where
        T: core::future::Future<Output = ()> + Send + 'js,
    {
        LocalExecutorTask(self.inner.spawn(Box::pin(work)))
    }
}

#[cfg(feature = "hyper")]
impl<'js, E, T> hyper::rt::Executor<T> for LocalExecutor<'js, E>
where
    E: Executor<LocalBoxFuture<'js, T::Output>>,
    T: core::future::Future + 'js,
{
    fn execute(&self, work: T) {
        self.inner.spawn(Box::pin(work)).detach();
    }
}
