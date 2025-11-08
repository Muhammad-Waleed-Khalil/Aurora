//! C ABI Support for Aurora
//!
//! This module provides stable C FFI bindings with:
//! - Name mangling for Aurora functions/types
//! - Type conversion between Aurora and C
//! - Safety shims for common C pitfalls

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during C ABI operations
#[derive(Debug, Error)]
pub enum CAbiError {
    /// Type cannot be represented in C
    #[error("Type {0} cannot be represented in C ABI")]
    UnsupportedType(String),

    /// Invalid function signature for C
    #[error("Invalid C function signature: {0}")]
    InvalidSignature(String),

    /// Name mangling failed
    #[error("Name mangling failed: {0}")]
    ManglingError(String),
}

/// Result type for C ABI operations
pub type Result<T> = std::result::Result<T, CAbiError>;

/// Represents a C-compatible type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CType {
    /// void
    Void,
    /// char
    Char,
    /// signed char
    SChar,
    /// unsigned char
    UChar,
    /// short
    Short,
    /// unsigned short
    UShort,
    /// int
    Int,
    /// unsigned int
    UInt,
    /// long
    Long,
    /// unsigned long
    ULong,
    /// long long
    LongLong,
    /// unsigned long long
    ULongLong,
    /// float
    Float,
    /// double
    Double,
    /// Pointer to another type
    Pointer(Box<CType>),
    /// Array of fixed size
    Array(Box<CType>, usize),
    /// Struct with fields
    Struct {
        /// Struct name
        name: String,
        /// Field types
        fields: Vec<(String, CType)>,
    },
    /// Function pointer
    FuncPtr {
        /// Return type
        ret: Box<CType>,
        /// Parameter types
        params: Vec<CType>,
    },
}

impl CType {
    /// Get the C declaration string for this type
    pub fn c_decl(&self, name: &str) -> String {
        match self {
            CType::Void => format!("void {}", name),
            CType::Char => format!("char {}", name),
            CType::SChar => format!("signed char {}", name),
            CType::UChar => format!("unsigned char {}", name),
            CType::Short => format!("short {}", name),
            CType::UShort => format!("unsigned short {}", name),
            CType::Int => format!("int {}", name),
            CType::UInt => format!("unsigned int {}", name),
            CType::Long => format!("long {}", name),
            CType::ULong => format!("unsigned long {}", name),
            CType::LongLong => format!("long long {}", name),
            CType::ULongLong => format!("unsigned long long {}", name),
            CType::Float => format!("float {}", name),
            CType::Double => format!("double {}", name),
            CType::Pointer(inner) => {
                let inner_str = inner.c_decl("");
                format!("{}* {}", inner_str.trim(), name)
            }
            CType::Array(inner, size) => {
                let inner_str = inner.c_decl("");
                format!("{} {}[{}]", inner_str.trim(), name, size)
            }
            CType::Struct { name: struct_name, .. } => {
                format!("struct {} {}", struct_name, name)
            }
            CType::FuncPtr { ret, params } => {
                let ret_str = ret.c_decl("").trim().to_string();
                let params_str = params
                    .iter()
                    .map(|p| p.c_decl("").trim().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} (*{})( {})", ret_str, name, params_str)
            }
        }
    }

    /// Get size in bytes (assuming 64-bit platform)
    pub fn size(&self) -> usize {
        match self {
            CType::Void => 0,
            CType::Char | CType::SChar | CType::UChar => 1,
            CType::Short | CType::UShort => 2,
            CType::Int | CType::UInt | CType::Float => 4,
            CType::Long | CType::ULong | CType::Double => 8,
            CType::LongLong | CType::ULongLong => 8,
            CType::Pointer(_) | CType::FuncPtr { .. } => 8, // 64-bit
            CType::Array(inner, count) => inner.size() * count,
            CType::Struct { fields, .. } => {
                fields.iter().map(|(_, t)| t.size()).sum()
            }
        }
    }

    /// Get alignment in bytes
    pub fn alignment(&self) -> usize {
        match self {
            CType::Void => 1,
            CType::Char | CType::SChar | CType::UChar => 1,
            CType::Short | CType::UShort => 2,
            CType::Int | CType::UInt | CType::Float => 4,
            CType::Long | CType::ULong | CType::Double => 8,
            CType::LongLong | CType::ULongLong => 8,
            CType::Pointer(_) | CType::FuncPtr { .. } => 8,
            CType::Array(inner, _) => inner.alignment(),
            CType::Struct { fields, .. } => {
                fields.iter().map(|(_, t)| t.alignment()).max().unwrap_or(1)
            }
        }
    }
}

/// C function signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CFunctionSignature {
    /// Function name
    pub name: String,
    /// Return type
    pub ret_type: CType,
    /// Parameters
    pub params: Vec<(String, CType)>,
    /// Is variadic (uses ...)
    pub is_variadic: bool,
}

impl CFunctionSignature {
    /// Generate C declaration
    pub fn c_declaration(&self) -> String {
        let params_str = if self.params.is_empty() {
            "void".to_string()
        } else {
            let mut params = self
                .params
                .iter()
                .map(|(name, ty)| ty.c_decl(name))
                .collect::<Vec<_>>();

            if self.is_variadic {
                params.push("...".to_string());
            }

            params.join(", ")
        };

        let ret_str = self.ret_type.c_decl("").trim().to_string();
        format!("{} {}({})", ret_str, self.name, params_str)
    }
}

/// Name mangling for Aurora symbols
#[derive(Clone)]
pub struct NameMangler {
    /// Namespace separator
    sep: &'static str,
}

impl NameMangler {
    /// Create a new name mangler
    pub fn new() -> Self {
        Self { sep: "::" }
    }

    /// Mangle an Aurora symbol name for C
    ///
    /// Format: aurora_<module>_<name>
    /// Special characters replaced with underscores
    pub fn mangle(&self, module_path: &[String], name: &str) -> String {
        let module_str = module_path.join("_");
        let safe_name = self.sanitize(name);

        if module_str.is_empty() {
            format!("aurora_{}", safe_name)
        } else {
            format!("aurora_{}_{}", module_str, safe_name)
        }
    }

    /// Sanitize a name for C (replace unsafe characters)
    fn sanitize(&self, name: &str) -> String {
        name.chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c
                } else {
                    '_'
                }
            })
            .collect()
    }

    /// Demangle a C symbol back to Aurora name
    pub fn demangle(&self, mangled: &str) -> Option<(Vec<String>, String)> {
        if !mangled.starts_with("aurora_") {
            return None;
        }

        let parts: Vec<&str> = mangled.strip_prefix("aurora_")?.split('_').collect();
        if parts.is_empty() {
            return None;
        }

        let name = parts.last()?.to_string();
        let module_path = parts[..parts.len() - 1]
            .iter()
            .map(|s| s.to_string())
            .collect();

        Some((module_path, name))
    }
}

impl Default for NameMangler {
    fn default() -> Self {
        Self::new()
    }
}

/// Safety shims for common C pitfalls
#[derive(Clone)]
pub struct SafetyShims {
    /// Checked pointer operations
    null_checks: bool,
    /// Bounds checking for arrays
    bounds_checks: bool,
}

impl SafetyShims {
    /// Create new safety shims with default settings
    pub fn new() -> Self {
        Self {
            null_checks: true,
            bounds_checks: true,
        }
    }

    /// Enable/disable null pointer checks
    pub fn with_null_checks(mut self, enabled: bool) -> Self {
        self.null_checks = enabled;
        self
    }

    /// Enable/disable array bounds checks
    pub fn with_bounds_checks(mut self, enabled: bool) -> Self {
        self.bounds_checks = enabled;
        self
    }

    /// Generate a safe wrapper for pointer dereference
    pub fn wrap_pointer_deref(&self, ptr_name: &str, ty: &CType) -> String {
        if self.null_checks {
            format!(
                "if ({} == NULL) {{ return NULL; }}\n{}",
                ptr_name,
                ty.c_decl(&format!("*{}", ptr_name))
            )
        } else {
            ty.c_decl(&format!("*{}", ptr_name))
        }
    }

    /// Generate safe array access
    pub fn wrap_array_access(
        &self,
        array_name: &str,
        index: &str,
        size: usize,
    ) -> String {
        if self.bounds_checks {
            format!(
                "if ({} >= {}) {{ return NULL; }}\n{}[{}]",
                index, size, array_name, index
            )
        } else {
            format!("{}[{}]", array_name, index)
        }
    }
}

impl Default for SafetyShims {
    fn default() -> Self {
        Self::new()
    }
}

/// C ABI context managing FFI operations
#[derive(Clone)]
pub struct CAbiContext {
    /// Name mangler
    pub mangler: NameMangler,
    /// Safety shims
    pub shims: SafetyShims,
    /// Registered functions
    functions: HashMap<String, CFunctionSignature>,
    /// Registered types
    types: HashMap<String, CType>,
}

impl CAbiContext {
    /// Create a new C ABI context
    pub fn new() -> Self {
        Self {
            mangler: NameMangler::new(),
            shims: SafetyShims::new(),
            functions: HashMap::new(),
            types: HashMap::new(),
        }
    }

    /// Register a function for FFI
    pub fn register_function(&mut self, sig: CFunctionSignature) {
        self.functions.insert(sig.name.clone(), sig);
    }

    /// Register a type for FFI
    pub fn register_type(&mut self, name: String, ty: CType) {
        self.types.insert(name, ty);
    }

    /// Get a registered function
    pub fn get_function(&self, name: &str) -> Option<&CFunctionSignature> {
        self.functions.get(name)
    }

    /// Get a registered type
    pub fn get_type(&self, name: &str) -> Option<&CType> {
        self.types.get(name)
    }

    /// List all registered functions
    pub fn functions(&self) -> Vec<&CFunctionSignature> {
        self.functions.values().collect()
    }

    /// List all registered types
    pub fn types(&self) -> Vec<(&String, &CType)> {
        self.types.iter().collect()
    }
}

impl Default for CAbiContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_type_decl() {
        assert_eq!(CType::Int.c_decl("x"), "int x");
        assert_eq!(CType::Pointer(Box::new(CType::Char)).c_decl("ptr"), "char* ptr");
        assert_eq!(CType::Array(Box::new(CType::Int), 10).c_decl("arr"), "int arr[10]");
    }

    #[test]
    fn test_c_type_size() {
        assert_eq!(CType::Char.size(), 1);
        assert_eq!(CType::Int.size(), 4);
        assert_eq!(CType::Long.size(), 8);
        assert_eq!(CType::Pointer(Box::new(CType::Void)).size(), 8);
        assert_eq!(CType::Array(Box::new(CType::Int), 5).size(), 20);
    }

    #[test]
    fn test_name_mangling() {
        let mangler = NameMangler::new();

        assert_eq!(mangler.mangle(&[], "foo"), "aurora_foo");
        assert_eq!(mangler.mangle(&["std".to_string()], "print"), "aurora_std_print");
        assert_eq!(
            mangler.mangle(&["std".to_string(), "io".to_string()], "read"),
            "aurora_std_io_read"
        );
    }

    #[test]
    fn test_name_demangling() {
        let mangler = NameMangler::new();

        assert_eq!(
            mangler.demangle("aurora_foo"),
            Some((vec![], "foo".to_string()))
        );
        assert_eq!(
            mangler.demangle("aurora_std_print"),
            Some((vec!["std".to_string()], "print".to_string()))
        );
        assert_eq!(
            mangler.demangle("aurora_std_io_read"),
            Some((vec!["std".to_string(), "io".to_string()], "read".to_string()))
        );
    }

    #[test]
    fn test_function_signature() {
        let sig = CFunctionSignature {
            name: "add".to_string(),
            ret_type: CType::Int,
            params: vec![
                ("a".to_string(), CType::Int),
                ("b".to_string(), CType::Int),
            ],
            is_variadic: false,
        };

        assert_eq!(sig.c_declaration(), "int add(int a, int b)");
    }

    #[test]
    fn test_variadic_function() {
        let sig = CFunctionSignature {
            name: "printf".to_string(),
            ret_type: CType::Int,
            params: vec![("fmt".to_string(), CType::Pointer(Box::new(CType::Char)))],
            is_variadic: true,
        };

        assert_eq!(sig.c_declaration(), "int printf(char* fmt, ...)");
    }

    #[test]
    fn test_safety_shims() {
        let shims = SafetyShims::new();

        let deref = shims.wrap_pointer_deref("ptr", &CType::Int);
        assert!(deref.contains("NULL"));

        let access = shims.wrap_array_access("arr", "i", 10);
        assert!(access.contains("10"));
    }

    #[test]
    fn test_abi_context() {
        let mut ctx = CAbiContext::new();

        let sig = CFunctionSignature {
            name: "test_func".to_string(),
            ret_type: CType::Void,
            params: vec![],
            is_variadic: false,
        };

        ctx.register_function(sig.clone());
        assert_eq!(ctx.get_function("test_func").unwrap().name, "test_func");
        assert_eq!(ctx.functions().len(), 1);
    }

    #[test]
    fn test_struct_type() {
        let point = CType::Struct {
            name: "Point".to_string(),
            fields: vec![
                ("x".to_string(), CType::Int),
                ("y".to_string(), CType::Int),
            ],
        };

        assert_eq!(point.size(), 8);
        assert_eq!(point.alignment(), 4);
        assert_eq!(point.c_decl("p"), "struct Point p");
    }

    #[test]
    fn test_function_pointer() {
        let fn_ptr = CType::FuncPtr {
            ret: Box::new(CType::Int),
            params: vec![CType::Int, CType::Int],
        };

        let decl = fn_ptr.c_decl("callback");
        assert!(decl.contains("callback"));
        assert!(decl.contains("int"));
    }
}
