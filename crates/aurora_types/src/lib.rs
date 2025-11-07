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
//! - `infer`: Hindley-Milner type inference (TODO)
//! - `unify`: Unification algorithm (TODO)
//! - `traits`: Typeclass resolution (TODO)
//! - `generics`: Generic instantiation (TODO)
//! - `option`: Null safety and exhaustiveness (TODO)
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

pub mod ty;

// Re-export main types
pub use ty::{
    Constraint, EffectSet, Lifetime, PrimitiveType, Type, TypeId, TypeVarId,
};
