//! Name Resolution for Aurora
//!
//! This crate implements the name resolution phase of the Aurora compiler,
//! including symbol tables, scope management, hygiene system, module graph
//! construction, and binding resolution.
//!
//! # Architecture
//!
//! The name resolution phase consists of several components:
//!
//! - **Symbol Table**: Stores all symbols (functions, types, variables, etc.)
//!   with support for shadowing and visibility rules.
//!
//! - **Scope Tree**: Manages the hierarchical structure of scopes (global,
//!   module, function, block, loop, match arm) and provides scope traversal.
//!
//! - **Hygiene System**: Prevents accidental variable capture in macro expansion
//!   by assigning unique hygiene IDs to identifiers based on their lexical context.
//!
//! - **Module Graph**: (TODO) Builds the dependency graph of modules and detects cycles.
//!
//! - **Name Resolver**: (TODO) Performs the actual name resolution, binding
//!   identifier uses to their definitions while respecting hygiene and visibility.
//!
//! # Example
//!
//! ```rust,ignore
//! use aurora_nameres::{SymbolTable, ScopeTree, HygieneContext};
//!
//! let mut symbols = SymbolTable::new();
//! let mut scopes = ScopeTree::new();
//! let mut hygiene = HygieneContext::new();
//!
//! // Resolution happens here...
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod hygiene;
pub mod modules;
pub mod resolver;
pub mod scopes;
pub mod symbols;

// Re-export main types
pub use hygiene::{ExpansionContext, HygieneBinding, HygieneContext, HygieneResolver};
pub use modules::{
    DependencyKind, Module, ModuleDependency, ModuleError, ModuleGraph, ModuleId, ModulePath,
};
pub use resolver::{ResolutionError, ResolutionMap, ResolutionResult, Resolver};
pub use scopes::{Scope, ScopeId, ScopeKind, ScopeTree};
pub use symbols::{Symbol, SymbolId, SymbolKind, SymbolTable, Visibility};

// Pipeline integration stub
use aurora_ast::Ast;
use std::sync::Arc;

/// Name resolver for pipeline integration
pub struct NameResolver {
    diagnostics: Arc<dyn Send + Sync>,
    symbol_count: usize,
}

impl NameResolver {
    /// Create a new name resolver with diagnostic collector
    pub fn new<D: Send + Sync + 'static>(diagnostics: Arc<D>) -> Self {
        Self {
            diagnostics: diagnostics as Arc<dyn Send + Sync>,
            symbol_count: 0,
        }
    }

    /// Resolve names in the AST
    pub fn resolve(&mut self, ast: Ast) -> Ast {
        // TODO: Implement actual name resolution
        // For now, just return the AST unchanged
        self.symbol_count = ast.items.len();
        ast
    }

    /// Get the number of resolved symbols
    pub fn symbol_count(&self) -> usize {
        self.symbol_count
    }
}
