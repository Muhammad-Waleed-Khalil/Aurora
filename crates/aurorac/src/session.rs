//! Compilation Session
//!
//! Manages state and coordination across all compilation phases.
//! The session is the central orchestrator that ensures proper sequencing
//! and data flow between compiler agents.

use anyhow::{Context, Result};
use aurora_diagnostics::DiagnosticCollector;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Compilation options and configuration
#[derive(Debug, Clone)]
pub struct CompilationOptions {
    /// Input source file
    pub input: PathBuf,

    /// Output file (if not specified, derived from input)
    pub output: Option<PathBuf>,

    /// Optimization level (0-3)
    pub opt_level: u8,

    /// Emit MIR dump for debugging
    pub emit_mir: bool,

    /// Emit AIR dump for debugging
    pub emit_air: bool,

    /// Emit AST dump for debugging
    pub emit_ast: bool,

    /// Emit LLVM IR
    pub emit_llvm: bool,

    /// Stop after type checking
    pub type_check_only: bool,

    /// Verbose output
    pub verbose: bool,

    /// Number of parallel codegen units
    pub codegen_units: usize,

    /// Enable debug information
    pub debug_info: bool,
}

impl CompilationOptions {
    /// Create options for a simple compile
    pub fn new(input: impl Into<PathBuf>) -> Self {
        Self {
            input: input.into(),
            output: None,
            opt_level: 0,
            emit_mir: false,
            emit_air: false,
            emit_ast: false,
            emit_llvm: false,
            type_check_only: false,
            verbose: false,
            codegen_units: 1,
            debug_info: true,
        }
    }

    /// Get the output path, deriving from input if not specified
    pub fn output_path(&self) -> PathBuf {
        if let Some(ref output) = self.output {
            output.clone()
        } else {
            self.input.with_extension("out")
        }
    }

    /// Get the MIR dump path
    pub fn mir_dump_path(&self) -> PathBuf {
        self.input.with_extension("mir")
    }

    /// Get the AIR dump path
    pub fn air_dump_path(&self) -> PathBuf {
        self.input.with_extension("air")
    }

    /// Get the AST dump path
    pub fn ast_dump_path(&self) -> PathBuf {
        self.input.with_extension("ast")
    }

    /// Get the LLVM IR dump path
    pub fn llvm_dump_path(&self) -> PathBuf {
        self.input.with_extension("ll")
    }
}

/// Compilation session state
///
/// This is the central coordinator for a single compilation unit.
/// It manages:
/// - Source code and file paths
/// - Diagnostic collection
/// - Intermediate representations at each stage
/// - Compilation options
pub struct CompilationSession {
    /// Compilation options
    pub options: CompilationOptions,

    /// Diagnostic collector (errors, warnings, notes)
    pub diagnostics: Arc<DiagnosticCollector>,

    /// Source code content
    pub source: String,

    /// Source file path
    pub source_path: PathBuf,
}

impl CompilationSession {
    /// Create a new compilation session
    pub fn new(options: CompilationOptions) -> Result<Self> {
        // Read source file
        let source = std::fs::read_to_string(&options.input)
            .with_context(|| format!("Failed to read source file: {}", options.input.display()))?;

        Ok(Self {
            source_path: options.input.clone(),
            options,
            diagnostics: Arc::new(DiagnosticCollector::new()),
            source,
        })
    }

    /// Get the source file name
    pub fn source_name(&self) -> &str {
        self.source_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("<unknown>")
    }

    /// Check if any errors have been reported
    pub fn has_errors(&self) -> bool {
        self.diagnostics.has_errors()
    }

    /// Get error count
    pub fn error_count(&self) -> usize {
        self.diagnostics.error_count()
    }

    /// Get warning count
    pub fn warning_count(&self) -> usize {
        self.diagnostics.warning_count()
    }

    /// Print all diagnostics to stderr
    pub fn emit_diagnostics(&self) {
        self.diagnostics.emit(&self.source, &self.source_path);
    }

    /// Return error if any errors occurred
    pub fn check_errors(&self) -> Result<()> {
        if self.has_errors() {
            anyhow::bail!("Compilation failed with {} error(s)", self.error_count());
        }
        Ok(())
    }
}

/// Result of a compilation phase
#[derive(Debug)]
pub enum PhaseResult<T> {
    /// Phase succeeded with result
    Success(T),

    /// Phase had warnings but succeeded
    SuccessWithWarnings(T),

    /// Phase failed with errors
    Failed,
}

impl<T> PhaseResult<T> {
    /// Check if the phase succeeded
    pub fn is_success(&self) -> bool {
        matches!(self, PhaseResult::Success(_) | PhaseResult::SuccessWithWarnings(_))
    }

    /// Unwrap the result or panic
    pub fn unwrap(self) -> T {
        match self {
            PhaseResult::Success(t) | PhaseResult::SuccessWithWarnings(t) => t,
            PhaseResult::Failed => panic!("Tried to unwrap a failed phase result"),
        }
    }

    /// Convert to Option
    pub fn ok(self) -> Option<T> {
        match self {
            PhaseResult::Success(t) | PhaseResult::SuccessWithWarnings(t) => Some(t),
            PhaseResult::Failed => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_compilation_options_new() {
        let opts = CompilationOptions::new("test.ax");
        assert_eq!(opts.input, PathBuf::from("test.ax"));
        assert_eq!(opts.opt_level, 0);
        assert!(!opts.emit_mir);
        assert!(opts.debug_info);
    }

    #[test]
    fn test_output_path_derivation() {
        let opts = CompilationOptions::new("test.ax");
        assert_eq!(opts.output_path(), PathBuf::from("test.out"));

        let opts_with_output = CompilationOptions {
            output: Some(PathBuf::from("custom.exe")),
            ..opts
        };
        assert_eq!(opts_with_output.output_path(), PathBuf::from("custom.exe"));
    }

    #[test]
    fn test_dump_paths() {
        let opts = CompilationOptions::new("test.ax");
        assert_eq!(opts.mir_dump_path(), PathBuf::from("test.mir"));
        assert_eq!(opts.air_dump_path(), PathBuf::from("test.air"));
        assert_eq!(opts.ast_dump_path(), PathBuf::from("test.ast"));
        assert_eq!(opts.llvm_dump_path(), PathBuf::from("test.ll"));
    }

    #[test]
    fn test_session_creation() -> Result<()> {
        // Create a temporary file
        let mut temp_file = NamedTempFile::new()?;
        std::io::Write::write_all(&mut temp_file, b"fn main() {}")?;
        let temp_path = temp_file.path().to_path_buf();

        let opts = CompilationOptions::new(&temp_path);
        let session = CompilationSession::new(opts)?;

        assert_eq!(session.source, "fn main() {}");
        assert!(!session.has_errors());
        assert_eq!(session.error_count(), 0);

        Ok(())
    }

    #[test]
    fn test_phase_result() {
        let success: PhaseResult<i32> = PhaseResult::Success(42);
        assert!(success.is_success());
        assert_eq!(success.unwrap(), 42);

        let failed: PhaseResult<i32> = PhaseResult::Failed;
        assert!(!failed.is_success());
        assert_eq!(failed.ok(), None);
    }
}
