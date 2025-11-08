//! aurora_interop - Foreign Function Interface and Interoperability
//!
//! This crate provides Aurora's FFI capabilities including:
//! - C ABI support with stable name mangling
//! - C header generation
//! - Python HPy bindings
//! - Node.js N-API bindings
//! - WebAssembly/WASI support
//!
//! # Example
//!
//! ```
//! use aurora_interop::c_abi::{CAbiContext, CFunctionSignature, CType};
//! use aurora_interop::header_gen::HeaderGenerator;
//!
//! // Create a C ABI context
//! let mut abi_ctx = CAbiContext::new();
//!
//! // Register a function
//! let sig = CFunctionSignature {
//!     name: "add".to_string(),
//!     ret_type: CType::Int,
//!     params: vec![
//!         ("a".to_string(), CType::Int),
//!         ("b".to_string(), CType::Int),
//!     ],
//!     is_variadic: false,
//! };
//! abi_ctx.register_function(sig);
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

/// C ABI support
pub mod c_abi;

/// C header generation
pub mod header_gen;

// Re-export main types
pub use c_abi::{CAbiContext, CAbiError, CFunctionSignature, CType, NameMangler, SafetyShims};
pub use header_gen::{BatchHeaderGenerator, HeaderGenConfig, HeaderGenerator};

#[cfg(test)]
mod integration_tests {
    use super::*;
    use c_abi::{CAbiContext, CFunctionSignature, CType};
    use header_gen::{HeaderGenConfig, HeaderGenerator};

    #[test]
    fn test_full_workflow() {
        // Create ABI context
        let mut abi_ctx = CAbiContext::new();

        // Mangle a name
        let mangled = abi_ctx.mangler.mangle(&["std".to_string()], "print");
        assert_eq!(mangled, "aurora_std_print");

        // Register a function
        let sig = CFunctionSignature {
            name: mangled.clone(),
            ret_type: CType::Void,
            params: vec![("msg".to_string(), CType::Pointer(Box::new(CType::Char)))],
            is_variadic: false,
        };
        abi_ctx.register_function(sig);

        // Generate header
        let config = HeaderGenConfig::new();
        let gen = HeaderGenerator::new(config, abi_ctx);
        let header = gen.generate("std").unwrap();

        // Verify header contains expected content
        assert!(header.contains(&mangled));
        assert!(header.contains("void"));
        assert!(header.contains("char*"));
    }

    #[test]
    fn test_type_registration() {
        let mut abi_ctx = CAbiContext::new();

        let vec_type = CType::Struct {
            name: "Vec".to_string(),
            fields: vec![
                ("data".to_string(), CType::Pointer(Box::new(CType::Void))),
                ("len".to_string(), CType::ULong),
                ("cap".to_string(), CType::ULong),
            ],
        };

        abi_ctx.register_type("Vec".to_string(), vec_type);
        assert!(abi_ctx.get_type("Vec").is_some());
    }

    #[test]
    fn test_safety_shims() {
        let shims = SafetyShims::new().with_null_checks(true);
        let code = shims.wrap_pointer_deref("ptr", &CType::Int);
        assert!(code.contains("NULL"));
    }
}
