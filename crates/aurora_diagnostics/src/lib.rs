//! aurora_diagnostics - Structured Diagnostics and LSP
//!
//! Provides JSON diagnostics, fix-its, and Language Server Protocol support.
//!
//! # Example
//!
//! ```
//! use aurora_diagnostics::diagnostic::{Diagnostic, Severity, Span};
//!
//! let span = Span::new(0, 10, 0);
//! let diag = Diagnostic::error("E0001", "undefined variable")
//!     .with_span(span)
//!     .with_note("consider declaring the variable");
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

/// Diagnostic system
pub mod diagnostic;

/// Language Server Protocol support
pub mod lsp;

// Re-export main types
pub use diagnostic::{Diagnostic, DiagnosticCollector, FixIt, Label, Severity, Span};
pub use lsp::{
    CodeAction, CompletionItem, CompletionKind, DocumentSymbol, Hover, Position, Range,
    SymbolKind, TextEdit,
};
