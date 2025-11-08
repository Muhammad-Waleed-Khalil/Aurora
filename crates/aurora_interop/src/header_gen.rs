//! C Header Generation
//!
//! This module generates C header files (.h) from Aurora declarations
//! for use in C projects that want to link against Aurora libraries.

use crate::c_abi::{CAbiContext, CFunctionSignature, CType};
use serde::{Deserialize, Serialize};
use std::fmt::Write as FmtWrite;
use thiserror::Error;

/// Errors that can occur during header generation
#[derive(Debug, Error)]
pub enum HeaderGenError {
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Formatting error
    #[error("Formatting error: {0}")]
    FmtError(#[from] std::fmt::Error),

    /// Invalid header name
    #[error("Invalid header name: {0}")]
    InvalidName(String),
}

/// Result type for header generation
pub type Result<T> = std::result::Result<T, HeaderGenError>;

/// C header generator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderGenConfig {
    /// Header guard prefix
    pub guard_prefix: String,
    /// Include standard headers
    pub include_stdint: bool,
    /// Include stddef.h
    pub include_stddef: bool,
    /// Add extern "C" for C++
    pub extern_c: bool,
    /// Add comments
    pub add_comments: bool,
}

impl HeaderGenConfig {
    /// Create default configuration
    pub fn new() -> Self {
        Self {
            guard_prefix: "AURORA".to_string(),
            include_stdint: true,
            include_stddef: true,
            extern_c: true,
            add_comments: true,
        }
    }
}

impl Default for HeaderGenConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// C header generator
pub struct HeaderGenerator {
    /// Configuration
    config: HeaderGenConfig,
    /// C ABI context
    abi_ctx: CAbiContext,
}

impl HeaderGenerator {
    /// Create a new header generator
    pub fn new(config: HeaderGenConfig, abi_ctx: CAbiContext) -> Self {
        Self { config, abi_ctx }
    }

    /// Generate a complete header file
    pub fn generate(&self, module_name: &str) -> Result<String> {
        let mut output = String::new();

        // Header guard
        let guard = self.make_guard(module_name)?;
        writeln!(output, "#ifndef {}", guard)?;
        writeln!(output, "#define {}", guard)?;
        writeln!(output)?;

        // Standard includes
        if self.config.include_stdint {
            writeln!(output, "#include <stdint.h>")?;
        }
        if self.config.include_stddef {
            writeln!(output, "#include <stddef.h>")?;
        }
        if self.config.include_stdint || self.config.include_stddef {
            writeln!(output)?;
        }

        // Extern C block start
        if self.config.extern_c {
            writeln!(output, "#ifdef __cplusplus")?;
            writeln!(output, "extern \"C\" {{")?;
            writeln!(output, "#endif")?;
            writeln!(output)?;
        }

        // Type declarations
        if !self.abi_ctx.types().is_empty() {
            if self.config.add_comments {
                writeln!(output, "/* Type Declarations */")?;
            }
            for (name, ty) in self.abi_ctx.types() {
                self.generate_type_decl(&mut output, name, ty)?;
                writeln!(output)?;
            }
            writeln!(output)?;
        }

        // Function declarations
        if !self.abi_ctx.functions().is_empty() {
            if self.config.add_comments {
                writeln!(output, "/* Function Declarations */")?;
            }
            for sig in self.abi_ctx.functions() {
                self.generate_function_decl(&mut output, sig)?;
            }
        }

        // Extern C block end
        if self.config.extern_c {
            writeln!(output)?;
            writeln!(output, "#ifdef __cplusplus")?;
            writeln!(output, "}}")?;
            writeln!(output, "#endif")?;
        }

        // Close header guard
        writeln!(output)?;
        if self.config.add_comments {
            writeln!(output, "#endif /* {} */", guard)?;
        } else {
            writeln!(output, "#endif")?;
        }

        Ok(output)
    }

    /// Make a header guard macro name
    fn make_guard(&self, module_name: &str) -> Result<String> {
        let sanitized = module_name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c.to_ascii_uppercase()
                } else {
                    '_'
                }
            })
            .collect::<String>();

        Ok(format!("{}_{}_H", self.config.guard_prefix, sanitized))
    }

    /// Generate a type declaration
    fn generate_type_decl(
        &self,
        output: &mut String,
        name: &str,
        ty: &CType,
    ) -> Result<()> {
        match ty {
            CType::Struct { name: struct_name, fields } => {
                writeln!(output, "typedef struct {} {{", struct_name)?;
                for (field_name, field_ty) in fields {
                    let decl = field_ty.c_decl(field_name);
                    writeln!(output, "    {};", decl)?;
                }
                writeln!(output, "}} {};", name)?;
            }
            _ => {
                // Simple typedef
                let decl = ty.c_decl(name);
                writeln!(output, "typedef {};", decl)?;
            }
        }
        Ok(())
    }

    /// Generate a function declaration
    fn generate_function_decl(
        &self,
        output: &mut String,
        sig: &CFunctionSignature,
    ) -> Result<()> {
        writeln!(output, "{};", sig.c_declaration())?;
        Ok(())
    }

    /// Generate inline documentation comment
    pub fn generate_doc_comment(&self, doc: &str) -> String {
        let lines: Vec<&str> = doc.lines().collect();
        if lines.is_empty() {
            return String::new();
        }

        let mut result = String::new();
        result.push_str("/**\n");
        for line in lines {
            result.push_str(" * ");
            result.push_str(line);
            result.push('\n');
        }
        result.push_str(" */\n");
        result
    }
}

/// Batch header generation for multiple modules
pub struct BatchHeaderGenerator {
    /// Configuration
    config: HeaderGenConfig,
}

impl BatchHeaderGenerator {
    /// Create a new batch generator
    pub fn new(config: HeaderGenConfig) -> Self {
        Self { config }
    }

    /// Generate headers for multiple modules
    pub fn generate_all(
        &self,
        modules: &[(String, CAbiContext)],
    ) -> Result<Vec<(String, String)>> {
        let mut headers = Vec::new();

        for (module_name, abi_ctx) in modules {
            let gen = HeaderGenerator::new(self.config.clone(), abi_ctx.clone());
            let header_content = gen.generate(module_name)?;
            headers.push((module_name.clone(), header_content));
        }

        Ok(headers)
    }

    /// Generate a master header that includes all module headers
    pub fn generate_master_header(&self, module_names: &[String]) -> Result<String> {
        let mut output = String::new();

        let guard = format!("{}_MASTER_H", self.config.guard_prefix);
        writeln!(output, "#ifndef {}", guard)?;
        writeln!(output, "#define {}", guard)?;
        writeln!(output)?;

        if self.config.add_comments {
            writeln!(output, "/* Aurora Master Header */")?;
            writeln!(output, "/* This file includes all Aurora module headers */")?;
            writeln!(output)?;
        }

        for module_name in module_names {
            let header_name = format!("{}.h", module_name.to_lowercase());
            writeln!(output, "#include \"{}\"", header_name)?;
        }

        writeln!(output)?;
        if self.config.add_comments {
            writeln!(output, "#endif /* {} */", guard)?;
        } else {
            writeln!(output, "#endif")?;
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::c_abi::{CAbiContext, CFunctionSignature, CType};

    #[test]
    fn test_header_guard() {
        let config = HeaderGenConfig::new();
        let abi_ctx = CAbiContext::new();
        let gen = HeaderGenerator::new(config, abi_ctx);

        let guard = gen.make_guard("my_module").unwrap();
        assert_eq!(guard, "AURORA_MY_MODULE_H");
    }

    #[test]
    fn test_simple_header() {
        let config = HeaderGenConfig::new();
        let mut abi_ctx = CAbiContext::new();

        let sig = CFunctionSignature {
            name: "test_func".to_string(),
            ret_type: CType::Int,
            params: vec![("x".to_string(), CType::Int)],
            is_variadic: false,
        };
        abi_ctx.register_function(sig);

        let gen = HeaderGenerator::new(config, abi_ctx);
        let header = gen.generate("test").unwrap();

        assert!(header.contains("#ifndef AURORA_TEST_H"));
        assert!(header.contains("int test_func(int x);"));
        assert!(header.contains("extern \"C\""));
    }

    #[test]
    fn test_struct_type_generation() {
        let config = HeaderGenConfig::new();
        let mut abi_ctx = CAbiContext::new();

        let point_type = CType::Struct {
            name: "Point".to_string(),
            fields: vec![
                ("x".to_string(), CType::Int),
                ("y".to_string(), CType::Int),
            ],
        };
        abi_ctx.register_type("Point".to_string(), point_type);

        let gen = HeaderGenerator::new(config, abi_ctx);
        let header = gen.generate("geometry").unwrap();

        assert!(header.contains("typedef struct Point"));
        assert!(header.contains("int x;"));
        assert!(header.contains("int y;"));
    }

    #[test]
    fn test_doc_comment() {
        let config = HeaderGenConfig::new();
        let abi_ctx = CAbiContext::new();
        let gen = HeaderGenerator::new(config, abi_ctx);

        let doc = gen.generate_doc_comment("This is a test function\nIt does something cool");
        assert!(doc.contains("/**"));
        assert!(doc.contains("This is a test function"));
        assert!(doc.contains("It does something cool"));
        assert!(doc.contains("*/"));
    }

    #[test]
    fn test_batch_generation() {
        let config = HeaderGenConfig::new();
        let batch_gen = BatchHeaderGenerator::new(config);

        let mut abi_ctx1 = CAbiContext::new();
        let sig1 = CFunctionSignature {
            name: "func1".to_string(),
            ret_type: CType::Void,
            params: vec![],
            is_variadic: false,
        };
        abi_ctx1.register_function(sig1);

        let mut abi_ctx2 = CAbiContext::new();
        let sig2 = CFunctionSignature {
            name: "func2".to_string(),
            ret_type: CType::Void,
            params: vec![],
            is_variadic: false,
        };
        abi_ctx2.register_function(sig2);

        let modules = vec![
            ("module1".to_string(), abi_ctx1),
            ("module2".to_string(), abi_ctx2),
        ];

        let headers = batch_gen.generate_all(&modules).unwrap();
        assert_eq!(headers.len(), 2);
        assert!(headers[0].1.contains("func1"));
        assert!(headers[1].1.contains("func2"));
    }

    #[test]
    fn test_master_header() {
        let config = HeaderGenConfig::new();
        let batch_gen = BatchHeaderGenerator::new(config);

        let modules = vec!["module1".to_string(), "module2".to_string()];
        let master = batch_gen.generate_master_header(&modules).unwrap();

        assert!(master.contains("#include \"module1.h\""));
        assert!(master.contains("#include \"module2.h\""));
        assert!(master.contains("AURORA_MASTER_H"));
    }

    #[test]
    fn test_no_extern_c() {
        let config = HeaderGenConfig {
            extern_c: false,
            ..HeaderGenConfig::new()
        };
        let abi_ctx = CAbiContext::new();
        let gen = HeaderGenerator::new(config, abi_ctx);
        let header = gen.generate("test").unwrap();

        assert!(!header.contains("extern \"C\""));
    }

    #[test]
    fn test_no_comments() {
        let config = HeaderGenConfig {
            add_comments: false,
            ..HeaderGenConfig::new()
        };
        let abi_ctx = CAbiContext::new();
        let gen = HeaderGenerator::new(config, abi_ctx);
        let header = gen.generate("test").unwrap();

        assert!(!header.contains("/* "));
    }
}
