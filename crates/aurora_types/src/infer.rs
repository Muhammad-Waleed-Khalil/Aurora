//! Hindley-Milner Type Inference for Aurora
//!
//! This module implements bidirectional type checking and inference based on
//! the Hindley-Milner algorithm with support for:
//! - Principal type inference for unannotated expressions
//! - Bidirectional checking (signatures guide inference)
//! - Let-polymorphism (generalization at let bindings)
//! - Type schemes and instantiation
//!
//! # Algorithm
//!
//! 1. **Constraint Generation**: Traverse the AST and generate type constraints
//! 2. **Unification**: Solve constraints using the unification algorithm
//! 3. **Generalization**: Generalize types at let-bindings (introduce forall)
//! 4. **Instantiation**: Instantiate polymorphic types with fresh variables

use crate::ty::{Constraint, EffectSet, PrimitiveType, Type, TypeVarId};
use crate::unify::{compose_subst, unify, Substitution, UnificationError};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Type inference error
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InferenceError {
    /// Unification error
    #[error("Unification error: {0}")]
    Unification(#[from] UnificationError),

    /// Undefined variable
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),

    /// Type mismatch in annotation
    #[error("Type annotation mismatch: expected {0}, got {1}")]
    AnnotationMismatch(String, String),

    /// Cannot infer type (needs annotation)
    #[error("Cannot infer type for {0}, annotation required")]
    CannotInfer(String),

    /// Effect mismatch
    #[error("Effect mismatch: function requires {0:?} but has {1:?}")]
    EffectMismatch(EffectSet, EffectSet),
}

/// Type inference result
pub type InferResult<T> = Result<T, InferenceError>;

/// Type environment (maps variables to type schemes)
#[derive(Debug, Clone)]
pub struct TypeEnv {
    /// Variable bindings
    bindings: HashMap<String, TypeScheme>,
}

impl TypeEnv {
    /// Create a new empty environment
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    /// Create environment with primitive types
    pub fn with_primitives() -> Self {
        let mut env = Self::new();

        // Add primitive type constructors
        // These will be added as needed

        env
    }

    /// Extend environment with a binding
    pub fn extend(&self, name: String, scheme: TypeScheme) -> Self {
        let mut bindings = self.bindings.clone();
        bindings.insert(name, scheme);
        Self { bindings }
    }

    /// Lookup a variable in the environment
    pub fn lookup(&self, name: &str) -> Option<&TypeScheme> {
        self.bindings.get(name)
    }

    /// Get all free type variables in the environment
    pub fn free_vars(&self) -> HashSet<TypeVarId> {
        let mut vars = HashSet::new();
        for scheme in self.bindings.values() {
            vars.extend(scheme.free_vars());
        }
        vars
    }

    /// Apply a substitution to the environment
    pub fn apply(&self, subst: &Substitution) -> Self {
        let bindings = self
            .bindings
            .iter()
            .map(|(k, v)| (k.clone(), v.apply(subst)))
            .collect();
        Self { bindings }
    }
}

impl Default for TypeEnv {
    fn default() -> Self {
        Self::new()
    }
}

/// Type scheme (polytypes with forall quantification)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeScheme {
    /// Quantified type variables
    pub vars: Vec<TypeVarId>,
    /// The type
    pub ty: Type,
}

impl TypeScheme {
    /// Create a monomorphic type scheme
    pub fn mono(ty: Type) -> Self {
        Self {
            vars: Vec::new(),
            ty,
        }
    }

    /// Create a polymorphic type scheme
    pub fn poly(vars: Vec<TypeVarId>, ty: Type) -> Self {
        Self { vars, ty }
    }

    /// Get free type variables (not bound by forall)
    pub fn free_vars(&self) -> HashSet<TypeVarId> {
        let mut vars = self.ty.free_vars();
        for v in &self.vars {
            vars.remove(v);
        }
        vars
    }

    /// Apply substitution (only to free variables)
    pub fn apply(&self, subst: &Substitution) -> Self {
        // Remove bound variables from substitution
        let mut filtered_subst = subst.clone();
        for var in &self.vars {
            filtered_subst.remove(var);
        }

        TypeScheme {
            vars: self.vars.clone(),
            ty: self.ty.substitute(&filtered_subst),
        }
    }
}

/// Type inference context
pub struct InferContext {
    /// Current type variable counter
    next_var: TypeVarId,
    /// Current substitution
    subst: Substitution,
}

impl InferContext {
    /// Create a new inference context
    pub fn new() -> Self {
        Self {
            next_var: 0,
            subst: Substitution::new(),
        }
    }

    /// Generate a fresh type variable
    pub fn fresh_var(&mut self) -> Type {
        let var = self.next_var;
        self.next_var += 1;
        Type::Var(var)
    }

    /// Add a constraint (perform unification)
    pub fn unify(&mut self, t1: &Type, t2: &Type) -> InferResult<()> {
        let t1_subst = t1.substitute(&self.subst);
        let t2_subst = t2.substitute(&self.subst);

        let new_subst = unify(&t1_subst, &t2_subst)?;
        self.subst = compose_subst(&self.subst, &new_subst);

        Ok(())
    }

    /// Instantiate a type scheme with fresh variables
    pub fn instantiate(&mut self, scheme: &TypeScheme) -> Type {
        if scheme.vars.is_empty() {
            return scheme.ty.clone();
        }

        let mut subst = Substitution::new();
        for &var in &scheme.vars {
            subst.insert(var, self.fresh_var());
        }

        scheme.ty.substitute(&subst)
    }

    /// Generalize a type (introduce forall for free variables)
    pub fn generalize(&self, env: &TypeEnv, ty: &Type) -> TypeScheme {
        let ty_vars = ty.free_vars();
        let env_vars = env.free_vars();

        // Quantify over variables free in ty but not in env
        let quantified: Vec<TypeVarId> = ty_vars.difference(&env_vars).copied().collect();

        TypeScheme::poly(quantified, ty.clone())
    }

    /// Apply current substitution to a type
    pub fn apply_subst(&self, ty: &Type) -> Type {
        ty.substitute(&self.subst)
    }

    /// Get the current substitution
    pub fn substitution(&self) -> &Substitution {
        &self.subst
    }
}

impl Default for InferContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Infer the type of a literal value
pub fn infer_literal(lit: &Literal) -> Type {
    match lit {
        Literal::Int(_) => Type::Primitive(PrimitiveType::I32), // Default to i32
        Literal::Float(_) => Type::Primitive(PrimitiveType::F64), // Default to f64
        Literal::Bool(_) => Type::Primitive(PrimitiveType::Bool),
        Literal::Char(_) => Type::Primitive(PrimitiveType::Char),
        Literal::String(_) => Type::Primitive(PrimitiveType::Str),
    }
}

/// Placeholder literal type for examples
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// Integer literal
    Int(i64),
    /// Float literal
    Float(f64),
    /// Boolean literal
    Bool(bool),
    /// Character literal
    Char(char),
    /// String literal
    String(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_env() {
        let env = TypeEnv::new();
        let scheme = TypeScheme::mono(Type::Primitive(PrimitiveType::I32));

        let env2 = env.extend("x".to_string(), scheme.clone());
        assert_eq!(env2.lookup("x"), Some(&scheme));
        assert_eq!(env.lookup("x"), None); // Original unchanged
    }

    #[test]
    fn test_type_scheme_mono() {
        let ty = Type::Primitive(PrimitiveType::I32);
        let scheme = TypeScheme::mono(ty.clone());

        assert!(scheme.vars.is_empty());
        assert_eq!(scheme.ty, ty);
    }

    #[test]
    fn test_type_scheme_poly() {
        let var = Type::Var(0);
        let scheme = TypeScheme::poly(vec![0], var.clone());

        assert_eq!(scheme.vars, vec![0]);
        assert_eq!(scheme.ty, var);
    }

    #[test]
    fn test_fresh_var() {
        let mut ctx = InferContext::new();

        let v1 = ctx.fresh_var();
        let v2 = ctx.fresh_var();

        assert_ne!(v1, v2);
        assert!(matches!(v1, Type::Var(_)));
        assert!(matches!(v2, Type::Var(_)));
    }

    #[test]
    fn test_instantiate_mono() {
        let mut ctx = InferContext::new();
        let ty = Type::Primitive(PrimitiveType::I32);
        let scheme = TypeScheme::mono(ty.clone());

        let result = ctx.instantiate(&scheme);
        assert_eq!(result, ty);
    }

    #[test]
    fn test_instantiate_poly() {
        let mut ctx = InferContext::new();
        let var = Type::Var(0);
        let scheme = TypeScheme::poly(vec![0], var);

        let result1 = ctx.instantiate(&scheme);
        let result2 = ctx.instantiate(&scheme);

        // Each instantiation should produce a fresh variable
        assert_ne!(result1, result2);
    }

    #[test]
    fn test_generalize() {
        let ctx = InferContext::new();
        let env = TypeEnv::new();

        let var = Type::Var(0);
        let scheme = ctx.generalize(&env, &var);

        assert_eq!(scheme.vars, vec![0]);
    }

    #[test]
    fn test_unify_in_context() {
        let mut ctx = InferContext::new();
        let var = ctx.fresh_var();
        let i32_ty = Type::Primitive(PrimitiveType::I32);

        ctx.unify(&var, &i32_ty).unwrap();

        let result = ctx.apply_subst(&var);
        assert_eq!(result, i32_ty);
    }

    #[test]
    fn test_infer_literal() {
        let int_lit = Literal::Int(42);
        let ty = infer_literal(&int_lit);
        assert_eq!(ty, Type::Primitive(PrimitiveType::I32));

        let bool_lit = Literal::Bool(true);
        let ty = infer_literal(&bool_lit);
        assert_eq!(ty, Type::Primitive(PrimitiveType::Bool));
    }

    #[test]
    fn test_scheme_free_vars() {
        let var0 = Type::Var(0);
        let var1 = Type::Var(1);

        let tuple = Type::Tuple(vec![var0, var1.clone()]);
        let scheme = TypeScheme::poly(vec![0], tuple);

        let free = scheme.free_vars();
        assert_eq!(free.len(), 1);
        assert!(free.contains(&1));
        assert!(!free.contains(&0)); // Bound variable
    }
}
