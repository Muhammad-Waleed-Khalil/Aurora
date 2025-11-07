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
