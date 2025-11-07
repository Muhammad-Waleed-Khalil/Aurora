//! Parser error types

use aurora_ast::Span;
use aurora_lexer::LexError;
use thiserror::Error;

/// Parser error type
#[derive(Debug, Error)]
pub enum ParseError {
    /// Lexer error
    #[error("Lexer error: {0}")]
    Lexer(#[from] LexError),

    /// Expected a specific token but found something else
    #[error("Expected {expected}, found {found} at {span:?}: {message}")]
    Expected {
        /// What was expected
        expected: String,
        /// What was found
        found: String,
        /// Location of error
        span: Span,
        /// Additional context
        message: String,
    },

    /// Unexpected token
    #[error("Unexpected token {token:?} at {span:?}: {message}")]
    Unexpected {
        /// The unexpected token
        token: String,
        /// Location of error
        span: Span,
        /// Additional context
        message: String,
    },

    /// Invalid syntax
    #[error("Invalid syntax at {span:?}: {message}")]
    InvalidSyntax {
        /// Location of error
        span: Span,
        /// Error message
        message: String,
    },

    /// Multiple errors collected
    #[error("Multiple parse errors: {}", .0.len())]
    Multiple(Vec<ParseError>),

    /// Incomplete parse (unexpected EOF)
    #[error("Unexpected end of input at {span:?}: {message}")]
    UnexpectedEof {
        /// Location of error
        span: Span,
        /// Error message
        message: String,
    },
}

/// Parser result type
pub type ParseResult<T> = Result<T, ParseError>;
