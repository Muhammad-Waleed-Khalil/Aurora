//! Exhaustiveness Checking for Pattern Matching
//!
//! This module implements exhaustiveness and reachability checking for
//! match expressions to ensure:
//! - All cases are covered (exhaustiveness)
//! - No unreachable patterns (reachability)
//! - Type safety through complete pattern matching

use crate::ty::{PrimitiveType, Type};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Pattern for exhaustiveness checking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Pattern {
    /// Wildcard pattern (_)
    Wildcard,
    /// Literal pattern (42, true, etc.)
    Literal(Literal),
    /// Constructor pattern (Some(x), None, Ok(y), etc.)
    Constructor {
        /// Constructor name
        name: String,
        /// Sub-patterns
        fields: Vec<Pattern>,
    },
    /// Tuple pattern
    Tuple(Vec<Pattern>),
    /// Or pattern (p1 | p2)
    Or(Vec<Pattern>),
}

/// Literal value in patterns
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Literal {
    /// Integer
    Int(i64),
    /// Boolean
    Bool(bool),
    /// Unit ()
    Unit,
}

/// Exhaustiveness error
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ExhaustivenessError {
    /// Non-exhaustive match
    #[error("Non-exhaustive match: missing pattern {0}")]
    NonExhaustive(String),

    /// Unreachable pattern
    #[error("Unreachable pattern at position {0}")]
    Unreachable(usize),

    /// Invalid pattern for type
    #[error("Invalid pattern {0} for type {1}")]
    InvalidPattern(String, String),
}

/// Exhaustiveness result
pub type ExhaustResult<T> = Result<T, ExhaustivenessError>;

/// Check if patterns are exhaustive for a type
pub fn check_exhaustive(ty: &Type, patterns: &[Pattern]) -> ExhaustResult<()> {
    let matrix = PatternMatrix::new(patterns.to_vec());

    if is_exhaustive(ty, &matrix) {
        Ok(())
    } else {
        let missing = construct_witness(ty, &matrix);
        Err(ExhaustivenessError::NonExhaustive(
            pattern_to_string(&missing),
        ))
    }
}

/// Check if any patterns are unreachable
pub fn check_reachable(patterns: &[Pattern]) -> ExhaustResult<()> {
    for (i, pattern) in patterns.iter().enumerate() {
        // Check if pattern is subsumed by previous patterns
        if is_subsumed(pattern, &patterns[..i]) {
            return Err(ExhaustivenessError::Unreachable(i));
        }
    }
    Ok(())
}

/// Pattern matrix for exhaustiveness checking
#[derive(Debug, Clone)]
struct PatternMatrix {
    rows: Vec<Vec<Pattern>>,
}

impl PatternMatrix {
    fn new(patterns: Vec<Pattern>) -> Self {
        Self {
            rows: patterns.into_iter().map(|p| vec![p]).collect(),
        }
    }

    fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    fn has_wildcard_row(&self) -> bool {
        self.rows
            .iter()
            .any(|row| row.first().map(|p| matches!(p, Pattern::Wildcard)).unwrap_or(false))
    }
}

/// Check if pattern matrix is exhaustive for a type
fn is_exhaustive(ty: &Type, matrix: &PatternMatrix) -> bool {
    // Empty matrix is not exhaustive (unless type is uninhabited)
    if matrix.is_empty() {
        return is_uninhabited(ty);
    }

    // Matrix with wildcard is exhaustive
    if matrix.has_wildcard_row() {
        return true;
    }

    // For specific types, check constructors
    match ty {
        Type::Primitive(PrimitiveType::Bool) => check_bool_exhaustive(matrix),
        Type::Option(_) => check_option_exhaustive(matrix),
        Type::Tuple(types) => check_tuple_exhaustive(types, matrix),
        _ => matrix.has_wildcard_row(), // Conservative: require wildcard for complex types
    }
}

/// Check if a type is uninhabited (has no values)
fn is_uninhabited(ty: &Type) -> bool {
    matches!(ty, Type::Never)
}

/// Check exhaustiveness for bool
fn check_bool_exhaustive(matrix: &PatternMatrix) -> bool {
    let mut has_true = false;
    let mut has_false = false;

    for row in &matrix.rows {
        if let Some(Pattern::Literal(Literal::Bool(b))) = row.first() {
            if *b {
                has_true = true;
            } else {
                has_false = true;
            }
        }
    }

    has_true && has_false
}

/// Check exhaustiveness for Option
fn check_option_exhaustive(matrix: &PatternMatrix) -> bool {
    let mut has_some = false;
    let mut has_none = false;

    for row in &matrix.rows {
        if let Some(Pattern::Constructor { name, .. }) = row.first() {
            if name == "Some" {
                has_some = true;
            } else if name == "None" {
                has_none = true;
            }
        }
    }

    has_some && has_none
}

/// Check exhaustiveness for tuple
fn check_tuple_exhaustive(_types: &[Type], matrix: &PatternMatrix) -> bool {
    // Simplified: require wildcard for tuples
    matrix.has_wildcard_row()
}

/// Construct a witness pattern for non-exhaustive match
fn construct_witness(ty: &Type, matrix: &PatternMatrix) -> Pattern {
    match ty {
        Type::Primitive(PrimitiveType::Bool) => {
            if !has_bool_pattern(matrix, true) {
                Pattern::Literal(Literal::Bool(true))
            } else {
                Pattern::Literal(Literal::Bool(false))
            }
        }
        Type::Option(_) => {
            if !has_constructor(matrix, "Some") {
                Pattern::Constructor {
                    name: "Some".to_string(),
                    fields: vec![Pattern::Wildcard],
                }
            } else {
                Pattern::Constructor {
                    name: "None".to_string(),
                    fields: vec![],
                }
            }
        }
        _ => Pattern::Wildcard,
    }
}

/// Check if matrix has a bool pattern
fn has_bool_pattern(matrix: &PatternMatrix, value: bool) -> bool {
    matrix.rows.iter().any(|row| {
        matches!(
            row.first(),
            Some(Pattern::Literal(Literal::Bool(b))) if *b == value
        )
    })
}

/// Check if matrix has a constructor
fn has_constructor(matrix: &PatternMatrix, name: &str) -> bool {
    matrix.rows.iter().any(|row| {
        matches!(
            row.first(),
            Some(Pattern::Constructor { name: n, .. }) if n == name
        )
    })
}

/// Check if a pattern is subsumed by a list of patterns
fn is_subsumed(pattern: &Pattern, previous: &[Pattern]) -> bool {
    previous.iter().any(|p| subsumes(p, pattern))
}

/// Check if pattern p1 subsumes p2 (p1 matches everything p2 matches)
fn subsumes(p1: &Pattern, p2: &Pattern) -> bool {
    match (p1, p2) {
        // Wildcard subsumes everything
        (Pattern::Wildcard, _) => true,

        // Same literals
        (Pattern::Literal(l1), Pattern::Literal(l2)) => l1 == l2,

        // Same constructors with subsumed fields
        (
            Pattern::Constructor {
                name: n1,
                fields: f1,
            },
            Pattern::Constructor {
                name: n2,
                fields: f2,
            },
        ) => n1 == n2 && f1.len() == f2.len() && f1.iter().zip(f2).all(|(a, b)| subsumes(a, b)),

        // Tuples
        (Pattern::Tuple(t1), Pattern::Tuple(t2)) => {
            t1.len() == t2.len() && t1.iter().zip(t2).all(|(a, b)| subsumes(a, b))
        }

        _ => false,
    }
}

/// Convert pattern to string for error messages
fn pattern_to_string(pattern: &Pattern) -> String {
    match pattern {
        Pattern::Wildcard => "_".to_string(),
        Pattern::Literal(Literal::Int(n)) => n.to_string(),
        Pattern::Literal(Literal::Bool(b)) => b.to_string(),
        Pattern::Literal(Literal::Unit) => "()".to_string(),
        Pattern::Constructor { name, fields } => {
            if fields.is_empty() {
                name.clone()
            } else {
                format!(
                    "{}({})",
                    name,
                    fields.iter().map(pattern_to_string).collect::<Vec<_>>().join(", ")
                )
            }
        }
        Pattern::Tuple(patterns) => {
            format!(
                "({})",
                patterns.iter().map(pattern_to_string).collect::<Vec<_>>().join(", ")
            )
        }
        Pattern::Or(patterns) => {
            patterns.iter().map(pattern_to_string).collect::<Vec<_>>().join(" | ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool_exhaustive() {
        let ty = Type::Primitive(PrimitiveType::Bool);

        let patterns = vec![
            Pattern::Literal(Literal::Bool(true)),
            Pattern::Literal(Literal::Bool(false)),
        ];

        assert!(check_exhaustive(&ty, &patterns).is_ok());
    }

    #[test]
    fn test_bool_non_exhaustive() {
        let ty = Type::Primitive(PrimitiveType::Bool);

        let patterns = vec![Pattern::Literal(Literal::Bool(true))];

        let result = check_exhaustive(&ty, &patterns);
        assert!(matches!(result, Err(ExhaustivenessError::NonExhaustive(_))));
    }

    #[test]
    fn test_option_exhaustive() {
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
    fn test_option_non_exhaustive() {
        let ty = Type::Option(Box::new(Type::Primitive(PrimitiveType::I32)));

        let patterns = vec![Pattern::Constructor {
            name: "Some".to_string(),
            fields: vec![Pattern::Wildcard],
        }];

        let result = check_exhaustive(&ty, &patterns);
        assert!(matches!(result, Err(ExhaustivenessError::NonExhaustive(_))));
    }

    #[test]
    fn test_wildcard_exhaustive() {
        let ty = Type::Primitive(PrimitiveType::I32);
        let patterns = vec![Pattern::Wildcard];

        assert!(check_exhaustive(&ty, &patterns).is_ok());
    }

    #[test]
    fn test_unreachable_pattern() {
        let patterns = vec![Pattern::Wildcard, Pattern::Literal(Literal::Bool(true))];

        let result = check_reachable(&patterns);
        assert!(matches!(result, Err(ExhaustivenessError::Unreachable(1))));
    }

    #[test]
    fn test_reachable_patterns() {
        let patterns = vec![
            Pattern::Literal(Literal::Bool(true)),
            Pattern::Literal(Literal::Bool(false)),
        ];

        assert!(check_reachable(&patterns).is_ok());
    }

    #[test]
    fn test_subsumes() {
        let wildcard = Pattern::Wildcard;
        let literal = Pattern::Literal(Literal::Bool(true));

        assert!(subsumes(&wildcard, &literal));
        assert!(!subsumes(&literal, &wildcard));
        assert!(subsumes(&literal, &literal));
    }

    #[test]
    fn test_pattern_to_string() {
        assert_eq!(pattern_to_string(&Pattern::Wildcard), "_");
        assert_eq!(
            pattern_to_string(&Pattern::Literal(Literal::Bool(true))),
            "true"
        );
        assert_eq!(
            pattern_to_string(&Pattern::Constructor {
                name: "Some".to_string(),
                fields: vec![Pattern::Wildcard],
            }),
            "Some(_)"
        );
    }

    #[test]
    fn test_never_type_exhaustive() {
        let ty = Type::Never;
        let patterns = vec![]; // Empty patterns exhaustive for Never

        assert!(check_exhaustive(&ty, &patterns).is_ok());
    }
}
