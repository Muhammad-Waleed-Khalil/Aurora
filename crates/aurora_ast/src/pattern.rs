//! Pattern AST nodes
//!
//! This module defines pattern forms used in let bindings, function
//! parameters, and match expressions.

use crate::expr::{Literal, Path};
use crate::span::{HygieneId, Span};
use serde::{Deserialize, Serialize};

/// Pattern node ID (index into arena)
pub type PatternId = u32;

/// A pattern in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pattern {
    /// The pattern kind
    pub kind: PatternKind,
    /// Source location
    pub span: Span,
    /// Hygiene context
    pub hygiene: HygieneId,
}

/// Pattern kinds
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PatternKind {
    /// Wildcard pattern (`_`)
    Wildcard,

    /// Identifier pattern (e.g., `x`, `mut y`)
    Ident {
        /// The identifier name
        name: String,
        /// Whether binding is mutable
        is_mut: bool,
    },

    /// Literal pattern (e.g., `42`, `"hello"`, `true`)
    Literal(Literal),

    /// Path pattern (e.g., `Some`, `None`, `Ok`)
    Path(Path),

    /// Tuple pattern (e.g., `(x, y, _)`)
    Tuple(Vec<PatternId>),

    /// Struct pattern (e.g., `Point { x, y }`, `Point { x: a, y: b }`)
    Struct {
        /// Path to struct
        path: Path,
        /// Field patterns
        fields: Vec<FieldPattern>,
        /// Whether to ignore remaining fields (`..`)
        has_rest: bool,
    },

    /// Tuple struct pattern (e.g., `Some(x)`, `Ok(value)`)
    TupleStruct {
        /// Path to tuple struct
        path: Path,
        /// Field patterns
        fields: Vec<PatternId>,
    },

    /// Or pattern (e.g., `Some(1) | Some(2)`)
    Or(Vec<PatternId>),

    /// Range pattern (e.g., `1..=10`, `'a'..='z'`)
    Range {
        /// Start pattern
        start: Box<PatternId>,
        /// End pattern
        end: Box<PatternId>,
        /// Whether range is inclusive
        inclusive: bool,
    },

    /// Reference pattern (e.g., `&x`, `&mut y`)
    Ref {
        /// Inner pattern
        inner: Box<PatternId>,
        /// Whether reference is mutable
        is_mut: bool,
    },

    /// Rest pattern in slices (`..`)
    Rest,
}

/// Field pattern for struct destructuring
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldPattern {
    /// Field name
    pub name: String,
    /// Pattern for field value (None = shorthand like `{ x }`)
    pub pattern: Option<PatternId>,
    /// Source span
    pub span: Span,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wildcard_pattern() {
        let pattern = PatternKind::Wildcard;
        assert!(matches!(pattern, PatternKind::Wildcard));
    }

    #[test]
    fn test_ident_pattern() {
        let pattern = PatternKind::Ident {
            name: "x".to_string(),
            is_mut: false,
        };
        assert!(matches!(pattern, PatternKind::Ident { .. }));
    }

    #[test]
    fn test_tuple_pattern() {
        let pattern = PatternKind::Tuple(vec![0, 1, 2]);
        assert!(matches!(pattern, PatternKind::Tuple(_)));
    }
}
