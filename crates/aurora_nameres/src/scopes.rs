//! Scope management for Aurora
//!
//! This module implements the scope hierarchy (modules, functions, blocks)
//! and provides APIs for managing lexical scoping during name resolution.

use aurora_ast::Span;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use crate::symbols::ScopeId;

/// Different kinds of scopes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScopeKind {
    /// Global/crate-level scope
    Global,
    /// Module scope
    Module,
    /// Function scope
    Function,
    /// Block scope (inside {})
    Block,
    /// Loop scope (for break/continue)
    Loop,
    /// Match arm scope
    MatchArm,
}

/// A lexical scope in the program
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scope {
    /// Unique identifier for this scope
    pub id: ScopeId,
    /// What kind of scope this is
    pub kind: ScopeKind,
    /// Parent scope (None for global scope)
    pub parent: Option<ScopeId>,
    /// Child scopes
    pub children: Vec<ScopeId>,
    /// Source span this scope covers
    pub span: Span,
    /// Optional name (for modules, functions)
    pub name: Option<String>,
}

impl Scope {
    /// Create a new scope
    pub fn new(id: ScopeId, kind: ScopeKind, parent: Option<ScopeId>, span: Span) -> Self {
        Self {
            id,
            kind,
            parent,
            children: Vec::new(),
            span,
            name: None,
        }
    }

    /// Create a named scope (for modules, functions)
    pub fn new_named(
        id: ScopeId,
        kind: ScopeKind,
        parent: Option<ScopeId>,
        span: Span,
        name: String,
    ) -> Self {
        Self {
            id,
            kind,
            parent,
            children: Vec::new(),
            span,
            name: Some(name),
        }
    }

    /// Add a child scope
    pub fn add_child(&mut self, child_id: ScopeId) {
        self.children.push(child_id);
    }

    /// Check if this is a loop scope (for break/continue validation)
    pub fn is_loop(&self) -> bool {
        matches!(self.kind, ScopeKind::Loop)
    }

    /// Check if this is a function scope (for return validation)
    pub fn is_function(&self) -> bool {
        matches!(self.kind, ScopeKind::Function)
    }
}

/// Scope tree managing all scopes in the program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeTree {
    /// All scopes indexed by ID
    scopes: HashMap<ScopeId, Scope>,
    /// The global scope ID
    global_scope: ScopeId,
    /// Current scope during traversal
    current_scope: ScopeId,
    /// Next available scope ID
    next_id: ScopeId,
}

impl ScopeTree {
    /// Create a new scope tree with a global scope
    pub fn new() -> Self {
        let mut tree = Self {
            scopes: HashMap::new(),
            global_scope: 0,
            current_scope: 0,
            next_id: 0,
        };

        // Create global scope
        let global = Scope::new(0, ScopeKind::Global, None, Span::dummy());
        tree.scopes.insert(0, global);
        tree.next_id = 1;

        tree
    }

    /// Create a new child scope and return its ID
    pub fn push_scope(&mut self, kind: ScopeKind, span: Span) -> ScopeId {
        let id = self.next_id;
        self.next_id += 1;

        let parent = self.current_scope;
        let scope = Scope::new(id, kind, Some(parent), span);

        // Add as child of current scope
        if let Some(parent_scope) = self.scopes.get_mut(&parent) {
            parent_scope.add_child(id);
        }

        self.scopes.insert(id, scope);
        self.current_scope = id;

        id
    }

    /// Create a new named child scope and return its ID
    pub fn push_named_scope(&mut self, kind: ScopeKind, span: Span, name: String) -> ScopeId {
        let id = self.next_id;
        self.next_id += 1;

        let parent = self.current_scope;
        let scope = Scope::new_named(id, kind, Some(parent), span, name);

        // Add as child of current scope
        if let Some(parent_scope) = self.scopes.get_mut(&parent) {
            parent_scope.add_child(id);
        }

        self.scopes.insert(id, scope);
        self.current_scope = id;

        id
    }

    /// Pop back to parent scope
    pub fn pop_scope(&mut self) {
        if let Some(scope) = self.scopes.get(&self.current_scope) {
            if let Some(parent) = scope.parent {
                self.current_scope = parent;
            }
        }
    }

    /// Get the current scope ID
    pub fn current_scope(&self) -> ScopeId {
        self.current_scope
    }

    /// Get a scope by ID
    pub fn get(&self, id: ScopeId) -> Option<&Scope> {
        self.scopes.get(&id)
    }

    /// Get a mutable reference to a scope by ID
    pub fn get_mut(&mut self, id: ScopeId) -> Option<&mut Scope> {
        self.scopes.get_mut(&id)
    }

    /// Get the global scope
    pub fn global_scope(&self) -> &Scope {
        self.scopes.get(&self.global_scope).unwrap()
    }

    /// Get all parent scopes from current scope to root
    pub fn parent_chain(&self, mut scope_id: ScopeId) -> Vec<ScopeId> {
        let mut chain = vec![scope_id];

        while let Some(scope) = self.scopes.get(&scope_id) {
            if let Some(parent) = scope.parent {
                chain.push(parent);
                scope_id = parent;
            } else {
                break;
            }
        }

        chain
    }

    /// Find the nearest enclosing loop scope (for break/continue)
    pub fn nearest_loop_scope(&self, from_scope: ScopeId) -> Option<ScopeId> {
        let chain = self.parent_chain(from_scope);
        chain
            .into_iter()
            .find(|&id| self.scopes.get(&id).map_or(false, |s| s.is_loop()))
    }

    /// Find the nearest enclosing function scope (for return)
    pub fn nearest_function_scope(&self, from_scope: ScopeId) -> Option<ScopeId> {
        let chain = self.parent_chain(from_scope);
        chain
            .into_iter()
            .find(|&id| self.scopes.get(&id).map_or(false, |s| s.is_function()))
    }

    /// Get the depth of a scope (distance from global scope)
    pub fn scope_depth(&self, scope_id: ScopeId) -> usize {
        self.parent_chain(scope_id).len() - 1
    }

    /// Export scope tree as JSON for debugging
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl Default for ScopeTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_tree_creation() {
        let tree = ScopeTree::new();
        assert_eq!(tree.current_scope(), 0);
        assert_eq!(tree.global_scope().kind, ScopeKind::Global);
    }

    #[test]
    fn test_push_pop_scope() {
        let mut tree = ScopeTree::new();

        // Push function scope
        let func_id = tree.push_scope(ScopeKind::Function, Span::dummy());
        assert_eq!(tree.current_scope(), func_id);
        assert_eq!(tree.get(func_id).unwrap().kind, ScopeKind::Function);

        // Push block scope
        let block_id = tree.push_scope(ScopeKind::Block, Span::dummy());
        assert_eq!(tree.current_scope(), block_id);

        // Pop back to function
        tree.pop_scope();
        assert_eq!(tree.current_scope(), func_id);

        // Pop back to global
        tree.pop_scope();
        assert_eq!(tree.current_scope(), 0);
    }

    #[test]
    fn test_named_scope() {
        let mut tree = ScopeTree::new();

        let module_id = tree.push_named_scope(
            ScopeKind::Module,
            Span::dummy(),
            "my_module".to_string(),
        );

        let scope = tree.get(module_id).unwrap();
        assert_eq!(scope.name, Some("my_module".to_string()));
        assert_eq!(scope.kind, ScopeKind::Module);
    }

    #[test]
    fn test_parent_chain() {
        let mut tree = ScopeTree::new();

        let func_id = tree.push_scope(ScopeKind::Function, Span::dummy());
        let block1_id = tree.push_scope(ScopeKind::Block, Span::dummy());
        let block2_id = tree.push_scope(ScopeKind::Block, Span::dummy());

        let chain = tree.parent_chain(block2_id);
        assert_eq!(chain, vec![block2_id, block1_id, func_id, 0]);
    }

    #[test]
    fn test_nearest_loop_scope() {
        let mut tree = ScopeTree::new();

        let _func_id = tree.push_scope(ScopeKind::Function, Span::dummy());
        let loop_id = tree.push_scope(ScopeKind::Loop, Span::dummy());
        let block_id = tree.push_scope(ScopeKind::Block, Span::dummy());

        let nearest = tree.nearest_loop_scope(block_id);
        assert_eq!(nearest, Some(loop_id));
    }

    #[test]
    fn test_nearest_function_scope() {
        let mut tree = ScopeTree::new();

        let func_id = tree.push_scope(ScopeKind::Function, Span::dummy());
        let _block1_id = tree.push_scope(ScopeKind::Block, Span::dummy());
        let block2_id = tree.push_scope(ScopeKind::Block, Span::dummy());

        let nearest = tree.nearest_function_scope(block2_id);
        assert_eq!(nearest, Some(func_id));
    }

    #[test]
    fn test_scope_depth() {
        let mut tree = ScopeTree::new();

        assert_eq!(tree.scope_depth(0), 0); // Global

        let func_id = tree.push_scope(ScopeKind::Function, Span::dummy());
        assert_eq!(tree.scope_depth(func_id), 1);

        let block_id = tree.push_scope(ScopeKind::Block, Span::dummy());
        assert_eq!(tree.scope_depth(block_id), 2);
    }

    #[test]
    fn test_child_tracking() {
        let mut tree = ScopeTree::new();

        let func_id = tree.push_scope(ScopeKind::Function, Span::dummy());
        tree.pop_scope();
        let module_id = tree.push_scope(ScopeKind::Module, Span::dummy());

        let global = tree.global_scope();
        assert_eq!(global.children.len(), 2);
        assert!(global.children.contains(&func_id));
        assert!(global.children.contains(&module_id));
    }
}
