//! Aurora Compiler Driver Library
//!
//! Core orchestration logic for the Aurora compiler.
//! This crate coordinates all compiler agents while enforcing boundaries.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod diagnostics;
pub mod driver;

/// Initialize the Aurora compiler
pub fn initialize_compiler() {
    tracing_subscriber::fmt::init();
    tracing::info!("Aurora compiler initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize() {
        initialize_compiler();
    }
}
