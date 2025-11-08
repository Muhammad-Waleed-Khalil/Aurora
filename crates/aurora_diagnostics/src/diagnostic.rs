//! Diagnostic system for Aurora compiler

use serde::{Deserialize, Serialize};
use std::fmt;

/// Diagnostic severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Error - compilation must stop
    Error,
    /// Warning - compilation continues
    Warning,
    /// Note - additional information
    Note,
    /// Help - suggestion for user
    Help,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Note => write!(f, "note"),
            Severity::Help => write!(f, "help"),
        }
    }
}

/// Source location span
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    /// Start byte offset
    pub start: usize,
    /// End byte offset
    pub end: usize,
    /// Source file ID
    pub file_id: usize,
}

impl Span {
    /// Create new span
    pub fn new(start: usize, end: usize, file_id: usize) -> Self {
        Self { start, end, file_id }
    }

    /// Get span length
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Check if span is empty
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

/// Fix-it suggestion for automatic repair
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixIt {
    /// Span to replace
    pub span: Span,
    /// Replacement text
    pub replacement: String,
    /// Description of fix
    pub description: String,
}

/// Diagnostic message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Severity level
    pub severity: Severity,
    /// Error code (e.g., "E0001")
    pub code: String,
    /// Primary message
    pub message: String,
    /// Primary span
    pub span: Option<Span>,
    /// Additional labels
    pub labels: Vec<Label>,
    /// Notes
    pub notes: Vec<String>,
    /// Fix-it suggestions
    pub fixes: Vec<FixIt>,
}

/// Label for additional context
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Label {
    /// Label span
    pub span: Span,
    /// Label message
    pub message: String,
    /// Is primary label
    pub primary: bool,
}

impl Diagnostic {
    /// Create new diagnostic
    pub fn new(severity: Severity, code: String, message: String) -> Self {
        Self {
            severity,
            code,
            message,
            span: None,
            labels: Vec::new(),
            notes: Vec::new(),
            fixes: Vec::new(),
        }
    }

    /// Create error diagnostic
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(Severity::Error, code.into(), message.into())
    }

    /// Create warning diagnostic
    pub fn warning(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(Severity::Warning, code.into(), message.into())
    }

    /// Set primary span
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    /// Add label
    pub fn with_label(mut self, span: Span, message: String, primary: bool) -> Self {
        self.labels.push(Label {
            span,
            message,
            primary,
        });
        self
    }

    /// Add note
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Add fix-it suggestion
    pub fn with_fix(mut self, fix: FixIt) -> Self {
        self.fixes.push(fix);
        self
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Diagnostic collector
#[derive(Debug, Default)]
pub struct DiagnosticCollector {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticCollector {
    /// Create new collector
    pub fn new() -> Self {
        Self::default()
    }

    /// Add diagnostic
    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Get all diagnostics
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Check if has errors
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

    /// Sort diagnostics by severity and span
    pub fn sort(&mut self) {
        self.diagnostics.sort_by(|a, b| {
            a.severity.cmp(&b.severity).then_with(|| {
                match (a.span, b.span) {
                    (Some(s1), Some(s2)) => s1.start.cmp(&s2.start),
                    _ => std::cmp::Ordering::Equal,
                }
            })
        });
    }

    /// Emit all diagnostics to stderr
    ///
    /// This formats and prints all diagnostics in a user-friendly format
    /// with source code context.
    pub fn emit(&self, source: &str, path: &std::path::Path) {
        use std::io::{self, Write};

        // Sort diagnostics first
        let mut sorted = self.diagnostics.clone();
        sorted.sort_by(|a, b| {
            a.severity.cmp(&b.severity).then_with(|| {
                match (a.span, b.span) {
                    (Some(s1), Some(s2)) => s1.start.cmp(&s2.start),
                    _ => std::cmp::Ordering::Equal,
                }
            })
        });

        let stderr = io::stderr();
        let mut handle = stderr.lock();

        for diag in &sorted {
            // Print severity and message
            let color = match diag.severity {
                Severity::Error => "\x1b[31;1m",     // Red
                Severity::Warning => "\x1b[33;1m",   // Yellow
                Severity::Note => "\x1b[36;1m",      // Cyan
                Severity::Help => "\x1b[32;1m",      // Green
            };
            let reset = "\x1b[0m";

            let _ = writeln!(
                handle,
                "{}{}: {}{}: {}",
                color,
                diag.severity,
                diag.code,
                reset,
                diag.message
            );

            // Print location if available
            if let Some(span) = diag.span {
                let (line, col) = get_line_col(source, span.start);
                let _ = writeln!(
                    handle,
                    "  {} {}:{}:{}",
                    "-->",
                    path.display(),
                    line,
                    col
                );

                // Print source context
                print_source_context(&mut handle, source, span);
            }

            // Print notes
            for note in &diag.notes {
                let _ = writeln!(handle, "  {} note: {}", "=", note);
            }

            // Print fix-its
            for fix in &diag.fixes {
                let _ = writeln!(handle, "  {} help: {}", "=", fix.description);
            }

            let _ = writeln!(handle);
        }
    }
}

/// Get line and column from byte offset
fn get_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;

    for (i, ch) in source.chars().enumerate() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }

    (line, col)
}

/// Print source context for a span
fn print_source_context(handle: &mut dyn std::io::Write, source: &str, span: Span) {
    let (start_line, _) = get_line_col(source, span.start);

    // Get the line containing the span
    let line_text = source.lines().nth(start_line - 1).unwrap_or("");

    // Print line number and source
    let _ = writeln!(handle, "{:5} | {}", start_line, line_text);

    // Print underline
    let (_, start_col) = get_line_col(source, span.start);
    let underline_len = span.len().max(1).min(line_text.len());

    let _ = write!(handle, "      | ");
    for _ in 0..start_col - 1 {
        let _ = write!(handle, " ");
    }
    for _ in 0..underline_len {
        let _ = write!(handle, "^");
    }
    let _ = writeln!(handle);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Error < Severity::Warning);
        assert!(Severity::Warning < Severity::Note);
        assert!(Severity::Note < Severity::Help);
    }

    #[test]
    fn test_span_operations() {
        let span = Span::new(10, 20, 0);
        assert_eq!(span.len(), 10);
        assert!(!span.is_empty());

        let empty_span = Span::new(10, 10, 0);
        assert!(empty_span.is_empty());
    }

    #[test]
    fn test_diagnostic_creation() {
        let diag = Diagnostic::error("E0001", "test error");
        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.code, "E0001");
        assert_eq!(diag.message, "test error");
    }

    #[test]
    fn test_diagnostic_builder() {
        let span = Span::new(0, 10, 0);
        let diag = Diagnostic::error("E0001", "test")
            .with_span(span)
            .with_note("additional info")
            .with_label(span, "here".to_string(), true);

        assert!(diag.span.is_some());
        assert_eq!(diag.notes.len(), 1);
        assert_eq!(diag.labels.len(), 1);
    }

    #[test]
    fn test_collector() {
        let mut collector = DiagnosticCollector::new();

        collector.add(Diagnostic::error("E0001", "error 1"));
        collector.add(Diagnostic::warning("W0001", "warning 1"));
        collector.add(Diagnostic::error("E0002", "error 2"));

        assert_eq!(collector.error_count(), 2);
        assert_eq!(collector.warning_count(), 1);
        assert!(collector.has_errors());
    }

    #[test]
    fn test_collector_sorting() {
        let mut collector = DiagnosticCollector::new();

        collector.add(Diagnostic::warning("W0001", "warning"));
        collector.add(Diagnostic::error("E0001", "error"));

        collector.sort();

        assert_eq!(collector.diagnostics()[0].severity, Severity::Error);
        assert_eq!(collector.diagnostics()[1].severity, Severity::Warning);
    }

    #[test]
    fn test_fixit() {
        let span = Span::new(0, 5, 0);
        let fix = FixIt {
            span,
            replacement: "fixed".to_string(),
            description: "Fix typo".to_string(),
        };

        assert_eq!(fix.replacement, "fixed");
        assert_eq!(fix.description, "Fix typo");
    }
}
