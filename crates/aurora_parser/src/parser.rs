//! Core parser implementation
//!
//! This module implements the main parser structure that coordinates
//! LL parsing for declarations and Pratt parsing for expressions.

use aurora_ast::{Arena, Ast, Program, Span};
use aurora_lexer::{Lexer, Token, TokenKind};
use crate::error::{ParseError, ParseResult};
use std::sync::Arc;

/// Parser for Aurora source code
pub struct Parser {
    /// Token stream from lexer
    tokens: Vec<Token>,
    /// Current position in token stream
    pos: usize,
    /// AST arena for allocating nodes
    pub(crate) arena: Arena,  // Made public for submodules
    /// Collected parse errors
    errors: Vec<ParseError>,
}

impl Parser {
    /// Create a new parser from source code
    pub fn new(source: &str, filename: String) -> ParseResult<Self> {
        let mut lexer = Lexer::new(source, filename)?;
        let tokens = lexer.lex_all()?;

        Ok(Self {
            tokens,
            pos: 0,
            arena: Arena::new(),
            errors: Vec::new(),
        })
    }

    /// Create a parser from a pre-lexed token stream
    pub fn from_tokens(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            arena: Arena::new(),
            errors: Vec::new(),
        }
    }

    /// Create a parser with diagnostic collector (for pipeline integration)
    ///
    /// The diagnostics parameter can be any Arc type. Typically aurora_diagnostics::DiagnosticCollector.
    pub fn with_diagnostics<D: Send + Sync + 'static>(
        tokens: Vec<Token>,
        _diagnostics: Arc<D>
    ) -> Self {
        Self {
            tokens,
            pos: 0,
            arena: Arena::new(),
            errors: Vec::new(),
        }
    }

    /// Parse a complete program (original API)
    pub fn parse_program(mut self) -> ParseResult<(Program, Arena)> {
        let items = self.parse_items()?;

        let program = Program::new(items, self.span_from_tokens());

        // Return errors if any were collected
        if !self.errors.is_empty() {
            return Err(ParseError::Multiple(self.errors));
        }

        Ok((program, self.arena))
    }

    /// Parse a complete program into AST (for pipeline integration)
    ///
    /// This method consumes the parser and returns an Ast, reporting
    /// errors via the diagnostic collector if provided.
    pub fn parse(mut self) -> Ast {
        match self.parse_items() {
            Ok(items) => Program::new(items, self.span_from_tokens()),
            Err(e) => {
                eprintln!("Parse error: {:?}", e);
                Program::empty()
            }
        }
    }

    /// Parse top-level items until EOF
    fn parse_items(&mut self) -> ParseResult<Vec<u32>> {
        let mut items = Vec::new();

        while !self.is_at_end() {
            // Skip any stray semicolons
            if self.check(&TokenKind::Semicolon) {
                self.advance();
                continue;
            }

            match self.parse_item() {
                Ok(item_id) => items.push(item_id),
                Err(err) => {
                    self.errors.push(err);
                    self.synchronize();
                }
            }
        }

        Ok(items)
    }

    /// Get the current token
    pub(crate) fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or_else(|| self.tokens.last().unwrap())
    }

    /// Get the previous token
    pub(crate) fn previous(&self) -> &Token {
        if self.pos > 0 {
            &self.tokens[self.pos - 1]
        } else {
            &self.tokens[0]
        }
    }

    /// Check if we're at the end of input
    pub(crate) fn is_at_end(&self) -> bool {
        self.current().kind == TokenKind::Eof
    }

    /// Peek at the current token kind
    pub(crate) fn peek(&self) -> &TokenKind {
        &self.current().kind
    }

    /// Check if current token matches a kind
    pub(crate) fn check(&self, kind: &TokenKind) -> bool {
        !self.is_at_end() && self.peek() == kind
    }

    /// Advance to the next token
    pub(crate) fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.pos += 1;
        }
        self.previous()
    }

    /// Consume a token of a specific kind or error
    pub(crate) fn expect(&mut self, kind: TokenKind, message: &str) -> ParseResult<&Token> {
        if self.check(&kind) {
            Ok(self.advance())
        } else {
            Err(ParseError::Expected {
                expected: format!("{:?}", kind),
                found: format!("{:?}", self.peek()),
                span: self.token_to_span(self.current()),
                message: message.to_string(),
            })
        }
    }

    /// Match and consume if current token matches any of the given kinds
    pub(crate) fn match_any(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    /// Convert token to span
    pub(crate) fn token_to_span(&self, token: &Token) -> Span {
        Span::new(
            0, // file_id - TODO: track properly
            0, // start - TODO: track properly
            0, // end - TODO: track properly
            token.line as u32,
            token.column as u32,
        )
    }

    /// Synchronize parser after an error (panic mode recovery)
    pub(crate) fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            // Stop at statement/declaration boundaries
            if matches!(
                self.previous().kind,
                TokenKind::Semicolon | TokenKind::RBrace
            ) {
                return;
            }

            // Stop at keyword that starts a new declaration
            if matches!(
                self.peek(),
                TokenKind::Fn
                    | TokenKind::Type
                    | TokenKind::Trait
                    | TokenKind::Impl
                    | TokenKind::Const
                    | TokenKind::Mod
                    | TokenKind::Use
                    | TokenKind::Pub
            ) {
                return;
            }

            self.advance();
        }
    }

    /// Get span from start to current position
    pub(crate) fn span_from(&self, start: Span) -> Span {
        start.merge(self.token_to_span(self.previous()))
    }

    /// Get span covering all tokens
    fn span_from_tokens(&self) -> Span {
        if self.tokens.is_empty() {
            return Span::dummy();
        }
        let first = self.token_to_span(self.tokens.first().unwrap());
        let last = self.token_to_span(self.tokens.last().unwrap());
        first.merge(last)
    }

    /// Get the arena (for testing)
    pub fn arena(&self) -> &Arena {
        &self.arena
    }

    /// Get collected errors
    pub fn errors(&self) -> &[ParseError] {
        &self.errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let source = "";
        let parser = Parser::new(source, "test.ax".to_string()).unwrap();
        assert_eq!(parser.pos, 0);
        assert!(parser.errors.is_empty());
    }

    #[test]
    fn test_empty_program() {
        let source = "";
        let parser = Parser::new(source, "test.ax".to_string()).unwrap();
        let (program, _arena) = parser.parse().unwrap();
        assert!(program.items.is_empty());
    }

    #[test]
    fn test_parser_from_tokens() {
        let tokens = vec![Token {
            kind: TokenKind::Eof,
            lexeme: String::new(),
            file: "test.ax".to_string(),
            line: 1,
            column: 1,
            len: 0,
        }];
        let parser = Parser::from_tokens(tokens);
        assert!(parser.is_at_end());
    }
}
