//! aurora_concurrency - Concurrency Primitives for Aurora
//!
//! This crate provides Aurora's concurrency features including:
//! - Work-stealing scheduler for goroutines
//! - Go-style channels for communication
//! - Async/await runtime with structured cancellation
//!
//! # Example
//!
//! ```
//! use aurora_concurrency::scheduler::{Scheduler, SchedulerConfig};
//! use aurora_concurrency::channels::buffered;
//!
//! // Create a scheduler
//! let mut scheduler = Scheduler::new(SchedulerConfig::default());
//!
//! // Spawn goroutines
//! scheduler.spawn(|| {
//!     println!("Hello from goroutine!");
//! });
//!
//! // Create a channel
//! let (tx, rx) = buffered::<i32>(10);
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

/// Work-stealing scheduler for goroutines
pub mod scheduler;

/// Go-style channels
pub mod channels;

/// Async/await runtime
pub mod async_rt;

// Re-export main types
pub use scheduler::{Goroutine, GoroutineId, GoroutineState, Scheduler, SchedulerConfig, SchedulerStats};
pub use channels::{channel, buffered, unbuffered, ChannelCapacity, ChannelError, Receiver, Sender};
pub use async_rt::{AsyncError, AsyncRuntime, CancellationToken, RuntimeStats, Task, TaskId, TaskState};
