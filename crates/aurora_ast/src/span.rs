// ! Source location tracking for AST nodes
//!
//! This module defines span information that tracks the location of AST nodes
//! in the source code, enabling precise error reporting and source mapping.

use serde::{Deserialize, Serialize};

/// A span representing a location in source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    /// File ID (index into file table)
    pub file_id: u32,
    /// Starting byte offset
    pub start: u32,
    /// Ending byte offset (exclusive)
    pub end: u32,
    /// Starting line number (1-indexed)
    pub line: u32,
    /// Starting column number (1-indexed)
    pub column: u32,
}

impl Span {
    /// Create a new span
    pub fn new(file_id: u32, start: u32, end: u32, line: u32, column: u32) -> Self {
        Self {
            file_id,
            start,
            end,
            line,
            column,
        }
    }

    /// Create a dummy span (for generated nodes)
    pub fn dummy() -> Self {
        Self {
            file_id: 0,
            start: 0,
            end: 0,
            line: 0,
            column: 0,
        }
    }

    /// Get the length of this span in bytes
    pub fn len(&self) -> u32 {
        self.end - self.start
    }

    /// Check if this span is empty
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Merge two spans into a span covering both
    pub fn merge(self, other: Span) -> Span {
        debug_assert_eq!(self.file_id, other.file_id);
        Span {
            file_id: self.file_id,
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            line: self.line.min(other.line),
            column: self.column.min(other.column),
        }
    }
}

impl Default for Span {
    fn default() -> Self {
        Self::dummy()
    }
}

/// Hygiene identifier for macro expansion
///
/// Each identifier gets a unique hygiene ID that tracks its lexical context.
/// This prevents accidental variable capture in macro expansion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HygieneId(pub u32);

impl HygieneId {
    /// Create a new hygiene ID
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the root hygiene context (no macros)
    pub fn root() -> Self {
        Self(0)
    }
}

impl Default for HygieneId {
    fn default() -> Self {
        Self::root()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_creation() {
        let span = Span::new(1, 10, 20, 1, 5);
        assert_eq!(span.file_id, 1);
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 20);
        assert_eq!(span.len(), 10);
        assert!(!span.is_empty());
    }

    #[test]
    fn test_span_merge() {
        let span1 = Span::new(1, 10, 20, 1, 5);
        let span2 = Span::new(1, 15, 30, 1, 10);
        let merged = span1.merge(span2);
        assert_eq!(merged.start, 10);
        assert_eq!(merged.end, 30);
    }

    #[test]
    fn test_hygiene_id() {
        let id = HygieneId::new(42);
        assert_eq!(id.0, 42);

        let root = HygieneId::root();
        assert_eq!(root.0, 0);
    }
}
