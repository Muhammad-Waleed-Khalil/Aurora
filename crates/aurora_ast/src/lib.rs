//! aurora_ast - Aurora Abstract Syntax Tree
//!
//! This crate defines the complete AST representation for Aurora programs.
//! The AST is designed for:
//! - **Deterministic parsing**: Every valid program has exactly one AST
//! - **Span tracking**: Every node carries source location information
//! - **Hygiene support**: Macro expansion preserves lexical scoping
//! - **Machine-readable**: Full serde support for tooling integration
//!
//! # Architecture
//!
//! The AST is organized into modules by node category:
//! - `nodes`: Top-level program structure and unified node types
//! - `expr`: Expression nodes (literals, operators, control flow)
//! - `stmt`: Statement nodes (let bindings, expression statements)
//! - `decl`: Declaration nodes (functions, types, traits, impls)
//! - `ty`: Type nodes (primitives, compounds, generics)
//! - `pattern`: Pattern nodes (destructuring, matching)
//! - `span`: Source location and hygiene tracking
//!
//! # Node IDs
//!
//! Nodes reference each other via typed IDs (u32 indices) rather than
//! direct pointers. This enables:
//! - Arena allocation for cache locality
//! - Serialization/deserialization
//! - Parent link precomputation
//! - Efficient traversal

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod arena;
pub mod decl;
pub mod expr;
pub mod nodes;
pub mod pattern;
pub mod pretty;
pub mod span;
pub mod stmt;
pub mod ty;
pub mod visit;

// Re-export commonly used types
pub use arena::Arena;
pub use pretty::{PrettyConfig, PrettyPrinter};
pub use visit::{ArenaExt, PostorderIter, PreorderIter, Visitor};
pub use decl::{Item, ItemId, ItemKind};
pub use expr::{Expr, ExprId, ExprKind};
pub use nodes::{AstNode, Program};
pub use pattern::{Pattern, PatternId, PatternKind};
pub use span::{HygieneId, Span};
pub use stmt::{Block, Stmt, StmtId, StmtKind};
pub use ty::{Type, TypeId, TypeKind};

/// Alias for Program (the complete AST)
pub type Ast = Program;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_creation() {
        let program = Program::empty();
        assert!(program.items.is_empty());
    }

    #[test]
    fn test_span_creation() {
        let span = Span::new(1, 10, 20, 1, 5);
        assert_eq!(span.len(), 10);
    }
}
