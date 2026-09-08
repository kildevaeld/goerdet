/// A trait for tasks that can be detached from the executor.
///
/// If the task is detached, it will continue to run in the background until it
/// completes or the task is dropped. This is useful for fire-and-forget tasks
/// that do not need to be awaited.
pub trait Task {
    /// Detaches the task, allowing it to run in the background.
    fn detach(self);
}

/// Spawns a single future or task onto an executor.
///
/// This is the low-level trait used by the higher-level [`LocalExecutor`] and
/// [`SendExecutor`] wrappers. Implementors receive the task directly and are
/// responsible for driving it to completion.
pub trait Executor<T> {
    type Task: Task;
    /// Spawns `work` onto the executor.
    fn spawn(&self, work: T) -> Self::Task;
}

impl<'a, T, V> Executor<T> for &'a V
where
    V: Executor<T> + ?Sized,
{
    type Task = V::Task;
    fn spawn(&self, work: T) -> V::Task {
        (**self).spawn(work)
    }
}

/// Spawns sendable futures that produce no value.
///
/// This is useful for contexts that only need to fire-and-forget sendable
/// futures with a unit output, such as background task loops.
pub trait Spawner<'a> {
    type Task: Task;
    /// Spawns `work` on the executor.
    fn spawn<T>(&self, work: T) -> Self::Task
    where
        T: core::future::Future<Output = ()> + Send + 'a;
}

impl<'a, 'b, T> Spawner<'a> for &'b T
where
    T: Spawner<'a> + ?Sized,
{
    type Task = T::Task;
    fn spawn<U>(&self, work: U) -> Self::Task
    where
        U: core::future::Future<Output = ()> + Send + 'a,
    {
        (**self).spawn(work)
    }
}

impl<'a, 'b, T> Spawner<'a> for &'b mut T
where
    T: Spawner<'a> + ?Sized,
{
    type Task = T::Task;
    fn spawn<U>(&self, work: U) -> Self::Task
    where
        U: core::future::Future<Output = ()> + Send + 'a,
    {
        (**self).spawn(work)
    }
}

/// Spawns local (non-sendable) futures that produce no value.
///
/// Like [`Spawner`], but does not require the future to be `Send`, allowing it
/// to be used with single-threaded executors.
pub trait LocalSpawner<'a> {
    type Task: Task;
    /// Spawns `work` on the local executor.
    fn spawn<T>(&self, work: T) -> Self::Task
    where
        T: core::future::Future<Output = ()> + 'a;
}

impl<'a, 'b, T> LocalSpawner<'a> for &'b T
where
    T: LocalSpawner<'a> + ?Sized,
{
    type Task = T::Task;
    fn spawn<U>(&self, work: U) -> Self::Task
    where
        U: core::future::Future<Output = ()> + 'a,
    {
        (**self).spawn(work)
    }
}

impl<'a, 'b, T> LocalSpawner<'a> for &'b mut T
where
    T: LocalSpawner<'a> + ?Sized,
{
    type Task = T::Task;
    fn spawn<U>(&self, work: U) -> Self::Task
    where
        U: core::future::Future<Output = ()> + 'a,
    {
        (**self).spawn(work)
    }
}

/// Spawns blocking work onto a thread pool.
///
/// The returned future resolves to the result of the blocking closure, or an
/// error if the executor failed to run or return it.
pub trait BlockingSpawner {
    /// Future that resolves once the blocking work finishes.
    type Future<R>: core::future::Future<Output = Result<R, Self::Error>>;

    /// Error returned when the blocking task cannot be completed.
    type Error;

    /// Spawns `work` on a blocking thread pool.
    fn spawn_blocking<T, R>(&self, work: T) -> Self::Future<R>
    where
        R: Send + 'static,
        T: FnOnce() -> R + Send + 'static;
}

impl<T> BlockingSpawner for &T
where
    T: BlockingSpawner + ?Sized,
{
    type Future<R> = T::Future<R>;
    type Error = T::Error;

    fn spawn_blocking<U, R>(&self, work: U) -> Self::Future<R>
    where
        R: Send + 'static,
        U: FnOnce() -> R + Send + 'static,
    {
        (**self).spawn_blocking(work)
    }
}

/// Provides access to a [`Spawner`].
pub trait HasSpawner<'a> {
    /// The spawner type.
    type Spawner: Spawner<'a>;

    /// Returns a reference to the spawner.
    fn spawner(&self) -> &Self::Spawner;
}

/// Provides access to a [`LocalSpawner`].
pub trait HasLocalSpawner<'a> {
    /// The local spawner type.
    type Spawner: LocalSpawner<'a>;

    /// Returns a reference to the local spawner.
    fn local_spawner(&self) -> &Self::Spawner;
}

/// Provides access to a [`BlockingSpawner`].
pub trait HasBlockingSpawner {
    /// The blocking spawner type.
    type Spawner: BlockingSpawner;

    /// Returns a reference to the blocking spawner.
    fn blocking_spawner(&self) -> &Self::Spawner;
}
