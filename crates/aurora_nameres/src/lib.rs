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
pub use resolver::{ResolutionChain, ResolutionError, ResolutionMap, ResolutionResult, Resolver};
pub use scopes::{Scope, ScopeId, ScopeKind, ScopeTree};
pub use symbols::{Symbol, SymbolId, SymbolKind, SymbolTable, Visibility};

// Pipeline integration
use aurora_ast::{Arena, Ast};
use std::sync::Arc;

/// Name resolver for pipeline integration
pub struct NameResolver {
    diagnostics: Arc<dyn Send + Sync>,
    /// Resolution result (populated after resolve() is called)
    result: Option<ResolutionResult>,
}

impl NameResolver {
    /// Create a new name resolver with diagnostic collector
    pub fn new<D: Send + Sync + 'static>(diagnostics: Arc<D>) -> Self {
        Self {
            diagnostics: diagnostics as Arc<dyn Send + Sync>,
            result: None,
        }
    }

    /// Resolve names in the AST
    ///
    /// Note: This requires access to the Arena. For now, this is a stub that
    /// will be integrated properly when the pipeline provides Arena access.
    pub fn resolve(&mut self, ast: Ast) -> Ast {
        // TODO: Once the pipeline provides Arena access, we can do:
        // let arena = ...; // Need arena from pipeline
        // let resolver = Resolver::new(arena, "crate_name".to_string());
        // self.result = Some(resolver.resolve(&ast));

        // For now, just return the AST unchanged
        ast
    }

    /// Resolve names with explicit arena access
    ///
    /// This is the full resolution implementation that requires the arena.
    pub fn resolve_with_arena(&mut self, ast: &Ast, arena: &Arena, crate_name: String) -> ResolutionResult {
        let resolver = Resolver::new(arena, crate_name);
        let result = resolver.resolve(ast);

        // Store result for later access
        self.result = Some(result.clone());

        result
    }

    /// Get the number of resolved symbols
    pub fn symbol_count(&self) -> usize {
        self.result.as_ref().map(|r| r.symbols.len()).unwrap_or(0)
    }

    /// Get the resolution result
    pub fn resolution_result(&self) -> Option<&ResolutionResult> {
        self.result.as_ref()
    }

    /// Get the symbol table
    pub fn symbols(&self) -> Option<&SymbolTable> {
        self.result.as_ref().map(|r| &r.symbols)
    }

    /// Get the scope tree
    pub fn scopes(&self) -> Option<&ScopeTree> {
        self.result.as_ref().map(|r| &r.scopes)
    }

    /// Get the module graph
    pub fn modules(&self) -> Option<&ModuleGraph> {
        self.result.as_ref().map(|r| &r.modules)
    }

    /// Get the resolution map
    pub fn resolution_map(&self) -> Option<&ResolutionMap> {
        self.result.as_ref().map(|r| &r.resolution_map)
    }

    /// Get diagnostics
    pub fn diagnostics(&self) -> Vec<ResolutionError> {
        self.result.as_ref().map(|r| r.diagnostics.clone()).unwrap_or_default()
    }
}
