//! Effect System for Aurora
//!
//! This module implements Aurora's effect system with:
//! - Effect polymorphism (effect variables)
//! - Subeffecting (partial order on effects)
//! - Effect tracking on function types
//! - Effect composition and normalization
//!
//! # Effect Hierarchy
//!
//! The subeffecting relation forms a partial order:
//! - PURE ⊆ IO ⊆ UNSAFE
//! - PURE ⊆ ALLOC ⊆ UNSAFE
//! - PURE ⊆ PARALLEL
//!
//! This allows pure functions to be used where effectful ones are expected.

use aurora_types::{EffectSet, Type};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Effect variable ID (for effect polymorphism)
pub type EffectVarId = u32;

/// Effect (either concrete or variable)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    /// Concrete effect set
    Concrete(EffectSet),
    /// Effect variable (polymorphic)
    Var(EffectVarId),
    /// Union of effects
    Union(Vec<Effect>),
}

impl Effect {
    /// Create a pure effect
    pub fn pure() -> Self {
        Effect::Concrete(EffectSet::PURE)
    }

    /// Create an IO effect
    pub fn io() -> Self {
        Effect::Concrete(EffectSet::IO)
    }

    /// Create an allocation effect
    pub fn alloc() -> Self {
        Effect::Concrete(EffectSet::ALLOC)
    }

    /// Create a parallel effect
    pub fn parallel() -> Self {
        Effect::Concrete(EffectSet::PARALLEL)
    }

    /// Create an unsafe effect
    pub fn unsafe_() -> Self {
        Effect::Concrete(EffectSet::UNSAFE)
    }

    /// Check if this effect is concrete
    pub fn is_concrete(&self) -> bool {
        matches!(self, Effect::Concrete(_))
    }

    /// Get concrete effect set, if available
    pub fn as_concrete(&self) -> Option<EffectSet> {
        match self {
            Effect::Concrete(e) => Some(*e),
            _ => None,
        }
    }

    /// Normalize effect (flatten unions, deduplicate)
    pub fn normalize(&self) -> Self {
        match self {
            Effect::Union(effects) => {
                let mut result = EffectSet::PURE;
                for eff in effects {
                    if let Effect::Concrete(e) = eff.normalize() {
                        result = result.union(e);
                    } else {
                        // Cannot normalize polymorphic effects
                        return self.clone();
                    }
                }
                Effect::Concrete(result)
            }
            _ => self.clone(),
        }
    }

    /// Apply substitution to effect
    pub fn substitute(&self, subst: &EffectSubstitution) -> Self {
        match self {
            Effect::Var(v) => subst.get(v).cloned().unwrap_or_else(|| self.clone()),
            Effect::Union(effects) => {
                Effect::Union(effects.iter().map(|e| e.substitute(subst)).collect())
            }
            Effect::Concrete(_) => self.clone(),
        }
    }
}

/// Effect substitution (maps effect variables to effects)
pub type EffectSubstitution = HashMap<EffectVarId, Effect>;

/// Effect error
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EffectError {
    /// Effect mismatch
    #[error("Effect mismatch: expected {0:?}, got {1:?}")]
    EffectMismatch(EffectSet, EffectSet),

    /// Cannot unify effects
    #[error("Cannot unify effects: {0:?} and {1:?}")]
    UnificationFailed(Effect, Effect),

    /// Insufficient effects
    #[error("Insufficient effects: {0:?} required but only {1:?} available")]
    InsufficientEffects(EffectSet, EffectSet),
}

/// Effect result type
pub type EffectResult<T> = Result<T, EffectError>;

/// Check if e1 is a subeffect of e2 (e1 ⊆ e2)
///
/// This implements the subeffecting partial order:
/// - PURE ⊆ everything
/// - IO ⊆ UNSAFE
/// - ALLOC ⊆ UNSAFE
/// - PARALLEL doesn't subsume anything except PURE
pub fn is_subeffect(e1: EffectSet, e2: EffectSet) -> bool {
    // PURE is subeffect of everything
    if e1.is_pure() {
        return true;
    }

    // If e1 == e2, they're equal
    if e1 == e2 {
        return true;
    }

    // UNSAFE subsumes IO and ALLOC
    if e2 == EffectSet::UNSAFE {
        return e1 == EffectSet::IO || e1 == EffectSet::ALLOC || e1 == EffectSet::PURE;
    }

    // For composite effects, check if all effects in e1 are in e2
    // or if each effect in e1 is a subeffect of something in e2
    if e1.has(EffectSet::IO) && !e2.has(EffectSet::IO) && !e2.has(EffectSet::UNSAFE) {
        return false;
    }
    if e1.has(EffectSet::ALLOC) && !e2.has(EffectSet::ALLOC) && !e2.has(EffectSet::UNSAFE) {
        return false;
    }
    if e1.has(EffectSet::PARALLEL) && !e2.has(EffectSet::PARALLEL) {
        return false;
    }
    if e1.has(EffectSet::UNSAFE) && !e2.has(EffectSet::UNSAFE) {
        return false;
    }

    true
}

/// Check if an effect set is allowed in a context
pub fn check_effect_allowed(required: EffectSet, available: EffectSet) -> EffectResult<()> {
    if is_subeffect(required, available) {
        Ok(())
    } else {
        Err(EffectError::InsufficientEffects(required, available))
    }
}

/// Effect tracker for function calls
#[derive(Debug, Clone)]
pub struct EffectTracker {
    /// Current effect context
    current: EffectSet,
    /// Stack of effect contexts (for nested scopes)
    stack: Vec<EffectSet>,
}

impl EffectTracker {
    /// Create a new effect tracker with pure context
    pub fn new() -> Self {
        Self {
            current: EffectSet::PURE,
            stack: Vec::new(),
        }
    }

    /// Push a new effect context
    pub fn push_context(&mut self, effects: EffectSet) {
        self.stack.push(self.current);
        self.current = effects;
    }

    /// Pop effect context
    pub fn pop_context(&mut self) -> Option<EffectSet> {
        let old = self.current;
        self.current = self.stack.pop()?;
        Some(old)
    }

    /// Get current effect context
    pub fn current(&self) -> EffectSet {
        self.current
    }

    /// Add an effect to current context
    pub fn add_effect(&mut self, effect: EffectSet) {
        self.current = self.current.union(effect);
    }

    /// Check if an effect is allowed in current context
    pub fn check_allowed(&self, required: EffectSet) -> EffectResult<()> {
        check_effect_allowed(required, self.current)
    }
}

impl Default for EffectTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Effect inference context
#[derive(Debug, Clone)]
pub struct EffectInferContext {
    /// Next effect variable ID
    next_var: EffectVarId,
    /// Effect substitution
    subst: EffectSubstitution,
}

impl EffectInferContext {
    /// Create new inference context
    pub fn new() -> Self {
        Self {
            next_var: 0,
            subst: EffectSubstitution::new(),
        }
    }

    /// Generate fresh effect variable
    pub fn fresh_var(&mut self) -> Effect {
        let var = self.next_var;
        self.next_var += 1;
        Effect::Var(var)
    }

    /// Unify two effects
    pub fn unify(&mut self, e1: &Effect, e2: &Effect) -> EffectResult<()> {
        let e1 = e1.substitute(&self.subst);
        let e2 = e2.substitute(&self.subst);

        match (&e1, &e2) {
            // Same concrete effects
            (Effect::Concrete(c1), Effect::Concrete(c2)) if c1 == c2 => Ok(()),

            // Bind variable to concrete
            (Effect::Var(v), Effect::Concrete(c)) | (Effect::Concrete(c), Effect::Var(v)) => {
                self.subst.insert(*v, Effect::Concrete(*c));
                Ok(())
            }

            // Bind variable to variable
            (Effect::Var(v1), Effect::Var(v2)) => {
                if v1 != v2 {
                    self.subst.insert(*v1, Effect::Var(*v2));
                }
                Ok(())
            }

            // Cannot unify different concrete effects
            _ => Err(EffectError::UnificationFailed(e1, e2)),
        }
    }

    /// Apply current substitution to effect
    pub fn apply(&self, effect: &Effect) -> Effect {
        effect.substitute(&self.subst)
    }

    /// Get substitution
    pub fn substitution(&self) -> &EffectSubstitution {
        &self.subst
    }
}

impl Default for EffectInferContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract effects from a function type
pub fn extract_effects(ty: &Type) -> Option<EffectSet> {
    match ty {
        Type::Function { effects, .. } => Some(*effects),
        _ => None,
    }
}

/// Compose effects (union)
pub fn compose_effects(e1: EffectSet, e2: EffectSet) -> EffectSet {
    e1.union(e2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subeffect_pure() {
        assert!(is_subeffect(EffectSet::PURE, EffectSet::PURE));
        assert!(is_subeffect(EffectSet::PURE, EffectSet::IO));
        assert!(is_subeffect(EffectSet::PURE, EffectSet::ALLOC));
        assert!(is_subeffect(EffectSet::PURE, EffectSet::UNSAFE));
    }

    #[test]
    fn test_subeffect_io() {
        assert!(!is_subeffect(EffectSet::IO, EffectSet::PURE));
        assert!(is_subeffect(EffectSet::IO, EffectSet::IO));
        assert!(is_subeffect(EffectSet::IO, EffectSet::UNSAFE));
    }

    #[test]
    fn test_subeffect_unsafe() {
        assert!(is_subeffect(EffectSet::UNSAFE, EffectSet::UNSAFE));
        assert!(!is_subeffect(EffectSet::UNSAFE, EffectSet::IO));
        assert!(!is_subeffect(EffectSet::UNSAFE, EffectSet::PURE));
    }

    #[test]
    fn test_effect_tracker() {
        let mut tracker = EffectTracker::new();
        assert_eq!(tracker.current(), EffectSet::PURE);

        tracker.add_effect(EffectSet::IO);
        assert_eq!(tracker.current(), EffectSet::IO);

        tracker.push_context(EffectSet::UNSAFE);
        assert_eq!(tracker.current(), EffectSet::UNSAFE);

        tracker.pop_context();
        assert_eq!(tracker.current(), EffectSet::IO);
    }

    #[test]
    fn test_effect_allowed() {
        let result = check_effect_allowed(EffectSet::PURE, EffectSet::IO);
        assert!(result.is_ok());

        let result = check_effect_allowed(EffectSet::UNSAFE, EffectSet::IO);
        assert!(result.is_err());
    }

    #[test]
    fn test_effect_var() {
        let mut ctx = EffectInferContext::new();
        let var1 = ctx.fresh_var();
        let var2 = ctx.fresh_var();

        assert_ne!(var1, var2);
        assert!(matches!(var1, Effect::Var(_)));
    }

    #[test]
    fn test_effect_unification() {
        let mut ctx = EffectInferContext::new();
        let var = ctx.fresh_var();
        let concrete = Effect::Concrete(EffectSet::IO);

        ctx.unify(&var, &concrete).unwrap();
        let result = ctx.apply(&var);
        assert_eq!(result, concrete);
    }

    #[test]
    fn test_effect_normalize() {
        let union = Effect::Union(vec![
            Effect::Concrete(EffectSet::IO),
            Effect::Concrete(EffectSet::ALLOC),
        ]);

        let normalized = union.normalize();
        assert!(matches!(normalized, Effect::Concrete(_)));
    }

    #[test]
    fn test_compose_effects() {
        let e1 = EffectSet::IO;
        let e2 = EffectSet::ALLOC;
        let composed = compose_effects(e1, e2);

        assert!(composed.has(EffectSet::IO));
        assert!(composed.has(EffectSet::ALLOC));
    }

    #[test]
    fn test_extract_effects() {
        let func_type = Type::Function {
            params: vec![],
            ret: Box::new(Type::Unit),
            effects: EffectSet::IO,
        };

        let effects = extract_effects(&func_type);
        assert_eq!(effects, Some(EffectSet::IO));
    }
}
