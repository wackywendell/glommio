//! Async I/O and timers.
//!
//! To wait for the next I/O event, the reactor calls [epoll] on Linux/Android, [kqueue] on
//! macOS/iOS/BSD, and [wepoll] on Windows.
//!
//! [epoll]: https://en.wikipedia.org/wiki/Epoll
//! [kqueue]: https://en.wikipedia.org/wiki/Kqueue
//! [wepoll]: https://github.com/piscisaureus/wepoll
//!
//! # Examples
//!
//! Connect to `example.com:80`, or time out after 10 seconds.
//!
//! ```
//! use async_io::{Async, Timer};
//! use futures_lite::{future::FutureExt, io};
//!
//! use std::net::{TcpStream, ToSocketAddrs};
//! use std::time::Duration;
//!
//! # futures_lite::future::block_on(async {
//! let addr = "example.com:80".to_socket_addrs()?.next().unwrap();
//!
//! let stream = Async::<TcpStream>::connect(addr).or(async {
//!     Timer::new(Duration::from_secs(10)).await;
//!     Err(io::ErrorKind::TimedOut.into())
//! })
//! .await?;
//! # std::io::Result::Ok(()) });
//! ```

#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]
#[macro_use]
extern crate nix;
extern crate alloc;

use crate::parking::Reactor;
use std::fmt::Debug;
use std::time::Duration;

pub mod parking;
mod sys;
pub mod task;

mod async_collections;
mod dma_file;
mod error;
mod executor;
mod local_semaphore;
mod multitask;
mod networking;
mod pollable;
mod timer;

pub use crate::async_collections::AsyncDeque;
pub use crate::dma_file::{Directory, DmaFile};
pub use crate::error::Error;
pub use crate::executor::{LocalExecutor, QueueNotFoundError, Task, TaskQueueHandle};
pub use crate::local_semaphore::Semaphore;
pub use crate::networking::*;
pub use crate::pollable::Async;
pub use crate::sys::DmaBuffer;
pub use crate::timer::Timer;

/// An attribute of a TaskQueue, passed during its creation.
///
/// This tells the executor whether or not tasks in this class are latency
/// sensitive. Latency sensitive tasks will be placed in their own I/O ring,
/// and tasks in background classes can cooperatively preempt themselves in
/// the faces of pending events for latency classes.
#[derive(Clone, Copy, Debug)]
pub enum Latency {
    /// Tasks marked as Latency::Matters will cooperatively signal to other tasks that the should
    /// preempt often
    Matters(Duration),

    /// Tasks marked as Latency::NotImportant will not signal to other tasks that the should
    /// preempt often
    NotImportant,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct IoRequirements {
    latency_req: Latency,
    io_handle: usize,
}

impl Default for IoRequirements {
    fn default() -> Self {
        Self {
            latency_req: Latency::NotImportant,
            io_handle: 0,
        }
    }
}

impl IoRequirements {
    fn new(latency: Latency, handle: usize) -> Self {
        Self {
            latency_req: latency,
            io_handle: handle,
        }
    }
}

/// Represents a wrapper around io::Result that packs more
/// information about the file being accessed.
pub type Result<T> = std::result::Result<T, Error>;