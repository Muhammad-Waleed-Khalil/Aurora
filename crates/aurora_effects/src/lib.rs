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

// Pipeline integration stub
use aurora_ast::Ast;
use std::sync::Arc;

/// Effect checker for pipeline integration
pub struct EffectChecker {
    diagnostics: Arc<dyn Send + Sync>,
}

impl EffectChecker {
    /// Create a new effect checker with diagnostic collector
    pub fn new<D: Send + Sync + 'static>(diagnostics: Arc<D>) -> Self {
        Self {
            diagnostics: diagnostics as Arc<dyn Send + Sync>,
        }
    }

    /// Check effects and borrow rules in the AST
    pub fn check(&mut self, ast: Ast) -> Ast {
        // TODO: Implement actual effect checking and borrow checking
        // For now, just return the AST unchanged
        ast
    }
}
