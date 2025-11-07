//! Lifetime Tracking for Borrow Checker
//!
//! This module implements lifetime tracking and analysis for Aurora's
//! borrow checker, including:
//! - Lifetime variables and constraints
//! - Lifetime inference
//! - Region-based lifetime analysis
//! - Lifetime variance checking

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Lifetime ID
pub type LifetimeId = u32;

/// Lifetime
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Lifetime {
    /// Static lifetime ('static)
    Static,
    /// Named lifetime ('a, 'b, etc.)
    Named(String),
    /// Anonymous lifetime (inferred)
    Anon(LifetimeId),
    /// Erased lifetime (runtime-determined)
    Erased,
}

impl Lifetime {
    /// Check if lifetime is static
    pub fn is_static(&self) -> bool {
        matches!(self, Lifetime::Static)
    }

    /// Check if lifetime is named
    pub fn is_named(&self) -> bool {
        matches!(self, Lifetime::Named(_))
    }

    /// Check if lifetime is anonymous
    pub fn is_anon(&self) -> bool {
        matches!(self, Lifetime::Anon(_))
    }
}

/// Lifetime constraint
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifetimeConstraint {
    /// 'a outlives 'b ('a: 'b)
    Outlives(Lifetime, Lifetime),
    /// 'a equals 'b ('a == 'b)
    Equals(Lifetime, Lifetime),
}

/// Lifetime error
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LifetimeError {
    /// Lifetime constraint violation
    #[error("Lifetime constraint violated: {0:?}")]
    ConstraintViolation(LifetimeConstraint),

    /// Conflicting lifetimes
    #[error("Conflicting lifetimes: {0:?} and {1:?}")]
    Conflict(Lifetime, Lifetime),

    /// Undefined lifetime
    #[error("Undefined lifetime: {0:?}")]
    UndefinedLifetime(Lifetime),

    /// Lifetime escapes scope
    #[error("Lifetime {0:?} escapes its scope")]
    Escapes(Lifetime),
}

/// Lifetime result type
pub type LifetimeResult<T> = Result<T, LifetimeError>;

/// Lifetime context for inference
#[derive(Debug, Clone)]
pub struct LifetimeContext {
    /// Next anonymous lifetime ID
    next_id: LifetimeId,
    /// Lifetime constraints
    constraints: Vec<LifetimeConstraint>,
    /// Lifetime scopes (lifetime -> scope depth)
    scopes: HashMap<Lifetime, usize>,
    /// Current scope depth
    depth: usize,
}

impl LifetimeContext {
    /// Create new lifetime context
    pub fn new() -> Self {
        Self {
            next_id: 0,
            constraints: Vec::new(),
            scopes: HashMap::new(),
            depth: 0,
        }
    }

    /// Generate fresh anonymous lifetime
    pub fn fresh(&mut self) -> Lifetime {
        let id = self.next_id;
        self.next_id += 1;
        let lifetime = Lifetime::Anon(id);
        self.scopes.insert(lifetime.clone(), self.depth);
        lifetime
    }

    /// Enter new scope
    pub fn push_scope(&mut self) {
        self.depth += 1;
    }

    /// Exit scope
    pub fn pop_scope(&mut self) {
        // Remove lifetimes from this scope
        self.scopes.retain(|_, d| *d < self.depth);
        self.depth = self.depth.saturating_sub(1);
    }

    /// Add outlives constraint ('a: 'b)
    pub fn add_outlives(&mut self, longer: Lifetime, shorter: Lifetime) {
        self.constraints
            .push(LifetimeConstraint::Outlives(longer, shorter));
    }

    /// Add equality constraint ('a == 'b)
    pub fn add_equals(&mut self, a: Lifetime, b: Lifetime) {
        self.constraints.push(LifetimeConstraint::Equals(a, b));
    }

    /// Check if constraints are satisfiable
    pub fn check_constraints(&self) -> LifetimeResult<()> {
        // Build outlives graph
        let mut outlives: HashMap<Lifetime, HashSet<Lifetime>> = HashMap::new();

        for constraint in &self.constraints {
            match constraint {
                LifetimeConstraint::Outlives(longer, shorter) => {
                    outlives
                        .entry(longer.clone())
                        .or_default()
                        .insert(shorter.clone());
                }
                LifetimeConstraint::Equals(a, b) => {
                    // a == b means a: b and b: a
                    outlives.entry(a.clone()).or_default().insert(b.clone());
                    outlives.entry(b.clone()).or_default().insert(a.clone());
                }
            }
        }

        // Check for cycles (would indicate impossible constraints)
        // Simplified: just check direct cycles
        for (longer, shorters) in &outlives {
            for shorter in shorters {
                if let Some(reverse) = outlives.get(shorter) {
                    if reverse.contains(longer) && longer != shorter {
                        return Err(LifetimeError::Conflict(longer.clone(), shorter.clone()));
                    }
                }
            }
        }

        Ok(())
    }

    /// Get scope depth of a lifetime
    pub fn scope_depth(&self, lifetime: &Lifetime) -> Option<usize> {
        if lifetime.is_static() {
            Some(0) // Static is outermost
        } else {
            self.scopes.get(lifetime).copied()
        }
    }

    /// Check if lifetime 'a outlives 'b
    pub fn outlives(&self, a: &Lifetime, b: &Lifetime) -> bool {
        if a.is_static() {
            return true; // Static outlives everything
        }

        let depth_a = self.scope_depth(a);
        let depth_b = self.scope_depth(b);

        match (depth_a, depth_b) {
            (Some(da), Some(db)) => da <= db, // Outer scopes outlive inner
            _ => false,
        }
    }

    /// Get all constraints
    pub fn constraints(&self) -> &[LifetimeConstraint] {
        &self.constraints
    }
}

impl Default for LifetimeContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Region (memory region with lifetime)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Region {
    /// Lifetime of this region
    pub lifetime: Lifetime,
    /// Mutability
    pub mutable: bool,
}

impl Region {
    /// Create a new region
    pub fn new(lifetime: Lifetime, mutable: bool) -> Self {
        Self { lifetime, mutable }
    }

    /// Create a static region
    pub fn static_region(mutable: bool) -> Self {
        Self {
            lifetime: Lifetime::Static,
            mutable,
        }
    }

    /// Check if region is static
    pub fn is_static(&self) -> bool {
        self.lifetime.is_static()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifetime_fresh() {
        let mut ctx = LifetimeContext::new();
        let l1 = ctx.fresh();
        let l2 = ctx.fresh();

        assert_ne!(l1, l2);
        assert!(l1.is_anon());
        assert!(l2.is_anon());
    }

    #[test]
    fn test_lifetime_static() {
        let static_lt = Lifetime::Static;
        assert!(static_lt.is_static());
        assert!(!static_lt.is_named());
    }

    #[test]
    fn test_lifetime_scopes() {
        let mut ctx = LifetimeContext::new();

        let l1 = ctx.fresh();
        assert_eq!(ctx.scope_depth(&l1), Some(0));

        ctx.push_scope();
        let l2 = ctx.fresh();
        assert_eq!(ctx.scope_depth(&l2), Some(1));

        ctx.pop_scope();
        assert_eq!(ctx.scope_depth(&l1), Some(0));
        assert_eq!(ctx.scope_depth(&l2), None); // l2 was removed
    }

    #[test]
    fn test_outlives() {
        let mut ctx = LifetimeContext::new();

        let l1 = ctx.fresh();
        ctx.push_scope();
        let l2 = ctx.fresh();

        // l1 is in outer scope, so it outlives l2
        assert!(ctx.outlives(&l1, &l2));
        assert!(!ctx.outlives(&l2, &l1));

        // Static outlives everything
        assert!(ctx.outlives(&Lifetime::Static, &l1));
        assert!(ctx.outlives(&Lifetime::Static, &l2));
    }

    #[test]
    fn test_constraints() {
        let mut ctx = LifetimeContext::new();
        let l1 = ctx.fresh();
        let l2 = ctx.fresh();

        ctx.add_outlives(l1.clone(), l2.clone());
        assert_eq!(ctx.constraints().len(), 1);

        ctx.add_equals(l1.clone(), l2.clone());
        assert_eq!(ctx.constraints().len(), 2);
    }

    #[test]
    fn test_check_constraints_valid() {
        let mut ctx = LifetimeContext::new();
        let l1 = ctx.fresh();
        let l2 = ctx.fresh();

        ctx.add_outlives(l1.clone(), l2.clone());
        assert!(ctx.check_constraints().is_ok());
    }

    #[test]
    fn test_region() {
        let region = Region::static_region(false);
        assert!(region.is_static());
        assert!(!region.mutable);

        let mut ctx = LifetimeContext::new();
        let lt = ctx.fresh();
        let region2 = Region::new(lt, true);
        assert!(!region2.is_static());
        assert!(region2.mutable);
    }

    #[test]
    fn test_lifetime_named() {
        let lt = Lifetime::Named("a".to_string());
        assert!(lt.is_named());
        assert!(!lt.is_static());
        assert!(!lt.is_anon());
    }
}
