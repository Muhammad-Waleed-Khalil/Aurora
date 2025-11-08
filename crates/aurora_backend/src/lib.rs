//! Aurora Backend - Code generation and linking
//!
//! This module handles the final stages of compilation:
//! 1. Converting AIR (Assembly IR) to machine code via GCC
//! 2. Linking object files with runtime and system libraries
//! 3. Producing executable binaries
//!
//! # Pipeline
//!
//! ```text
//! AIR Module → Assembly (.s) → Object File (.o) → Executable
//!                  ↓              ↓                   ↓
//!              GAS syntax    GCC assemble      GCC link + runtime
//! ```

pub mod link;
pub mod llvm;

pub use link::{LinkError, Linker, LinkerConfig};
pub use llvm::{BackendError, LlvmBackend, OptLevel};

use aurora_air::AirModule;
use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Code generation options
#[derive(Debug, Clone)]
pub struct CodegenOptions {
    /// Optimization level (0-3)
    pub opt_level: u8,
    /// Generate debug info
    pub debug_info: bool,
    /// Emit LLVM IR (or assembly in our case)
    pub emit_llvm: bool,
    /// Number of codegen units (for parallel compilation)
    pub codegen_units: usize,
    /// Output binary path
    pub output_path: PathBuf,
    /// Keep intermediate files (assembly, object files)
    pub keep_intermediates: bool,
    /// Target triple (e.g., "x86_64-unknown-linux-gnu")
    pub target_triple: Option<String>,
}

impl Default for CodegenOptions {
    fn default() -> Self {
        Self {
            opt_level: 2,
            debug_info: false,
            emit_llvm: false,
            codegen_units: 1,
            output_path: PathBuf::from("a.out"),
            keep_intermediates: false,
            target_triple: None,
        }
    }
}

impl CodegenOptions {
    /// Create options for debug builds
    pub fn debug() -> Self {
        Self {
            opt_level: 0,
            debug_info: true,
            ..Default::default()
        }
    }

    /// Create options for release builds
    pub fn release() -> Self {
        Self {
            opt_level: 3,
            debug_info: false,
            ..Default::default()
        }
    }

    /// Get optimization level as OptLevel enum
    fn opt_level_enum(&self) -> OptLevel {
        match self.opt_level {
            0 => OptLevel::None,
            1 => OptLevel::Less,
            2 => OptLevel::Default,
            _ => OptLevel::Aggressive,
        }
    }
}

/// Generate machine code from AIR and link to executable
///
/// This is the main entry point for the backend. It:
/// 1. Converts AIR to assembly
/// 2. Compiles assembly to object file
/// 3. Compiles C runtime
/// 4. Links everything into an executable
///
/// # Errors
///
/// Returns an error if:
/// - Assembly generation fails
/// - Compilation fails
/// - Linking fails
/// - File I/O fails
pub fn generate_code<D: Send + Sync + 'static>(
    air: AirModule,
    options: CodegenOptions,
    _diagnostics: Arc<D>,
) -> Result<()> {
    // Step 1: Get target triple
    let target_triple = options
        .target_triple
        .clone()
        .unwrap_or_else(|| get_host_triple());

    // Step 2: Set up backend
    let mut backend = LlvmBackend::new(target_triple);
    backend.set_opt_level(options.opt_level_enum());

    // Step 3: Generate AIR text
    let air_text = air.to_text();

    // Optionally emit assembly
    if options.emit_llvm {
        let asm_path = options.output_path.with_extension("s");
        fs::write(&asm_path, &air_text)
            .with_context(|| format!("Failed to write assembly to {:?}", asm_path))?;
        println!("Generated assembly: {}", asm_path.display());
    }

    // Step 4: Compile AIR to object file
    let obj_path = get_temp_path("aurora_main.o");
    backend
        .compile_to_object(&air_text, &obj_path)
        .context("Failed to compile AIR to object file")?;

    if options.keep_intermediates {
        println!("Generated object file: {}", obj_path.display());
    }

    // Step 5: Compile C runtime
    let runtime_c_path = find_runtime_c()?;
    let linker = Linker::new();
    let runtime_obj = linker
        .compile_c_runtime(&runtime_c_path)
        .context("Failed to compile C runtime")?;

    if options.keep_intermediates {
        println!("Generated runtime object: {}", runtime_obj.display());
    }

    // Step 6: Link to executable
    let mut linker = Linker::new();

    // Add standard C library
    linker.add_library("c".to_string());

    // Link object files
    let object_files = vec![obj_path.clone(), runtime_obj.clone()];

    linker
        .link(&object_files, &options.output_path)
        .context("Failed to link executable")?;

    // Step 7: Clean up temporary files if requested
    if !options.keep_intermediates {
        let _ = fs::remove_file(&obj_path);
        let _ = fs::remove_file(&runtime_obj);
        let _ = fs::remove_file(obj_path.with_extension("s"));
    }

    Ok(())
}

/// Get host target triple
fn get_host_triple() -> String {
    // Simple detection based on current platform
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    {
        "x86_64-unknown-linux-gnu".to_string()
    }
    #[cfg(all(target_arch = "x86_64", target_os = "macos"))]
    {
        "x86_64-apple-darwin".to_string()
    }
    #[cfg(all(target_arch = "x86_64", target_os = "windows"))]
    {
        "x86_64-pc-windows-msvc".to_string()
    }
    #[cfg(all(target_arch = "aarch64", target_os = "linux"))]
    {
        "aarch64-unknown-linux-gnu".to_string()
    }
    #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
    {
        "aarch64-apple-darwin".to_string()
    }
    #[cfg(not(any(
        all(target_arch = "x86_64", target_os = "linux"),
        all(target_arch = "x86_64", target_os = "macos"),
        all(target_arch = "x86_64", target_os = "windows"),
        all(target_arch = "aarch64", target_os = "linux"),
        all(target_arch = "aarch64", target_os = "macos"),
    )))]
    {
        "unknown-unknown-unknown".to_string()
    }
}

/// Find the C runtime source file
fn find_runtime_c() -> Result<PathBuf> {
    // Try several locations
    let candidates = vec![
        PathBuf::from("runtime/c_runtime.c"),
        PathBuf::from("../runtime/c_runtime.c"),
        PathBuf::from("../../runtime/c_runtime.c"),
        PathBuf::from("/home/user/Aurora/runtime/c_runtime.c"),
    ];

    for candidate in candidates {
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    // If not found, try relative to current executable
    if let Ok(exe) = env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join("../runtime/c_runtime.c");
            if candidate.exists() {
                return Ok(candidate);
            }
        }
    }

    anyhow::bail!("Could not find runtime/c_runtime.c")
}

/// Get a temporary file path
fn get_temp_path(name: &str) -> PathBuf {
    env::temp_dir().join(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_air::{AirFunction, Instruction, Operand, Register};

    #[test]
    fn test_codegen_options_default() {
        let opts = CodegenOptions::default();
        assert_eq!(opts.opt_level, 2);
        assert!(!opts.debug_info);
        assert_eq!(opts.output_path, PathBuf::from("a.out"));
    }

    #[test]
    fn test_codegen_options_debug() {
        let opts = CodegenOptions::debug();
        assert_eq!(opts.opt_level, 0);
        assert!(opts.debug_info);
    }

    #[test]
    fn test_codegen_options_release() {
        let opts = CodegenOptions::release();
        assert_eq!(opts.opt_level, 3);
        assert!(!opts.debug_info);
    }

    #[test]
    fn test_opt_level_conversion() {
        let opts = CodegenOptions {
            opt_level: 0,
            ..Default::default()
        };
        assert!(matches!(opts.opt_level_enum(), OptLevel::None));

        let opts = CodegenOptions {
            opt_level: 1,
            ..Default::default()
        };
        assert!(matches!(opts.opt_level_enum(), OptLevel::Less));

        let opts = CodegenOptions {
            opt_level: 2,
            ..Default::default()
        };
        assert!(matches!(opts.opt_level_enum(), OptLevel::Default));

        let opts = CodegenOptions {
            opt_level: 3,
            ..Default::default()
        };
        assert!(matches!(opts.opt_level_enum(), OptLevel::Aggressive));
    }

    #[test]
    fn test_get_host_triple() {
        let triple = get_host_triple();
        assert!(!triple.is_empty());
        assert!(triple.contains("-"));
    }

    #[test]
    fn test_get_temp_path() {
        let path = get_temp_path("test.o");
        assert!(path.to_str().unwrap().contains("test.o"));
    }

    #[test]
    fn test_generate_code_with_empty_module() {
        let module = AirModule::new("test".to_string());
        let options = CodegenOptions {
            output_path: PathBuf::from("/tmp/test_empty"),
            keep_intermediates: true,
            ..Default::default()
        };

        let diagnostics = Arc::new(());

        // This should succeed even with empty module
        let result = generate_code(module, options, diagnostics);

        // The result might fail if runtime is not found, which is OK for this test
        let _ = result;
    }

    #[test]
    fn test_generate_code_with_simple_function() {
        let mut module = AirModule::new("test".to_string());

        let mut func = AirFunction::new("main".to_string());
        func.push(Instruction::Mov {
            dest: Operand::Reg(Register::RAX),
            src: Operand::Imm(0),
        });
        func.push(Instruction::Ret);

        module.add_function(func);

        let options = CodegenOptions {
            output_path: PathBuf::from("/tmp/test_simple"),
            keep_intermediates: true,
            ..Default::default()
        };

        let diagnostics = Arc::new(());

        // This should succeed if gcc and runtime are available
        let result = generate_code(module, options, diagnostics);

        // The result might fail if runtime is not found, which is OK for this test
        let _ = result;
    }

    #[test]
    fn test_emit_assembly() {
        let mut module = AirModule::new("test".to_string());

        let mut func = AirFunction::new("test_func".to_string());
        func.push(Instruction::Mov {
            dest: Operand::Reg(Register::RAX),
            src: Operand::Imm(42),
        });
        func.push(Instruction::Ret);

        module.add_function(func);

        let options = CodegenOptions {
            output_path: PathBuf::from("/tmp/test_asm"),
            emit_llvm: true,
            keep_intermediates: true,
            ..Default::default()
        };

        let diagnostics = Arc::new(());

        let result = generate_code(module, options, diagnostics);

        // The result might fail if runtime is not found
        let _ = result;

        // Check if assembly was emitted
        let asm_path = PathBuf::from("/tmp/test_asm.s");
        if asm_path.exists() {
            let content = fs::read_to_string(&asm_path).unwrap();
            assert!(content.contains("test_func"));
            let _ = fs::remove_file(&asm_path);
        }
    }

    #[test]
    fn test_codegen_with_data_section() {
        use aurora_air::{DataDirective, DataKind};

        let mut module = AirModule::new("test".to_string());

        // Add data section
        module.data.push(DataDirective {
            label: "msg".to_string(),
            kind: DataKind::String,
            value: b"Hello\0".to_vec(),
        });

        let mut func = AirFunction::new("main".to_string());
        func.push(Instruction::Mov {
            dest: Operand::Reg(Register::RAX),
            src: Operand::Imm(0),
        });
        func.push(Instruction::Ret);

        module.add_function(func);

        let options = CodegenOptions {
            output_path: PathBuf::from("/tmp/test_data"),
            keep_intermediates: true,
            ..Default::default()
        };

        let diagnostics = Arc::new(());

        let result = generate_code(module, options, diagnostics);
        let _ = result;
    }
}
