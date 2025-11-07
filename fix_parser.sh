#!/bin/bash
cd /home/user/Aurora

echo "=== Comprehensive Parser Fix Script ==="

# Step 1: Fix parser.rs - make methods/fields public and add helper
echo "Step 1: Fixing parser.rs..."

cat > crates/aurora_parser/src/parser_fixed.rs << 'EOF'
//! Core parser implementation
//!
//! This module implements the main parser structure that coordinates
//! LL parsing for declarations and Pratt parsing for expressions.

use aurora_ast::{Arena, Program, Span};
use aurora_lexer::{Lexer, Token, TokenKind};
use crate::error::{ParseError, ParseResult};

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

    /// Parse a complete program
    pub fn parse(mut self) -> ParseResult<(Program, Arena)> {
        let items = self.parse_items()?;

        let program = Program::new(items, self.span_from_tokens());

        // Return errors if any were collected
        if !self.errors.is_empty() {
            return Err(ParseError::Multiple(self.errors));
        }

        Ok((program, self.arena))
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
        }];
        let parser = Parser::from_tokens(tokens);
        assert!(parser.is_at_end());
    }
}
EOF

mv crates/aurora_parser/src/parser_fixed.rs crates/aurora_parser/src/parser.rs
echo "✓ parser.rs fixed"

# Step 2: Fix decls.rs - replace .span with token_to_span()
echo "Step 2: Fixing decls.rs spans..."
sed -i 's/self\.current()\.span/self.token_to_span(self.current())/g' crates/aurora_parser/src/decls.rs
sed -i 's/self\.previous()\.span/self.token_to_span(self.previous())/g' crates/aurora_parser/src/decls.rs
echo "✓ decls.rs spans fixed"

# Step 3: Fix exprs.rs - same span fixes
echo "Step 3: Fixing exprs.rs spans..."
sed -i 's/self\.current()\.span/self.token_to_span(self.current())/g' crates/aurora_parser/src/exprs.rs
sed -i 's/self\.previous()\.span/self.token_to_span(self.previous())/g' crates/aurora_parser/src/exprs.rs
echo "✓ exprs.rs spans fixed"

# Step 4: Fix stmts.rs - span fixes
echo "Step 4: Fixing stmts.rs spans..."
sed -i 's/self\.current()\.span/self.token_to_span(self.current())/g' crates/aurora_parser/src/stmts.rs
sed -i 's/self\.previous()\.span/self.token_to_span(self.previous())/g' crates/aurora_parser/src/stmts.rs
echo "✓ stmts.rs spans fixed"

# Step 5: Fix patterns.rs - span fixes
echo "Step 5: Fixing patterns.rs spans..."
sed -i 's/self\.current()\.span/self.token_to_span(self.current())/g' crates/aurora_parser/src/patterns.rs
sed -i 's/self\.previous()\.span/self.token_to_span(self.previous())/g' crates/aurora_parser/src/patterns.rs
echo "✓ patterns.rs spans fixed"

echo ""
echo "=== All fixes applied! ==="
