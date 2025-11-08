//! Testing Framework for Aurora
//!
//! Provides unit testing, property testing, and golden snapshot testing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::panic;
use std::time::{Duration, Instant};
use thiserror::Error;

/// Testing errors
#[derive(Debug, Error)]
pub enum TestError {
    /// Assertion failed
    #[error("Assertion failed: {0}")]
    AssertionFailed(String),

    /// Test panicked
    #[error("Test panicked: {0}")]
    Panicked(String),

    /// Timeout
    #[error("Test timed out after {0:?}")]
    Timeout(Duration),
}

/// Result type for tests
pub type TestResult = Result<(), TestError>;

/// Test case
pub struct Test {
    /// Test name
    pub name: String,
    /// Test function
    pub func: Box<dyn FnOnce() -> TestResult + Send>,
    /// Timeout
    pub timeout: Option<Duration>,
}

impl Test {
    /// Create a new test
    pub fn new<F>(name: impl Into<String>, func: F) -> Self
    where
        F: FnOnce() -> TestResult + Send + 'static,
    {
        Self {
            name: name.into(),
            func: Box::new(func),
            timeout: None,
        }
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Run the test
    pub fn run(self) -> TestOutcome {
        let start = Instant::now();

        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            (self.func)()
        }));

        let duration = start.elapsed();

        match result {
            Ok(Ok(())) => TestOutcome {
                name: self.name,
                result: TestResult::Ok(()),
                duration,
            },
            Ok(Err(e)) => TestOutcome {
                name: self.name,
                result: Err(e),
                duration,
            },
            Err(panic_info) => {
                let msg = if let Some(s) = panic_info.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(s) = panic_info.downcast_ref::<&str>() {
                    s.to_string()
                } else {
                    "Unknown panic".to_string()
                };

                TestOutcome {
                    name: self.name,
                    result: Err(TestError::Panicked(msg)),
                    duration,
                }
            }
        }
    }
}

/// Test outcome
#[derive(Debug)]
pub struct TestOutcome {
    /// Test name
    pub name: String,
    /// Result
    pub result: TestResult,
    /// Duration
    pub duration: Duration,
}

impl TestOutcome {
    /// Check if test passed
    pub fn passed(&self) -> bool {
        self.result.is_ok()
    }
}

/// Test suite
pub struct TestSuite {
    /// Suite name
    pub name: String,
    /// Tests
    tests: Vec<Test>,
}

impl TestSuite {
    /// Create a new test suite
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tests: Vec::new(),
        }
    }

    /// Add a test
    pub fn add_test(&mut self, test: Test) {
        self.tests.push(test);
    }

    /// Run all tests
    pub fn run(self) -> TestSuiteResult {
        let mut outcomes = Vec::new();
        let start = Instant::now();

        for test in self.tests {
            outcomes.push(test.run());
        }

        let duration = start.elapsed();
        let passed = outcomes.iter().filter(|o| o.passed()).count();
        let failed = outcomes.len() - passed;

        TestSuiteResult {
            name: self.name,
            outcomes,
            passed,
            failed,
            duration,
        }
    }
}

/// Test suite result
#[derive(Debug)]
pub struct TestSuiteResult {
    /// Suite name
    pub name: String,
    /// Test outcomes
    pub outcomes: Vec<TestOutcome>,
    /// Number of passed tests
    pub passed: usize,
    /// Number of failed tests
    pub failed: usize,
    /// Total duration
    pub duration: Duration,
}

impl TestSuiteResult {
    /// Print summary
    pub fn print_summary(&self) {
        println!("\nTest suite: {}", self.name);
        println!("Passed: {}, Failed: {}", self.passed, self.failed);
        println!("Duration: {:?}", self.duration);

        if self.failed > 0 {
            println!("\nFailed tests:");
            for outcome in &self.outcomes {
                if !outcome.passed() {
                    println!("  - {}: {:?}", outcome.name, outcome.result);
                }
            }
        }
    }

    /// Check if all tests passed
    pub fn success(&self) -> bool {
        self.failed == 0
    }
}

/// Assertion helper
pub fn assert_eq<T: PartialEq + fmt::Debug>(left: T, right: T, msg: &str) -> TestResult {
    if left == right {
        Ok(())
    } else {
        Err(TestError::AssertionFailed(format!(
            "{}\n  left: {:?}\n  right: {:?}",
            msg, left, right
        )))
    }
}

/// Assert true
pub fn assert_true(condition: bool, msg: &str) -> TestResult {
    if condition {
        Ok(())
    } else {
        Err(TestError::AssertionFailed(msg.to_string()))
    }
}

/// Property-based testing
pub mod property {
    use super::*;

    /// Property test
    pub struct PropertyTest<T> {
        /// Generator
        generator: Box<dyn Fn() -> T>,
        /// Number of iterations
        iterations: usize,
    }

    impl<T> PropertyTest<T> {
        /// Create a new property test
        pub fn new<F>(generator: F) -> Self
        where
            F: Fn() -> T + 'static,
        {
            Self {
                generator: Box::new(generator),
                iterations: 100,
            }
        }

        /// Set number of iterations
        pub fn iterations(mut self, n: usize) -> Self {
            self.iterations = n;
            self
        }

        /// Check property
        pub fn check<F>(self, property: F) -> TestResult
        where
            F: Fn(&T) -> bool,
        {
            for i in 0..self.iterations {
                let value = (self.generator)();
                if !property(&value) {
                    return Err(TestError::AssertionFailed(format!(
                        "Property failed on iteration {}",
                        i
                    )));
                }
            }
            Ok(())
        }
    }
}

/// Golden snapshot testing
pub mod golden {
    use super::*;
    use std::fs;
    use std::path::Path;

    /// Golden snapshot
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Snapshot {
        /// Snapshot name
        pub name: String,
        /// Content
        pub content: String,
    }

    impl Snapshot {
        /// Create a new snapshot
        pub fn new(name: impl Into<String>, content: impl Into<String>) -> Self {
            Self {
                name: name.into(),
                content: content.into(),
            }
        }

        /// Save to file
        pub fn save(&self, dir: impl AsRef<Path>) -> std::io::Result<()> {
            let path = dir.as_ref().join(format!("{}.snap", self.name));
            fs::write(path, &self.content)
        }

        /// Load from file
        pub fn load(name: &str, dir: impl AsRef<Path>) -> std::io::Result<Self> {
            let path = dir.as_ref().join(format!("{}.snap", name));
            let content = fs::read_to_string(path)?;
            Ok(Self {
                name: name.to_string(),
                content,
            })
        }

        /// Compare with expected
        pub fn compare(&self, expected: &str) -> TestResult {
            if self.content == expected {
                Ok(())
            } else {
                Err(TestError::AssertionFailed(format!(
                    "Snapshot mismatch for '{}'\nExpected:\n{}\nActual:\n{}",
                    self.name, expected, self.content
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_test() {
        let test = Test::new("simple", || Ok(()));
        let outcome = test.run();
        assert!(outcome.passed());
    }

    #[test]
    fn test_failing_test() {
        let test = Test::new("failing", || {
            Err(TestError::AssertionFailed("test failed".to_string()))
        });
        let outcome = test.run();
        assert!(!outcome.passed());
    }

    #[test]
    fn test_suite() {
        let mut suite = TestSuite::new("my_suite");
        suite.add_test(Test::new("test1", || Ok(())));
        suite.add_test(Test::new("test2", || Ok(())));

        let result = suite.run();
        assert_eq!(result.passed, 2);
        assert_eq!(result.failed, 0);
    }

    #[test]
    fn test_assert_eq() {
        assert!(assert_eq(1, 1, "should be equal").is_ok());
        assert!(assert_eq(1, 2, "should not be equal").is_err());
    }

    #[test]
    fn test_assert_true() {
        assert!(assert_true(true, "should be true").is_ok());
        assert!(assert_true(false, "should be false").is_err());
    }

    #[test]
    fn test_property_test() {
        let prop = property::PropertyTest::new(|| 42)
            .iterations(10)
            .check(|x| *x == 42);
        assert!(prop.is_ok());
    }

    #[test]
    fn test_snapshot() {
        let snap = golden::Snapshot::new("test", "content");
        assert_eq!(snap.name, "test");
        assert_eq!(snap.content, "content");
    }
}
