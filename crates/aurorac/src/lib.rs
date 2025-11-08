//! Aurora Compiler Driver Library
//!
//! Core orchestration logic for the Aurora compiler.
//! This crate coordinates all compiler agents while enforcing boundaries.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod diagnostics;
pub mod driver;
pub mod session;
pub mod pipeline;

pub use session::{CompilationOptions, CompilationSession, PhaseResult};
pub use pipeline::Pipeline;

/// Initialize the Aurora compiler
pub fn initialize_compiler() {
    tracing_subscriber::fmt::init();
    tracing::info!("Aurora compiler initialized");
}

/// Compile a single source file
///
/// This is the main entry point for compiling Aurora programs.
pub fn compile_file(options: CompilationOptions) -> anyhow::Result<()> {
    let mut session = CompilationSession::new(options)?;
    let mut pipeline = Pipeline::new(&mut session);

    pipeline.compile()?;

    // Emit diagnostics even on success (for warnings)
    if session.warning_count() > 0 {
        session.emit_diagnostics();
    }

    Ok(())
}

/// Check syntax of a source file without full compilation
pub fn check_file(options: CompilationOptions) -> anyhow::Result<()> {
    let mut session = CompilationSession::new(options)?;
    pipeline::check_syntax(&mut session)?;

    if session.warning_count() > 0 {
        session.emit_diagnostics();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize() {
        initialize_compiler();
    }

    #[test]
    fn test_compilation_options() {
        let opts = CompilationOptions::new("test.ax");
        assert_eq!(opts.input.to_str().unwrap(), "test.ax");
        assert_eq!(opts.opt_level, 0);
    }
}
