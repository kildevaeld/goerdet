use alloc::boxed::Box;
use core::pin::Pin;

/// A pinned, boxed, sendable future with lifetime `'a` and output `T`.
pub type BoxFuture<'a, T> = Pin<Box<dyn core::future::Future<Output = T> + Send + 'a>>;

/// A pinned, boxed future with lifetime `'a` and output `T`.
///
/// Unlike [`BoxFuture`], this type is not required to be `Send`, so it can
/// hold non-sendable state such as `Rc` or local runtime handles.
pub type LocalBoxFuture<'a, T> = Pin<Box<dyn core::future::Future<Output = T> + 'a>>;

pub type BoxError = Box<dyn core::error::Error + Send + Sync + 'static>;
