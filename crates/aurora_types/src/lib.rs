//! Type System for Aurora
//!
//! This crate implements the Aurora type system including:
//! - **Type Representation**: Primitives, compounds, functions, generics
//! - **Hindley-Milner Inference**: Principal type inference with bidirectional checking
//! - **Typeclasses**: Trait constraints with coherence checking
//! - **Generics & Monomorphization**: Generic instantiation and specialization
//! - **Null Safety**: Option types with exhaustiveness checking
//! - **Effect System**: Effect tracking and subeffecting
//!
//! # Architecture
//!
//! The type system is organized into modules:
//! - `ty`: Core type representation, equality, subtyping
//! - `unify`: Unification algorithm for type inference
//! - `infer`: Hindley-Milner type inference with generalization
//! - `traits`: Typeclass resolution with coherence checking
//! - `generics`: Generic instantiation and monomorphization tracking
//! - `exhaustive`: Exhaustiveness checking for pattern matching
//!
//! # Example
//!
//! ```rust,ignore
//! use aurora_types::ty::{Type, PrimitiveType};
//!
//! let i32_ty = Type::Primitive(PrimitiveType::I32);
//! let bool_ty = Type::Primitive(PrimitiveType::Bool);
//!
//! // Type equality
//! assert!(i32_ty.equals(&i32_ty));
//! assert!(!i32_ty.equals(&bool_ty));
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod exhaustive;
pub mod generics;
pub mod infer;
pub mod traits;
pub mod ty;
pub mod unify;

// Re-export main types
pub use exhaustive::{ExhaustivenessError, Pattern as ExhaustPattern};
pub use generics::{GenericDef, GenericError, GenericParam, MonoInstance, MonoTracker};
pub use infer::{InferContext, InferenceError, TypeEnv, TypeScheme};
pub use traits::{AssocType, MethodSignature, Trait, TraitBound, TraitError, TraitImpl, TraitRegistry};
pub use ty::{Constraint, EffectSet, Lifetime, PrimitiveType, Type, TypeId, TypeVarId};
pub use unify::{Substitution, UnificationError};

// Pipeline integration
use aurora_ast::expr::{BinaryOp, Literal};
use aurora_ast::{Ast, ExprId, ExprKind};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

/// Type checking error
#[derive(Debug, Clone, Error)]
pub enum TypeError {
    /// Inference error
    #[error("Type inference error: {0}")]
    Inference(#[from] InferenceError),

    /// Unification error
    #[error("Type unification error: {0}")]
    Unification(#[from] UnificationError),

    /// Type mismatch
    #[error("Type mismatch: expected {expected}, found {found}")]
    Mismatch {
        /// Expected type
        expected: String,
        /// Found type
        found: String,
    },

    /// Undefined function
    #[error("Undefined function: {0}")]
    UndefinedFunction(String),

    /// Undefined variable
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),

    /// Wrong number of arguments
    #[error("Wrong number of arguments: expected {expected}, got {got}")]
    WrongArgCount {
        /// Expected count
        expected: usize,
        /// Got count
        got: usize,
    },

    /// Non-exhaustive pattern match
    #[error("Non-exhaustive pattern match: {0}")]
    NonExhaustive(String),
}

/// Type annotation map (maps AST node IDs to inferred types)
#[derive(Debug, Clone, Default)]
pub struct TypeMap {
    /// Expression types
    expr_types: HashMap<ExprId, Type>,
}

impl TypeMap {
    /// Create a new empty type map
    pub fn new() -> Self {
        Self {
            expr_types: HashMap::new(),
        }
    }

    /// Insert a type for an expression
    pub fn insert_expr(&mut self, expr_id: ExprId, ty: Type) {
        self.expr_types.insert(expr_id, ty);
    }

    /// Get the type of an expression
    pub fn get_expr(&self, expr_id: ExprId) -> Option<&Type> {
        self.expr_types.get(&expr_id)
    }
}

/// Type checker for pipeline integration
pub struct TypeChecker {
    diagnostics: Arc<dyn Send + Sync>,
    /// Type environment
    env: TypeEnv,
    /// Inference context
    ctx: InferContext,
    /// Type annotations
    type_map: TypeMap,
    /// Trait registry
    trait_registry: TraitRegistry,
    /// Monomorphization tracker
    mono_tracker: MonoTracker,
}

impl TypeChecker {
    /// Create a new type checker with diagnostic collector
    pub fn new<D: Send + Sync + 'static>(diagnostics: Arc<D>) -> Self {
        let mut env = TypeEnv::new();

        // Add built-in functions
        Self::add_builtins(&mut env);

        Self {
            diagnostics: diagnostics as Arc<dyn Send + Sync>,
            env,
            ctx: InferContext::new(),
            type_map: TypeMap::new(),
            trait_registry: TraitRegistry::new("current_crate".to_string()),
            mono_tracker: MonoTracker::new(),
        }
    }

    /// Add built-in functions to the environment
    fn add_builtins(env: &mut TypeEnv) {
        // println: (str, ...) -> ()
        // For simplicity, we'll type it as (str) -> ()
        let println_ty = Type::Function {
            params: vec![Type::Primitive(PrimitiveType::Str)],
            ret: Box::new(Type::Unit),
            effects: EffectSet::IO,
        };
        *env = env.extend("println".to_string(), TypeScheme::mono(println_ty));

        // len: [T] -> usize
        let len_ty = Type::Forall {
            vars: vec![0],
            constraints: vec![],
            inner: Box::new(Type::Function {
                params: vec![Type::Array {
                    elem: Box::new(Type::Var(0)),
                    size: None,
                }],
                ret: Box::new(Type::Primitive(PrimitiveType::USize)),
                effects: EffectSet::PURE,
            }),
        };
        *env = env.extend("len".to_string(), TypeScheme::poly(vec![0], len_ty));
    }

    /// Type check the AST
    pub fn check(&mut self, ast: Ast) -> Ast {
        // TODO: Implement actual type checking with Hindley-Milner inference
        // For now, this is a stub that returns the AST unchanged
        // A full implementation would:
        // 1. Traverse all items in the AST
        // 2. Type check function declarations
        // 3. Infer types for expressions
        // 4. Check exhaustiveness of pattern matches
        // 5. Resolve trait constraints
        // 6. Track monomorphization instances

        // This is intentionally minimal for now - will be expanded
        ast
    }

    /// Type check an expression (helper for testing and internal use)
    pub fn check_expr(&mut self, expr: &ExprKind) -> Result<Type, TypeError> {
        match expr {
            ExprKind::Literal(lit) => Ok(self.infer_literal(lit)),

            ExprKind::Ident(name) => {
                // Look up the variable in the environment
                if let Some(scheme) = self.env.lookup(name) {
                    Ok(self.ctx.instantiate(scheme))
                } else {
                    Err(TypeError::UndefinedVariable(name.clone()))
                }
            }

            ExprKind::Binary { op, left, right } => {
                // Type check binary operations
                let left_ty = self.check_expr_id(*left)?;
                let right_ty = self.check_expr_id(*right)?;

                // For now, require both operands to be the same type
                self.ctx.unify(&left_ty, &right_ty)?;

                // Return type depends on the operator
                Ok(match op {
                    // Comparison operators return bool
                    BinaryOp::Eq | BinaryOp::Ne |
                    BinaryOp::Lt | BinaryOp::Le |
                    BinaryOp::Gt | BinaryOp::Ge => {
                        Type::Primitive(PrimitiveType::Bool)
                    }
                    // Logical operators return bool
                    BinaryOp::And | BinaryOp::Or => {
                        Type::Primitive(PrimitiveType::Bool)
                    }
                    // Arithmetic operators return the same type as operands
                    _ => left_ty,
                })
            }

            ExprKind::Call { func, args } => {
                // Type check function call
                let func_ty = self.check_expr_id(*func)?;

                // Unify with a function type
                let ret_ty = self.ctx.fresh_var();
                let mut param_tys = Vec::new();
                for arg in args {
                    let arg_ty = self.check_expr_id(*arg)?;
                    param_tys.push(arg_ty);
                }

                let expected_func_ty = Type::Function {
                    params: param_tys,
                    ret: Box::new(ret_ty.clone()),
                    effects: EffectSet::PURE, // TODO: track effects
                };

                self.ctx.unify(&func_ty, &expected_func_ty)?;

                Ok(self.ctx.apply_subst(&ret_ty))
            }

            ExprKind::If { condition, then_block, else_block } => {
                // Type check if expression
                let cond_ty = self.check_expr_id(*condition)?;
                self.ctx.unify(&cond_ty, &Type::Primitive(PrimitiveType::Bool))?;

                // TODO: Type check blocks
                // For now, return unit type
                Ok(Type::Unit)
            }

            _ => {
                // TODO: Implement type checking for other expression kinds
                Ok(Type::Unit)
            }
        }
    }

    /// Helper to check an expression by ID (stub)
    fn check_expr_id(&mut self, _expr_id: ExprId) -> Result<Type, TypeError> {
        // TODO: Look up the expression in the AST and type check it
        Ok(Type::Unit)
    }

    /// Infer the type of a literal
    fn infer_literal(&self, lit: &Literal) -> Type {
        match lit {
            Literal::Int(_) => Type::Primitive(PrimitiveType::I32),
            Literal::Float(_) => Type::Primitive(PrimitiveType::F64),
            Literal::Bool(_) => Type::Primitive(PrimitiveType::Bool),
            Literal::Char(_) => Type::Primitive(PrimitiveType::Char),
            Literal::String(_) => Type::Primitive(PrimitiveType::Str),
        }
    }

    /// Get the type map
    pub fn type_map(&self) -> &TypeMap {
        &self.type_map
    }

    /// Get the trait registry
    pub fn trait_registry(&self) -> &TraitRegistry {
        &self.trait_registry
    }

    /// Get the monomorphization tracker
    pub fn mono_tracker(&self) -> &MonoTracker {
        &self.mono_tracker
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_checker_creation() {
        struct DummyDiagnostics;
        let diagnostics = Arc::new(DummyDiagnostics);
        let checker = TypeChecker::new(diagnostics);

        // Should have built-in functions
        assert!(checker.env.lookup("println").is_some());
    }

    #[test]
    fn test_infer_literal() {
        struct DummyDiagnostics;
        let diagnostics = Arc::new(DummyDiagnostics);
        let checker = TypeChecker::new(diagnostics);

        let lit = Literal::Int(42);
        let ty = checker.infer_literal(&lit);
        assert_eq!(ty, Type::Primitive(PrimitiveType::I32));
    }

    #[test]
    fn test_type_map() {
        let mut map = TypeMap::new();
        let ty = Type::Primitive(PrimitiveType::I32);

        map.insert_expr(0, ty.clone());
        assert_eq!(map.get_expr(0), Some(&ty));
        assert_eq!(map.get_expr(1), None);
    }

    #[test]
    fn test_hindley_milner_inference() {
        // Test basic HM inference
        let mut ctx = InferContext::new();

        // Create a variable and unify it with i32
        let var = ctx.fresh_var();
        let i32_ty = Type::Primitive(PrimitiveType::I32);

        ctx.unify(&var, &i32_ty).unwrap();

        let result = ctx.apply_subst(&var);
        assert_eq!(result, i32_ty);
    }

    #[test]
    fn test_function_type_unification() {
        // Test function type unification
        let mut ctx = InferContext::new();

        let var = ctx.fresh_var();
        let func1 = Type::Function {
            params: vec![var.clone()],
            ret: Box::new(Type::Primitive(PrimitiveType::I32)),
            effects: EffectSet::PURE,
        };

        let func2 = Type::Function {
            params: vec![Type::Primitive(PrimitiveType::Bool)],
            ret: Box::new(Type::Primitive(PrimitiveType::I32)),
            effects: EffectSet::PURE,
        };

        ctx.unify(&func1, &func2).unwrap();

        let result = ctx.apply_subst(&var);
        assert_eq!(result, Type::Primitive(PrimitiveType::Bool));
    }

    #[test]
    fn test_generalization() {
        // Test let-polymorphism generalization
        let ctx = InferContext::new();
        let env = TypeEnv::new();

        let var = Type::Var(0);
        let scheme = ctx.generalize(&env, &var);

        assert_eq!(scheme.vars, vec![0]);
        assert_eq!(scheme.ty, var);
    }

    #[test]
    fn test_instantiation() {
        // Test polymorphic instantiation
        let mut ctx = InferContext::new();

        let var = Type::Var(0);
        let scheme = TypeScheme::poly(vec![0], var);

        let inst1 = ctx.instantiate(&scheme);
        let inst2 = ctx.instantiate(&scheme);

        // Each instantiation should produce a fresh variable
        assert_ne!(inst1, inst2);
    }

    #[test]
    fn test_option_type() {
        // Test Option type
        let i32_ty = Type::Primitive(PrimitiveType::I32);
        let option_ty = Type::Option(Box::new(i32_ty.clone()));

        assert_eq!(option_ty.to_string(), "Option<i32>");
    }

    #[test]
    fn test_result_type() {
        // Test Result type
        let i32_ty = Type::Primitive(PrimitiveType::I32);
        let str_ty = Type::Primitive(PrimitiveType::Str);
        let result_ty = Type::Result {
            ok: Box::new(i32_ty.clone()),
            err: Box::new(str_ty.clone()),
        };

        assert_eq!(result_ty.to_string(), "Result<i32, str>");
    }

    #[test]
    fn test_trait_registry() {
        // Test trait registration
        let mut registry = TraitRegistry::new("test_crate".to_string());

        let trait_def = Trait {
            id: 0,
            name: "test_crate::Display".to_string(),
            type_params: vec![],
            supertraits: vec![],
            assoc_types: vec![],
            methods: vec![],
        };

        let trait_id = registry.register_trait(trait_def);
        assert!(registry.get_trait(trait_id).is_some());
    }

    #[test]
    fn test_trait_implementation() {
        // Test trait implementation registration
        let mut registry = TraitRegistry::new("test_crate".to_string());

        let trait_def = Trait {
            id: 0,
            name: "test_crate::Show".to_string(),
            type_params: vec![],
            supertraits: vec![],
            assoc_types: vec![],
            methods: vec![],
        };
        let trait_id = registry.register_trait(trait_def);

        let impl_def = TraitImpl {
            id: 0,
            trait_id,
            self_type: Type::Named {
                name: "test_crate::MyType".to_string(),
                args: vec![],
            },
            type_args: vec![],
            assoc_type_defs: HashMap::new(),
            defining_crate: "test_crate".to_string(),
        };

        let result = registry.register_impl(impl_def);
        assert!(result.is_ok());
    }

    #[test]
    fn test_monomorphization_tracker() {
        // Test monomorphization tracking
        let mut tracker = MonoTracker::new();

        let instance = MonoInstance {
            generic_name: "Vec".to_string(),
            type_args: vec![Type::Primitive(PrimitiveType::I32)],
        };

        // First request should be new
        assert!(tracker.request(instance.clone()));
        assert_eq!(tracker.count(), 1);

        // Second request should be duplicate
        assert!(!tracker.request(instance));
        assert_eq!(tracker.count(), 1);
    }

    #[test]
    fn test_generic_instantiation() {
        // Test generic instantiation
        use crate::generics::{instantiate_generic, GenericParam};

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
                effects: EffectSet::PURE,
            },
        };

        let i32_ty = Type::Primitive(PrimitiveType::I32);
        let result = instantiate_generic(&generic_def, &[i32_ty.clone()]).unwrap();

        let expected = Type::Function {
            params: vec![i32_ty.clone()],
            ret: Box::new(i32_ty),
            effects: EffectSet::PURE,
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn test_exhaustiveness_bool() {
        // Test exhaustiveness checking for bool
        use crate::exhaustive::{check_exhaustive, Literal as ExhaustLit, Pattern};

        let ty = Type::Primitive(PrimitiveType::Bool);
        let patterns = vec![
            Pattern::Literal(ExhaustLit::Bool(true)),
            Pattern::Literal(ExhaustLit::Bool(false)),
        ];

        assert!(check_exhaustive(&ty, &patterns).is_ok());
    }

    #[test]
    fn test_exhaustiveness_option() {
        // Test exhaustiveness checking for Option
        use crate::exhaustive::{check_exhaustive, Pattern};

        let ty = Type::Option(Box::new(Type::Primitive(PrimitiveType::I32)));
        let patterns = vec![
            Pattern::Constructor {
                name: "Some".to_string(),
                fields: vec![Pattern::Wildcard],
            },
            Pattern::Constructor {
                name: "None".to_string(),
                fields: vec![],
            },
        ];

        assert!(check_exhaustive(&ty, &patterns).is_ok());
    }

    #[test]
    fn test_effect_system() {
        // Test effect system
        let mut effects = EffectSet::new();
        assert!(effects.is_pure());

        effects.add(EffectSet::IO);
        assert!(effects.has(EffectSet::IO));
        assert!(!effects.is_pure());

        // Test subeffecting
        let pure = EffectSet::PURE;
        let io = EffectSet::IO;
        assert!(pure.is_subeffect_of(io));
        assert!(!io.is_subeffect_of(pure));
    }

    #[test]
    fn test_occurs_check() {
        // Test occurs check prevents infinite types
        let mut ctx = InferContext::new();
        let var = Type::Var(0);
        let list = Type::Named {
            name: "List".to_string(),
            args: vec![var.clone()],
        };

        let result = ctx.unify(&var, &list);
        assert!(result.is_err());
    }

    #[test]
    fn test_principal_types() {
        // Test that inference produces principal (most general) types
        let mut ctx = InferContext::new();
        let env = TypeEnv::new();

        // id = λx. x should infer to ∀a. a -> a
        let var = ctx.fresh_var();
        let id_ty = Type::Function {
            params: vec![var.clone()],
            ret: Box::new(var.clone()),
            effects: EffectSet::PURE,
        };

        let scheme = ctx.generalize(&env, &id_ty);

        // Should have one quantified variable
        assert_eq!(scheme.vars.len(), 1);
    }

    #[test]
    fn test_type_substitution() {
        // Test type substitution
        let var = Type::Var(0);
        let i32_ty = Type::Primitive(PrimitiveType::I32);

        let mut subst = HashMap::new();
        subst.insert(0, i32_ty.clone());

        let result = var.substitute(&subst);
        assert_eq!(result, i32_ty);
    }

    #[test]
    fn test_free_variables() {
        // Test free variable computation
        let var0 = Type::Var(0);
        let var1 = Type::Var(1);
        let func = Type::Function {
            params: vec![var0.clone()],
            ret: Box::new(var1.clone()),
            effects: EffectSet::PURE,
        };

        let vars = func.free_vars();
        assert_eq!(vars.len(), 2);
        assert!(vars.contains(&0));
        assert!(vars.contains(&1));
    }

    #[test]
    fn test_subtyping() {
        // Test subtyping relation
        let i32_ty = Type::Primitive(PrimitiveType::I32);
        let never = Type::Never;

        // Never is subtype of everything
        assert!(never.is_subtype_of(&i32_ty));
        assert!(!i32_ty.is_subtype_of(&never));

        // Reflexivity
        assert!(i32_ty.is_subtype_of(&i32_ty));
    }

    #[test]
    fn test_tuple_types() {
        // Test tuple types
        let i32_ty = Type::Primitive(PrimitiveType::I32);
        let bool_ty = Type::Primitive(PrimitiveType::Bool);
        let tuple = Type::Tuple(vec![i32_ty, bool_ty]);

        assert_eq!(tuple.to_string(), "(i32, bool)");
    }

    #[test]
    fn test_array_types() {
        // Test array types
        let i32_ty = Type::Primitive(PrimitiveType::I32);
        let array = Type::Array {
            elem: Box::new(i32_ty),
            size: Some(10),
        };

        assert_eq!(array.to_string(), "[i32; 10]");
    }

    #[test]
    fn test_forall_types() {
        // Test universal quantification
        let forall = Type::Forall {
            vars: vec![0],
            constraints: vec![],
            inner: Box::new(Type::Function {
                params: vec![Type::Var(0)],
                ret: Box::new(Type::Var(0)),
                effects: EffectSet::PURE,
            }),
        };

        let display = forall.to_string();
        assert!(display.contains("forall"));
    }

    #[test]
    fn test_coherence_checking() {
        // Test that coherence is enforced
        let mut registry = TraitRegistry::new("test_crate".to_string());

        let trait_def = Trait {
            id: 0,
            name: "test_crate::Eq".to_string(),
            type_params: vec![],
            supertraits: vec![],
            assoc_types: vec![],
            methods: vec![],
        };
        let trait_id = registry.register_trait(trait_def);

        let ty = Type::Named {
            name: "test_crate::MyType".to_string(),
            args: vec![],
        };

        let impl1 = TraitImpl {
            id: 0,
            trait_id,
            self_type: ty.clone(),
            type_args: vec![],
            assoc_type_defs: HashMap::new(),
            defining_crate: "test_crate".to_string(),
        };

        let impl2 = TraitImpl {
            id: 0,
            trait_id,
            self_type: ty,
            type_args: vec![],
            assoc_type_defs: HashMap::new(),
            defining_crate: "test_crate".to_string(),
        };

        // First impl should succeed
        assert!(registry.register_impl(impl1).is_ok());

        // Second impl should fail (coherence violation)
        assert!(matches!(
            registry.register_impl(impl2),
            Err(TraitError::Coherence(_))
        ));
    }

    #[test]
    fn test_deterministic_inference() {
        // Test that inference is deterministic
        let mut ctx1 = InferContext::new();
        let mut ctx2 = InferContext::new();

        let var1 = ctx1.fresh_var();
        let var2 = ctx2.fresh_var();

        let i32_ty = Type::Primitive(PrimitiveType::I32);

        ctx1.unify(&var1, &i32_ty).unwrap();
        ctx2.unify(&var2, &i32_ty).unwrap();

        let result1 = ctx1.apply_subst(&var1);
        let result2 = ctx2.apply_subst(&var2);

        // Both should infer the same type
        assert_eq!(result1, result2);
    }
}
