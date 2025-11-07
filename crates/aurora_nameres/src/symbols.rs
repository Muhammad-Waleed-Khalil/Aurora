//! Symbol table implementation for Aurora
//!
//! This module defines the symbol representation and provides APIs
//! for storing and retrieving symbols during name resolution.

use aurora_ast::Span;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a symbol
pub type SymbolId = u32;

/// Unique identifier for a scope
pub type ScopeId = u32;

/// A symbol in the program (variable, function, type, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Symbol {
    /// Unique identifier for this symbol
    pub id: SymbolId,
    /// The name of the symbol
    pub name: String,
    /// What kind of symbol this is
    pub kind: SymbolKind,
    /// Visibility level
    pub visibility: Visibility,
    /// Where this symbol was defined
    pub def_span: Span,
    /// The scope this symbol belongs to
    pub scope_id: ScopeId,
    /// Whether this symbol has been used
    pub is_used: bool,
    /// Optional type information (filled in during type checking)
    pub ty: Option<String>, // Will be replaced with actual Type in phase 4
}

/// Different kinds of symbols
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolKind {
    /// Function declaration
    Function,
    /// Type declaration (struct, enum, type alias)
    Type,
    /// Trait declaration
    Trait,
    /// Constant declaration
    Const,
    /// Static variable
    Static,
    /// Local variable
    Variable,
    /// Function parameter
    Parameter,
    /// Module
    Module,
    /// Type parameter (generic)
    TypeParam,
    /// Trait method
    Method,
    /// Struct field
    Field,
    /// Enum variant
    Variant,
}

/// Visibility levels for symbols
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Visibility {
    /// Private to current module
    Private,
    /// Visible within crate
    Crate,
    /// Publicly exported
    Public,
}

impl Symbol {
    /// Create a new symbol
    pub fn new(
        id: SymbolId,
        name: String,
        kind: SymbolKind,
        visibility: Visibility,
        def_span: Span,
        scope_id: ScopeId,
    ) -> Self {
        Self {
            id,
            name,
            kind,
            visibility,
            def_span,
            scope_id,
            is_used: false,
            ty: None,
        }
    }

    /// Check if this symbol is visible from another scope
    pub fn is_visible_from(&self, from_scope: ScopeId, _current_scope: ScopeId) -> bool {
        match self.visibility {
            Visibility::Private => self.scope_id == from_scope,
            Visibility::Crate => true, // Same crate
            Visibility::Public => true,
        }
    }

    /// Mark this symbol as used
    pub fn mark_used(&mut self) {
        self.is_used = true;
    }
}

/// Symbol table storing all symbols in the program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolTable {
    /// All symbols indexed by ID
    symbols: HashMap<SymbolId, Symbol>,
    /// Symbol name resolution: (scope_id, name) -> symbol_id
    /// This allows shadowing - each scope can have its own binding
    bindings: HashMap<(ScopeId, String), SymbolId>,
    /// Next available symbol ID
    next_id: SymbolId,
}

impl SymbolTable {
    /// Create a new empty symbol table
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            bindings: HashMap::new(),
            next_id: 0,
        }
    }

    /// Insert a new symbol into the table
    ///
    /// Returns the symbol ID, or None if a symbol with the same name
    /// already exists in the given scope (duplicate definition error)
    pub fn insert(&mut self, mut symbol: Symbol) -> Option<SymbolId> {
        let key = (symbol.scope_id, symbol.name.clone());

        // Check for duplicate definition in same scope
        if self.bindings.contains_key(&key) {
            return None; // Duplicate definition
        }

        symbol.id = self.next_id;
        let id = symbol.id;
        self.next_id += 1;

        self.symbols.insert(id, symbol.clone());
        self.bindings.insert(key, id);

        Some(id)
    }

    /// Look up a symbol by name in a given scope
    ///
    /// This will search the current scope and parent scopes
    pub fn lookup(&self, scope_id: ScopeId, name: &str) -> Option<SymbolId> {
        let key = (scope_id, name.to_string());
        self.bindings.get(&key).copied()
    }

    /// Get a symbol by ID
    pub fn get(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbols.get(&id)
    }

    /// Get a mutable reference to a symbol by ID
    pub fn get_mut(&mut self, id: SymbolId) -> Option<&mut Symbol> {
        self.symbols.get_mut(&id)
    }

    /// Mark a symbol as used
    pub fn mark_used(&mut self, id: SymbolId) {
        if let Some(symbol) = self.symbols.get_mut(&id) {
            symbol.mark_used();
        }
    }

    /// Get all symbols in a specific scope
    pub fn symbols_in_scope(&self, scope_id: ScopeId) -> Vec<&Symbol> {
        self.symbols
            .values()
            .filter(|s| s.scope_id == scope_id)
            .collect()
    }

    /// Get all unused symbols (for warnings)
    pub fn unused_symbols(&self) -> Vec<&Symbol> {
        self.symbols
            .values()
            .filter(|s| !s.is_used && !s.name.starts_with('_'))
            .collect()
    }

    /// Get the total number of symbols
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Check if the symbol table is empty
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    /// Export symbol table as JSON for debugging/tooling
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_creation() {
        let symbol = Symbol::new(
            0,
            "test".to_string(),
            SymbolKind::Function,
            Visibility::Public,
            Span::dummy(),
            0,
        );

        assert_eq!(symbol.name, "test");
        assert_eq!(symbol.kind, SymbolKind::Function);
        assert_eq!(symbol.visibility, Visibility::Public);
        assert!(!symbol.is_used);
    }

    #[test]
    fn test_symbol_table_insert() {
        let mut table = SymbolTable::new();

        let symbol = Symbol::new(
            0,
            "foo".to_string(),
            SymbolKind::Variable,
            Visibility::Private,
            Span::dummy(),
            0,
        );

        let id = table.insert(symbol).unwrap();
        assert_eq!(id, 0);

        // Duplicate insertion should fail
        let duplicate = Symbol::new(
            0,
            "foo".to_string(),
            SymbolKind::Variable,
            Visibility::Private,
            Span::dummy(),
            0,
        );

        assert!(table.insert(duplicate).is_none());
    }

    #[test]
    fn test_symbol_lookup() {
        let mut table = SymbolTable::new();

        let symbol = Symbol::new(
            0,
            "bar".to_string(),
            SymbolKind::Function,
            Visibility::Public,
            Span::dummy(),
            0,
        );

        let id = table.insert(symbol).unwrap();

        let found = table.lookup(0, "bar");
        assert_eq!(found, Some(id));

        let not_found = table.lookup(0, "nonexistent");
        assert_eq!(not_found, None);
    }

    #[test]
    fn test_symbol_shadowing() {
        let mut table = SymbolTable::new();

        // Outer scope
        let outer = Symbol::new(
            0,
            "x".to_string(),
            SymbolKind::Variable,
            Visibility::Private,
            Span::dummy(),
            0,
        );
        let outer_id = table.insert(outer).unwrap();

        // Inner scope (shadowing)
        let inner = Symbol::new(
            0,
            "x".to_string(),
            SymbolKind::Variable,
            Visibility::Private,
            Span::dummy(),
            1,
        );
        let inner_id = table.insert(inner).unwrap();

        assert_ne!(outer_id, inner_id);

        // Each scope has its own binding
        assert_eq!(table.lookup(0, "x"), Some(outer_id));
        assert_eq!(table.lookup(1, "x"), Some(inner_id));
    }

    #[test]
    fn test_mark_used() {
        let mut table = SymbolTable::new();

        let symbol = Symbol::new(
            0,
            "used".to_string(),
            SymbolKind::Function,
            Visibility::Public,
            Span::dummy(),
            0,
        );

        let id = table.insert(symbol).unwrap();
        assert!(!table.get(id).unwrap().is_used);

        table.mark_used(id);
        assert!(table.get(id).unwrap().is_used);
    }

    #[test]
    fn test_unused_symbols() {
        let mut table = SymbolTable::new();

        // Used symbol
        let used = Symbol::new(
            0,
            "used".to_string(),
            SymbolKind::Variable,
            Visibility::Private,
            Span::dummy(),
            0,
        );
        let used_id = table.insert(used).unwrap();
        table.mark_used(used_id);

        // Unused symbol
        let unused = Symbol::new(
            0,
            "unused".to_string(),
            SymbolKind::Variable,
            Visibility::Private,
            Span::dummy(),
            0,
        );
        table.insert(unused).unwrap();

        // Symbol starting with _ (should be ignored)
        let ignored = Symbol::new(
            0,
            "_ignored".to_string(),
            SymbolKind::Variable,
            Visibility::Private,
            Span::dummy(),
            0,
        );
        table.insert(ignored).unwrap();

        let unused_list = table.unused_symbols();
        assert_eq!(unused_list.len(), 1);
        assert_eq!(unused_list[0].name, "unused");
    }
}
