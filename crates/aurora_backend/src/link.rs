//! Linker - Link object files to executables
//!
//! This module handles linking object files with runtime libraries and system libraries
//! to produce final executables.

use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LinkError {
    #[error("Linker not found: {0}")]
    LinkerNotFound(String),

    #[error("Link failed: {0}")]
    LinkFailed(String),

    #[error("IO error: {0}")]
    IoError(String),
}

/// Platform-specific linker configuration
#[derive(Debug, Clone)]
pub struct LinkerConfig {
    /// Linker executable (gcc, ld, lld, etc.)
    pub linker: String,
    /// Additional linker arguments
    pub args: Vec<String>,
    /// Library search paths
    pub library_paths: Vec<PathBuf>,
    /// Libraries to link
    pub libraries: Vec<String>,
    /// Static libraries (.a files)
    pub static_libs: Vec<PathBuf>,
}

impl Default for LinkerConfig {
    fn default() -> Self {
        Self {
            linker: "gcc".to_string(),
            args: Vec::new(),
            library_paths: Vec::new(),
            libraries: Vec::new(),
            static_libs: Vec::new(),
        }
    }
}

/// Linker
pub struct Linker {
    config: LinkerConfig,
}

impl Linker {
    pub fn new() -> Self {
        Self {
            config: LinkerConfig::default(),
        }
    }

    /// Create linker with custom configuration
    pub fn with_config(config: LinkerConfig) -> Self {
        Self { config }
    }

    /// Set linker executable
    pub fn set_linker(&mut self, linker: String) {
        self.config.linker = linker;
    }

    /// Add link argument
    pub fn add_arg(&mut self, arg: String) {
        self.config.args.push(arg);
    }

    /// Add library search path
    pub fn add_library_path(&mut self, path: PathBuf) {
        self.config.library_paths.push(path);
    }

    /// Add library to link
    pub fn add_library(&mut self, lib: String) {
        self.config.libraries.push(lib);
    }

    /// Add static library
    pub fn add_static_library(&mut self, path: PathBuf) {
        self.config.static_libs.push(path);
    }

    /// Link object files to executable
    ///
    /// This uses GCC as the linker driver, which handles:
    /// - Invoking the system linker (ld)
    /// - Linking with C runtime (crt0.o, etc.)
    /// - Linking with system libraries (libc, etc.)
    pub fn link(
        &self,
        object_files: &[PathBuf],
        output: &Path,
    ) -> Result<(), LinkError> {
        // Use GCC as linker driver for simplicity
        let mut cmd = Command::new(&self.config.linker);

        // Output path
        cmd.arg("-o").arg(output);

        // Object files
        for obj in object_files {
            cmd.arg(obj);
        }

        // Static libraries
        for lib in &self.config.static_libs {
            cmd.arg(lib);
        }

        // Library search paths
        for path in &self.config.library_paths {
            cmd.arg(format!("-L{}", path.display()));
        }

        // Libraries
        for lib in &self.config.libraries {
            cmd.arg(format!("-l{}", lib));
        }

        // Additional arguments
        for arg in &self.config.args {
            cmd.arg(arg);
        }

        // Run linker
        let output = cmd.output().map_err(|e| {
            LinkError::LinkerNotFound(format!("Failed to run {}: {}", self.config.linker, e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(LinkError::LinkFailed(format!(
                "Linker failed with exit code {}:\n{}",
                output.status.code().unwrap_or(-1),
                stderr
            )));
        }

        Ok(())
    }

    /// Link for Windows (PE/COFF)
    pub fn link_pe(
        &self,
        object_files: &[PathBuf],
        output: &Path,
    ) -> Result<(), LinkError> {
        // Windows-specific linking
        let mut cmd = Command::new("gcc");

        cmd.arg("-o").arg(output);

        for obj in object_files {
            cmd.arg(obj);
        }

        // Add Windows-specific libraries
        cmd.arg("-lkernel32")
            .arg("-luser32")
            .arg("-lmsvcrt");

        let output = cmd.output().map_err(|e| {
            LinkError::LinkerNotFound(format!("Failed to run gcc: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(LinkError::LinkFailed(format!(
                "Windows linking failed:\n{}",
                stderr
            )));
        }

        Ok(())
    }

    /// Link for macOS (Mach-O)
    pub fn link_macho(
        &self,
        object_files: &[PathBuf],
        output: &Path,
    ) -> Result<(), LinkError> {
        // macOS-specific linking
        let mut cmd = Command::new("gcc");

        cmd.arg("-o").arg(output);

        for obj in object_files {
            cmd.arg(obj);
        }

        // macOS-specific flags
        cmd.arg("-Wl,-platform_version,macos,11.0,11.0");

        let output = cmd.output().map_err(|e| {
            LinkError::LinkerNotFound(format!("Failed to run gcc: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(LinkError::LinkFailed(format!(
                "macOS linking failed:\n{}",
                stderr
            )));
        }

        Ok(())
    }

    /// Compile C runtime and return object file path
    pub fn compile_c_runtime(&self, runtime_c: &Path) -> Result<PathBuf, LinkError> {
        let obj_path = runtime_c.with_extension("o");

        let status = Command::new("gcc")
            .arg("-c")
            .arg("-O2")
            .arg("-o")
            .arg(&obj_path)
            .arg(runtime_c)
            .status()
            .map_err(|e| LinkError::IoError(format!("Failed to compile C runtime: {}", e)))?;

        if !status.success() {
            return Err(LinkError::LinkFailed(
                "Failed to compile C runtime".to_string(),
            ));
        }

        Ok(obj_path)
    }
}

impl Default for Linker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linker_creation() {
        let linker = Linker::new();
        assert_eq!(linker.config.linker, "gcc");
    }

    #[test]
    fn test_add_library() {
        let mut linker = Linker::new();
        linker.add_library("m".to_string());
        assert_eq!(linker.config.libraries.len(), 1);
    }

    #[test]
    fn test_add_library_path() {
        let mut linker = Linker::new();
        linker.add_library_path(PathBuf::from("/usr/lib"));
        assert_eq!(linker.config.library_paths.len(), 1);
    }

    #[test]
    fn test_add_static_library() {
        let mut linker = Linker::new();
        linker.add_static_library(PathBuf::from("/tmp/libtest.a"));
        assert_eq!(linker.config.static_libs.len(), 1);
    }

    #[test]
    fn test_linker_config_default() {
        let config = LinkerConfig::default();
        assert_eq!(config.linker, "gcc");
        assert!(config.args.is_empty());
        assert!(config.libraries.is_empty());
    }

    #[test]
    fn test_custom_config() {
        let mut config = LinkerConfig::default();
        config.linker = "clang".to_string();
        config.args.push("-v".to_string());

        let linker = Linker::with_config(config);
        assert_eq!(linker.config.linker, "clang");
        assert_eq!(linker.config.args.len(), 1);
    }

    #[test]
    fn test_set_linker() {
        let mut linker = Linker::new();
        linker.set_linker("lld".to_string());
        assert_eq!(linker.config.linker, "lld");
    }

    #[test]
    fn test_add_arg() {
        let mut linker = Linker::new();
        linker.add_arg("-static".to_string());
        assert_eq!(linker.config.args.len(), 1);
        assert_eq!(linker.config.args[0], "-static");
    }
}
