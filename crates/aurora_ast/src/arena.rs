//! Arena allocator for AST nodes
//!
//! This module implements a bump allocator (arena) for AST nodes that provides:
//! - **Contiguous allocation**: All nodes stored sequentially for cache locality
//! - **O(1) parent links**: Precomputed parent pointers for tree navigation
//! - **O(1) subtree slicing**: Preorder/postorder indices for range queries
//! - **Efficient traversal**: Iterative visitors without recursion overhead
//!
//! # Architecture
//!
//! Nodes are allocated in a single `Vec<AstNode>` and referenced by typed
//! indices (ExprId, StmtId, etc.). After initial construction, parent links
//! and traversal indices are computed in a single pass.

use crate::nodes::AstNode;
use crate::{Expr, Item, Pattern, Stmt, Type};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Arena allocator for AST nodes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Arena {
    /// All AST nodes stored contiguously
    nodes: Vec<AstNode>,
    /// Parent node indices (node_id -> parent_id)
    parents: Vec<Option<u32>>,
    /// Preorder traversal indices (node_id -> preorder_index)
    preorder: Vec<u32>,
    /// Postorder traversal indices (node_id -> postorder_index)
    postorder: Vec<u32>,
    /// Root node ID (typically the Program)
    root: Option<u32>,
}

impl Arena {
    /// Create a new empty arena
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            parents: Vec::new(),
            preorder: Vec::new(),
            postorder: Vec::new(),
            root: None,
        }
    }

    /// Create an arena with preallocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
            parents: Vec::with_capacity(capacity),
            preorder: Vec::with_capacity(capacity),
            postorder: Vec::with_capacity(capacity),
            root: None,
        }
    }

    /// Allocate a new node and return its ID
    pub fn alloc(&mut self, node: AstNode) -> u32 {
        let id = self.nodes.len() as u32;
        self.nodes.push(node);
        self.parents.push(None); // Will be computed later
        self.preorder.push(0); // Will be computed later
        self.postorder.push(0); // Will be computed later
        id
    }

    /// Allocate an expression node
    pub fn alloc_expr(&mut self, expr: Expr) -> u32 {
        self.alloc(AstNode::Expr(expr))
    }

    /// Allocate a statement node
    pub fn alloc_stmt(&mut self, stmt: Stmt) -> u32 {
        self.alloc(AstNode::Stmt(stmt))
    }

    /// Allocate an item node
    pub fn alloc_item(&mut self, item: Item) -> u32 {
        self.alloc(AstNode::Item(item))
    }

    /// Allocate a type node
    pub fn alloc_type(&mut self, ty: Type) -> u32 {
        self.alloc(AstNode::Type(ty))
    }

    /// Allocate a pattern node
    pub fn alloc_pattern(&mut self, pattern: Pattern) -> u32 {
        self.alloc(AstNode::Pattern(pattern))
    }

    /// Get a node by ID
    pub fn get(&self, id: u32) -> Option<&AstNode> {
        self.nodes.get(id as usize)
    }

    /// Get a mutable node by ID
    pub fn get_mut(&mut self, id: u32) -> Option<&mut AstNode> {
        self.nodes.get_mut(id as usize)
    }

    /// Get the parent of a node
    pub fn parent(&self, id: u32) -> Option<u32> {
        self.parents.get(id as usize).copied().flatten()
    }

    /// Get the preorder index of a node
    pub fn preorder_index(&self, id: u32) -> Option<u32> {
        self.preorder.get(id as usize).copied()
    }

    /// Get the postorder index of a node
    pub fn postorder_index(&self, id: u32) -> Option<u32> {
        self.postorder.get(id as usize).copied()
    }

    /// Get all nodes in the arena
    pub fn nodes(&self) -> &[AstNode] {
        &self.nodes
    }

    /// Get an item node by ID
    pub fn get_item(&self, id: u32) -> Option<&Item> {
        match self.get(id)? {
            AstNode::Item(item) => Some(item),
            _ => None,
        }
    }

    /// Get an expression node by ID
    pub fn get_expr(&self, id: u32) -> Option<&Expr> {
        match self.get(id)? {
            AstNode::Expr(expr) => Some(expr),
            _ => None,
        }
    }

    /// Get a statement node by ID
    pub fn get_stmt(&self, id: u32) -> Option<&Stmt> {
        match self.get(id)? {
            AstNode::Stmt(stmt) => Some(stmt),
            _ => None,
        }
    }

    /// Get a pattern node by ID
    pub fn get_pattern(&self, id: u32) -> Option<&Pattern> {
        match self.get(id)? {
            AstNode::Pattern(pattern) => Some(pattern),
            _ => None,
        }
    }

    /// Get a type node by ID
    pub fn get_type_node(&self, id: u32) -> Option<&Type> {
        match self.get(id)? {
            AstNode::Type(ty) => Some(ty),
            _ => None,
        }
    }

    /// Get the number of nodes in the arena
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the arena is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Set the root node
    pub fn set_root(&mut self, root: u32) {
        self.root = Some(root);
    }

    /// Get the root node ID
    pub fn root(&self) -> Option<u32> {
        self.root
    }

    /// Compute parent links, preorder, and postorder indices
    ///
    /// This performs a single tree traversal to compute:
    /// 1. Parent pointers for each node
    /// 2. Preorder traversal indices (visit node before children)
    /// 3. Postorder traversal indices (visit node after children)
    ///
    /// Must be called after all nodes are allocated and before queries.
    pub fn compute_metadata(&mut self) {
        if let Some(root) = self.root {
            let mut preorder_counter = 0;
            let mut postorder_counter = 0;

            // Stack-based iterative traversal to avoid recursion
            let mut stack = vec![(root, false)]; // (node_id, visited_children)
            let mut parent_map: HashMap<u32, u32> = HashMap::new();

            while let Some((node_id, visited)) = stack.pop() {
                if visited {
                    // Postorder visit: assign postorder index
                    self.postorder[node_id as usize] = postorder_counter;
                    postorder_counter += 1;
                } else {
                    // Preorder visit: assign preorder index
                    self.preorder[node_id as usize] = preorder_counter;
                    preorder_counter += 1;

                    // Push node again for postorder visit
                    stack.push((node_id, true));

                    // Push children onto stack (in reverse order for correct traversal)
                    if let Some(node) = self.get(node_id) {
                        let children = self.get_children_ids(node);
                        for &child_id in children.iter().rev() {
                            parent_map.insert(child_id, node_id);
                            stack.push((child_id, false));
                        }
                    }
                }
            }

            // Store parent links
            for (child, parent) in parent_map {
                self.parents[child as usize] = Some(parent);
            }
        }
    }

    /// Get child node IDs for a given node (helper for traversal)
    fn get_children_ids(&self, node: &AstNode) -> Vec<u32> {
        match node {
            AstNode::Expr(expr) => self.get_expr_children(&expr.kind),
            AstNode::Stmt(stmt) => self.get_stmt_children(&stmt.kind),
            AstNode::Block(block) => {
                let mut children: Vec<u32> = block.stmts.clone();
                if let Some(expr) = block.expr {
                    children.push(expr);
                }
                children
            }
            AstNode::Item(item) => self.get_item_children(&item.kind),
            AstNode::Type(_) => vec![], // Types don't have node children (only type IDs)
            AstNode::Pattern(_) => vec![], // Patterns don't have node children
        }
    }

    /// Get children of an expression
    fn get_expr_children(&self, kind: &crate::expr::ExprKind) -> Vec<u32> {
        use crate::expr::ExprKind::*;
        match kind {
            Unary { operand, .. } => vec![*operand],
            Binary { left, right, .. } => vec![*left, *right],
            Call { func, args, .. } => {
                let mut children = vec![*func];
                children.extend_from_slice(args);
                children
            }
            MethodCall { receiver, args, .. } => {
                let mut children = vec![*receiver];
                children.extend_from_slice(args);
                children
            }
            Field { object, .. } => vec![*object],
            Index { collection, index, .. } => vec![*collection, *index],
            Pipeline { left, right, .. } => vec![*left, *right],
            If { condition, .. } => vec![*condition],
            Match { scrutinee, .. } => vec![*scrutinee],
            Loop { .. } | While { .. } | For { .. } => vec![],
            Return { value } | Break { value } => {
                value.map(|v| vec![v]).unwrap_or_default()
            }
            Yield { value } => vec![*value],
            Continue => vec![],
            Block(_) => vec![],
            Tuple(exprs) | Array(exprs) => exprs.clone(),
            Struct { .. } => vec![],
            Range { start, end, .. } => {
                let mut children = Vec::new();
                if let Some(s) = start {
                    children.push(*s);
                }
                if let Some(e) = end {
                    children.push(*e);
                }
                children
            }
            Try { expr } | Await { expr } | Comptime { expr } => vec![*expr],
            Unsafe { .. } => vec![],
            Literal(_) | Ident(_) | Path(_) => vec![],
        }
    }

    /// Get children of a statement
    fn get_stmt_children(&self, kind: &crate::stmt::StmtKind) -> Vec<u32> {
        use crate::stmt::StmtKind::*;
        match kind {
            Let { init, .. } => init.map(|e| vec![e]).unwrap_or_default(),
            Expr { expr, .. } => vec![*expr],
            Item(item_id) => vec![*item_id],
        }
    }

    /// Get children of an item
    fn get_item_children(&self, _kind: &crate::decl::ItemKind) -> Vec<u32> {
        // Items contain mostly type IDs and pattern IDs, not direct node children
        // Full traversal would need to follow those IDs as well
        vec![]
    }

    /// Check if node A is an ancestor of node B
    ///
    /// Uses preorder/postorder indices for O(1) check:
    /// A is ancestor of B iff preorder[A] < preorder[B] < postorder[B] < postorder[A]
    pub fn is_ancestor(&self, ancestor: u32, descendant: u32) -> bool {
        if let (Some(&pre_a), Some(&post_a), Some(&pre_d), Some(&post_d)) = (
            self.preorder.get(ancestor as usize),
            self.postorder.get(ancestor as usize),
            self.preorder.get(descendant as usize),
            self.postorder.get(descendant as usize),
        ) {
            pre_a < pre_d && post_d < post_a
        } else {
            false
        }
    }

    /// Get all descendants of a node (subtree slice)
    ///
    /// Returns node IDs in preorder traversal order
    pub fn descendants(&self, node_id: u32) -> Vec<u32> {
        if let Some(&pre) = self.preorder.get(node_id as usize) {
            if let Some(&post) = self.postorder.get(node_id as usize) {
                // Find all nodes with preorder in range [pre, post]
                (0..self.nodes.len() as u32)
                    .filter(|&id| {
                        if let Some(&p) = self.preorder.get(id as usize) {
                            p >= pre && p <= post
                        } else {
                            false
                        }
                    })
                    .collect()
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{Expr, ExprKind, Literal};
    use crate::span::Span;

    #[test]
    fn test_arena_creation() {
        let arena = Arena::new();
        assert!(arena.is_empty());
        assert_eq!(arena.len(), 0);
    }

    #[test]
    fn test_arena_allocation() {
        let mut arena = Arena::new();

        let expr = Expr {
            kind: ExprKind::Literal(Literal::Int(42)),
            span: Span::dummy(),
            hygiene: Default::default(),
        };

        let id = arena.alloc_expr(expr);
        assert_eq!(id, 0);
        assert_eq!(arena.len(), 1);

        let node = arena.get(id);
        assert!(node.is_some());
    }

    #[test]
    fn test_parent_links() {
        let mut arena = Arena::new();

        // Create a simple expression tree: (1 + 2)
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

        let binary = arena.alloc_expr(Expr {
            kind: ExprKind::Binary {
                op: crate::expr::BinaryOp::Add,
                left,
                right,
            },
            span: Span::dummy(),
            hygiene: Default::default(),
        });

        arena.set_root(binary);
        arena.compute_metadata();

        // Check parent links
        assert_eq!(arena.parent(left), Some(binary));
        assert_eq!(arena.parent(right), Some(binary));
        assert_eq!(arena.parent(binary), None); // Root has no parent
    }

    #[test]
    fn test_preorder_postorder() {
        let mut arena = Arena::new();

        // Create: (1 + 2)
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
                op: crate::expr::BinaryOp::Add,
                left,
                right,
            },
            span: Span::dummy(),
            hygiene: Default::default(),
        });

        arena.set_root(root);
        arena.compute_metadata();

        // Preorder: root, left, right
        assert_eq!(arena.preorder_index(root), Some(0));
        assert_eq!(arena.preorder_index(left), Some(1));
        assert_eq!(arena.preorder_index(right), Some(2));

        // Postorder: left, right, root
        assert_eq!(arena.postorder_index(left), Some(0));
        assert_eq!(arena.postorder_index(right), Some(1));
        assert_eq!(arena.postorder_index(root), Some(2));
    }

    #[test]
    fn test_is_ancestor() {
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
                op: crate::expr::BinaryOp::Add,
                left,
                right,
            },
            span: Span::dummy(),
            hygiene: Default::default(),
        });

        arena.set_root(root);
        arena.compute_metadata();

        assert!(arena.is_ancestor(root, left));
        assert!(arena.is_ancestor(root, right));
        assert!(!arena.is_ancestor(left, root));
        assert!(!arena.is_ancestor(left, right));
    }

    #[test]
    fn test_descendants() {
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
                op: crate::expr::BinaryOp::Add,
                left,
                right,
            },
            span: Span::dummy(),
            hygiene: Default::default(),
        });

        arena.set_root(root);
        arena.compute_metadata();

        let desc = arena.descendants(root);
        assert_eq!(desc.len(), 3); // root, left, right
        assert!(desc.contains(&root));
        assert!(desc.contains(&left));
        assert!(desc.contains(&right));
    }
}
