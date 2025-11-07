///! Diagnostics - Structured Error and Warning Reporting
///!
///! This module defines the machine-readable diagnostic format used
///! throughout the Aurora compiler. All diagnostics follow a stable JSON schema.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Diagnostic severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Fatal error preventing compilation
    Error,
    /// Warning about potential issues
    Warning,
    /// Informational message
    Info,
    /// Hint or suggestion
    Hint,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
            Severity::Hint => write!(f, "hint"),
        }
    }
}

/// Source code span (location in file)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    /// File path
    pub file: String,
    /// Start line (1-indexed)
    pub line: usize,
    /// Start column (1-indexed)
    pub column: usize,
    /// Length in characters
    pub length: usize,
    /// Optional label for this span
    pub label: Option<String>,
}

impl Span {
    /// Create a new span
    pub fn new(file: String, line: usize, column: usize, length: usize) -> Self {
        Self {
            file,
            line,
            column,
            length,
            label: None,
        }
    }

    /// Add a label to this span
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// Code fix suggestion
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FixIt {
    /// Description of the fix
    pub description: String,
    /// Text edits to apply
    pub edits: Vec<TextEdit>,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
}

/// Text edit for a fix-it
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextEdit {
    /// Span to replace
    pub span: Span,
    /// Replacement text
    pub replacement: String,
}

/// Diagnostic category
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticCategory {
    /// Lexer errors
    Lexer,
    /// Parser errors
    Parser,
    /// Name resolution errors
    NameResolution,
    /// Type system errors
    TypeSystem,
    /// Borrow checker messages
    BorrowChecker,
    /// Effect system messages
    EffectSystem,
    /// MIR errors
    Mir,
    /// AIR errors
    Air,
    /// Codegen errors
    Codegen,
    /// Linker errors
    Linker,
    /// General compiler errors
    General,
}

/// A single diagnostic message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Unique diagnostic ID (e.g., "E0501")
    pub id: String,
    /// Category
    pub category: DiagnosticCategory,
    /// Severity level
    pub severity: Severity,
    /// Human-readable message
    pub message: String,
    /// Primary span (main error location)
    pub primary_span: Span,
    /// Secondary spans (related locations)
    pub secondary_spans: Vec<Span>,
    /// Fix-it suggestions
    pub fix_its: Vec<FixIt>,
    /// Documentation URL
    pub doc_url: Option<String>,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
}

impl Diagnostic {
    /// Create a new diagnostic
    pub fn new(
        id: impl Into<String>,
        category: DiagnosticCategory,
        severity: Severity,
        message: impl Into<String>,
        primary_span: Span,
    ) -> Self {
        Self {
            id: id.into(),
            category,
            severity,
            message: message.into(),
            primary_span,
            secondary_spans: Vec::new(),
            fix_its: Vec::new(),
            doc_url: None,
            confidence: 1.0,
        }
    }

    /// Add a secondary span
    pub fn with_secondary_span(mut self, span: Span) -> Self {
        self.secondary_spans.push(span);
        self
    }

    /// Add a fix-it suggestion
    pub fn with_fix_it(mut self, fix_it: FixIt) -> Self {
        self.fix_its.push(fix_it);
        self
    }

    /// Add documentation URL
    pub fn with_doc_url(mut self, url: impl Into<String>) -> Self {
        self.doc_url = Some(url.into());
        self
    }

    /// Set confidence score
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {}: {} ({}:{}:{})",
            self.severity,
            self.id,
            self.message,
            self.primary_span.file,
            self.primary_span.line,
            self.primary_span.column
        )
    }
}

/// Bundle of diagnostics from a compilation phase
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagnosticsBundle {
    /// All diagnostics
    pub diagnostics: Vec<Diagnostic>,
}

impl DiagnosticsBundle {
    /// Create an empty bundle
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }

    /// Add a diagnostic
    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }

    /// Get error count
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count()
    }

    /// Get warning count
    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count()
    }

    /// Export to JSON
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    /// Import from JSON
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }
}

impl fmt::Display for DiagnosticsBundle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for diag in &self.diagnostics {
            writeln!(f, "{}", diag)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_creation() {
        let span = Span::new("test.ax".to_string(), 10, 5, 8);
        let diag = Diagnostic::new(
            "E0001",
            DiagnosticCategory::TypeSystem,
            Severity::Error,
            "Type mismatch",
            span,
        );

        assert_eq!(diag.id, "E0001");
        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.message, "Type mismatch");
    }

    #[test]
    fn test_diagnostics_bundle() {
        let mut bundle = DiagnosticsBundle::new();
        assert!(!bundle.has_errors());

        let span = Span::new("test.ax".to_string(), 10, 5, 8);
        let diag = Diagnostic::new(
            "E0001",
            DiagnosticCategory::TypeSystem,
            Severity::Error,
            "Type mismatch",
            span,
        );
        bundle.add(diag);

        assert!(bundle.has_errors());
        assert_eq!(bundle.error_count(), 1);
    }

    #[test]
    fn test_json_roundtrip() {
        let mut bundle = DiagnosticsBundle::new();
        let span = Span::new("test.ax".to_string(), 10, 5, 8);
        let diag = Diagnostic::new(
            "E0001",
            DiagnosticCategory::TypeSystem,
            Severity::Error,
            "Type mismatch",
            span,
        );
        bundle.add(diag);

        let json = bundle.to_json().unwrap();
        let restored = DiagnosticsBundle::from_json(&json).unwrap();

        assert_eq!(bundle.diagnostics.len(), restored.diagnostics.len());
        assert_eq!(bundle.diagnostics[0].id, restored.diagnostics[0].id);
    }
}
