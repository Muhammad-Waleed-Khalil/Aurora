//! Aurora Backend - Code generation and linking

pub mod link;
pub mod llvm;

pub use link::{LinkError, Linker};
pub use llvm::{BackendError, LlvmBackend, OptLevel};

// Pipeline integration stubs
use aurora_air::AirModule;
use std::path::PathBuf;
use std::sync::Arc;
use anyhow::Result;

/// Code generation options
#[derive(Debug, Clone)]
pub struct CodegenOptions {
    /// Optimization level (0-3)
    pub opt_level: u8,
    /// Generate debug info
    pub debug_info: bool,
    /// Emit LLVM IR
    pub emit_llvm: bool,
    /// Number of codegen units (for parallel compilation)
    pub codegen_units: usize,
    /// Output binary path
    pub output_path: PathBuf,
}

/// Generate machine code from AIR
pub fn generate_code<D: Send + Sync + 'static>(
    _air: AirModule,
    _options: CodegenOptions,
    _diagnostics: Arc<D>,
) -> Result<()> {
    // TODO: Implement actual code generation
    // For now, just succeed
    Ok(())
}
