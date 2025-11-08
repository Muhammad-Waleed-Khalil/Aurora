//! aurora_testing - Testing Framework for Aurora
//!
//! This crate provides comprehensive testing capabilities including:
//! - Unit testing framework
//! - Property-based testing
//! - Golden snapshot testing
//! - Differential testing against C implementations
//!
//! # Example
//!
//! ```
//! use aurora_testing::framework::{Test, TestSuite};
//!
//! let mut suite = TestSuite::new("my_tests");
//! suite.add_test(Test::new("test_addition", || {
//!     aurora_testing::framework::assert_eq(2 + 2, 4, "math works")
//! }));
//!
//! let result = suite.run();
//! assert!(result.success());
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

/// Core testing framework
pub mod framework;

/// Differential testing
pub mod differential;

// Re-export main types
pub use framework::{Test, TestError, TestOutcome, TestResult, TestSuite, TestSuiteResult};
pub use differential::{DiffError, DiffResult, DifferentialRunner, DifferentialTest};
