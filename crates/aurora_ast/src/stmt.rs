//! Statement AST nodes
//!
//! This module defines statement forms including let bindings, expression
//! statements, and items that can appear in block scope.

use crate::expr::{ExprId, PatternId, TypeId};
use crate::span::Span;
use serde::{Deserialize, Serialize};

/// Statement node ID (index into arena)
pub type StmtId = u32;

/// A statement in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stmt {
    /// The statement kind
    pub kind: StmtKind,
    /// Source location
    pub span: Span,
}

/// Statement kinds
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StmtKind {
    /// Let binding (e.g., `let x = 42;`, `let mut y: i64 = 10;`)
    Let {
        /// Pattern to bind
        pattern: PatternId,
        /// Optional type annotation
        ty: Option<TypeId>,
        /// Optional initializer expression
        init: Option<ExprId>,
        /// Whether binding is mutable
        mutable: bool,
    },

    /// Expression statement (e.g., `foo();`, `x + 1;`)
    Expr {
        /// The expression
        expr: ExprId,
        /// Whether there's a trailing semicolon
        has_semi: bool,
    },

    /// Item declaration in statement position
    Item(ItemId),
}

/// Item ID (declaration node ID)
pub type ItemId = u32;

/// A block (sequence of statements with optional trailing expression)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    /// Statements in the block
    pub stmts: Vec<StmtId>,
    /// Optional trailing expression (return value)
    pub expr: Option<ExprId>,
    /// Source location
    pub span: Span,
}

impl Block {
    /// Create a new block
    pub fn new(stmts: Vec<StmtId>, expr: Option<ExprId>, span: Span) -> Self {
        Self { stmts, expr, span }
    }

    /// Create an empty block
    pub fn empty(span: Span) -> Self {
        Self {
            stmts: Vec::new(),
            expr: None,
            span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Span;

    #[test]
    fn test_block_creation() {
        let block = Block::empty(Span::dummy());
        assert!(block.stmts.is_empty());
        assert!(block.expr.is_none());
    }

    #[test]
    fn test_block_with_expr() {
        let block = Block::new(vec![], Some(0), Span::dummy());
        assert_eq!(block.expr, Some(0));
    }
}
