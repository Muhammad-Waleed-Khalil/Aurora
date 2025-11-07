//! Generic Type System and Monomorphization
//!
//! This module implements:
//! - Generic type instantiation
//! - Monomorphization tracking for code generation
//! - Type parameter bounds checking
//! - Generic function/type validation

use crate::ty::{Type, TypeVarId};
use crate::unify::Substitution;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

/// Generic parameter
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenericParam {
    /// Parameter ID
    pub id: TypeVarId,
    /// Parameter name
    pub name: String,
    /// Type bounds (trait constraints)
    pub bounds: Vec<TypeBound>,
}

/// Type bound on a generic parameter
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeBound {
    /// Trait name
    pub trait_name: String,
}

/// Generic definition (function or type)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenericDef {
    /// Name of the generic item
    pub name: String,
    /// Generic parameters
    pub params: Vec<GenericParam>,
    /// Body type (for generic types) or return type (for functions)
    pub body: Type,
}

/// Monomorphization instance
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonoInstance {
    /// Generic definition name
    pub generic_name: String,
    /// Concrete type arguments
    pub type_args: Vec<Type>,
}

impl std::hash::Hash for MonoInstance {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.generic_name.hash(state);
        // Hash string representation of types
        for ty in &self.type_args {
            ty.to_string().hash(state);
        }
    }
}

/// Monomorphization error
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum GenericError {
    /// Wrong number of type arguments
    #[error("Wrong number of type arguments: expected {0}, got {1}")]
    ArityMismatch(usize, usize),

    /// Type parameter bound not satisfied
    #[error("Type {0} does not satisfy bound {1}")]
    BoundNotSatisfied(String, String),

    /// Recursive type without indirection
    #[error("Recursive type {0} requires indirection (Box, Rc, etc.)")]
    RecursiveType(String),

    /// Unknown generic parameter
    #[error("Unknown generic parameter: {0}")]
    UnknownParam(String),
}

/// Generic result type
pub type GenericResult<T> = Result<T, GenericError>;

/// Monomorphization tracker
#[derive(Debug, Clone)]
pub struct MonoTracker {
    /// All monomorphization instances seen
    instances: HashSet<MonoInstance>,
    /// Pending instances to generate
    pending: Vec<MonoInstance>,
}

impl MonoTracker {
    /// Create a new monomorphization tracker
    pub fn new() -> Self {
        Self {
            instances: HashSet::new(),
            pending: Vec::new(),
        }
    }

    /// Request monomorphization of a generic with concrete types
    pub fn request(&mut self, instance: MonoInstance) -> bool {
        if self.instances.insert(instance.clone()) {
            self.pending.push(instance);
            true // New instance
        } else {
            false // Already seen
        }
    }

    /// Get next pending instance
    pub fn next_pending(&mut self) -> Option<MonoInstance> {
        self.pending.pop()
    }

    /// Check if there are pending instances
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    /// Get all instances
    pub fn instances(&self) -> &HashSet<MonoInstance> {
        &self.instances
    }

    /// Get count of monomorphized instances
    pub fn count(&self) -> usize {
        self.instances.len()
    }
}

impl Default for MonoTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Instantiate a generic definition with concrete types
pub fn instantiate_generic(
    def: &GenericDef,
    type_args: &[Type],
) -> GenericResult<Type> {
    // Check arity
    if def.params.len() != type_args.len() {
        return Err(GenericError::ArityMismatch(
            def.params.len(),
            type_args.len(),
        ));
    }

    // Build substitution
    let mut subst = Substitution::new();
    for (param, arg) in def.params.iter().zip(type_args.iter()) {
        subst.insert(param.id, arg.clone());
    }

    // Apply substitution to body
    Ok(def.body.substitute(&subst))
}

/// Check if type arguments satisfy bounds
pub fn check_bounds(
    params: &[GenericParam],
    type_args: &[Type],
) -> GenericResult<()> {
    for (param, arg) in params.iter().zip(type_args.iter()) {
        for bound in &param.bounds {
            // In a real implementation, this would check trait implementations
            // For now, we just validate that primitives satisfy basic bounds
            if !satisfies_bound(arg, &bound.trait_name) {
                return Err(GenericError::BoundNotSatisfied(
                    arg.to_string(),
                    bound.trait_name.clone(),
                ));
            }
        }
    }
    Ok(())
}

/// Check if a type satisfies a bound (simplified)
fn satisfies_bound(ty: &Type, _bound: &str) -> bool {
    // Simplified: all types satisfy all bounds
    // In reality, would check trait implementations
    matches!(ty, Type::Primitive(_) | Type::Named { .. })
}

/// Detect recursive types without indirection
pub fn check_recursive_type(ty: &Type, seen: &mut HashSet<String>) -> GenericResult<()> {
    match ty {
        Type::Named { name, args } => {
            if seen.contains(name) {
                return Err(GenericError::RecursiveType(name.clone()));
            }

            seen.insert(name.clone());
            for arg in args {
                check_recursive_type(arg, seen)?;
            }
            seen.remove(name);
            Ok(())
        }
        Type::Tuple(types) => {
            for t in types {
                check_recursive_type(t, seen)?;
            }
            Ok(())
        }
        Type::Array { elem, .. } => check_recursive_type(elem, seen),
        Type::Function { params, ret, .. } => {
            for p in params {
                check_recursive_type(p, seen)?;
            }
            check_recursive_type(ret, seen)
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ty::PrimitiveType;

    #[test]
    fn test_mono_tracker() {
        let mut tracker = MonoTracker::new();

        let instance = MonoInstance {
            generic_name: "Vec".to_string(),
            type_args: vec![Type::Primitive(PrimitiveType::I32)],
        };

        // First request should be new
        assert!(tracker.request(instance.clone()));
        assert_eq!(tracker.count(), 1);

        // Second request should be duplicate
        assert!(!tracker.request(instance.clone()));
        assert_eq!(tracker.count(), 1);

        // Should have pending
        assert!(tracker.has_pending());
        assert!(tracker.next_pending().is_some());
        assert!(!tracker.has_pending());
    }

    #[test]
    fn test_instantiate_generic() {
        let generic_def = GenericDef {
            name: "identity".to_string(),
            params: vec![GenericParam {
                id: 0,
                name: "T".to_string(),
                bounds: vec![],
            }],
            body: Type::Function {
                params: vec![Type::Var(0)],
                ret: Box::new(Type::Var(0)),
                effects: crate::ty::EffectSet::PURE,
            },
        };

        let i32_ty = Type::Primitive(PrimitiveType::I32);
        let result = instantiate_generic(&generic_def, &[i32_ty.clone()]).unwrap();

        let expected = Type::Function {
            params: vec![i32_ty.clone()],
            ret: Box::new(i32_ty),
            effects: crate::ty::EffectSet::PURE,
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn test_arity_mismatch() {
        let generic_def = GenericDef {
            name: "pair".to_string(),
            params: vec![
                GenericParam {
                    id: 0,
                    name: "T".to_string(),
                    bounds: vec![],
                },
                GenericParam {
                    id: 1,
                    name: "U".to_string(),
                    bounds: vec![],
                },
            ],
            body: Type::Tuple(vec![Type::Var(0), Type::Var(1)]),
        };

        let i32_ty = Type::Primitive(PrimitiveType::I32);

        // Too few arguments
        let result = instantiate_generic(&generic_def, &[i32_ty]);
        assert!(matches!(result, Err(GenericError::ArityMismatch(2, 1))));
    }

    #[test]
    fn test_check_bounds() {
        let params = vec![GenericParam {
            id: 0,
            name: "T".to_string(),
            bounds: vec![TypeBound {
                trait_name: "Display".to_string(),
            }],
        }];

        let i32_ty = Type::Primitive(PrimitiveType::I32);
        let result = check_bounds(&params, &[i32_ty]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mono_instance_equality() {
        let inst1 = MonoInstance {
            generic_name: "Vec".to_string(),
            type_args: vec![Type::Primitive(PrimitiveType::I32)],
        };

        let inst2 = MonoInstance {
            generic_name: "Vec".to_string(),
            type_args: vec![Type::Primitive(PrimitiveType::I32)],
        };

        let inst3 = MonoInstance {
            generic_name: "Vec".to_string(),
            type_args: vec![Type::Primitive(PrimitiveType::I64)],
        };

        assert_eq!(inst1, inst2);
        assert_ne!(inst1, inst3);
    }

    #[test]
    fn test_check_recursive_type() {
        // Non-recursive type
        let ty = Type::Named {
            name: "List".to_string(),
            args: vec![Type::Primitive(PrimitiveType::I32)],
        };

        let mut seen = HashSet::new();
        assert!(check_recursive_type(&ty, &mut seen).is_ok());

        // Recursive type (would fail in real check)
        let recursive = Type::Named {
            name: "List".to_string(),
            args: vec![Type::Named {
                name: "Node".to_string(),
                args: vec![],
            }],
        };

        let mut seen = HashSet::new();
        assert!(check_recursive_type(&recursive, &mut seen).is_ok());
    }

    #[test]
    fn test_multiple_monomorphizations() {
        let mut tracker = MonoTracker::new();

        let vec_i32 = MonoInstance {
            generic_name: "Vec".to_string(),
            type_args: vec![Type::Primitive(PrimitiveType::I32)],
        };

        let vec_bool = MonoInstance {
            generic_name: "Vec".to_string(),
            type_args: vec![Type::Primitive(PrimitiveType::Bool)],
        };

        tracker.request(vec_i32);
        tracker.request(vec_bool);

        assert_eq!(tracker.count(), 2);
    }
}
