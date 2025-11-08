//! Aurora Runtime Library
//!
//! Provides core runtime support for Aurora programs including:
//! - Memory allocation
//! - Panic handling
//! - Program startup
//! - Thread-local storage
//!
//! This library is linked with all Aurora programs to provide essential runtime services.

#![warn(missing_docs)]

pub mod allocator;
pub mod panic;
pub mod start;

// Re-export main functions
pub use allocator::{aurora_alloc, aurora_alloc_zeroed, aurora_free, aurora_realloc};
pub use panic::{
    aurora_panic, aurora_panic_bounds_check, aurora_panic_msg, aurora_panic_unwrap_err,
    aurora_panic_unwrap_none,
};
pub use start::{aurora_atexit, aurora_get_arg, aurora_get_args, aurora_getenv, aurora_start};

/// Runtime version information
pub const RUNTIME_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize Aurora runtime
///
/// This should be called automatically by the compiler-generated startup code.
#[no_mangle]
pub extern "C" fn aurora_runtime_init() {
    // Currently a no-op, but reserved for future initialization
}

/// Shutdown Aurora runtime
///
/// This should be called automatically during program exit.
#[no_mangle]
pub extern "C" fn aurora_runtime_shutdown() {
    // Currently a no-op, but reserved for future cleanup
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_version() {
        assert!(!RUNTIME_VERSION.is_empty());
    }

    #[test]
    fn test_runtime_init() {
        aurora_runtime_init();
        aurora_runtime_shutdown();
    }
}
