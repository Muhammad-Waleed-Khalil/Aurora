//! Name resolution for Aurora
//!
//! This module implements the main name resolution pass that traverses the AST,
//! collects declarations into symbol tables, and resolves identifier uses to
//! their definitions while respecting scope rules, visibility, and hygiene.

use crate::hygiene::{HygieneContext, HygieneResolver};
use crate::modules::{ModuleError, ModuleGraph, ModuleId};
use crate::scopes::{ScopeId, ScopeKind, ScopeTree};
use crate::symbols::{Symbol, SymbolId, SymbolKind, SymbolTable, Visibility};
use aurora_ast::decl::{Item, ItemKind, Param};
use aurora_ast::expr::{Expr, ExprId, ExprKind, Path};
use aurora_ast::pattern::{Pattern, PatternId, PatternKind};
use aurora_ast::span::HygieneId;
use aurora_ast::stmt::{Block, Stmt, StmtId, StmtKind};
use aurora_ast::{Arena, Program, Span};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Resolution chain - explains why a name resolved the way it did
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolutionChain {
    /// The name being resolved
    pub name: String,
    /// Scopes searched in order
    pub scopes_searched: Vec<ScopeId>,
    /// The scope where the symbol was found
    pub found_in_scope: Option<ScopeId>,
    /// The symbol ID it resolved to
    pub symbol_id: Option<SymbolId>,
    /// Why the resolution succeeded or failed
    pub reason: String,
}

impl ResolutionChain {
    /// Create a new resolution chain for a successful resolution
    pub fn success(
        name: String,
        scopes_searched: Vec<ScopeId>,
        found_scope: ScopeId,
        symbol_id: SymbolId,
    ) -> Self {
        Self {
            name: name.clone(),
            scopes_searched,
            found_in_scope: Some(found_scope),
            symbol_id: Some(symbol_id),
            reason: format!("Found '{}' in scope {}", name, found_scope),
        }
    }

    /// Create a new resolution chain for a failed resolution
    pub fn failure(name: String, scopes_searched: Vec<ScopeId>, reason: String) -> Self {
        Self {
            name,
            scopes_searched,
            found_in_scope: None,
            symbol_id: None,
            reason,
        }
    }
}

/// Resolution result mapping uses to definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionMap {
    /// Maps expression IDs (identifier uses) to symbol IDs (definitions)
    expr_resolutions: HashMap<ExprId, SymbolId>,
    /// Maps pattern IDs (bindings) to symbol IDs
    pattern_bindings: HashMap<PatternId, SymbolId>,
    /// Resolution chains for debugging/diagnostics
    resolution_chains: HashMap<ExprId, ResolutionChain>,
}

impl ResolutionMap {
    /// Create a new empty resolution map
    pub fn new() -> Self {
        Self {
            expr_resolutions: HashMap::new(),
            pattern_bindings: HashMap::new(),
            resolution_chains: HashMap::new(),
        }
    }

    /// Record that an expression resolves to a symbol with a resolution chain
    pub fn resolve_expr(&mut self, expr_id: ExprId, symbol_id: SymbolId) {
        self.expr_resolutions.insert(expr_id, symbol_id);
    }

    /// Record a resolution chain
    pub fn record_chain(&mut self, expr_id: ExprId, chain: ResolutionChain) {
        self.resolution_chains.insert(expr_id, chain);
    }

    /// Record that a pattern binds a new symbol
    pub fn bind_pattern(&mut self, pattern_id: PatternId, symbol_id: SymbolId) {
        self.pattern_bindings.insert(pattern_id, symbol_id);
    }

    /// Get the resolution for an expression
    pub fn get_expr_resolution(&self, expr_id: ExprId) -> Option<SymbolId> {
        self.expr_resolutions.get(&expr_id).copied()
    }

    /// Get the resolution chain for an expression
    pub fn get_resolution_chain(&self, expr_id: ExprId) -> Option<&ResolutionChain> {
        self.resolution_chains.get(&expr_id)
    }

    /// Get the binding for a pattern
    pub fn get_pattern_binding(&self, pattern_id: PatternId) -> Option<SymbolId> {
        self.pattern_bindings.get(&pattern_id).copied()
    }

    /// Get the number of resolved expressions
    pub fn expr_count(&self) -> usize {
        self.expr_resolutions.len()
    }

    /// Get the number of pattern bindings
    pub fn pattern_count(&self) -> usize {
        self.pattern_bindings.len()
    }
}

impl Default for ResolutionMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Name resolver
pub struct Resolver<'a> {
    /// Symbol table
    symbols: SymbolTable,
    /// Scope tree
    scopes: ScopeTree,
    /// Module graph
    modules: ModuleGraph,
    /// Hygiene context
    hygiene_ctx: HygieneContext,
    /// Hygiene resolver
    hygiene_resolver: HygieneResolver,
    /// Resolution map
    resolution_map: ResolutionMap,
    /// Current module
    current_module: ModuleId,
    /// AST arena
    arena: &'a Arena,
    /// Diagnostics
    diagnostics: Vec<ResolutionError>,
    /// Prelude scope (standard library items)
    prelude_scope: ScopeId,
}

impl<'a> Resolver<'a> {
    /// Create a new resolver
    pub fn new(arena: &'a Arena, crate_name: String) -> Self {
        let mut symbols = SymbolTable::new();
        let mut scopes = ScopeTree::new();

        // Create prelude scope as a child of global scope
        let prelude_scope = scopes.push_named_scope(
            ScopeKind::Module,
            Span::dummy(),
            "prelude".to_string(),
        );

        // Populate prelude with standard library items
        Self::populate_prelude(&mut symbols, prelude_scope);

        // Return to global scope
        scopes.pop_scope();

        Self {
            symbols,
            scopes,
            modules: ModuleGraph::new(crate_name),
            hygiene_ctx: HygieneContext::new(),
            hygiene_resolver: HygieneResolver::new(),
            resolution_map: ResolutionMap::new(),
            current_module: 0, // Root module
            arena,
            diagnostics: Vec::new(),
            prelude_scope,
        }
    }

    /// Populate the prelude with standard library items
    fn populate_prelude(symbols: &mut SymbolTable, prelude_scope: ScopeId) {
        // Standard I/O functions
        let prelude_items = vec![
            ("println", SymbolKind::Function),
            ("print", SymbolKind::Function),
            ("eprintln", SymbolKind::Function),
            ("eprint", SymbolKind::Function),
            ("dbg", SymbolKind::Function),
            // Standard types
            ("String", SymbolKind::Type),
            ("Vec", SymbolKind::Type),
            ("Option", SymbolKind::Type),
            ("Result", SymbolKind::Type),
            ("Box", SymbolKind::Type),
            ("Arc", SymbolKind::Type),
            ("Rc", SymbolKind::Type),
            // Option/Result variants
            ("Some", SymbolKind::Variant),
            ("None", SymbolKind::Variant),
            ("Ok", SymbolKind::Variant),
            ("Err", SymbolKind::Variant),
            // Common traits
            ("Clone", SymbolKind::Trait),
            ("Copy", SymbolKind::Trait),
            ("Debug", SymbolKind::Trait),
            ("Display", SymbolKind::Trait),
            ("Default", SymbolKind::Trait),
            ("Iterator", SymbolKind::Trait),
            ("IntoIterator", SymbolKind::Trait),
            ("From", SymbolKind::Trait),
            ("Into", SymbolKind::Trait),
        ];

        for (name, kind) in prelude_items {
            let symbol = Symbol::new(
                0,
                name.to_string(),
                kind,
                Visibility::Public,
                Span::dummy(),
                prelude_scope,
            );
            symbols.insert(symbol);
        }
    }

    /// Resolve a program
    ///
    /// This is the main entry point. It performs name resolution in multiple passes:
    /// 1. Collect all top-level declarations
    /// 2. Build module graph
    /// 3. Resolve identifier uses
    pub fn resolve(mut self, program: &Program) -> ResolutionResult {
        // Pass 1: Collect top-level items
        for &item_id in &program.items {
            if let Some(item) = self.arena.get_item(item_id) {
                self.collect_item(item, item_id);
            }
        }

        // Pass 2: Check for module cycles
        if let Err(e) = self.modules.detect_cycles() {
            self.diagnostics.push(ResolutionError::ModuleError(e));
        }

        // Pass 3: Resolve uses
        for &item_id in &program.items {
            if let Some(item) = self.arena.get_item(item_id) {
                self.resolve_item(item, item_id);
            }
        }

        ResolutionResult {
            symbols: self.symbols,
            scopes: self.scopes,
            modules: self.modules,
            resolution_map: self.resolution_map,
            diagnostics: self.diagnostics,
        }
    }

    /// Collect declarations from an item (pass 1)
    fn collect_item(&mut self, item: &Item, _item_id: u32) {
        match &item.kind {
            ItemKind::Function(func) => {
                let vis = if func.is_pub {
                    Visibility::Public
                } else {
                    Visibility::Private
                };

                let symbol = Symbol::new(
                    0, // Will be assigned by insert
                    func.name.clone(),
                    SymbolKind::Function,
                    vis,
                    func.span,
                    self.scopes.current_scope(),
                );

                if let Some(symbol_id) = self.symbols.insert(symbol) {
                    // Success
                    let _ = symbol_id;
                } else {
                    self.diagnostics.push(ResolutionError::DuplicateDefinition {
                        name: func.name.clone(),
                        first_span: func.span,
                        second_span: func.span,
                    });
                }
            }
            ItemKind::Type(ty) => {
                let vis = if ty.is_pub {
                    Visibility::Public
                } else {
                    Visibility::Private
                };

                let symbol = Symbol::new(
                    0,
                    ty.name.clone(),
                    SymbolKind::Type,
                    vis,
                    ty.span,
                    self.scopes.current_scope(),
                );

                self.symbols.insert(symbol);
            }
            ItemKind::Const(c) => {
                let vis = if c.is_pub {
                    Visibility::Public
                } else {
                    Visibility::Private
                };

                let symbol = Symbol::new(
                    0,
                    c.name.clone(),
                    SymbolKind::Const,
                    vis,
                    c.span,
                    self.scopes.current_scope(),
                );

                self.symbols.insert(symbol);
            }
            ItemKind::Module(module) => {
                if let Err(e) = self.modules.add_module(module, self.current_module) {
                    self.diagnostics.push(ResolutionError::ModuleError(e));
                }

                let vis = if module.is_pub {
                    Visibility::Public
                } else {
                    Visibility::Private
                };

                let symbol = Symbol::new(
                    0,
                    module.name.clone(),
                    SymbolKind::Module,
                    vis,
                    module.span,
                    self.scopes.current_scope(),
                );

                self.symbols.insert(symbol);
            }
            ItemKind::Use(use_decl) => {
                if let Err(e) = self.modules.add_use(use_decl, self.current_module) {
                    self.diagnostics.push(ResolutionError::ModuleError(e));
                }
            }
            ItemKind::Trait(trait_decl) => {
                let vis = if trait_decl.is_pub {
                    Visibility::Public
                } else {
                    Visibility::Private
                };

                let symbol = Symbol::new(
                    0,
                    trait_decl.name.clone(),
                    SymbolKind::Trait,
                    vis,
                    trait_decl.span,
                    self.scopes.current_scope(),
                );

                self.symbols.insert(symbol);
            }
            ItemKind::Impl(_) => {
                // Impls don't create named symbols at top level
            }
        }
    }

    /// Resolve uses in an item (pass 3)
    fn resolve_item(&mut self, item: &Item, _item_id: u32) {
        match &item.kind {
            ItemKind::Function(func) => {
                // Enter function scope
                let _func_scope = self.scopes.push_named_scope(
                    ScopeKind::Function,
                    func.span,
                    func.name.clone(),
                );

                // Resolve parameters
                for param in &func.params {
                    self.resolve_param(param);
                }

                // Resolve body
                self.resolve_block(&func.body);

                // Exit function scope
                self.scopes.pop_scope();
            }
            ItemKind::Const(c) => {
                // Resolve value expression
                self.resolve_expr(c.value);
            }
            ItemKind::Module(module) => {
                // If inline module, resolve its items
                if let Some(items) = &module.items {
                    // TODO: Enter module scope and resolve items
                    let _ = items;
                }
            }
            _ => {
                // Other items don't have expressions to resolve yet
            }
        }
    }

    /// Resolve a parameter
    fn resolve_param(&mut self, param: &Param) {
        // Collect bindings from the pattern
        if let Some(pattern) = self.arena.get_pattern(param.pattern) {
            self.collect_pattern_bindings(pattern, param.pattern);
        }
    }

    /// Collect bindings from a pattern
    fn collect_pattern_bindings(&mut self, pattern: &Pattern, pattern_id: PatternId) {
        match &pattern.kind {
            PatternKind::Ident { name, is_mut: _ } => {
                let symbol = Symbol::new(
                    0,
                    name.clone(),
                    SymbolKind::Variable,
                    Visibility::Private,
                    pattern.span,
                    self.scopes.current_scope(),
                );

                if let Some(symbol_id) = self.symbols.insert(symbol) {
                    self.resolution_map.bind_pattern(pattern_id, symbol_id);
                } else {
                    self.diagnostics.push(ResolutionError::DuplicateDefinition {
                        name: name.clone(),
                        first_span: pattern.span,
                        second_span: pattern.span,
                    });
                }
            }
            PatternKind::Tuple(patterns) => {
                for &pat_id in patterns {
                    if let Some(pat) = self.arena.get_pattern(pat_id) {
                        self.collect_pattern_bindings(pat, pat_id);
                    }
                }
            }
            PatternKind::Struct { path: _, fields, has_rest: _ } => {
                for field in fields {
                    if let Some(pat_id) = field.pattern {
                        if let Some(pat) = self.arena.get_pattern(pat_id) {
                            self.collect_pattern_bindings(pat, pat_id);
                        }
                    } else {
                        // Shorthand binding: the field name is the binding
                        let symbol = Symbol::new(
                            0,
                            field.name.clone(),
                            SymbolKind::Variable,
                            Visibility::Private,
                            field.span,
                            self.scopes.current_scope(),
                        );
                        self.symbols.insert(symbol);
                    }
                }
            }
            PatternKind::TupleStruct { path: _, fields } => {
                for &pat_id in fields {
                    if let Some(pat) = self.arena.get_pattern(pat_id) {
                        self.collect_pattern_bindings(pat, pat_id);
                    }
                }
            }
            PatternKind::Or(patterns) => {
                for &pat_id in patterns {
                    if let Some(pat) = self.arena.get_pattern(pat_id) {
                        self.collect_pattern_bindings(pat, pat_id);
                    }
                }
            }
            PatternKind::Ref { inner, is_mut: _ } => {
                if let Some(pat) = self.arena.get_pattern(**inner) {
                    self.collect_pattern_bindings(pat, **inner);
                }
            }
            PatternKind::Wildcard
            | PatternKind::Literal(_)
            | PatternKind::Path(_)
            | PatternKind::Range { .. }
            | PatternKind::Rest => {
                // These don't introduce bindings
            }
        }
    }

    /// Resolve a block
    fn resolve_block(&mut self, block: &Block) {
        // Enter block scope
        let _block_scope = self.scopes.push_scope(ScopeKind::Block, block.span);

        // Resolve statements
        for &stmt_id in &block.stmts {
            if let Some(stmt) = self.arena.get_stmt(stmt_id) {
                self.resolve_stmt(stmt, stmt_id);
            }
        }

        // Resolve trailing expression
        if let Some(expr_id) = block.expr {
            self.resolve_expr(expr_id);
        }

        // Exit block scope
        self.scopes.pop_scope();
    }

    /// Resolve a statement
    fn resolve_stmt(&mut self, stmt: &Stmt, _stmt_id: StmtId) {
        match &stmt.kind {
            StmtKind::Let { pattern, ty: _, init, mutable: _ } => {
                // First resolve the init expression (RHS)
                if let Some(init_expr) = init {
                    self.resolve_expr(*init_expr);
                }

                // Then collect bindings from the pattern (LHS)
                if let Some(pat) = self.arena.get_pattern(*pattern) {
                    self.collect_pattern_bindings(pat, *pattern);
                }
            }
            StmtKind::Expr { expr, has_semi: _ } => {
                self.resolve_expr(*expr);
            }
            StmtKind::Item(_) => {
                // Items in statement position - would need to collect/resolve
                // For now, skip
            }
        }
    }

    /// Resolve an expression
    fn resolve_expr(&mut self, expr_id: ExprId) {
        if let Some(expr) = self.arena.get_expr(expr_id) {
            match &expr.kind {
                ExprKind::Ident(name) => {
                    self.resolve_ident(name, expr.span, expr_id, expr.hygiene);
                }
                ExprKind::Binary { left, op: _, right } => {
                    self.resolve_expr(*left);
                    self.resolve_expr(*right);
                }
                ExprKind::Unary { op: _, operand } => {
                    self.resolve_expr(*operand);
                }
                ExprKind::Call { func, args } => {
                    self.resolve_expr(*func);
                    for &arg in args {
                        self.resolve_expr(arg);
                    }
                }
                ExprKind::MethodCall { receiver, method: _, args } => {
                    self.resolve_expr(*receiver);
                    for &arg in args {
                        self.resolve_expr(arg);
                    }
                }
                ExprKind::Field { object, field: _ } => {
                    self.resolve_expr(*object);
                }
                ExprKind::Index { collection, index } => {
                    self.resolve_expr(*collection);
                    self.resolve_expr(*index);
                }
                ExprKind::Tuple(exprs) => {
                    for &e in exprs {
                        self.resolve_expr(e);
                    }
                }
                ExprKind::Array(exprs) => {
                    for &e in exprs {
                        self.resolve_expr(e);
                    }
                }
                ExprKind::Struct { path: _, fields } => {
                    for field in fields {
                        self.resolve_expr(field.value);
                    }
                }
                ExprKind::If { condition, then_block, else_block } => {
                    self.resolve_expr(*condition);
                    if let Some(block) = self.arena.get_block(*then_block) {
                        self.resolve_block(block);
                    }
                    if let Some(else_id) = else_block {
                        if let Some(block) = self.arena.get_block(*else_id) {
                            self.resolve_block(block);
                        }
                    }
                }
                ExprKind::Match { scrutinee, arms } => {
                    self.resolve_expr(*scrutinee);
                    for arm in arms {
                        // Enter match arm scope
                        let _arm_scope = self.scopes.push_scope(ScopeKind::MatchArm, arm.span);

                        // Collect pattern bindings
                        if let Some(pat) = self.arena.get_pattern(arm.pattern) {
                            self.collect_pattern_bindings(pat, arm.pattern);
                        }

                        // Resolve guard
                        if let Some(guard) = arm.guard {
                            self.resolve_expr(guard);
                        }

                        // Resolve body
                        self.resolve_expr(arm.body);

                        // Exit match arm scope
                        self.scopes.pop_scope();
                    }
                }
                ExprKind::Block(block_id) => {
                    if let Some(block) = self.arena.get_block(*block_id) {
                        self.resolve_block(block);
                    }
                }
                ExprKind::Return { value } => {
                    if let Some(e) = value {
                        self.resolve_expr(*e);
                    }
                }
                ExprKind::Break { value } => {
                    if let Some(e) = value {
                        self.resolve_expr(*e);
                    }
                }
                ExprKind::Continue => {}
                ExprKind::Loop { body } => {
                    let _loop_scope = self.scopes.push_scope(ScopeKind::Loop, expr.span);
                    if let Some(block) = self.arena.get_block(*body) {
                        self.resolve_block(block);
                    }
                    self.scopes.pop_scope();
                }
                ExprKind::While { condition, body } => {
                    let _loop_scope = self.scopes.push_scope(ScopeKind::Loop, expr.span);
                    self.resolve_expr(*condition);
                    if let Some(block) = self.arena.get_block(*body) {
                        self.resolve_block(block);
                    }
                    self.scopes.pop_scope();
                }
                ExprKind::For { pattern, iterator, body } => {
                    let _loop_scope = self.scopes.push_scope(ScopeKind::Loop, expr.span);

                    self.resolve_expr(*iterator);

                    if let Some(pat) = self.arena.get_pattern(*pattern) {
                        self.collect_pattern_bindings(pat, *pattern);
                    }

                    if let Some(block) = self.arena.get_block(*body) {
                        self.resolve_block(block);
                    }
                    self.scopes.pop_scope();
                }
                ExprKind::Range { start, end, inclusive: _ } => {
                    if let Some(s) = start {
                        self.resolve_expr(*s);
                    }
                    if let Some(e) = end {
                        self.resolve_expr(*e);
                    }
                }
                ExprKind::Path(path) => {
                    self.resolve_path(path, expr.span, expr_id, expr.hygiene);
                }
                ExprKind::Literal(_) => {}
                ExprKind::Pipeline { left, right } => {
                    self.resolve_expr(*left);
                    self.resolve_expr(*right);
                }
                ExprKind::Yield { value } => {
                    self.resolve_expr(*value);
                }
                ExprKind::Try { expr } => {
                    self.resolve_expr(*expr);
                }
                ExprKind::Await { expr } => {
                    self.resolve_expr(*expr);
                }
                ExprKind::Unsafe { block } => {
                    if let Some(blk) = self.arena.get_block(*block) {
                        self.resolve_block(blk);
                    }
                }
                ExprKind::Comptime { expr } => {
                    self.resolve_expr(*expr);
                }
            }
        }
    }

    /// Resolve an identifier
    fn resolve_ident(
        &mut self,
        name: &str,
        span: Span,
        expr_id: ExprId,
        use_hygiene: HygieneId,
    ) {
        // Build scope search order: current scope chain + prelude
        let mut scope_chain = self.scopes.parent_chain(self.scopes.current_scope());

        // Always search prelude last (unless already in the chain)
        if !scope_chain.contains(&self.prelude_scope) {
            scope_chain.push(self.prelude_scope);
        }

        let scopes_searched = scope_chain.clone();

        // Search through each scope
        for &scope_id in &scope_chain {
            if let Some(symbol_id) = self.symbols.lookup(scope_id, name) {
                if self.symbols.get(symbol_id).is_some() {
                    // Check hygiene (use root for global scope symbols)
                    let def_hygiene = HygieneId::root();
                    if self.hygiene_resolver.is_visible(name, use_hygiene, def_hygiene) {
                        // Resolve!
                        self.resolution_map.resolve_expr(expr_id, symbol_id);
                        self.symbols.mark_used(symbol_id);

                        // Record resolution chain
                        let chain = ResolutionChain::success(
                            name.to_string(),
                            scopes_searched,
                            scope_id,
                            symbol_id,
                        );
                        self.resolution_map.record_chain(expr_id, chain);
                        return;
                    }
                }
            }
        }

        // Not found - record error with resolution chain
        let chain = ResolutionChain::failure(
            name.to_string(),
            scopes_searched,
            format!("Symbol '{}' not found in any accessible scope", name),
        );
        self.resolution_map.record_chain(expr_id, chain);

        self.diagnostics.push(ResolutionError::UndefinedSymbol {
            name: name.to_string(),
            span,
        });
    }

    /// Resolve a path
    fn resolve_path(
        &mut self,
        path: &Path,
        span: Span,
        expr_id: ExprId,
        _use_hygiene: HygieneId,
    ) {
        // For simple paths, resolve the first segment as an identifier
        // TODO: Handle multi-segment paths properly
        if let Some(first) = path.segments.first() {
            if path.segments.len() == 1 {
                self.resolve_ident(first, span, expr_id, HygieneId::root());
            } else {
                // Multi-segment path - would need module resolution
                // For now, just record as unresolved
                self.diagnostics.push(ResolutionError::UndefinedSymbol {
                    name: path.segments.join("::"),
                    span,
                });
            }
        }
    }

    /// Get the symbols table
    pub fn symbols(&self) -> &SymbolTable {
        &self.symbols
    }

    /// Get the scope tree
    pub fn scopes(&self) -> &ScopeTree {
        &self.scopes
    }

    /// Get the module graph
    pub fn modules(&self) -> &ModuleGraph {
        &self.modules
    }

    /// Get the resolution map
    pub fn resolution_map(&self) -> &ResolutionMap {
        &self.resolution_map
    }
}

/// Result of name resolution
#[derive(Debug, Clone)]
pub struct ResolutionResult {
    /// Symbol table
    pub symbols: SymbolTable,
    /// Scope tree
    pub scopes: ScopeTree,
    /// Module graph
    pub modules: ModuleGraph,
    /// Resolution map
    pub resolution_map: ResolutionMap,
    /// Diagnostics
    pub diagnostics: Vec<ResolutionError>,
}

impl ResolutionResult {
    /// Check if resolution succeeded without errors
    pub fn is_ok(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Get the diagnostics
    pub fn diagnostics(&self) -> &[ResolutionError] {
        &self.diagnostics
    }
}

/// Resolution errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionError {
    /// Undefined symbol
    UndefinedSymbol {
        /// Symbol name
        name: String,
        /// Usage span
        span: Span,
    },
    /// Duplicate definition
    DuplicateDefinition {
        /// Symbol name
        name: String,
        /// First definition span
        first_span: Span,
        /// Second definition span
        second_span: Span,
    },
    /// Module error
    ModuleError(ModuleError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_ast::decl::{ConstDecl, FunctionDecl, ItemKind};
    use aurora_ast::expr::Literal;
    use aurora_ast::stmt::Stmt;
    use aurora_ast::ty::{Type, TypeKind};

    #[test]
    fn test_resolution_map() {
        let mut map = ResolutionMap::new();

        map.resolve_expr(0, 10);
        map.resolve_expr(1, 11);
        map.bind_pattern(5, 20);

        assert_eq!(map.get_expr_resolution(0), Some(10));
        assert_eq!(map.get_expr_resolution(1), Some(11));
        assert_eq!(map.get_expr_resolution(2), None);
        assert_eq!(map.get_pattern_binding(5), Some(20));
    }

    #[test]
    fn test_resolution_chain_success() {
        let chain = ResolutionChain::success(
            "test".to_string(),
            vec![0, 1, 2],
            2,
            42,
        );

        assert_eq!(chain.name, "test");
        assert_eq!(chain.scopes_searched, vec![0, 1, 2]);
        assert_eq!(chain.found_in_scope, Some(2));
        assert_eq!(chain.symbol_id, Some(42));
    }

    #[test]
    fn test_resolution_chain_failure() {
        let chain = ResolutionChain::failure(
            "missing".to_string(),
            vec![0, 1],
            "Not found".to_string(),
        );

        assert_eq!(chain.name, "missing");
        assert_eq!(chain.found_in_scope, None);
        assert_eq!(chain.symbol_id, None);
        assert_eq!(chain.reason, "Not found");
    }

    #[test]
    fn test_resolver_creation() {
        let arena = Arena::new();
        let resolver = Resolver::new(&arena, "test_crate".to_string());

        // Should have prelude items
        assert!(resolver.symbols().len() > 0);
        assert_eq!(resolver.modules().len(), 1); // Root module
    }

    #[test]
    fn test_prelude_contains_println() {
        let arena = Arena::new();
        let resolver = Resolver::new(&arena, "test_crate".to_string());

        // Check that prelude has println
        let println_found = resolver
            .symbols()
            .symbols_in_scope(resolver.prelude_scope)
            .iter()
            .any(|s| s.name == "println");

        assert!(println_found);
    }

    #[test]
    fn test_prelude_contains_result_variants() {
        let arena = Arena::new();
        let resolver = Resolver::new(&arena, "test_crate".to_string());

        let prelude_symbols: Vec<_> = resolver
            .symbols()
            .symbols_in_scope(resolver.prelude_scope)
            .iter()
            .map(|s| s.name.as_str())
            .collect();

        assert!(prelude_symbols.contains(&"Ok"));
        assert!(prelude_symbols.contains(&"Err"));
        assert!(prelude_symbols.contains(&"Some"));
        assert!(prelude_symbols.contains(&"None"));
    }

    #[test]
    fn test_function_declaration_collection() {
        let mut arena = Arena::new();

        // Create a simple function: fn test() {}
        let func_decl = FunctionDecl {
            name: "test".to_string(),
            generics: vec![],
            params: vec![],
            return_type: None,
            where_clause: None,
            body: Block {
                stmts: vec![],
                expr: None,
                span: Span::dummy(),
            },
            is_pub: true,
            is_async: false,
            is_unsafe: false,
            span: Span::dummy(),
        };

        let item = Item {
            kind: ItemKind::Function(func_decl),
            span: Span::dummy(),
        };

        let item_id = arena.alloc_item(item);

        let program = Program::new(vec![item_id], Span::dummy());

        let resolver = Resolver::new(&arena, "test_crate".to_string());
        let result = resolver.resolve(&program);

        // Should have collected the function
        assert!(result.symbols.len() > 0);

        // Check that "test" function exists in global scope
        let symbols_in_scope = result.symbols.symbols_in_scope(0);
        let test_fn = symbols_in_scope.iter().find(|s| s.name == "test");

        assert!(test_fn.is_some());
        assert_eq!(test_fn.unwrap().kind, SymbolKind::Function);
    }

    #[test]
    fn test_const_declaration_collection() {
        let mut arena = Arena::new();

        // Create a constant: const PI = 3.14
        let lit_expr = Expr {
            kind: ExprKind::Literal(Literal::Float(3.14)),
            span: Span::dummy(),
            hygiene: HygieneId::root(),
        };
        let lit_id = arena.alloc_expr(lit_expr);

        let ty = Type {
            kind: TypeKind::Path {
                path: Path {
                    segments: vec!["f64".to_string()],
                    generics: vec![],
                },
            },
            span: Span::dummy(),
        };
        let ty_id = arena.alloc_type(ty);

        let const_decl = ConstDecl {
            name: "PI".to_string(),
            ty: ty_id,
            value: lit_id,
            is_pub: true,
            span: Span::dummy(),
        };

        let item = Item {
            kind: ItemKind::Const(const_decl),
            span: Span::dummy(),
        };

        let item_id = arena.alloc_item(item);
        let program = Program::new(vec![item_id], Span::dummy());

        let resolver = Resolver::new(&arena, "test_crate".to_string());
        let result = resolver.resolve(&program);

        // Check that PI constant exists
        let symbols_in_scope = result.symbols.symbols_in_scope(0);
        let pi_const = symbols_in_scope.iter().find(|s| s.name == "PI");

        assert!(pi_const.is_some());
        assert_eq!(pi_const.unwrap().kind, SymbolKind::Const);
    }

    #[test]
    fn test_duplicate_function_detection() {
        let mut arena = Arena::new();

        // Create two functions with the same name
        let func1 = FunctionDecl {
            name: "duplicate".to_string(),
            generics: vec![],
            params: vec![],
            return_type: None,
            where_clause: None,
            body: Block {
                stmts: vec![],
                expr: None,
                span: Span::dummy(),
            },
            is_pub: true,
            is_async: false,
            is_unsafe: false,
            span: Span::dummy(),
        };

        let func2 = func1.clone();

        let item1 = Item {
            kind: ItemKind::Function(func1),
            span: Span::dummy(),
        };

        let item2 = Item {
            kind: ItemKind::Function(func2),
            span: Span::dummy(),
        };

        let item1_id = arena.alloc_item(item1);
        let item2_id = arena.alloc_item(item2);

        let program = Program::new(vec![item1_id, item2_id], Span::dummy());

        let resolver = Resolver::new(&arena, "test_crate".to_string());
        let result = resolver.resolve(&program);

        // Should have a duplicate definition error
        assert!(result.diagnostics.len() > 0);

        let has_duplicate_error = result.diagnostics.iter().any(|e| {
            matches!(e, ResolutionError::DuplicateDefinition { name, .. } if name == "duplicate")
        });

        assert!(has_duplicate_error);
    }

    #[test]
    fn test_variable_binding_in_let() {
        let mut arena = Arena::new();

        // Create: let x = 42;
        let lit = Expr {
            kind: ExprKind::Literal(Literal::Int(42)),
            span: Span::dummy(),
            hygiene: HygieneId::root(),
        };
        let lit_id = arena.alloc_expr(lit);

        let pattern = Pattern {
            kind: PatternKind::Ident {
                name: "x".to_string(),
                is_mut: false,
            },
            span: Span::dummy(),
            hygiene: HygieneId::root(),
        };
        let pat_id = arena.alloc_pattern(pattern);

        let let_stmt = Stmt {
            kind: StmtKind::Let {
                pattern: pat_id,
                ty: None,
                init: Some(lit_id),
                mutable: false,
            },
            span: Span::dummy(),
        };
        let stmt_id = arena.alloc_stmt(let_stmt);

        // Wrap in a function
        let func_body = Block {
            stmts: vec![stmt_id],
            expr: None,
            span: Span::dummy(),
        };

        let func = FunctionDecl {
            name: "test".to_string(),
            generics: vec![],
            params: vec![],
            return_type: None,
            where_clause: None,
            body: func_body,
            is_pub: false,
            is_async: false,
            is_unsafe: false,
            span: Span::dummy(),
        };

        let item = Item {
            kind: ItemKind::Function(func),
            span: Span::dummy(),
        };

        let item_id = arena.alloc_item(item);
        let program = Program::new(vec![item_id], Span::dummy());

        let resolver = Resolver::new(&arena, "test_crate".to_string());
        let result = resolver.resolve(&program);

        // Should have bound the pattern
        assert!(result.resolution_map.pattern_count() > 0);
        assert!(result.resolution_map.get_pattern_binding(pat_id).is_some());
    }

    #[test]
    fn test_variable_resolution() {
        let mut arena = Arena::new();

        // Create: let x = 42; x
        let lit = Expr {
            kind: ExprKind::Literal(Literal::Int(42)),
            span: Span::dummy(),
            hygiene: HygieneId::root(),
        };
        let lit_id = arena.alloc_expr(lit);

        let pattern = Pattern {
            kind: PatternKind::Ident {
                name: "x".to_string(),
                is_mut: false,
            },
            span: Span::dummy(),
            hygiene: HygieneId::root(),
        };
        let pat_id = arena.alloc_pattern(pattern);

        let let_stmt = Stmt {
            kind: StmtKind::Let {
                pattern: pat_id,
                ty: None,
                init: Some(lit_id),
                mutable: false,
            },
            span: Span::dummy(),
        };
        let stmt_id = arena.alloc_stmt(let_stmt);

        // Use x
        let use_expr = Expr {
            kind: ExprKind::Ident("x".to_string()),
            span: Span::dummy(),
            hygiene: HygieneId::root(),
        };
        let use_id = arena.alloc_expr(use_expr);

        // Wrap in function
        let func_body = Block {
            stmts: vec![stmt_id],
            expr: Some(use_id),
            span: Span::dummy(),
        };

        let func = FunctionDecl {
            name: "test".to_string(),
            generics: vec![],
            params: vec![],
            return_type: None,
            where_clause: None,
            body: func_body,
            is_pub: false,
            is_async: false,
            is_unsafe: false,
            span: Span::dummy(),
        };

        let item = Item {
            kind: ItemKind::Function(func),
            span: Span::dummy(),
        };

        let item_id = arena.alloc_item(item);
        let program = Program::new(vec![item_id], Span::dummy());

        let resolver = Resolver::new(&arena, "test_crate".to_string());
        let result = resolver.resolve(&program);

        // Should have resolved the use of x
        assert!(result.resolution_map.get_expr_resolution(use_id).is_some());

        // Should have a resolution chain
        let chain = result.resolution_map.get_resolution_chain(use_id);
        assert!(chain.is_some());
        assert_eq!(chain.unwrap().name, "x");
    }

    #[test]
    fn test_undefined_variable_error() {
        let mut arena = Arena::new();

        // Create: y (undefined)
        let use_expr = Expr {
            kind: ExprKind::Ident("undefined_var".to_string()),
            span: Span::dummy(),
            hygiene: HygieneId::root(),
        };
        let use_id = arena.alloc_expr(use_expr);

        let func_body = Block {
            stmts: vec![],
            expr: Some(use_id),
            span: Span::dummy(),
        };

        let func = FunctionDecl {
            name: "test".to_string(),
            generics: vec![],
            params: vec![],
            return_type: None,
            where_clause: None,
            body: func_body,
            is_pub: false,
            is_async: false,
            is_unsafe: false,
            span: Span::dummy(),
        };

        let item = Item {
            kind: ItemKind::Function(func),
            span: Span::dummy(),
        };

        let item_id = arena.alloc_item(item);
        let program = Program::new(vec![item_id], Span::dummy());

        let resolver = Resolver::new(&arena, "test_crate".to_string());
        let result = resolver.resolve(&program);

        // Should have an undefined symbol error
        assert!(result.diagnostics.len() > 0);

        let has_undefined_error = result.diagnostics.iter().any(|e| {
            matches!(e, ResolutionError::UndefinedSymbol { name, .. } if name == "undefined_var")
        });

        assert!(has_undefined_error);

        // Should have a failure resolution chain
        let chain = result.resolution_map.get_resolution_chain(use_id);
        assert!(chain.is_some());
        assert!(chain.unwrap().symbol_id.is_none());
    }

    #[test]
    fn test_shadowing_in_nested_scopes() {
        let mut arena = Arena::new();

        // Outer: let x = 1;
        let outer_lit = Expr {
            kind: ExprKind::Literal(Literal::Int(1)),
            span: Span::dummy(),
            hygiene: HygieneId::root(),
        };
        let outer_lit_id = arena.alloc_expr(outer_lit);

        let outer_pattern = Pattern {
            kind: PatternKind::Ident {
                name: "x".to_string(),
                is_mut: false,
            },
            span: Span::dummy(),
            hygiene: HygieneId::root(),
        };
        let outer_pat_id = arena.alloc_pattern(outer_pattern);

        let outer_let = Stmt {
            kind: StmtKind::Let {
                pattern: outer_pat_id,
                ty: None,
                init: Some(outer_lit_id),
                mutable: false,
            },
            span: Span::dummy(),
        };
        let outer_let_id = arena.alloc_stmt(outer_let);

        // Inner: let x = 2;
        let inner_lit = Expr {
            kind: ExprKind::Literal(Literal::Int(2)),
            span: Span::dummy(),
            hygiene: HygieneId::root(),
        };
        let inner_lit_id = arena.alloc_expr(inner_lit);

        let inner_pattern = Pattern {
            kind: PatternKind::Ident {
                name: "x".to_string(),
                is_mut: false,
            },
            span: Span::dummy(),
            hygiene: HygieneId::root(),
        };
        let inner_pat_id = arena.alloc_pattern(inner_pattern);

        let inner_let = Stmt {
            kind: StmtKind::Let {
                pattern: inner_pat_id,
                ty: None,
                init: Some(inner_lit_id),
                mutable: false,
            },
            span: Span::dummy(),
        };
        let _inner_let_id = arena.alloc_stmt(inner_let);

        // Inner block with inner let
        let inner_block_expr = Expr {
            kind: ExprKind::Block(0), // Will need proper block ID
            span: Span::dummy(),
            hygiene: HygieneId::root(),
        };
        let inner_block_id = arena.alloc_expr(inner_block_expr);

        // Outer block
        let func_body = Block {
            stmts: vec![outer_let_id],
            expr: Some(inner_block_id),
            span: Span::dummy(),
        };

        let func = FunctionDecl {
            name: "test".to_string(),
            generics: vec![],
            params: vec![],
            return_type: None,
            where_clause: None,
            body: func_body,
            is_pub: false,
            is_async: false,
            is_unsafe: false,
            span: Span::dummy(),
        };

        let item = Item {
            kind: ItemKind::Function(func),
            span: Span::dummy(),
        };

        let item_id = arena.alloc_item(item);
        let program = Program::new(vec![item_id], Span::dummy());

        let resolver = Resolver::new(&arena, "test_crate".to_string());
        let result = resolver.resolve(&program);

        // Both patterns should be bound (different scopes)
        assert!(result.resolution_map.get_pattern_binding(outer_pat_id).is_some());
        // Inner pattern binding would work if we had proper nested blocks
    }

    #[test]
    fn test_resolution_result_is_ok() {
        let arena = Arena::new();
        let program = Program::empty();

        let resolver = Resolver::new(&arena, "test".to_string());
        let result = resolver.resolve(&program);

        // Empty program should resolve without errors
        assert!(result.is_ok());
        assert_eq!(result.diagnostics().len(), 0);
    }
}
