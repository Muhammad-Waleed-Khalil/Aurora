//! Differential Testing Against C
//!
//! Compare Aurora compiler output with reference C implementations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Command, Output};
use thiserror::Error;

/// Differential testing errors
#[derive(Debug, Error)]
pub enum DiffError {
    /// Compilation failed
    #[error("Compilation failed: {0}")]
    CompilationFailed(String),

    /// Execution failed
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    /// Output mismatch
    #[error("Output mismatch:\n  Aurora: {aurora}\n  Reference: {reference}")]
    OutputMismatch { aurora: String, reference: String },

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type
pub type Result<T> = std::result::Result<T, DiffError>;

/// Test case for differential testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifferentialTest {
    /// Test name
    pub name: String,
    /// Aurora source code
    pub aurora_source: String,
    /// C reference source code
    pub c_source: String,
    /// Expected output
    pub expected_output: Option<String>,
}

impl DifferentialTest {
    /// Create a new differential test
    pub fn new(
        name: impl Into<String>,
        aurora_source: impl Into<String>,
        c_source: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            aurora_source: aurora_source.into(),
            c_source: c_source.into(),
            expected_output: None,
        }
    }

    /// Set expected output
    pub fn with_expected_output(mut self, output: impl Into<String>) -> Self {
        self.expected_output = Some(output.into());
        self
    }
}

/// Differential test runner
pub struct DifferentialRunner {
    /// Aurora compiler path
    aurora_compiler: String,
    /// C compiler path (gcc/clang)
    c_compiler: String,
    /// Test results
    results: HashMap<String, DiffResult>,
}

impl DifferentialRunner {
    /// Create a new differential runner
    pub fn new() -> Self {
        Self {
            aurora_compiler: "aurorac".to_string(),
            c_compiler: "gcc".to_string(),
            results: HashMap::new(),
        }
    }

    /// Set Aurora compiler path
    pub fn with_aurora_compiler(mut self, path: impl Into<String>) -> Self {
        self.aurora_compiler = path.into();
        self
    }

    /// Set C compiler path
    pub fn with_c_compiler(mut self, path: impl Into<String>) -> Self {
        self.c_compiler = path.into();
        self
    }

    /// Run a differential test
    pub fn run_test(&mut self, test: &DifferentialTest) -> Result<()> {
        // Compile and run Aurora
        let aurora_output = self.compile_and_run_aurora(&test.aurora_source)?;

        // Compile and run C reference
        let c_output = self.compile_and_run_c(&test.c_source)?;

        // Compare outputs
        let aurora_stdout = String::from_utf8_lossy(&aurora_output.stdout);
        let c_stdout = String::from_utf8_lossy(&c_output.stdout);

        let result = if aurora_stdout == c_stdout {
            DiffResult::Passed {
                output: aurora_stdout.to_string(),
            }
        } else {
            DiffResult::Failed {
                aurora: aurora_stdout.to_string(),
                reference: c_stdout.to_string(),
            }
        };

        self.results.insert(test.name.clone(), result.clone());

        match result {
            DiffResult::Passed { .. } => Ok(()),
            DiffResult::Failed { aurora, reference } => {
                Err(DiffError::OutputMismatch { aurora, reference })
            }
        }
    }

    /// Compile and run Aurora code (simulated)
    fn compile_and_run_aurora(&self, _source: &str) -> Result<Output> {
        // In a real implementation, this would compile and run Aurora code
        // For now, return a simulated output
        Ok(Output {
            status: std::process::ExitStatus::default(),
            stdout: b"42\n".to_vec(),
            stderr: Vec::new(),
        })
    }

    /// Compile and run C code
    fn compile_and_run_c(&self, source: &str) -> Result<Output> {
        // Write C source to temp file
        let temp_dir = std::env::temp_dir();
        let source_path = temp_dir.join("test.c");
        let binary_path = temp_dir.join("test_bin");

        std::fs::write(&source_path, source)?;

        // Compile
        let compile_output = Command::new(&self.c_compiler)
            .args(&[
                source_path.to_str().unwrap(),
                "-o",
                binary_path.to_str().unwrap(),
            ])
            .output()?;

        if !compile_output.status.success() {
            let stderr = String::from_utf8_lossy(&compile_output.stderr);
            return Err(DiffError::CompilationFailed(stderr.to_string()));
        }

        // Run
        let run_output = Command::new(&binary_path).output()?;

        // Cleanup
        let _ = std::fs::remove_file(source_path);
        let _ = std::fs::remove_file(binary_path);

        Ok(run_output)
    }

    /// Get test results
    pub fn results(&self) -> &HashMap<String, DiffResult> {
        &self.results
    }

    /// Print summary
    pub fn print_summary(&self) {
        let passed = self.results.values().filter(|r| r.passed()).count();
        let failed = self.results.len() - passed;

        println!("\nDifferential Testing Summary:");
        println!("Passed: {}, Failed: {}", passed, failed);

        if failed > 0 {
            println!("\nFailed tests:");
            for (name, result) in &self.results {
                if !result.passed() {
                    println!("  - {}: {:?}", name, result);
                }
            }
        }
    }
}

impl Default for DifferentialRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// Differential test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffResult {
    /// Test passed
    Passed { output: String },
    /// Test failed
    Failed { aurora: String, reference: String },
}

impl DiffResult {
    /// Check if test passed
    pub fn passed(&self) -> bool {
        matches!(self, DiffResult::Passed { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_differential_test_creation() {
        let test = DifferentialTest::new(
            "simple",
            "fn main() { println!(\"42\"); }",
            "int main() { printf(\"42\\n\"); return 0; }",
        );

        assert_eq!(test.name, "simple");
        assert!(test.expected_output.is_none());
    }

    #[test]
    fn test_with_expected_output() {
        let test = DifferentialTest::new("test", "aurora", "c")
            .with_expected_output("42\n");

        assert_eq!(test.expected_output, Some("42\n".to_string()));
    }

    #[test]
    fn test_runner_creation() {
        let runner = DifferentialRunner::new();
        assert_eq!(runner.aurora_compiler, "aurorac");
        assert_eq!(runner.c_compiler, "gcc");
    }

    #[test]
    fn test_custom_compilers() {
        let runner = DifferentialRunner::new()
            .with_aurora_compiler("/usr/bin/aurorac")
            .with_c_compiler("clang");

        assert_eq!(runner.aurora_compiler, "/usr/bin/aurorac");
        assert_eq!(runner.c_compiler, "clang");
    }

    #[test]
    fn test_diff_result() {
        let passed = DiffResult::Passed {
            output: "42".to_string(),
        };
        assert!(passed.passed());

        let failed = DiffResult::Failed {
            aurora: "42".to_string(),
            reference: "43".to_string(),
        };
        assert!(!failed.passed());
    }
}
