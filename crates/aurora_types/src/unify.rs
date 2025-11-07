//! Type Unification for Hindley-Milner Inference
//!
//! This module implements the unification algorithm that finds a substitution
//! to make two types equal, if possible. Unification is the core operation
//! in Hindley-Milner type inference.
//!
//! # Algorithm
//!
//! The unification algorithm works by:
//! 1. If both types are equal, return empty substitution
//! 2. If one is a type variable, bind it to the other (with occurs check)
//! 3. If both are compound types, recursively unify subcomponents
//! 4. Otherwise, the types cannot be unified (type error)

use crate::ty::{EffectSet, PrimitiveType, Type, TypeVarId};
use std::collections::HashMap;
use thiserror::Error;

/// Type unification error
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum UnificationError {
    /// Type mismatch (cannot unify)
    #[error("Cannot unify {0} with {1}")]
    TypeMismatch(String, String),

    /// Occurs check failed (infinite type)
    #[error("Occurs check failed: '{0} occurs in {1}")]
    OccursCheck(TypeVarId, String),

    /// Effect mismatch
    #[error("Effect mismatch: {0:?} vs {1:?}")]
    EffectMismatch(EffectSet, EffectSet),

    /// Arity mismatch (different number of type arguments)
    #[error("Arity mismatch: expected {0} arguments, got {1}")]
    ArityMismatch(usize, usize),
}

/// Type substitution (maps type variables to types)
pub type Substitution = HashMap<TypeVarId, Type>;

/// Unification result
pub type UnifyResult<T> = Result<T, UnificationError>;

/// Apply a substitution to a type
pub fn apply_subst(ty: &Type, subst: &Substitution) -> Type {
    ty.substitute(subst)
}

/// Compose two substitutions (s2 ∘ s1)
pub fn compose_subst(s1: &Substitution, s2: &Substitution) -> Substitution {
    let mut result = HashMap::new();

    // Apply s2 to all bindings in s1
    for (var, ty) in s1 {
        result.insert(*var, apply_subst(ty, s2));
    }

    // Add bindings from s2 that aren't in s1
    for (var, ty) in s2 {
        if !s1.contains_key(var) {
            result.insert(*var, ty.clone());
        }
    }

    result
}

/// Unify two types, returning a substitution that makes them equal
pub fn unify(t1: &Type, t2: &Type) -> UnifyResult<Substitution> {
    match (t1, t2) {
        // Equal types unify with empty substitution
        _ if t1 == t2 => Ok(Substitution::new()),

        // Unify type variable with another type
        (Type::Var(v), ty) | (ty, Type::Var(v)) => bind_var(*v, ty),

        // Unify primitives (must be equal, already handled above)
        (Type::Primitive(_), Type::Primitive(_)) => Err(UnificationError::TypeMismatch(
            t1.to_string(),
            t2.to_string(),
        )),

        // Unify named types (must have same name and unify arguments)
        (
            Type::Named { name: n1, args: a1 },
            Type::Named { name: n2, args: a2 },
        ) => {
            if n1 != n2 {
                return Err(UnificationError::TypeMismatch(
                    t1.to_string(),
                    t2.to_string(),
                ));
            }
            unify_many(a1, a2)
        }

        // Unify function types
        (
            Type::Function {
                params: p1,
                ret: r1,
                effects: e1,
            },
            Type::Function {
                params: p2,
                ret: r2,
                effects: e2,
            },
        ) => {
            // Effects must match exactly
            if e1 != e2 {
                return Err(UnificationError::EffectMismatch(*e1, *e2));
            }

            // Unify parameters
            let s1 = unify_many(p1, p2)?;

            // Unify return types (with substitution applied)
            let r1_subst = apply_subst(r1, &s1);
            let r2_subst = apply_subst(r2, &s1);
            let s2 = unify(&r1_subst, &r2_subst)?;

            Ok(compose_subst(&s1, &s2))
        }

        // Unify tuple types
        (Type::Tuple(ts1), Type::Tuple(ts2)) => unify_many(ts1, ts2),

        // Unify array types
        (
            Type::Array {
                elem: e1,
                size: sz1,
            },
            Type::Array {
                elem: e2,
                size: sz2,
            },
        ) => {
            if sz1 != sz2 {
                return Err(UnificationError::ArityMismatch(
                    sz1.unwrap_or(0),
                    sz2.unwrap_or(0),
                ));
            }
            unify(e1, e2)
        }

        // Unify reference types
        (
            Type::Ref {
                inner: i1,
                mutable: m1,
                ..
            },
            Type::Ref {
                inner: i2,
                mutable: m2,
                ..
            },
        ) => {
            if m1 != m2 {
                return Err(UnificationError::TypeMismatch(
                    t1.to_string(),
                    t2.to_string(),
                ));
            }
            unify(i1, i2)
        }

        // Unify pointer types
        (
            Type::Ptr {
                inner: i1,
                mutable: m1,
            },
            Type::Ptr {
                inner: i2,
                mutable: m2,
            },
        ) => {
            if m1 != m2 {
                return Err(UnificationError::TypeMismatch(
                    t1.to_string(),
                    t2.to_string(),
                ));
            }
            unify(i1, i2)
        }

        // Unify Option types
        (Type::Option(t1), Type::Option(t2)) => unify(t1, t2),

        // Unify Result types
        (
            Type::Result { ok: o1, err: e1 },
            Type::Result { ok: o2, err: e2 },
        ) => {
            let s1 = unify(o1, o2)?;
            let e1_subst = apply_subst(e1, &s1);
            let e2_subst = apply_subst(e2, &s1);
            let s2 = unify(&e1_subst, &e2_subst)?;
            Ok(compose_subst(&s1, &s2))
        }

        // Cannot unify different type constructors
        _ => Err(UnificationError::TypeMismatch(
            t1.to_string(),
            t2.to_string(),
        )),
    }
}

/// Unify a list of type pairs
fn unify_many(types1: &[Type], types2: &[Type]) -> UnifyResult<Substitution> {
    if types1.len() != types2.len() {
        return Err(UnificationError::ArityMismatch(
            types1.len(),
            types2.len(),
        ));
    }

    let mut subst = Substitution::new();
    for (t1, t2) in types1.iter().zip(types2.iter()) {
        let t1_subst = apply_subst(t1, &subst);
        let t2_subst = apply_subst(t2, &subst);
        let new_subst = unify(&t1_subst, &t2_subst)?;
        subst = compose_subst(&subst, &new_subst);
    }
    Ok(subst)
}

/// Bind a type variable to a type (with occurs check)
fn bind_var(var: TypeVarId, ty: &Type) -> UnifyResult<Substitution> {
    match ty {
        // Variable binds to itself - empty substitution
        Type::Var(v) if *v == var => Ok(Substitution::new()),

        // Check if var occurs in ty (prevents infinite types)
        _ if ty.occurs(var) => Err(UnificationError::OccursCheck(var, ty.to_string())),

        // Bind var to ty
        _ => {
            let mut subst = Substitution::new();
            subst.insert(var, ty.clone());
            Ok(subst)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unify_equal_types() {
        let i32_ty = Type::Primitive(PrimitiveType::I32);
        let result = unify(&i32_ty, &i32_ty);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_unify_var_with_type() {
        let var = Type::Var(0);
        let i32_ty = Type::Primitive(PrimitiveType::I32);

        let result = unify(&var, &i32_ty).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(&0), Some(&i32_ty));
    }

    #[test]
    fn test_unify_two_vars() {
        let var1 = Type::Var(0);
        let var2 = Type::Var(1);

        let result = unify(&var1, &var2).unwrap();
        assert_eq!(result.len(), 1);
        // Either var1 -> var2 or var2 -> var1
        assert!(result.get(&0).is_some() || result.get(&1).is_some());
    }

    #[test]
    fn test_occurs_check() {
        let var = Type::Var(0);
        let list = Type::Named {
            name: "List".to_string(),
            args: vec![var.clone()],
        };

        let result = unify(&var, &list);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            UnificationError::OccursCheck(0, _)
        ));
    }

    #[test]
    fn test_unify_tuples() {
        let var = Type::Var(0);
        let i32_ty = Type::Primitive(PrimitiveType::I32);
        let bool_ty = Type::Primitive(PrimitiveType::Bool);

        let tuple1 = Type::Tuple(vec![var.clone(), bool_ty.clone()]);
        let tuple2 = Type::Tuple(vec![i32_ty.clone(), bool_ty.clone()]);

        let result = unify(&tuple1, &tuple2).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(&0), Some(&i32_ty));
    }

    #[test]
    fn test_unify_functions() {
        let var = Type::Var(0);
        let i32_ty = Type::Primitive(PrimitiveType::I32);

        let func1 = Type::Function {
            params: vec![var.clone()],
            ret: Box::new(i32_ty.clone()),
            effects: EffectSet::PURE,
        };

        let func2 = Type::Function {
            params: vec![i32_ty.clone()],
            ret: Box::new(i32_ty.clone()),
            effects: EffectSet::PURE,
        };

        let result = unify(&func1, &func2).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(&0), Some(&i32_ty));
    }

    #[test]
    fn test_unify_mismatch() {
        let i32_ty = Type::Primitive(PrimitiveType::I32);
        let bool_ty = Type::Primitive(PrimitiveType::Bool);

        let result = unify(&i32_ty, &bool_ty);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            UnificationError::TypeMismatch(_, _)
        ));
    }

    #[test]
    fn test_apply_subst() {
        let var = Type::Var(0);
        let i32_ty = Type::Primitive(PrimitiveType::I32);

        let mut subst = Substitution::new();
        subst.insert(0, i32_ty.clone());

        let result = apply_subst(&var, &subst);
        assert_eq!(result, i32_ty);
    }

    #[test]
    fn test_compose_subst() {
        let var0 = Type::Var(0);
        let var1 = Type::Var(1);
        let i32_ty = Type::Primitive(PrimitiveType::I32);

        let mut s1 = Substitution::new();
        s1.insert(0, var1.clone());

        let mut s2 = Substitution::new();
        s2.insert(1, i32_ty.clone());

        let composed = compose_subst(&s1, &s2);

        // After composition, var0 should map to i32
        let result = apply_subst(&var0, &composed);
        assert_eq!(result, i32_ty);
    }

    #[test]
    fn test_unify_nested_types() {
        let var = Type::Var(0);
        let i32_ty = Type::Primitive(PrimitiveType::I32);

        let option_var = Type::Option(Box::new(var.clone()));
        let option_i32 = Type::Option(Box::new(i32_ty.clone()));

        let result = unify(&option_var, &option_i32).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(&0), Some(&i32_ty));
    }
}
