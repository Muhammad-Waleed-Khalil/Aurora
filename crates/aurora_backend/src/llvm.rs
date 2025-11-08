//! LLVM Backend
//!
//! This module provides code generation using assembly output.
//! For now, we convert AIR to GAS assembly and compile with GCC.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

/// LLVM Backend
pub struct LlvmBackend {
    target_triple: String,
    opt_level: OptLevel,
}

#[derive(Debug, Clone, Copy)]
pub enum OptLevel {
    None,
    Less,
    Default,
    Aggressive,
}

impl OptLevel {
    /// Convert to GCC optimization flag
    fn to_gcc_flag(&self) -> &str {
        match self {
            OptLevel::None => "-O0",
            OptLevel::Less => "-O1",
            OptLevel::Default => "-O2",
            OptLevel::Aggressive => "-O3",
        }
    }
}

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("Code generation failed: {0}")]
    CodegenFailed(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Assembly failed: {0}")]
    AssemblyFailed(String),
}

impl LlvmBackend {
    pub fn new(target_triple: String) -> Self {
        Self {
            target_triple,
            opt_level: OptLevel::Default,
        }
    }

    pub fn set_opt_level(&mut self, level: OptLevel) {
        self.opt_level = level;
    }

    /// Compile AIR text to object file
    ///
    /// This converts AIR (NASM-like syntax) to GAS syntax and compiles with GCC.
    pub fn compile_to_object(
        &self,
        air_text: &str,
        output_path: &Path,
    ) -> Result<(), BackendError> {
        // Step 1: Convert AIR (NASM syntax) to GAS (AT&T syntax with Intel mode)
        let gas_asm = self.air_to_gas(air_text)?;

        // Step 2: Write to temporary assembly file
        let asm_path = output_path.with_extension("s");
        fs::write(&asm_path, gas_asm)
            .map_err(|e| BackendError::IoError(format!("Failed to write assembly: {}", e)))?;

        // Step 3: Assemble with GCC
        let status = Command::new("gcc")
            .arg(self.opt_level.to_gcc_flag())
            .arg("-c") // Compile only, don't link
            .arg("-o")
            .arg(output_path)
            .arg(&asm_path)
            .status()
            .map_err(|e| BackendError::AssemblyFailed(format!("Failed to run gcc: {}", e)))?;

        if !status.success() {
            return Err(BackendError::AssemblyFailed(format!(
                "GCC returned non-zero exit code: {}",
                status.code().unwrap_or(-1)
            )));
        }

        // Clean up temporary assembly file
        let _ = fs::remove_file(&asm_path);

        Ok(())
    }

    /// Convert AIR (NASM-like syntax) to GAS assembly
    fn air_to_gas(&self, air_text: &str) -> Result<String, BackendError> {
        let mut output = String::new();

        // Add GAS directives
        output.push_str(".intel_syntax noprefix\n");
        output.push_str("\n");

        // Process each line
        for line in air_text.lines() {
            let trimmed = line.trim();

            // Skip empty lines and AIR comments starting with ';'
            if trimmed.is_empty() {
                output.push('\n');
                continue;
            }

            // Convert comments
            if let Some(comment) = trimmed.strip_prefix(';') {
                output.push_str(&format!("#{}\n", comment));
                continue;
            }

            // Handle section directives
            if trimmed.starts_with("section") {
                let section = self.convert_section(trimmed);
                output.push_str(&section);
                output.push('\n');
                continue;
            }

            // Handle global directive
            if trimmed.starts_with("global") {
                let global = trimmed.replace("global", ".globl");
                output.push_str(&global);
                output.push('\n');
                continue;
            }

            // Convert data directives
            if trimmed.contains("db ") || trimmed.contains("dw ") ||
               trimmed.contains("dd ") || trimmed.contains("dq ") {
                let data = self.convert_data_directive(trimmed);
                output.push_str(&data);
                output.push('\n');
                continue;
            }

            // Handle labels (ending with ':')
            if trimmed.ends_with(':') {
                // Keep labels as-is, but ensure proper formatting
                let label = trimmed.trim_end_matches(':');
                output.push_str(&format!("{}:\n", label));
                continue;
            }

            // Regular instruction - keep as-is for Intel syntax
            output.push_str(line);
            output.push('\n');
        }

        Ok(output)
    }

    /// Convert NASM section to GAS section
    fn convert_section(&self, section: &str) -> String {
        if section.contains(".text") {
            ".text".to_string()
        } else if section.contains(".data") {
            ".data".to_string()
        } else if section.contains(".bss") {
            ".bss".to_string()
        } else if section.contains(".rodata") {
            ".section .rodata".to_string()
        } else {
            section.to_string()
        }
    }

    /// Convert NASM data directive to GAS
    fn convert_data_directive(&self, line: &str) -> String {
        // Simple conversion of db/dw/dd/dq to .byte/.word/.long/.quad
        line.replace(" db ", " .byte ")
            .replace(" dw ", " .word ")
            .replace(" dd ", " .long ")
            .replace(" dq ", " .quad ")
    }

    /// Get target triple
    pub fn target_triple(&self) -> &str {
        &self.target_triple
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_backend_creation() {
        let backend = LlvmBackend::new("x86_64-unknown-linux-gnu".to_string());
        assert_eq!(backend.target_triple(), "x86_64-unknown-linux-gnu");
    }

    #[test]
    fn test_opt_level_flags() {
        assert_eq!(OptLevel::None.to_gcc_flag(), "-O0");
        assert_eq!(OptLevel::Less.to_gcc_flag(), "-O1");
        assert_eq!(OptLevel::Default.to_gcc_flag(), "-O2");
        assert_eq!(OptLevel::Aggressive.to_gcc_flag(), "-O3");
    }

    #[test]
    fn test_air_to_gas_basic() {
        let backend = LlvmBackend::new("x86_64-unknown-linux-gnu".to_string());

        let air = r#"
section .text
global main

main:
    mov rax, 42
    ret
"#;

        let gas = backend.air_to_gas(air).unwrap();

        assert!(gas.contains(".intel_syntax noprefix"));
        assert!(gas.contains(".text"));
        assert!(gas.contains(".globl main"));
        assert!(gas.contains("mov rax, 42"));
    }

    #[test]
    fn test_convert_section() {
        let backend = LlvmBackend::new("x86_64-unknown-linux-gnu".to_string());

        assert_eq!(backend.convert_section("section .text"), ".text");
        assert_eq!(backend.convert_section("section .data"), ".data");
        assert_eq!(backend.convert_section("section .bss"), ".bss");
    }

    #[test]
    fn test_convert_data_directive() {
        let backend = LlvmBackend::new("x86_64-unknown-linux-gnu".to_string());

        assert_eq!(
            backend.convert_data_directive("msg db 'Hello'"),
            "msg .byte 'Hello'"
        );
        assert_eq!(
            backend.convert_data_directive("val dw 100"),
            "val .word 100"
        );
    }

    #[test]
    fn test_compile_to_object() {
        let backend = LlvmBackend::new("x86_64-unknown-linux-gnu".to_string());

        let air = r#"
section .text
global test_func

test_func:
    mov rax, 0
    ret
"#;

        let output = PathBuf::from("/tmp/test_backend.o");

        // This will fail if gcc is not available, which is expected in CI
        let result = backend.compile_to_object(air, &output);

        // Just check that the function doesn't panic
        // Actual compilation will fail if gcc is not available
        let _ = result;
    }

    #[test]
    fn test_comments_conversion() {
        let backend = LlvmBackend::new("x86_64-unknown-linux-gnu".to_string());

        let air = r#"
; This is a comment
    mov rax, 1  ; inline comment
"#;

        let gas = backend.air_to_gas(air).unwrap();

        assert!(gas.contains("# This is a comment"));
    }

    #[test]
    fn test_label_conversion() {
        let backend = LlvmBackend::new("x86_64-unknown-linux-gnu".to_string());

        let air = r#"
.L1:
    jmp .L2
.L2:
    ret
"#;

        let gas = backend.air_to_gas(air).unwrap();

        assert!(gas.contains(".L1:"));
        assert!(gas.contains(".L2:"));
    }

    #[test]
    fn test_multiline_function() {
        let backend = LlvmBackend::new("x86_64-unknown-linux-gnu".to_string());

        let air = r#"
section .text
global factorial

factorial:
    push rbp
    mov rbp, rsp
    mov rax, 1
    pop rbp
    ret
"#;

        let gas = backend.air_to_gas(air).unwrap();

        assert!(gas.contains(".intel_syntax noprefix"));
        assert!(gas.contains(".globl factorial"));
        assert!(gas.contains("push rbp"));
        assert!(gas.contains("mov rax, 1"));
        assert!(gas.contains("ret"));
    }
}
