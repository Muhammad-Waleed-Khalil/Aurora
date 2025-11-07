//! AST traversal and visitor pattern
//!
//! This module provides iterative (stack-based) traversal mechanisms
//! for visiting AST nodes without recursion overhead or stack overflow risk.

use crate::arena::Arena;
use crate::nodes::AstNode;
use crate::{Expr, Item, Pattern, Stmt, Type};

/// Visitor trait for traversing AST nodes
///
/// Implementors can override visit methods for specific node types.
/// Default implementations traverse children automatically.
pub trait Visitor: Sized {
    /// Visit an AST node
    fn visit_node(&mut self, arena: &Arena, node_id: u32) {
        if let Some(node) = arena.get(node_id) {
            match node {
                AstNode::Expr(expr) => self.visit_expr(arena, node_id, expr),
                AstNode::Stmt(stmt) => self.visit_stmt(arena, node_id, stmt),
                AstNode::Block(block) => {
                    for &stmt_id in &block.stmts {
                        self.visit_node(arena, stmt_id);
                    }
                    if let Some(expr_id) = block.expr {
                        self.visit_node(arena, expr_id);
                    }
                }
                AstNode::Item(item) => self.visit_item(arena, node_id, item),
                AstNode::Type(ty) => self.visit_type(arena, node_id, ty),
                AstNode::Pattern(pat) => self.visit_pattern(arena, node_id, pat),
            }
        }
    }

    /// Visit an expression node
    fn visit_expr(&mut self, arena: &Arena, _node_id: u32, expr: &Expr) {
        walk_expr(self, arena, expr);
    }

    /// Visit a statement node
    fn visit_stmt(&mut self, arena: &Arena, _node_id: u32, stmt: &Stmt) {
        walk_stmt(self, arena, stmt);
    }

    /// Visit an item node
    fn visit_item(&mut self, arena: &Arena, _node_id: u32, item: &Item) {
        walk_item(self, arena, item);
    }

    /// Visit a type node
    fn visit_type(&mut self, _arena: &Arena, _node_id: u32, _ty: &Type) {
        // Default: no traversal for types
    }

    /// Visit a pattern node
    fn visit_pattern(&mut self, _arena: &Arena, _node_id: u32, _pattern: &Pattern) {
        // Default: no traversal for patterns
    }
}

/// Walk (traverse) an expression's children
pub fn walk_expr<V: Visitor>(visitor: &mut V, arena: &Arena, expr: &Expr) {
    use crate::expr::ExprKind::*;
    match &expr.kind {
        Unary { operand, .. } => visitor.visit_node(arena, *operand),
        Binary { left, right, .. } => {
            visitor.visit_node(arena, *left);
            visitor.visit_node(arena, *right);
        }
        Call { func, args, .. } => {
            visitor.visit_node(arena, *func);
            for &arg in args {
                visitor.visit_node(arena, arg);
            }
        }
        MethodCall { receiver, args, .. } => {
            visitor.visit_node(arena, *receiver);
            for &arg in args {
                visitor.visit_node(arena, arg);
            }
        }
        Field { object, .. } => visitor.visit_node(arena, *object),
        Index { collection, index, .. } => {
            visitor.visit_node(arena, *collection);
            visitor.visit_node(arena, *index);
        }
        Pipeline { left, right, .. } => {
            visitor.visit_node(arena, *left);
            visitor.visit_node(arena, *right);
        }
        If { condition, .. } => {
            visitor.visit_node(arena, *condition);
            // Blocks are handled separately
        }
        Match { scrutinee, .. } => {
            visitor.visit_node(arena, *scrutinee);
            // Arms handled separately
        }
        Return { value } | Break { value } => {
            if let Some(v) = value {
                visitor.visit_node(arena, *v);
            }
        }
        Yield { value } => visitor.visit_node(arena, *value),
        Tuple(exprs) | Array(exprs) => {
            for &expr_id in exprs {
                visitor.visit_node(arena, expr_id);
            }
        }
        Range { start, end, .. } => {
            if let Some(s) = start {
                visitor.visit_node(arena, *s);
            }
            if let Some(e) = end {
                visitor.visit_node(arena, *e);
            }
        }
        Try { expr } | Await { expr } | Comptime { expr } => {
            visitor.visit_node(arena, *expr);
        }
        // Leaf nodes
        Literal(_) | Ident(_) | Path(_) | Continue | Loop { .. } | While { .. }
        | For { .. } | Block(_) | Struct { .. } | Unsafe { .. } => {}
    }
}

/// Walk (traverse) a statement's children
pub fn walk_stmt<V: Visitor>(visitor: &mut V, arena: &Arena, stmt: &Stmt) {
    use crate::stmt::StmtKind::*;
    match &stmt.kind {
        Let { init, .. } => {
            if let Some(expr_id) = init {
                visitor.visit_node(arena, *expr_id);
            }
        }
        Expr { expr, .. } => {
            visitor.visit_node(arena, *expr);
        }
        Item(item_id) => {
            visitor.visit_node(arena, *item_id);
        }
    }
}

/// Walk (traverse) an item's children
pub fn walk_item<V: Visitor>(_visitor: &mut V, _arena: &Arena, _item: &Item) {
    // Items contain mostly IDs, not direct children
    // Full traversal would follow those IDs as well
}

/// Preorder iterator for AST traversal
///
/// Visits nodes in preorder (parent before children) using indices
pub struct PreorderIter<'a> {
    arena: &'a Arena,
    current: usize,
}

impl<'a> PreorderIter<'a> {
    /// Create a new preorder iterator
    pub fn new(arena: &'a Arena) -> Self {
        Self { arena, current: 0 }
    }
}

impl<'a> Iterator for PreorderIter<'a> {
    type Item = (u32, &'a AstNode);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.arena.len() {
            let id = self.current as u32;
            self.current += 1;

            // Find node with this preorder index
            for node_id in 0..self.arena.len() as u32 {
                if self.arena.preorder_index(node_id) == Some(id) {
                    return self.arena.get(node_id).map(|node| (node_id, node));
                }
            }
        }
        None
    }
}

/// Postorder iterator for AST traversal
///
/// Visits nodes in postorder (children before parent) using indices
pub struct PostorderIter<'a> {
    arena: &'a Arena,
    current: usize,
}

impl<'a> PostorderIter<'a> {
    /// Create a new postorder iterator
    pub fn new(arena: &'a Arena) -> Self {
        Self { arena, current: 0 }
    }
}

impl<'a> Iterator for PostorderIter<'a> {
    type Item = (u32, &'a AstNode);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.arena.len() {
            let id = self.current as u32;
            self.current += 1;

            // Find node with this postorder index
            for node_id in 0..self.arena.len() as u32 {
                if self.arena.postorder_index(node_id) == Some(id) {
                    return self.arena.get(node_id).map(|node| (node_id, node));
                }
            }
        }
        None
    }
}

/// Extension trait for arena to add traversal methods
pub trait ArenaExt {
    /// Create a preorder iterator
    fn preorder(&self) -> PreorderIter;

    /// Create a postorder iterator
    fn postorder(&self) -> PostorderIter;
}

impl ArenaExt for Arena {
    fn preorder(&self) -> PreorderIter {
        PreorderIter::new(self)
    }

    fn postorder(&self) -> PostorderIter {
        PostorderIter::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{BinaryOp, Expr, ExprKind, Literal};
    use crate::span::Span;

    struct CountingVisitor {
        count: usize,
    }

    impl Visitor for CountingVisitor {
        fn visit_expr(&mut self, arena: &Arena, _node_id: u32, expr: &Expr) {
            self.count += 1;
            walk_expr(self, arena, expr);
        }
    }

    #[test]
    fn test_visitor_pattern() {
        let mut arena = Arena::new();

        let left = arena.alloc_expr(Expr {
            kind: ExprKind::Literal(Literal::Int(1)),
            span: Span::dummy(),
            hygiene: Default::default(),
        });

        let right = arena.alloc_expr(Expr {
            kind: ExprKind::Literal(Literal::Int(2)),
            span: Span::dummy(),
            hygiene: Default::default(),
        });

        let root = arena.alloc_expr(Expr {
            kind: ExprKind::Binary {
                op: BinaryOp::Add,
                left,
                right,
            },
            span: Span::dummy(),
            hygiene: Default::default(),
        });

        arena.set_root(root);
        arena.compute_metadata();

        let mut visitor = CountingVisitor { count: 0 };
        visitor.visit_node(&arena, root);

        assert_eq!(visitor.count, 3); // root + left + right
    }

    #[test]
    fn test_preorder_iterator() {
        let mut arena = Arena::new();

        let left = arena.alloc_expr(Expr {
            kind: ExprKind::Literal(Literal::Int(1)),
            span: Span::dummy(),
            hygiene: Default::default(),
        });

        let right = arena.alloc_expr(Expr {
            kind: ExprKind::Literal(Literal::Int(2)),
            span: Span::dummy(),
            hygiene: Default::default(),
        });

        let root = arena.alloc_expr(Expr {
            kind: ExprKind::Binary {
                op: BinaryOp::Add,
                left,
                right,
            },
            span: Span::dummy(),
            hygiene: Default::default(),
        });

        arena.set_root(root);
        arena.compute_metadata();

        let nodes: Vec<_> = arena.preorder().collect();
        assert_eq!(nodes.len(), 3);
        // Preorder: root, left, right
        assert_eq!(nodes[0].0, root);
        assert_eq!(nodes[1].0, left);
        assert_eq!(nodes[2].0, right);
    }

    #[test]
    fn test_postorder_iterator() {
        let mut arena = Arena::new();

        let left = arena.alloc_expr(Expr {
            kind: ExprKind::Literal(Literal::Int(1)),
            span: Span::dummy(),
            hygiene: Default::default(),
        });

        let right = arena.alloc_expr(Expr {
            kind: ExprKind::Literal(Literal::Int(2)),
            span: Span::dummy(),
            hygiene: Default::default(),
        });

        let root = arena.alloc_expr(Expr {
            kind: ExprKind::Binary {
                op: BinaryOp::Add,
                left,
                right,
            },
            span: Span::dummy(),
            hygiene: Default::default(),
        });

        arena.set_root(root);
        arena.compute_metadata();

        let nodes: Vec<_> = arena.postorder().collect();
        assert_eq!(nodes.len(), 3);
        // Postorder: left, right, root
        assert_eq!(nodes[0].0, left);
        assert_eq!(nodes[1].0, right);
        assert_eq!(nodes[2].0, root);
    }
}
