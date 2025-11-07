//! Declaration AST nodes
//!
//! This module defines top-level item declarations: functions, types,
//! traits, impls, constants, modules, and use statements.

use crate::expr::{ExprId, Path, TypeId};
use crate::span::Span;
use crate::stmt::Block;
use serde::{Deserialize, Serialize};

/// Item (top-level declaration) ID
pub type ItemId = u32;

/// A top-level item declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    /// The item kind
    pub kind: ItemKind,
    /// Source location
    pub span: Span,
}

/// Item kinds
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ItemKind {
    /// Function declaration
    Function(FunctionDecl),
    /// Type declaration (type alias)
    Type(TypeDecl),
    /// Trait declaration
    Trait(TraitDecl),
    /// Implementation
    Impl(ImplDecl),
    /// Constant declaration
    Const(ConstDecl),
    /// Module declaration
    Module(ModuleDecl),
    /// Use (import) declaration
    Use(UseDecl),
}

/// Function declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionDecl {
    /// Function name
    pub name: String,
    /// Generic parameters
    pub generics: Vec<GenericParam>,
    /// Function parameters
    pub params: Vec<Param>,
    /// Return type (None = unit type)
    pub return_type: Option<TypeId>,
    /// Where clause constraints
    pub where_clause: Option<WhereClause>,
    /// Function body
    pub body: Block,
    /// Whether function is public
    pub is_pub: bool,
    /// Whether function is async
    pub is_async: bool,
    /// Whether function is unsafe
    pub is_unsafe: bool,
    /// Source span
    pub span: Span,
}

/// Function parameter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Param {
    /// Parameter pattern
    pub pattern: PatternId,
    /// Parameter type
    pub ty: TypeId,
    /// Whether parameter is mutable
    pub is_mut: bool,
    /// Source span
    pub span: Span,
}

/// Pattern ID (index into arena)
pub type PatternId = u32;

/// Generic parameter
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenericParam {
    /// Parameter name
    pub name: String,
    /// Type bounds
    pub bounds: Vec<TypeBound>,
    /// Source span
    pub span: Span,
}

/// Type bound (trait constraint)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeBound {
    /// Trait path
    pub trait_path: Path,
    /// Source span
    pub span: Span,
}

/// Where clause
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhereClause {
    /// Predicates
    pub predicates: Vec<WherePredicate>,
    /// Source span
    pub span: Span,
}

/// Where predicate
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WherePredicate {
    /// Type being constrained
    pub ty: TypeId,
    /// Bounds on the type
    pub bounds: Vec<TypeBound>,
    /// Source span
    pub span: Span,
}

/// Type declaration (type alias)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeDecl {
    /// Type name
    pub name: String,
    /// Generic parameters
    pub generics: Vec<GenericParam>,
    /// Type alias target
    pub ty: TypeId,
    /// Whether type is public
    pub is_pub: bool,
    /// Source span
    pub span: Span,
}

/// Trait declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitDecl {
    /// Trait name
    pub name: String,
    /// Generic parameters
    pub generics: Vec<GenericParam>,
    /// Where clause constraints
    pub where_clause: Option<WhereClause>,
    /// Trait items
    pub items: Vec<TraitItem>,
    /// Whether trait is public
    pub is_pub: bool,
    /// Source span
    pub span: Span,
}

/// Trait item
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TraitItem {
    /// Function signature
    Function(FunctionSignature),
    /// Associated type
    Type(AssocType),
    /// Associated constant
    Const(ConstDecl),
}

/// Function signature (for trait declarations)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionSignature {
    /// Function name
    pub name: String,
    /// Generic parameters
    pub generics: Vec<GenericParam>,
    /// Function parameters
    pub params: Vec<Param>,
    /// Return type
    pub return_type: Option<TypeId>,
    /// Where clause
    pub where_clause: Option<WhereClause>,
    /// Optional default implementation
    pub body: Option<Block>,
    /// Whether function is async
    pub is_async: bool,
    /// Whether function is unsafe
    pub is_unsafe: bool,
    /// Source span
    pub span: Span,
}

/// Associated type declaration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssocType {
    /// Type name
    pub name: String,
    /// Type bounds
    pub bounds: Vec<TypeBound>,
    /// Optional default type
    pub default: Option<TypeId>,
    /// Source span
    pub span: Span,
}

/// Implementation declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImplDecl {
    /// Generic parameters
    pub generics: Vec<GenericParam>,
    /// Type being implemented
    pub self_ty: TypeId,
    /// Trait being implemented (None for inherent impl)
    pub trait_ref: Option<TraitRef>,
    /// Where clause constraints
    pub where_clause: Option<WhereClause>,
    /// Implementation items
    pub items: Vec<ImplItem>,
    /// Source span
    pub span: Span,
}

/// Trait reference
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraitRef {
    /// Trait path
    pub path: Path,
    /// Source span
    pub span: Span,
}

/// Implementation item
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImplItem {
    /// Function implementation
    Function(FunctionDecl),
    /// Associated constant
    Const(ConstDecl),
    /// Associated type
    Type(TypeDecl),
}

/// Constant declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstDecl {
    /// Constant name
    pub name: String,
    /// Constant type
    pub ty: TypeId,
    /// Constant value
    pub value: ExprId,
    /// Whether constant is public
    pub is_pub: bool,
    /// Source span
    pub span: Span,
}

/// Module declaration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleDecl {
    /// Module name
    pub name: String,
    /// Module items (None for external modules)
    pub items: Option<Vec<ItemId>>,
    /// Whether module is public
    pub is_pub: bool,
    /// Source span
    pub span: Span,
}

/// Use (import) declaration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UseDecl {
    /// Use tree
    pub tree: UseTree,
    /// Whether use is public (re-export)
    pub is_pub: bool,
    /// Source span
    pub span: Span,
}

/// Use tree (import path)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UseTree {
    /// Simple path (e.g., `use std::io`)
    Path {
        /// The path
        path: Path,
        /// Optional alias
        alias: Option<String>,
    },
    /// Glob import (e.g., `use std::io::*`)
    Glob {
        /// The path prefix
        path: Path,
    },
    /// Nested imports (e.g., `use std::io::{Read, Write}`)
    Nested {
        /// The path prefix
        path: Path,
        /// Nested use trees
        trees: Vec<UseTree>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Span;

    #[test]
    fn test_generic_param() {
        let param = GenericParam {
            name: "T".to_string(),
            bounds: vec![],
            span: Span::dummy(),
        };
        assert_eq!(param.name, "T");
    }

    #[test]
    fn test_const_decl() {
        let const_decl = ConstDecl {
            name: "MAX_SIZE".to_string(),
            ty: 0,
            value: 0,
            is_pub: true,
            span: Span::dummy(),
        };
        assert_eq!(const_decl.name, "MAX_SIZE");
        assert!(const_decl.is_pub);
    }
}
