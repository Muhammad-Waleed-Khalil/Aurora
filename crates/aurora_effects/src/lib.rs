//! Aurora Effects & Borrow Checking System
//!
//! This crate implements Aurora's effect system and borrow checker:
//!
//! # Effect System
//! - Effect tracking (IO, Alloc, Parallel, Unsafe)
//! - Effect polymorphism with effect variables
//! - Subeffecting partial order
//! - Effect composition and normalization
//!
//! # Borrow Checker
//! - Advisory mode (warnings, not errors)
//! - Dataflow analysis for borrows
//! - Lifetime tracking and inference
//! - Borrow conflict detection
//!
//! # ARC Insertion
//! - Automatic reference counting at uncertain escape points
//! - Escape analysis
//! - Advisory emission for ARC sites
//!
//! # Strict Mode
//! - Convert advisories to errors
//! - Require explicit lifetimes
//! - Disallow ARC insertion
//! - Enforce all borrow rules strictly
//!
//! # Example
//!
//! ```rust
//! use aurora_effects::{BorrowChecker, BorrowKind, Region};
//!
//! let mut checker = BorrowChecker::new();
//! let region = Region::static_region(false);
//!
//! // Record shared borrows
//! checker.record_borrow(BorrowKind::Shared, "x".to_string(), region.clone(), 1);
//! checker.record_borrow(BorrowKind::Shared, "x".to_string(), region, 2);
//!
//! // Check for advisories
//! assert!(!checker.has_advisories());
//! ```

pub mod arc;
pub mod borrow;
pub mod effects;
pub mod lifetimes;
pub mod strict;

// Re-export commonly used types
pub use arc::{ArcContext, ArcError, ArcOp, ArcSite, EscapeInfo, EscapeKind};
pub use borrow::{
    Advisory, Borrow, BorrowChecker, BorrowConflict, BorrowDataflow, BorrowError, BorrowKind,
};
pub use effects::{
    check_effect_allowed, compose_effects, extract_effects, is_subeffect, Effect,
    EffectInferContext, EffectSubstitution, EffectTracker,
};
pub use lifetimes::{Lifetime, LifetimeConstraint, LifetimeContext, LifetimeError, Region};
pub use strict::{StrictChecker, StrictConfig, StrictError, StrictModeEnforcer};

// Pipeline integration
use aurora_ast::{Ast, ExprId, ExprKind, ItemKind, StmtKind};
use aurora_types::{EffectSet, Type, TypeMap};
use std::collections::HashMap;
use std::sync::Arc;

/// Diagnostic collector trait for reporting advisories
pub trait DiagnosticCollector: Send + Sync {
    /// Report an advisory message
    fn report_advisory(&self, message: String, location: usize, severity: u8);
}

/// Effect checker for pipeline integration
pub struct EffectChecker {
    diagnostics: Arc<dyn DiagnosticCollector>,
    /// Effect tracker for current context
    effect_tracker: EffectTracker,
    /// Borrow checker
    borrow_checker: BorrowChecker,
    /// ARC context
    arc_context: ArcContext,
    /// Lifetime context
    lifetime_context: LifetimeContext,
    /// Strict mode checker
    strict_checker: Option<StrictChecker>,
    /// Type map (from type checker)
    type_map: TypeMap,
    /// Expression effects cache
    expr_effects: HashMap<ExprId, EffectSet>,
    /// Current scope depth
    scope_depth: usize,
}

impl EffectChecker {
    /// Create a new effect checker with diagnostic collector
    pub fn new<D: DiagnosticCollector + 'static>(diagnostics: Arc<D>) -> Self {
        Self {
            diagnostics: diagnostics as Arc<dyn DiagnosticCollector>,
            effect_tracker: EffectTracker::new(),
            borrow_checker: BorrowChecker::new(),
            arc_context: ArcContext::new(false),
            lifetime_context: LifetimeContext::new(),
            strict_checker: None,
            type_map: TypeMap::new(),
            expr_effects: HashMap::new(),
            scope_depth: 0,
        }
    }

    /// Enable strict mode
    pub fn with_strict_mode(mut self, config: StrictConfig) -> Self {
        let disallow_arc = config.disallow_arc;
        self.strict_checker = Some(StrictChecker::new(config));
        self.arc_context.set_strict_mode(disallow_arc);
        self
    }

    /// Set type map from type checker
    pub fn with_type_map(mut self, type_map: TypeMap) -> Self {
        self.type_map = type_map;
        self
    }

    /// Check effects and borrow rules in the AST
    pub fn check(&mut self, ast: Ast) -> Ast {
        // Traverse all items in the program
        for &item_id in &ast.items {
            self.check_item(item_id);
        }

        // Report all advisories
        self.report_advisories();

        // In strict mode, check all rules
        if let Some(ref mut strict) = self.strict_checker {
            let _ = strict.check_all(
                &self.borrow_checker,
                &self.arc_context,
                &self.lifetime_context,
            );

            // Report strict mode errors
            for error in strict.errors() {
                self.diagnostics.report_advisory(
                    error.to_string(),
                    0,
                    2, // Error severity
                );
            }
        }

        ast
    }

    /// Check an item (function, struct, etc.)
    fn check_item(&mut self, item_id: u32) {
        // In a real implementation, we'd look up the item in the AST arena
        // For now, this is a simplified stub that demonstrates the structure
        // The actual implementation would traverse ItemKind variants

        // Stub: we would check function bodies, struct definitions, etc.
        // Each would push/pop effect contexts and scopes appropriately
    }

    /// Check an expression and return its effect
    fn check_expr(&mut self, expr_id: ExprId, expr: &ExprKind) -> EffectSet {
        // Check if we've already computed this
        if let Some(&effects) = self.expr_effects.get(&expr_id) {
            return effects;
        }

        let effects = match expr {
            ExprKind::Literal(_) => EffectSet::PURE,

            ExprKind::Ident(name) => {
                // Variable access is pure, but record as borrow
                let region = Region::new(self.lifetime_context.fresh(), false);
                self.borrow_checker.record_borrow(
                    BorrowKind::Shared,
                    name.clone(),
                    region,
                    0,
                );
                EffectSet::PURE
            }

            ExprKind::Call { func, args } => {
                // Check function and arguments
                let func_effects = self.check_expr_by_id(*func);
                let mut effects = func_effects;

                for &arg in args {
                    let arg_effects = self.check_expr_by_id(arg);
                    effects = compose_effects(effects, arg_effects);
                }

                // Function calls may have IO, Alloc, etc. based on type
                // For now, conservatively assume IO
                effects.union(EffectSet::IO)
            }

            ExprKind::Binary { op, left, right } => {
                let left_effects = self.check_expr_by_id(*left);
                let right_effects = self.check_expr_by_id(*right);
                compose_effects(left_effects, right_effects)
            }

            ExprKind::Unary { op, operand } => {
                self.check_expr_by_id(*operand)
            }

            ExprKind::If { condition, then_block, else_block } => {
                self.push_scope();
                let cond_effects = self.check_expr_by_id(*condition);
                // Would check blocks here
                self.pop_scope();
                cond_effects
            }

            ExprKind::Match { scrutinee, arms } => {
                let scrut_effects = self.check_expr_by_id(*scrutinee);
                // Would check match arms here
                scrut_effects
            }

            ExprKind::Return { value } => {
                if let Some(val_id) = value {
                    let effects = self.check_expr_by_id(*val_id);
                    // Analyze escape for ARC insertion
                    self.arc_context.process_escape(
                        format!("return_{}", val_id),
                        0,
                    ).ok();
                    effects
                } else {
                    EffectSet::PURE
                }
            }

            _ => EffectSet::PURE,
        };

        // Cache the result
        self.expr_effects.insert(expr_id, effects);

        // Check if effects are allowed in current context
        if let Err(e) = self.effect_tracker.check_allowed(effects) {
            self.report_effect_error(e, 0);
        }

        effects
    }

    /// Check expression by ID (stub - would look up in AST)
    fn check_expr_by_id(&mut self, expr_id: ExprId) -> EffectSet {
        // In a real implementation, would look up the expression in the AST arena
        // For now, return PURE as a stub
        EffectSet::PURE
    }

    /// Push a new scope
    fn push_scope(&mut self) {
        self.scope_depth += 1;
        self.borrow_checker.push_scope();
        self.lifetime_context.push_scope();
    }

    /// Pop current scope
    fn pop_scope(&mut self) {
        self.scope_depth = self.scope_depth.saturating_sub(1);
        self.borrow_checker.pop_scope();
        self.lifetime_context.pop_scope();
    }

    /// Report effect error
    fn report_effect_error(&self, error: effects::EffectError, location: usize) {
        self.diagnostics.report_advisory(
            format!("Effect error: {}", error),
            location,
            1, // Warning severity
        );
    }

    /// Report all collected advisories
    fn report_advisories(&self) {
        // Report borrow checker advisories
        for advisory in self.borrow_checker.advisories() {
            self.diagnostics.report_advisory(
                advisory.error.clone(),
                advisory.location,
                advisory.severity,
            );
        }

        // Report ARC advisories
        for advisory in self.arc_context.advisories() {
            self.diagnostics.report_advisory(
                advisory.error.clone(),
                advisory.location,
                advisory.severity,
            );
        }
    }

    /// Get borrow checker reference
    pub fn borrow_checker(&self) -> &BorrowChecker {
        &self.borrow_checker
    }

    /// Get ARC context reference
    pub fn arc_context(&self) -> &ArcContext {
        &self.arc_context
    }

    /// Get lifetime context reference
    pub fn lifetime_context(&self) -> &LifetimeContext {
        &self.lifetime_context
    }
}

/// Null diagnostic collector for testing
pub struct NullDiagnostics;

impl DiagnosticCollector for NullDiagnostics {
    fn report_advisory(&self, _message: String, _location: usize, _severity: u8) {
        // Do nothing
    }
}

/// Collecting diagnostic collector for testing
#[derive(Debug, Clone, Default)]
pub struct CollectingDiagnostics {
    messages: Arc<std::sync::Mutex<Vec<(String, usize, u8)>>>,
}

impl CollectingDiagnostics {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub fn messages(&self) -> Vec<(String, usize, u8)> {
        self.messages.lock().unwrap().clone()
    }

    pub fn count(&self) -> usize {
        self.messages.lock().unwrap().len()
    }
}

impl DiagnosticCollector for CollectingDiagnostics {
    fn report_advisory(&self, message: String, location: usize, severity: u8) {
        self.messages.lock().unwrap().push((message, location, severity));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_checker_creation() {
        let diag = Arc::new(NullDiagnostics);
        let checker = EffectChecker::new(diag);

        assert!(checker.borrow_checker().advisories().is_empty());
        assert!(checker.arc_context().sites().is_empty());
    }

    #[test]
    fn test_effect_checker_with_strict_mode() {
        let diag = Arc::new(NullDiagnostics);
        let checker = EffectChecker::new(diag).with_strict_mode(StrictConfig::strict());

        assert!(checker.strict_checker.is_some());
    }

    #[test]
    fn test_effect_checker_advisory_mode() {
        let diag = Arc::new(CollectingDiagnostics::new());
        let mut checker = EffectChecker::new(diag.clone());

        // Create an empty AST
        let ast = Ast::empty();
        let result = checker.check(ast);

        // Should complete without errors
        assert_eq!(result.items.len(), 0);
    }

    #[test]
    fn test_borrow_checking_shared() {
        let diag = Arc::new(NullDiagnostics);
        let mut checker = EffectChecker::new(diag);

        // Simulate shared borrows
        let region = Region::static_region(false);
        checker.borrow_checker.record_borrow(
            BorrowKind::Shared,
            "x".to_string(),
            region.clone(),
            1,
        );
        checker.borrow_checker.record_borrow(
            BorrowKind::Shared,
            "x".to_string(),
            region,
            2,
        );

        // Shared borrows should not generate advisories
        assert_eq!(checker.borrow_checker.advisory_count(), 0);
    }

    #[test]
    fn test_borrow_checking_mutable_conflict() {
        let diag = Arc::new(NullDiagnostics);
        let mut checker = EffectChecker::new(diag);

        // Simulate conflicting mutable borrows
        let region = Region::static_region(true);
        checker.borrow_checker.record_borrow(
            BorrowKind::Mutable,
            "x".to_string(),
            region.clone(),
            1,
        );
        checker.borrow_checker.record_borrow(
            BorrowKind::Mutable,
            "x".to_string(),
            region,
            2,
        );

        // Should generate advisory
        assert!(checker.borrow_checker.has_advisories());
    }

    #[test]
    fn test_effect_tracking_pure() {
        let diag = Arc::new(NullDiagnostics);
        let checker = EffectChecker::new(diag);

        assert_eq!(checker.effect_tracker.current(), EffectSet::PURE);
    }

    #[test]
    fn test_effect_tracking_io() {
        let diag = Arc::new(NullDiagnostics);
        let mut checker = EffectChecker::new(diag);

        checker.effect_tracker.add_effect(EffectSet::IO);
        assert_eq!(checker.effect_tracker.current(), EffectSet::IO);
    }

    #[test]
    fn test_arc_insertion_advisory() {
        let diag = Arc::new(NullDiagnostics);
        let mut checker = EffectChecker::new(diag);

        // Process an escape that requires ARC
        checker.arc_context.process_escape("heap_value".to_string(), 1).ok();

        // Should have inserted ARC sites
        assert!(checker.arc_context.sites().len() > 0);
    }

    #[test]
    fn test_arc_blocked_in_strict_mode() {
        let diag = Arc::new(NullDiagnostics);
        let mut checker = EffectChecker::new(diag)
            .with_strict_mode(StrictConfig::strict());

        // Try to insert ARC in strict mode
        let result = checker.arc_context.insert_arc(
            "x".to_string(),
            1,
            "test".to_string(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_lifetime_tracking() {
        let diag = Arc::new(NullDiagnostics);
        let mut checker = EffectChecker::new(diag);

        let lt1 = checker.lifetime_context.fresh();
        checker.push_scope();
        let lt2 = checker.lifetime_context.fresh();

        // Outer lifetime should outlive inner
        assert!(checker.lifetime_context.outlives(&lt1, &lt2));
    }

    #[test]
    fn test_scope_management() {
        let diag = Arc::new(NullDiagnostics);
        let mut checker = EffectChecker::new(diag);

        assert_eq!(checker.scope_depth, 0);

        checker.push_scope();
        assert_eq!(checker.scope_depth, 1);

        checker.push_scope();
        assert_eq!(checker.scope_depth, 2);

        checker.pop_scope();
        assert_eq!(checker.scope_depth, 1);

        checker.pop_scope();
        assert_eq!(checker.scope_depth, 0);
    }

    #[test]
    fn test_effect_composition() {
        let e1 = EffectSet::IO;
        let e2 = EffectSet::ALLOC;
        let composed = compose_effects(e1, e2);

        assert!(composed.has(EffectSet::IO));
        assert!(composed.has(EffectSet::ALLOC));
    }

    #[test]
    fn test_subeffecting() {
        assert!(is_subeffect(EffectSet::PURE, EffectSet::IO));
        assert!(is_subeffect(EffectSet::IO, EffectSet::UNSAFE));
        assert!(!is_subeffect(EffectSet::UNSAFE, EffectSet::IO));
    }

    #[test]
    fn test_move_tracking() {
        let diag = Arc::new(NullDiagnostics);
        let mut checker = EffectChecker::new(diag);

        // Record a move
        checker.borrow_checker.record_move("x".to_string(), 1);

        // Try to borrow after move
        let region = Region::static_region(false);
        checker.borrow_checker.record_borrow(
            BorrowKind::Shared,
            "x".to_string(),
            region,
            2,
        );

        // Should generate advisory about use-after-move
        assert!(checker.borrow_checker.has_advisories());
    }

    #[test]
    fn test_escape_analysis() {
        let diag = Arc::new(NullDiagnostics);
        let mut checker = EffectChecker::new(diag);

        // Analyze different escape kinds
        let kind1 = checker.arc_context.analyze_escape("return_value".to_string(), 1);
        assert_eq!(kind1, EscapeKind::ReturnEscape);

        let kind2 = checker.arc_context.analyze_escape("heap_alloc".to_string(), 2);
        assert_eq!(kind2, EscapeKind::HeapEscape);

        let kind3 = checker.arc_context.analyze_escape("local_var".to_string(), 3);
        assert_eq!(kind3, EscapeKind::NoEscape);
    }

    #[test]
    fn test_advisory_reporting() {
        let diag = Arc::new(CollectingDiagnostics::new());
        let mut checker = EffectChecker::new(diag.clone());

        // Generate some advisories
        let region = Region::static_region(true);
        checker.borrow_checker.record_borrow(
            BorrowKind::Mutable,
            "x".to_string(),
            region.clone(),
            1,
        );
        checker.borrow_checker.record_borrow(
            BorrowKind::Mutable,
            "x".to_string(),
            region,
            2,
        );

        // Report them
        checker.report_advisories();

        // Should have reported the advisory
        assert!(diag.count() > 0);
    }

    #[test]
    fn test_strict_mode_errors() {
        let diag = Arc::new(CollectingDiagnostics::new());
        let mut checker = EffectChecker::new(diag.clone())
            .with_strict_mode(StrictConfig::strict());

        // Create borrow violations
        let region = Region::static_region(true);
        checker.borrow_checker.record_borrow(
            BorrowKind::Mutable,
            "x".to_string(),
            region.clone(),
            1,
        );
        checker.borrow_checker.record_borrow(
            BorrowKind::Mutable,
            "x".to_string(),
            region,
            2,
        );

        // Check the AST (empty)
        let ast = Ast::empty();
        checker.check(ast);

        // Should have reported errors (severity 2)
        let messages = diag.messages();
        assert!(messages.iter().any(|(_, _, severity)| *severity == 2));
    }

    #[test]
    fn test_effect_caching() {
        let diag = Arc::new(NullDiagnostics);
        let mut checker = EffectChecker::new(diag);

        // Insert cached effect
        checker.expr_effects.insert(0, EffectSet::IO);

        // Should retrieve from cache
        assert_eq!(checker.expr_effects.get(&0), Some(&EffectSet::IO));
    }

    #[test]
    fn test_path_overlap_detection() {
        // Test internal path overlap logic through public API
        let diag = Arc::new(NullDiagnostics);
        let mut checker = EffectChecker::new(diag);

        // Create borrows to test overlap
        let region = Region::static_region(false);
        checker.borrow_checker.record_borrow(
            BorrowKind::Shared,
            "x".to_string(),
            region.clone(),
            1,
        );

        // Borrow of same variable should not conflict for shared
        checker.borrow_checker.record_borrow(
            BorrowKind::Shared,
            "x".to_string(),
            region,
            2,
        );

        // No advisories should be generated
        assert_eq!(checker.borrow_checker.advisory_count(), 0);
    }

    #[test]
    fn test_lifetime_constraints() {
        let diag = Arc::new(NullDiagnostics);
        let mut checker = EffectChecker::new(diag);

        let l1 = checker.lifetime_context.fresh();
        let l2 = checker.lifetime_context.fresh();

        // Add constraint
        checker.lifetime_context.add_outlives(l1.clone(), l2.clone());

        // Check constraints
        assert!(checker.lifetime_context.check_constraints().is_ok());
    }

    #[test]
    fn test_diagnostic_collector_trait() {
        let diag = Arc::new(CollectingDiagnostics::new());

        // Test the trait
        diag.report_advisory("Test message".to_string(), 42, 1);

        let messages = diag.messages();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].0, "Test message");
        assert_eq!(messages[0].1, 42);
        assert_eq!(messages[0].2, 1);
    }

    #[test]
    fn test_comprehensive_checking() {
        // Test a complete checking workflow
        let diag = Arc::new(CollectingDiagnostics::new());
        let mut checker = EffectChecker::new(diag.clone());

        // Simulate checking an expression with effects
        checker.push_scope();

        // Variable access
        let region = Region::static_region(false);
        checker.borrow_checker.record_borrow(
            BorrowKind::Shared,
            "x".to_string(),
            region,
            1,
        );

        // Add effect
        checker.effect_tracker.add_effect(EffectSet::IO);

        // Process escape
        checker.arc_context.process_escape("value".to_string(), 2).ok();

        checker.pop_scope();

        // Report advisories
        checker.report_advisories();

        // Check the AST
        let ast = Ast::empty();
        let result = checker.check(ast);

        // Should complete successfully
        assert_eq!(result.items.len(), 0);
    }

    #[test]
    fn test_effect_polymorphism() {
        let mut ctx = EffectInferContext::new();

        // Create effect variable
        let var = ctx.fresh_var();
        assert!(matches!(var, Effect::Var(_)));

        // Unify with concrete effect
        ctx.unify(&var, &Effect::Concrete(EffectSet::IO)).unwrap();

        // Apply substitution
        let result = ctx.apply(&var);
        assert_eq!(result, Effect::Concrete(EffectSet::IO));
    }

    #[test]
    fn test_borrow_dataflow() {
        let mut dataflow = BorrowDataflow::new();

        let mut borrows = std::collections::HashSet::new();
        borrows.insert(1);
        borrows.insert(2);

        dataflow.set_borrows(10, borrows.clone());

        // Check live borrows
        assert_eq!(dataflow.live_borrows(10), borrows);
    }

    #[test]
    fn test_arc_optimizer() {
        use crate::arc::{ArcOptimizer, ArcSite, ArcOp};

        let mut optimizer = ArcOptimizer::new();
        let mut sites = vec![
            ArcSite {
                path: "x".to_string(),
                operation: ArcOp::Retain,
                location: 1,
                reason: "test".to_string(),
            },
            ArcSite {
                path: "x".to_string(),
                operation: ArcOp::Release,
                location: 1,
                reason: "test".to_string(),
            },
        ];

        optimizer.optimize(&mut sites);

        // Should remove redundant pair
        assert_eq!(sites.len(), 0);
    }
}
