//! Hygiene system for Aurora
//!
//! This module implements macro hygiene to prevent accidental variable capture.
//! Each identifier created during macro expansion receives a unique hygiene ID
//! that tracks its lexical context, ensuring that names don't accidentally
//! bind to variables from outer scopes.

use aurora_ast::span::HygieneId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a macro expansion context
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpansionContext {
    /// Unique ID for this expansion
    pub id: u32,
    /// Name of the macro being expanded
    pub macro_name: String,
    /// Parent expansion context (None if top-level)
    pub parent: Option<u32>,
    /// Depth of nesting (0 = top-level, 1 = first expansion, etc.)
    pub depth: usize,
}

/// Hygiene context managing macro expansion hygiene
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HygieneContext {
    /// All expansion contexts indexed by ID
    expansions: HashMap<u32, ExpansionContext>,
    /// Next available hygiene ID
    next_hygiene_id: u32,
    /// Next available expansion context ID
    next_expansion_id: u32,
    /// Stack of active expansion contexts
    expansion_stack: Vec<u32>,
}

impl HygieneContext {
    /// Create a new hygiene context
    pub fn new() -> Self {
        Self {
            expansions: HashMap::new(),
            next_hygiene_id: 1, // 0 is reserved for root context
            next_expansion_id: 0,
            expansion_stack: Vec::new(),
        }
    }

    /// Generate a fresh hygiene ID for an identifier
    ///
    /// This should be called for every identifier created during macro expansion
    /// to ensure it has a unique context that prevents accidental capture.
    pub fn fresh_hygiene_id(&mut self) -> HygieneId {
        let id = self.next_hygiene_id;
        self.next_hygiene_id += 1;
        HygieneId::new(id)
    }

    /// Enter a new macro expansion context
    ///
    /// This should be called when beginning to expand a macro. All identifiers
    /// created within this context will be associated with it.
    ///
    /// Returns the expansion context ID.
    pub fn enter_expansion(&mut self, macro_name: String) -> u32 {
        let id = self.next_expansion_id;
        self.next_expansion_id += 1;

        let parent = self.expansion_stack.last().copied();
        let depth = if let Some(parent_id) = parent {
            self.expansions.get(&parent_id).map(|c| c.depth + 1).unwrap_or(0)
        } else {
            0
        };

        let context = ExpansionContext {
            id,
            macro_name,
            parent,
            depth,
        };

        self.expansions.insert(id, context);
        self.expansion_stack.push(id);

        id
    }

    /// Exit the current macro expansion context
    ///
    /// This should be called when macro expansion is complete.
    pub fn exit_expansion(&mut self) -> Option<u32> {
        self.expansion_stack.pop()
    }

    /// Get the current expansion context
    pub fn current_expansion(&self) -> Option<&ExpansionContext> {
        self.expansion_stack
            .last()
            .and_then(|id| self.expansions.get(id))
    }

    /// Get an expansion context by ID
    pub fn get_expansion(&self, id: u32) -> Option<&ExpansionContext> {
        self.expansions.get(&id)
    }

    /// Get the current expansion depth
    pub fn expansion_depth(&self) -> usize {
        self.expansion_stack.len()
    }

    /// Check if currently inside a macro expansion
    pub fn in_expansion(&self) -> bool {
        !self.expansion_stack.is_empty()
    }

    /// Get the root hygiene ID (no expansion)
    pub fn root_hygiene(&self) -> HygieneId {
        HygieneId::root()
    }
}

impl Default for HygieneContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Hygiene binding represents a name binding with hygiene information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HygieneBinding {
    /// The identifier name
    pub name: String,
    /// The hygiene context this binding was created in
    pub hygiene_id: HygieneId,
    /// The scope this binding belongs to
    pub scope_id: crate::scopes::ScopeId,
}

/// Hygiene resolver for checking if a name reference can bind to a definition
pub struct HygieneResolver {
    /// Maps hygiene IDs to their expansion context
    hygiene_to_expansion: HashMap<u32, u32>,
}

impl HygieneResolver {
    /// Create a new hygiene resolver
    pub fn new() -> Self {
        Self {
            hygiene_to_expansion: HashMap::new(),
        }
    }

    /// Register a hygiene ID with its expansion context
    pub fn register_hygiene(&mut self, hygiene_id: HygieneId, expansion_id: u32) {
        self.hygiene_to_expansion.insert(hygiene_id.0, expansion_id);
    }

    /// Check if a use site can bind to a definition site
    ///
    /// Returns true if the hygiene allows the binding, false if it would
    /// cause accidental capture and should be prevented.
    pub fn can_bind(&self, use_hygiene: HygieneId, def_hygiene: HygieneId) -> bool {
        // Root context (non-macro code) can bind to anything
        if use_hygiene == HygieneId::root() {
            return true;
        }

        // Same hygiene ID means same expansion context - always allowed
        if use_hygiene == def_hygiene {
            return true;
        }

        // Definitions from root context can be used by macro expansions
        if def_hygiene == HygieneId::root() {
            return true;
        }

        // Different non-root hygiene IDs cannot bind to each other
        // This prevents accidental capture across different macro expansions
        false
    }

    /// Check if a name from one hygiene context should be visible
    /// in another hygiene context
    pub fn is_visible(
        &self,
        name: &str,
        use_hygiene: HygieneId,
        def_hygiene: HygieneId,
    ) -> bool {
        // Names starting with underscore are always visible (convention)
        if name.starts_with('_') {
            return true;
        }

        self.can_bind(use_hygiene, def_hygiene)
    }
}

impl Default for HygieneResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hygiene_context_creation() {
        let ctx = HygieneContext::new();
        assert_eq!(ctx.expansion_depth(), 0);
        assert!(!ctx.in_expansion());
    }

    #[test]
    fn test_fresh_hygiene_id() {
        let mut ctx = HygieneContext::new();

        let id1 = ctx.fresh_hygiene_id();
        let id2 = ctx.fresh_hygiene_id();
        let id3 = ctx.fresh_hygiene_id();

        // Each ID should be unique
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);

        // IDs should be sequential (starting from 1, since 0 is root)
        assert_eq!(id1.0, 1);
        assert_eq!(id2.0, 2);
        assert_eq!(id3.0, 3);
    }

    #[test]
    fn test_expansion_context() {
        let mut ctx = HygieneContext::new();

        // Enter first expansion
        let exp1 = ctx.enter_expansion("my_macro".to_string());
        assert_eq!(ctx.expansion_depth(), 1);
        assert!(ctx.in_expansion());

        let current = ctx.current_expansion().unwrap();
        assert_eq!(current.macro_name, "my_macro");
        assert_eq!(current.depth, 0);
        assert_eq!(current.parent, None);

        // Enter nested expansion
        let _exp2 = ctx.enter_expansion("nested_macro".to_string());
        assert_eq!(ctx.expansion_depth(), 2);

        let nested = ctx.current_expansion().unwrap();
        assert_eq!(nested.macro_name, "nested_macro");
        assert_eq!(nested.depth, 1);
        assert_eq!(nested.parent, Some(exp1));

        // Exit nested expansion
        ctx.exit_expansion();
        assert_eq!(ctx.expansion_depth(), 1);
        assert_eq!(ctx.current_expansion().unwrap().macro_name, "my_macro");

        // Exit first expansion
        ctx.exit_expansion();
        assert_eq!(ctx.expansion_depth(), 0);
        assert!(!ctx.in_expansion());
        assert!(ctx.current_expansion().is_none());
    }

    #[test]
    fn test_hygiene_prevents_capture() {
        let mut ctx = HygieneContext::new();
        let resolver = HygieneResolver::new();

        // Root context identifier
        let root_hygiene = ctx.root_hygiene();

        // Enter macro expansion and create identifier
        ctx.enter_expansion("test_macro".to_string());
        let macro_hygiene = ctx.fresh_hygiene_id();

        // Root can bind to anything
        assert!(resolver.can_bind(root_hygiene, root_hygiene));
        assert!(resolver.can_bind(root_hygiene, macro_hygiene));

        // Macro-created identifier can bind to root
        assert!(resolver.can_bind(macro_hygiene, root_hygiene));

        // Macro-created identifier can bind to itself
        assert!(resolver.can_bind(macro_hygiene, macro_hygiene));

        // Different macro expansions cannot bind to each other
        ctx.enter_expansion("another_macro".to_string());
        let other_macro_hygiene = ctx.fresh_hygiene_id();

        assert!(!resolver.can_bind(macro_hygiene, other_macro_hygiene));
        assert!(!resolver.can_bind(other_macro_hygiene, macro_hygiene));
    }

    #[test]
    fn test_hygiene_binding() {
        let mut ctx = HygieneContext::new();

        let binding = HygieneBinding {
            name: "x".to_string(),
            hygiene_id: ctx.fresh_hygiene_id(),
            scope_id: 0,
        };

        assert_eq!(binding.name, "x");
        assert_eq!(binding.scope_id, 0);
        assert_ne!(binding.hygiene_id, HygieneId::root());
    }

    #[test]
    fn test_underscore_visibility() {
        let resolver = HygieneResolver::new();
        let h1 = HygieneId::new(1);
        let h2 = HygieneId::new(2);

        // Regular names from different hygiene contexts cannot see each other
        assert!(!resolver.is_visible("regular", h1, h2));

        // Names starting with underscore are always visible
        assert!(resolver.is_visible("_special", h1, h2));
        assert!(resolver.is_visible("_ignored", h1, h2));
    }

    #[test]
    fn test_hygiene_id_equality() {
        let id1 = HygieneId::new(42);
        let id2 = HygieneId::new(42);
        let id3 = HygieneId::new(43);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_expansion_context_chain() {
        let mut ctx = HygieneContext::new();

        // Build chain: macro_a -> macro_b -> macro_c
        let exp_a = ctx.enter_expansion("macro_a".to_string());
        let exp_b = ctx.enter_expansion("macro_b".to_string());
        let exp_c = ctx.enter_expansion("macro_c".to_string());

        // Check depths
        let ctx_a = ctx.get_expansion(exp_a).unwrap();
        let ctx_b = ctx.get_expansion(exp_b).unwrap();
        let ctx_c = ctx.get_expansion(exp_c).unwrap();

        assert_eq!(ctx_a.depth, 0);
        assert_eq!(ctx_b.depth, 1);
        assert_eq!(ctx_c.depth, 2);

        // Check parents
        assert_eq!(ctx_a.parent, None);
        assert_eq!(ctx_b.parent, Some(exp_a));
        assert_eq!(ctx_c.parent, Some(exp_b));
    }

    #[test]
    fn test_root_hygiene() {
        let ctx = HygieneContext::new();
        let root = ctx.root_hygiene();

        assert_eq!(root, HygieneId::root());
        assert_eq!(root.0, 0);
    }

    #[test]
    fn test_multiple_expansions_same_level() {
        let mut ctx = HygieneContext::new();

        // Create two independent macro expansions
        let exp1 = ctx.enter_expansion("macro1".to_string());
        let hygiene1 = ctx.fresh_hygiene_id();
        ctx.exit_expansion();

        let exp2 = ctx.enter_expansion("macro2".to_string());
        let hygiene2 = ctx.fresh_hygiene_id();
        ctx.exit_expansion();

        // Both should be at depth 0
        let ctx1 = ctx.get_expansion(exp1).unwrap();
        let ctx2 = ctx.get_expansion(exp2).unwrap();
        assert_eq!(ctx1.depth, 0);
        assert_eq!(ctx2.depth, 0);

        // Hygiene IDs should be different
        assert_ne!(hygiene1, hygiene2);

        // They shouldn't be able to bind to each other
        let resolver = HygieneResolver::new();
        assert!(!resolver.can_bind(hygiene1, hygiene2));
        assert!(!resolver.can_bind(hygiene2, hygiene1));
    }
}
