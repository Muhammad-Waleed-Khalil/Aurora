//! Operator Precedence Table for Aurora
//!
//! Defines the 16 precedence levels and associativity rules for Aurora operators.
//! This is used by the Pratt parser for expression parsing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Operator associativity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Associativity {
    /// Left-associative (a + b + c = (a + b) + c)
    Left,
    /// Right-associative (a = b = c =>  a = (b = c))
    Right,
    /// Non-associative (a < b < c is error)
    None,
}

/// Precedence level (1 = lowest, 16 = highest)
pub type Precedence = u8;

/// Operator precedence entry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrecedenceEntry {
    /// Precedence level (1-16)
    pub level: Precedence,
    /// Associativity
    pub assoc: Associativity,
}

impl PrecedenceEntry {
    /// Create a new precedence entry
    pub const fn new(level: Precedence, assoc: Associativity) -> Self {
        Self { level, assoc }
    }
}

/// Precedence table for Aurora operators
///
/// Aurora uses 16 precedence levels, following Rust's model with adjustments:
///
/// 1.  Assignment (=, +=, -=, etc.) - Right
/// 2.  Pipeline operators (|>, <|) - Left
/// 3.  Range operators (.., ..=) - None
/// 4.  Logical OR (||) - Left
/// 5.  Logical AND (&&) - Left
/// 6.  Comparison (==, !=, <, >, <=, >=) - None
/// 7.  Bitwise OR (|) - Left
/// 8.  Bitwise XOR (^) - Left
/// 9.  Bitwise AND (&) - Left
/// 10. Bit shift (<<, >>) - Left
/// 11. Addition (+, -) - Left
/// 12. Multiplication (*, /, %) - Left
/// 13. Exponentiation (**) - Right
/// 14. Unary (!, -, ~) - Right
/// 15. Function call, array index, field access - Left
/// 16. Path (::), arrow (->, =>) - Left
#[derive(Debug, Clone)]
pub struct PrecedenceTable {
    /// Operator to precedence mapping
    table: HashMap<&'static str, PrecedenceEntry>,
}

impl PrecedenceTable {
    /// Create a new precedence table with Aurora's operator precedence
    pub fn new() -> Self {
        let mut table = HashMap::new();

        // Level 1: Assignment (right-associative)
        table.insert("=", PrecedenceEntry::new(1, Associativity::Right));
        table.insert("+=", PrecedenceEntry::new(1, Associativity::Right));
        table.insert("-=", PrecedenceEntry::new(1, Associativity::Right));
        table.insert("*=", PrecedenceEntry::new(1, Associativity::Right));
        table.insert("/=", PrecedenceEntry::new(1, Associativity::Right));
        table.insert("%=", PrecedenceEntry::new(1, Associativity::Right));
        table.insert("&=", PrecedenceEntry::new(1, Associativity::Right));
        table.insert("|=", PrecedenceEntry::new(1, Associativity::Right));
        table.insert("^=", PrecedenceEntry::new(1, Associativity::Right));
        table.insert("<<=", PrecedenceEntry::new(1, Associativity::Right));
        table.insert(">>=", PrecedenceEntry::new(1, Associativity::Right));

        // Level 2: Pipeline (left-associative)
        table.insert("|>", PrecedenceEntry::new(2, Associativity::Left));
        table.insert("<|", PrecedenceEntry::new(2, Associativity::Left));

        // Level 3: Range (non-associative)
        table.insert("..", PrecedenceEntry::new(3, Associativity::None));
        table.insert("..=", PrecedenceEntry::new(3, Associativity::None));
        table.insert("...", PrecedenceEntry::new(3, Associativity::None));

        // Level 4: Logical OR (left-associative)
        table.insert("||", PrecedenceEntry::new(4, Associativity::Left));

        // Level 5: Logical AND (left-associative)
        table.insert("&&", PrecedenceEntry::new(5, Associativity::Left));

        // Level 6: Comparison (non-associative)
        table.insert("==", PrecedenceEntry::new(6, Associativity::None));
        table.insert("!=", PrecedenceEntry::new(6, Associativity::None));
        table.insert("<", PrecedenceEntry::new(6, Associativity::None));
        table.insert(">", PrecedenceEntry::new(6, Associativity::None));
        table.insert("<=", PrecedenceEntry::new(6, Associativity::None));
        table.insert(">=", PrecedenceEntry::new(6, Associativity::None));

        // Level 7: Bitwise OR (left-associative)
        table.insert("|", PrecedenceEntry::new(7, Associativity::Left));

        // Level 8: Bitwise XOR (left-associative)
        table.insert("^", PrecedenceEntry::new(8, Associativity::Left));

        // Level 9: Bitwise AND (left-associative)
        table.insert("&", PrecedenceEntry::new(9, Associativity::Left));

        // Level 10: Bit shift (left-associative)
        table.insert("<<", PrecedenceEntry::new(10, Associativity::Left));
        table.insert(">>", PrecedenceEntry::new(10, Associativity::Left));

        // Level 11: Addition (left-associative)
        table.insert("+", PrecedenceEntry::new(11, Associativity::Left));
        table.insert("-", PrecedenceEntry::new(11, Associativity::Left));

        // Level 12: Multiplication (left-associative)
        table.insert("*", PrecedenceEntry::new(12, Associativity::Left));
        table.insert("/", PrecedenceEntry::new(12, Associativity::Left));
        table.insert("%", PrecedenceEntry::new(12, Associativity::Left));

        // Level 13: Exponentiation (right-associative)
        table.insert("**", PrecedenceEntry::new(13, Associativity::Right));

        // Level 14: Unary operators (right-associative, handled specially)
        table.insert("!", PrecedenceEntry::new(14, Associativity::Right));
        table.insert("~", PrecedenceEntry::new(14, Associativity::Right));
        // Unary - handled in parser context

        // Level 15: Postfix (function call, index, field - left-associative)
        table.insert(".", PrecedenceEntry::new(15, Associativity::Left));
        table.insert("?", PrecedenceEntry::new(15, Associativity::Left));
        table.insert("??", PrecedenceEntry::new(15, Associativity::Left));
        // ( [ handled specially in parser

        // Level 16: Path operators (left-associative)
        table.insert("::", PrecedenceEntry::new(16, Associativity::Left));
        table.insert("->", PrecedenceEntry::new(16, Associativity::Left));
        table.insert("=>", PrecedenceEntry::new(16, Associativity::Left));

        Self { table }
    }

    /// Get precedence for an operator
    pub fn get(&self, op: &str) -> Option<PrecedenceEntry> {
        self.table.get(op).copied()
    }

    /// Get precedence level for an operator
    pub fn level(&self, op: &str) -> Option<Precedence> {
        self.table.get(op).map(|e| e.level)
    }

    /// Get associativity for an operator
    pub fn assoc(&self, op: &str) -> Option<Associativity> {
        self.table.get(op).map(|e| e.assoc)
    }

    /// Check if operator is left-associative
    pub fn is_left_assoc(&self, op: &str) -> bool {
        matches!(self.assoc(op), Some(Associativity::Left))
    }

    /// Check if operator is right-associative
    pub fn is_right_assoc(&self, op: &str) -> bool {
        matches!(self.assoc(op), Some(Associativity::Right))
    }

    /// Check if operator is non-associative
    pub fn is_non_assoc(&self, op: &str) -> bool {
        matches!(self.assoc(op), Some(Associativity::None))
    }

    /// Export as JSON
    pub fn to_json(&self) -> serde_json::Result<String> {
        let mut entries: Vec<_> = self
            .table
            .iter()
            .map(|(op, entry)| (op.to_string(), entry))
            .collect();
        entries.sort_by_key(|(_, entry)| entry.level);

        serde_json::to_string_pretty(&entries)
    }
}

impl Default for PrecedenceTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precedence_levels() {
        let table = PrecedenceTable::new();

        // Assignment lowest
        assert_eq!(table.level("="), Some(1));
        assert_eq!(table.level("+="), Some(1));

        // Logical OR lower than AND
        assert_eq!(table.level("||"), Some(4));
        assert_eq!(table.level("&&"), Some(5));
        assert!(table.level("&&").unwrap() > table.level("||").unwrap());

        // Multiplication higher than addition
        assert_eq!(table.level("+"), Some(11));
        assert_eq!(table.level("*"), Some(12));
        assert!(table.level("*").unwrap() > table.level("+").unwrap());

        // Exponentiation higher than multiplication
        assert_eq!(table.level("**"), Some(13));
        assert!(table.level("**").unwrap() > table.level("*").unwrap());

        // Path/access highest
        assert_eq!(table.level("::"), Some(16));
        assert_eq!(table.level("."), Some(15));
    }

    #[test]
    fn test_associativity() {
        let table = PrecedenceTable::new();

        // Assignment is right-associative
        assert_eq!(table.assoc("="), Some(Associativity::Right));
        assert!(table.is_right_assoc("="));

        // Addition is left-associative
        assert_eq!(table.assoc("+"), Some(Associativity::Left));
        assert!(table.is_left_assoc("+"));

        // Comparison is non-associative
        assert_eq!(table.assoc("=="), Some(Associativity::None));
        assert!(table.is_non_assoc("=="));
    }

    #[test]
    fn test_all_operators_defined() {
        let table = PrecedenceTable::new();

        let operators = vec![
            "=", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "<<=", ">>=",
            "|>", "<|", "..", "..=", "...", "||", "&&", "==", "!=", "<", ">",
            "<=", ">=", "|", "^", "&", "<<", ">>", "+", "-", "*", "/", "%",
            "**", "!", "~", ".", "?", "??", "::", "->", "=>",
        ];

        for op in operators {
            assert!(
                table.get(op).is_some(),
                "Operator {} not defined",
                op
            );
        }
    }

    #[test]
    fn test_precedence_ordering() {
        let table = PrecedenceTable::new();

        // Verify precedence ordering makes sense
        assert!(table.level("=").unwrap() < table.level("|>").unwrap());
        assert!(table.level("|>").unwrap() < table.level("||").unwrap());
        assert!(table.level("||").unwrap() < table.level("&&").unwrap());
        assert!(table.level("&&").unwrap() < table.level("==").unwrap());
        assert!(table.level("==").unwrap() < table.level("|").unwrap());
        assert!(table.level("|").unwrap() < table.level("^").unwrap());
        assert!(table.level("^").unwrap() < table.level("&").unwrap());
        assert!(table.level("&").unwrap() < table.level("<<").unwrap());
        assert!(table.level("<<").unwrap() < table.level("+").unwrap());
        assert!(table.level("+").unwrap() < table.level("*").unwrap());
        assert!(table.level("*").unwrap() < table.level("**").unwrap());
        assert!(table.level("**").unwrap() < table.level("!").unwrap());
        assert!(table.level("!").unwrap() < table.level(".").unwrap());
        assert!(table.level(".").unwrap() < table.level("::").unwrap());
    }

    #[test]
    fn test_json_export() {
        let table = PrecedenceTable::new();
        let json = table.to_json().unwrap();

        assert!(json.contains("\"=\""));
        assert!(json.contains("\"level\":"));
        assert!(json.contains("\"assoc\":"));
    }
}
