//! Executor abstractions and runtime adapters for `bycat`.
//!
//! This crate provides a small set of traits for spawning futures and blocking
//! work, plus concrete adapters for popular async runtimes. It is `no_std` and
//! only requires `alloc`.
//!
//! # Features
//!
//! - `tokio` (default): adapters for the Tokio runtime.
//! - `smol`: adapters for the `smol` runtime.
//! - `compio`: adapters for the `compio` runtime.
//! - `hyper`: implements `hyper::rt::Executor` for the enabled runtime adapters.

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
mod boxed;
mod executors;
mod traits;
#[cfg(feature = "alloc")]
mod types;

#[cfg(feature = "alloc")]
pub use boxed::*;
#[cfg(any(
    feature = "alloc",
    feature = "compio",
    feature = "smol",
    feature = "tokio"
))]
pub use executors::*;
pub use traits::*;
#[cfg(feature = "alloc")]
pub use types::*;
