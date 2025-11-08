//! aurora_optimizer - Performance Tuning
//!
//! CPU-profiled optimizations, performance gates, and benchmark enforcement.
//!
//! # Example
//!
//! ```
//! use aurora_optimizer::profile::{CpuCharacteristics, OptimizationStrategy};
//! use aurora_optimizer::perf_gate::PerfMetric;
//!
//! // Create CPU-specific optimization strategy
//! let cpu = CpuCharacteristics::skylake();
//! let strategy = OptimizationStrategy::for_cpu(cpu);
//!
//! // Create performance metric
//! let metric = PerfMetric::throughput("compute", 1000.0);
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

/// CPU-specific optimization profiles
pub mod profile;

/// Performance gates and benchmarks
pub mod perf_gate;

// Re-export main types
pub use perf_gate::{BenchResult, PerfError, PerfGate, PerfMetric};
pub use profile::{CpuCharacteristics, CpuProfile, OptimizationStrategy};
