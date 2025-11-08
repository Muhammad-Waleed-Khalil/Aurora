//! Work-Stealing Scheduler for Aurora Goroutines
//!
//! Implements M:N threading with work-stealing for efficient goroutine execution.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use thiserror::Error;

/// Scheduler errors
#[derive(Debug, Error)]
pub enum SchedulerError {
    /// Worker thread panicked
    #[error("Worker thread panicked")]
    WorkerPanic,

    /// Failed to spawn goroutine
    #[error("Failed to spawn goroutine: {0}")]
    SpawnError(String),
}

/// Result type for scheduler operations
pub type Result<T> = std::result::Result<T, SchedulerError>;

/// Unique ID for a goroutine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GoroutineId(pub u64);

/// Goroutine state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoroutineState {
    /// Ready to run
    Ready,
    /// Currently running
    Running,
    /// Blocked on I/O or channel
    Blocked,
    /// Completed execution
    Completed,
}

/// A goroutine task
pub struct Goroutine {
    /// Unique ID
    pub id: GoroutineId,
    /// Current state
    pub state: GoroutineState,
    /// Task to execute
    task: Box<dyn FnOnce() + Send + 'static>,
}

impl Goroutine {
    /// Create a new goroutine
    pub fn new<F>(id: GoroutineId, task: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        Self {
            id,
            state: GoroutineState::Ready,
            task: Box::new(task),
        }
    }

    /// Execute the goroutine
    pub fn run(mut self) {
        self.state = GoroutineState::Running;
        (self.task)();
        self.state = GoroutineState::Completed;
    }
}

/// Work queue for a worker thread
#[derive(Clone)]
struct WorkQueue {
    queue: Arc<Mutex<VecDeque<Goroutine>>>,
}

impl WorkQueue {
    fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    fn push(&self, goroutine: Goroutine) {
        self.queue.lock().unwrap().push_back(goroutine);
    }

    fn pop(&self) -> Option<Goroutine> {
        self.queue.lock().unwrap().pop_front()
    }

    fn steal(&self) -> Option<Goroutine> {
        self.queue.lock().unwrap().pop_back()
    }

    fn len(&self) -> usize {
        self.queue.lock().unwrap().len()
    }
}

/// Work-stealing scheduler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// Number of worker threads
    pub num_workers: usize,
    /// Maximum goroutines per worker
    pub max_goroutines_per_worker: usize,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            num_workers: num_cpus(),
            max_goroutines_per_worker: 1000,
        }
    }
}

/// Get number of CPUs (fallback to 4 if detection fails)
fn num_cpus() -> usize {
    thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

/// M:N work-stealing scheduler
pub struct Scheduler {
    /// Configuration
    config: SchedulerConfig,
    /// Work queues for each worker
    work_queues: Vec<WorkQueue>,
    /// Next goroutine ID
    next_id: Arc<Mutex<u64>>,
    /// Worker threads
    workers: Vec<Option<thread::JoinHandle<()>>>,
}

impl Scheduler {
    /// Create a new scheduler
    pub fn new(config: SchedulerConfig) -> Self {
        let work_queues: Vec<WorkQueue> = (0..config.num_workers)
            .map(|_| WorkQueue::new())
            .collect();

        Self {
            config,
            work_queues,
            next_id: Arc::new(Mutex::new(0)),
            workers: Vec::new(),
        }
    }

    /// Spawn a new goroutine
    pub fn spawn<F>(&mut self, task: F) -> GoroutineId
    where
        F: FnOnce() + Send + 'static,
    {
        let id = self.allocate_id();
        let goroutine = Goroutine::new(id, task);

        // Round-robin assignment to workers
        let worker_idx = (id.0 as usize) % self.config.num_workers;
        self.work_queues[worker_idx].push(goroutine);

        id
    }

    /// Allocate a new goroutine ID
    fn allocate_id(&self) -> GoroutineId {
        let mut next = self.next_id.lock().unwrap();
        let id = *next;
        *next += 1;
        GoroutineId(id)
    }

    /// Start the scheduler
    pub fn start(&mut self) {
        for worker_id in 0..self.config.num_workers {
            let queues = self.work_queues.clone();
            let num_workers = self.config.num_workers;

            let handle = thread::spawn(move || {
                Self::worker_loop(worker_id, queues, num_workers);
            });

            self.workers.push(Some(handle));
        }
    }

    /// Worker thread main loop
    fn worker_loop(worker_id: usize, queues: Vec<WorkQueue>, num_workers: usize) {
        loop {
            // Try to get work from own queue
            if let Some(goroutine) = queues[worker_id].pop() {
                goroutine.run();
                continue;
            }

            // Try to steal from other workers
            let mut found_work = false;
            for steal_from in 0..num_workers {
                if steal_from == worker_id {
                    continue;
                }

                if let Some(goroutine) = queues[steal_from].steal() {
                    goroutine.run();
                    found_work = true;
                    break;
                }
            }

            if !found_work {
                // No work available, sleep briefly
                thread::sleep(std::time::Duration::from_micros(100));

                // Check if all queues are empty to potentially exit
                let total_work: usize = queues.iter().map(|q| q.len()).sum();
                if total_work == 0 {
                    break;
                }
            }
        }
    }

    /// Wait for all workers to complete
    pub fn join(mut self) -> Result<()> {
        for worker in self.workers.drain(..) {
            if let Some(handle) = worker {
                handle.join().map_err(|_| SchedulerError::WorkerPanic)?;
            }
        }
        Ok(())
    }

    /// Get scheduler statistics
    pub fn stats(&self) -> SchedulerStats {
        let queue_lengths: Vec<usize> = self.work_queues.iter().map(|q| q.len()).collect();
        let total_pending: usize = queue_lengths.iter().sum();

        SchedulerStats {
            num_workers: self.config.num_workers,
            total_pending,
            queue_lengths,
        }
    }
}

/// Scheduler statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStats {
    /// Number of worker threads
    pub num_workers: usize,
    /// Total pending goroutines
    pub total_pending: usize,
    /// Queue lengths per worker
    pub queue_lengths: Vec<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_scheduler_creation() {
        let config = SchedulerConfig::default();
        let scheduler = Scheduler::new(config.clone());
        assert_eq!(scheduler.config.num_workers, config.num_workers);
    }

    #[test]
    fn test_goroutine_spawn() {
        let config = SchedulerConfig {
            num_workers: 2,
            max_goroutines_per_worker: 100,
        };
        let mut scheduler = Scheduler::new(config);

        let id = scheduler.spawn(|| {
            println!("Hello from goroutine!");
        });

        assert_eq!(id.0, 0);
    }

    #[test]
    fn test_multiple_goroutines() {
        let config = SchedulerConfig {
            num_workers: 4,
            max_goroutines_per_worker: 100,
        };
        let mut scheduler = Scheduler::new(config);

        let counter = Arc::new(AtomicUsize::new(0));

        for _ in 0..10 {
            let counter_clone = Arc::clone(&counter);
            scheduler.spawn(move || {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            });
        }

        scheduler.start();
        scheduler.join().unwrap();

        assert_eq!(counter.load(Ordering::SeqCst), 10);
    }

    #[test]
    fn test_work_stealing() {
        let config = SchedulerConfig {
            num_workers: 2,
            max_goroutines_per_worker: 100,
        };
        let mut scheduler = Scheduler::new(config);

        let counter = Arc::new(AtomicUsize::new(0));

        // Spawn many tasks to ensure work stealing occurs
        for _ in 0..100 {
            let counter_clone = Arc::clone(&counter);
            scheduler.spawn(move || {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                thread::sleep(std::time::Duration::from_micros(10));
            });
        }

        scheduler.start();
        scheduler.join().unwrap();

        assert_eq!(counter.load(Ordering::SeqCst), 100);
    }

    #[test]
    fn test_goroutine_states() {
        let id = GoroutineId(42);
        let goroutine = Goroutine::new(id, || {});

        assert_eq!(goroutine.id, id);
        assert_eq!(goroutine.state, GoroutineState::Ready);
    }

    #[test]
    fn test_scheduler_stats() {
        let config = SchedulerConfig {
            num_workers: 3,
            max_goroutines_per_worker: 100,
        };
        let mut scheduler = Scheduler::new(config);

        for _ in 0..5 {
            scheduler.spawn(|| {});
        }

        let stats = scheduler.stats();
        assert_eq!(stats.num_workers, 3);
        assert_eq!(stats.total_pending, 5);
    }

    #[test]
    fn test_goroutine_id_allocation() {
        let config = SchedulerConfig::default();
        let mut scheduler = Scheduler::new(config);

        let id1 = scheduler.spawn(|| {});
        let id2 = scheduler.spawn(|| {});
        let id3 = scheduler.spawn(|| {});

        assert_eq!(id1.0, 0);
        assert_eq!(id2.0, 1);
        assert_eq!(id3.0, 2);
    }
}
