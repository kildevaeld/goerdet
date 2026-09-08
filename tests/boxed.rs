use core::{
    future::{Future, Ready, ready},
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};
use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use goerdet::{
    BlockingSpawner, BoxTask, BoxedBlockingSpawner, BoxedLocalSpawner, BoxedSpawner,
    DynBlockingSpawner, DynResult, LocalSpawner, Spawner, Task,
};

#[derive(Clone)]
struct CountingTask(Arc<AtomicUsize>);

impl Task for CountingTask {
    fn detach(self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[derive(Clone)]
struct CountingSpawner {
    spawned: Arc<AtomicUsize>,
    detached: Arc<AtomicUsize>,
}

impl<'a> Spawner<'a> for CountingSpawner {
    type Task = CountingTask;

    fn spawn<T>(&self, _work: T) -> Self::Task
    where
        T: Future<Output = ()> + Send + 'a,
    {
        self.spawned.fetch_add(1, Ordering::SeqCst);
        CountingTask(self.detached.clone())
    }
}

#[derive(Clone)]
struct CountingLocalSpawner {
    spawned: Arc<AtomicUsize>,
    detached: Arc<AtomicUsize>,
}

impl<'a> LocalSpawner<'a> for CountingLocalSpawner {
    type Task = CountingTask;

    fn spawn<T>(&self, _work: T) -> Self::Task
    where
        T: Future<Output = ()> + 'a,
    {
        self.spawned.fetch_add(1, Ordering::SeqCst);
        CountingTask(self.detached.clone())
    }
}

struct ImmediateBlockingSpawner;

impl BlockingSpawner for ImmediateBlockingSpawner {
    type Error = io::Error;
    type Future<R> = Ready<Result<R, Self::Error>>;

    fn spawn_blocking<T, R>(&self, work: T) -> Self::Future<R>
    where
        R: Send + 'static,
        T: FnOnce() -> R + Send + 'static,
    {
        ready(Ok(work()))
    }
}

#[test]
fn box_task_detach_forwards_to_dyn_task() {
    let detached = Arc::new(AtomicUsize::new(0));
    let task: BoxTask = Box::new(CountingTask(detached.clone()));

    task.detach();

    assert_eq!(detached.load(Ordering::SeqCst), 1);
}

#[test]
fn boxed_spawner_returns_box_task_and_forwards_spawn() {
    let spawned = Arc::new(AtomicUsize::new(0));
    let detached = Arc::new(AtomicUsize::new(0));

    let inner = CountingSpawner {
        spawned: spawned.clone(),
        detached: detached.clone(),
    };

    let boxed = BoxedSpawner::new(inner);
    let task = boxed.spawn(async {});

    assert_eq!(spawned.load(Ordering::SeqCst), 1);
    task.detach();
    assert_eq!(detached.load(Ordering::SeqCst), 1);
}

#[test]
fn boxed_local_spawner_returns_box_task_and_forwards_spawn() {
    let spawned = Arc::new(AtomicUsize::new(0));
    let detached = Arc::new(AtomicUsize::new(0));

    let inner = CountingLocalSpawner {
        spawned: spawned.clone(),
        detached: detached.clone(),
    };

    let boxed = BoxedLocalSpawner::new(inner);
    let task = boxed.spawn(async {});

    assert_eq!(spawned.load(Ordering::SeqCst), 1);
    task.detach();
    assert_eq!(detached.load(Ordering::SeqCst), 1);
}

#[test]
fn boxed_blocking_spawner_roundtrips_result_type() {
    let boxed = BoxedBlockingSpawner::new(ImmediateBlockingSpawner);
    let result = block_on(boxed.spawn_blocking(|| 42_usize)).unwrap();

    assert_eq!(result, 42);
}

#[test]
fn boxed_blocking_spawner_dyn_api_returns_dyn_result() {
    let boxed = BoxedBlockingSpawner::new(ImmediateBlockingSpawner);
    let result =
        block_on(boxed.spawn_blocking_boxed(Box::new(|| DynResult(Box::new(7_u32))))).unwrap();

    let value = result.0.downcast::<u32>().expect("expected u32");
    assert_eq!(*value, 7);
}

fn block_on<F: Future>(mut fut: F) -> F::Output {
    // SAFETY: we never move `fut` after pinning.
    let mut fut = unsafe { Pin::new_unchecked(&mut fut) };
    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);

    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(value) => return value,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}

fn noop_waker() -> Waker {
    unsafe fn clone(_: *const ()) -> RawWaker {
        RawWaker::new(core::ptr::null(), &VTABLE)
    }
    unsafe fn wake(_: *const ()) {}
    unsafe fn wake_by_ref(_: *const ()) {}
    unsafe fn drop(_: *const ()) {}

    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

    let raw = RawWaker::new(core::ptr::null(), &VTABLE);
    // SAFETY: `VTABLE` functions are valid no-op implementations.
    unsafe { Waker::from_raw(raw) }
}
