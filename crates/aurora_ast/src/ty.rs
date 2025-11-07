//! Type AST nodes
//!
//! This module defines type representations in the AST, including
//! primitive types, compound types, generics, and function types.

use crate::expr::{ExprId, Path};
use crate::span::Span;
use serde::{Deserialize, Serialize};

/// Type node ID (index into arena)
pub type TypeId = u32;

/// A type in the AST
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Type {
    /// The type kind
    pub kind: TypeKind,
    /// Source location
    pub span: Span,
}

/// Type kinds
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeKind {
    // Primitive types
    /// Signed integer types (`i8`, `i16`, `i32`, `i64`)
    Int(IntType),
    /// Unsigned integer types (`u8`, `u16`, `u32`, `u64`)
    Uint(UintType),
    /// Floating point types (`f32`, `f64`)
    Float(FloatType),
    /// Boolean type (`bool`)
    Bool,
    /// Character type (`char`)
    Char,
    /// String slice type (`str`)
    Str,

    // Path types
    /// Named type reference (e.g., `Vec<T>`, `Option<i64>`)
    Path {
        /// Type path
        path: Path,
    },

    // Compound types
    /// Tuple type (e.g., `(i64, String, bool)`)
    Tuple(Vec<TypeId>),

    /// Array type (e.g., `[u8; 1024]`)
    Array {
        /// Element type
        element: Box<TypeId>,
        /// Array length
        length: ExprId,
    },

    /// Slice type (e.g., `[u8]`)
    Slice {
        /// Element type
        element: Box<TypeId>,
    },

    /// Reference type (e.g., `&T`, `&mut T`)
    Reference {
        /// Referenced type
        inner: Box<TypeId>,
        /// Whether reference is mutable
        is_mut: bool,
    },

    /// Pointer type (e.g., `*const T`, `*mut T`)
    Pointer {
        /// Pointed-to type
        inner: Box<TypeId>,
        /// Whether pointer is mutable
        is_mut: bool,
    },

    /// Function type (e.g., `fn(i64, i64) -> i64`)
    Function {
        /// Parameter types
        params: Vec<TypeId>,
        /// Return type (None = unit type)
        return_type: Option<Box<TypeId>>,
    },

    /// Trait object type (e.g., `dyn Iterator<Item=i64>`)
    TraitObject {
        /// Trait bounds
        bounds: Vec<TypeBound>,
    },

    /// Impl trait type (e.g., `impl Iterator<Item=i64>`)
    ImplTrait {
        /// Trait bounds
        bounds: Vec<TypeBound>,
    },

    /// Inferred type (`_`)
    Infer,

    /// Never type (`!`)
    Never,
}

/// Signed integer type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntType {
    /// 8-bit signed integer
    I8,
    /// 16-bit signed integer
    I16,
    /// 32-bit signed integer
    I32,
    /// 64-bit signed integer
    I64,
}

/// Unsigned integer type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UintType {
    /// 8-bit unsigned integer
    U8,
    /// 16-bit unsigned integer
    U16,
    /// 32-bit unsigned integer
    U32,
    /// 64-bit unsigned integer
    U64,
}

/// Floating point type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FloatType {
    /// 32-bit float
    F32,
    /// 64-bit float
    F64,
}

/// Type bound (for trait objects and impl trait)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeBound {
    /// Trait path
    pub trait_path: Path,
    /// Source span
    pub span: Span,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_types() {
        let int_ty = TypeKind::Int(IntType::I64);
        assert!(matches!(int_ty, TypeKind::Int(IntType::I64)));

        let bool_ty = TypeKind::Bool;
        assert!(matches!(bool_ty, TypeKind::Bool));
    }

    #[test]
    fn test_compound_types() {
        let tuple_ty = TypeKind::Tuple(vec![0, 1, 2]);
        assert!(matches!(tuple_ty, TypeKind::Tuple(_)));
    }
}
