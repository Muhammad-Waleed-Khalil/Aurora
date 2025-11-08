//! Top-level AST node definitions
//!
//! This module provides the root AST structure and unified node types.

use crate::decl::{Item, ItemId};
use crate::expr::Expr;
use crate::pattern::Pattern;
use crate::span::Span;
use crate::stmt::{Block, Stmt};
use crate::ty::Type;
use serde::{Deserialize, Serialize};

/// The root of an Aurora AST (a complete program or module)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    /// Top-level items in the program
    pub items: Vec<ItemId>,
    /// Source span covering the entire program
    pub span: Span,
}

impl Program {
    /// Create a new program
    pub fn new(items: Vec<ItemId>, span: Span) -> Self {
        Self { items, span }
    }

    /// Create an empty program
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            span: Span::dummy(),
        }
    }

    /// Get the number of top-level items (nodes) in this program
    ///
    /// Note: This returns the count of top-level items, not all AST nodes.
    /// For a complete node count, you would need to traverse the entire tree.
    pub fn node_count(&self) -> usize {
        self.items.len()
    }
}

/// Unified AST node type (for arena storage)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AstNode {
    /// Expression node
    Expr(Expr),
    /// Statement node
    Stmt(Stmt),
    /// Block node
    Block(Block),
    /// Item (declaration) node
    Item(Item),
    /// Type node
    Type(Type),
    /// Pattern node
    Pattern(Pattern),
}

impl AstNode {
    /// Get the span of this node
    pub fn span(&self) -> Span {
        match self {
            AstNode::Expr(expr) => expr.span,
            AstNode::Stmt(stmt) => stmt.span,
            AstNode::Block(block) => block.span,
            AstNode::Item(item) => item.span,
            AstNode::Type(ty) => ty.span,
            AstNode::Pattern(pat) => pat.span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_program() {
        let program = Program::empty();
        assert!(program.items.is_empty());
    }

    #[test]
    fn test_program_with_items() {
        let program = Program::new(vec![0, 1, 2], Span::dummy());
        assert_eq!(program.items.len(), 3);
    }
}
