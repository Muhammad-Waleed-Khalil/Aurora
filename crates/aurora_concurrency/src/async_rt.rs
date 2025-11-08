//! Async/Await Runtime for Aurora
//!
//! Implements state machine lowering and structured cancellation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};
use thiserror::Error;

/// Async runtime errors
#[derive(Debug, Error)]
pub enum AsyncError {
    /// Task was cancelled
    #[error("Task was cancelled")]
    Cancelled,

    /// Task panicked
    #[error("Task panicked: {0}")]
    Panic(String),

    /// Runtime error
    #[error("Runtime error: {0}")]
    RuntimeError(String),
}

/// Result type for async operations
pub type Result<T> = std::result::Result<T, AsyncError>;

/// Unique ID for an async task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub u64);

/// Task state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    /// Task is pending execution
    Pending,
    /// Task is running
    Running,
    /// Task is waiting
    Waiting,
    /// Task completed successfully
    Completed,
    /// Task was cancelled
    Cancelled,
    /// Task panicked
    Panicked,
}

/// Cancellation token for structured cancellation
#[derive(Clone)]
pub struct CancellationToken {
    cancelled: Arc<Mutex<bool>>,
    children: Arc<Mutex<Vec<CancellationToken>>>,
}

impl CancellationToken {
    /// Create a new cancellation token
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(Mutex::new(false)),
            children: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Cancel this token and all children
    pub fn cancel(&self) {
        *self.cancelled.lock().unwrap() = true;

        // Cancel all children
        let children = self.children.lock().unwrap();
        for child in children.iter() {
            child.cancel();
        }
    }

    /// Check if cancelled
    pub fn is_cancelled(&self) -> bool {
        *self.cancelled.lock().unwrap()
    }

    /// Create a child token
    pub fn child(&self) -> Self {
        let child = Self::new();
        self.children.lock().unwrap().push(child.clone());
        child
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

/// Async task
pub struct Task {
    /// Unique ID
    pub id: TaskId,
    /// Current state
    pub state: TaskState,
    /// Cancellation token
    pub cancel_token: CancellationToken,
    /// Future to execute
    future: Pin<Box<dyn Future<Output = ()> + Send>>,
}

impl Task {
    /// Create a new task
    pub fn new<F>(id: TaskId, future: F, cancel_token: CancellationToken) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
    {
        Self {
            id,
            state: TaskState::Pending,
            cancel_token,
            future: Box::pin(future),
        }
    }

    /// Poll the task
    pub fn poll(&mut self, waker: &Waker) -> Poll<()> {
        if self.cancel_token.is_cancelled() {
            self.state = TaskState::Cancelled;
            return Poll::Ready(());
        }

        self.state = TaskState::Running;
        let mut context = Context::from_waker(waker);

        match self.future.as_mut().poll(&mut context) {
            Poll::Ready(()) => {
                self.state = TaskState::Completed;
                Poll::Ready(())
            }
            Poll::Pending => {
                self.state = TaskState::Waiting;
                Poll::Pending
            }
        }
    }
}

/// Simple waker implementation
struct SimpleWaker {
    task_id: TaskId,
}

impl Wake for SimpleWaker {
    fn wake(self: Arc<Self>) {
        // In a real implementation, this would notify the runtime
        // to re-poll the task
    }
}

/// Async runtime
pub struct AsyncRuntime {
    /// All tasks
    tasks: Arc<Mutex<HashMap<TaskId, Task>>>,
    /// Next task ID
    next_id: Arc<Mutex<u64>>,
    /// Root cancellation token
    root_token: CancellationToken,
}

impl AsyncRuntime {
    /// Create a new async runtime
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(0)),
            root_token: CancellationToken::new(),
        }
    }

    /// Spawn an async task
    pub fn spawn<F>(&mut self, future: F) -> TaskId
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let id = self.allocate_id();
        let cancel_token = self.root_token.child();
        let task = Task::new(id, future, cancel_token);

        self.tasks.lock().unwrap().insert(id, task);
        id
    }

    /// Spawn with a custom cancellation token
    pub fn spawn_with_token<F>(&mut self, future: F, cancel_token: CancellationToken) -> TaskId
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let id = self.allocate_id();
        let task = Task::new(id, future, cancel_token);

        self.tasks.lock().unwrap().insert(id, task);
        id
    }

    /// Allocate a new task ID
    fn allocate_id(&self) -> TaskId {
        let mut next = self.next_id.lock().unwrap();
        let id = *next;
        *next += 1;
        TaskId(id)
    }

    /// Run all pending tasks to completion
    pub fn block_on<F, T>(&mut self, future: F) -> Result<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        // Simple implementation: poll until ready
        let result = Arc::new(Mutex::new(None));
        let result_clone = Arc::clone(&result);

        let wrapped_future = async move {
            let value = future.await;
            *result_clone.lock().unwrap() = Some(value);
        };

        let id = self.spawn(wrapped_future);
        let waker = Arc::new(SimpleWaker { task_id: id }).into();

        // Poll until complete
        loop {
            let mut tasks = self.tasks.lock().unwrap();
            if let Some(task) = tasks.get_mut(&id) {
                match task.poll(&waker) {
                    Poll::Ready(()) => {
                        if task.state == TaskState::Cancelled {
                            return Err(AsyncError::Cancelled);
                        }
                        break;
                    }
                    Poll::Pending => {
                        drop(tasks);
                        std::thread::sleep(std::time::Duration::from_micros(10));
                    }
                }
            } else {
                break;
            }
        }

        let value = result
            .lock()
            .unwrap()
            .take();

        value.ok_or_else(|| AsyncError::RuntimeError("Task did not complete".to_string()))
    }

    /// Cancel all tasks
    pub fn cancel_all(&self) {
        self.root_token.cancel();
    }

    /// Get task state
    pub fn task_state(&self, id: TaskId) -> Option<TaskState> {
        self.tasks.lock().unwrap().get(&id).map(|t| t.state.clone())
    }

    /// Get runtime statistics
    pub fn stats(&self) -> RuntimeStats {
        let tasks = self.tasks.lock().unwrap();
        let mut stats = RuntimeStats {
            total_tasks: tasks.len(),
            pending: 0,
            running: 0,
            waiting: 0,
            completed: 0,
            cancelled: 0,
            panicked: 0,
        };

        for task in tasks.values() {
            match task.state {
                TaskState::Pending => stats.pending += 1,
                TaskState::Running => stats.running += 1,
                TaskState::Waiting => stats.waiting += 1,
                TaskState::Completed => stats.completed += 1,
                TaskState::Cancelled => stats.cancelled += 1,
                TaskState::Panicked => stats.panicked += 1,
            }
        }

        stats
    }
}

impl Default for AsyncRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// Runtime statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStats {
    /// Total number of tasks
    pub total_tasks: usize,
    /// Pending tasks
    pub pending: usize,
    /// Running tasks
    pub running: usize,
    /// Waiting tasks
    pub waiting: usize,
    /// Completed tasks
    pub completed: usize,
    /// Cancelled tasks
    pub cancelled: usize,
    /// Panicked tasks
    pub panicked: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = AsyncRuntime::new();
        let stats = runtime.stats();
        assert_eq!(stats.total_tasks, 0);
    }

    #[test]
    fn test_spawn_task() {
        let mut runtime = AsyncRuntime::new();

        let id = runtime.spawn(async {
            // Simple async task
        });

        assert_eq!(id.0, 0);
        assert_eq!(runtime.stats().total_tasks, 1);
    }

    #[test]
    fn test_block_on() {
        let mut runtime = AsyncRuntime::new();

        let result = runtime.block_on(async { 42 }).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_cancellation_token() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());

        token.cancel();
        assert!(token.is_cancelled());
    }

    #[test]
    fn test_child_cancellation() {
        let parent = CancellationToken::new();
        let child = parent.child();

        assert!(!parent.is_cancelled());
        assert!(!child.is_cancelled());

        parent.cancel();
        assert!(parent.is_cancelled());
        assert!(child.is_cancelled());
    }

    #[test]
    fn test_task_states() {
        let token = CancellationToken::new();
        let id = TaskId(1);

        let task = Task::new(id, async {}, token);
        assert_eq!(task.state, TaskState::Pending);
        assert_eq!(task.id, id);
    }

    #[test]
    fn test_runtime_stats() {
        let mut runtime = AsyncRuntime::new();

        runtime.spawn(async {});
        runtime.spawn(async {});

        let stats = runtime.stats();
        assert_eq!(stats.total_tasks, 2);
    }

    #[test]
    fn test_cancel_all() {
        let runtime = AsyncRuntime::new();

        assert!(!runtime.root_token.is_cancelled());
        runtime.cancel_all();
        assert!(runtime.root_token.is_cancelled());
    }

    #[test]
    fn test_task_id_allocation() {
        let mut runtime = AsyncRuntime::new();

        let id1 = runtime.spawn(async {});
        let id2 = runtime.spawn(async {});
        let id3 = runtime.spawn(async {});

        assert_eq!(id1.0, 0);
        assert_eq!(id2.0, 1);
        assert_eq!(id3.0, 2);
    }
}
