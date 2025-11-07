//! Statement parsing
//!
//! This module implements parsing for Aurora statements including:
//! - Let bindings
//! - Expression statements
//! - Items in statement position

use aurora_ast::{Stmt, StmtKind};
use aurora_lexer::TokenKind;
use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;

impl Parser {
    /// Parse a statement
    pub(crate) fn parse_stmt(&mut self) -> ParseResult<u32> {
        let start = self.token_to_span(self.current());
        
        // Check for let binding
        if self.check(&TokenKind::Let) {
            return self.parse_let_stmt(start);
        }
        
        // Check for item in statement position
        if self.check(&TokenKind::Fn) || self.check(&TokenKind::Type)
            || self.check(&TokenKind::Trait) || self.check(&TokenKind::Impl)
            || self.check(&TokenKind::Const) || self.check(&TokenKind::Mod)
            || self.check(&TokenKind::Use) {
            let item_id = self.parse_item()?;
            let span = self.span_from(start);
            let stmt = Stmt {
                kind: StmtKind::Item(item_id),
                span,
            };
            return Ok(self.arena.alloc_stmt(stmt));
        }
        
        // Otherwise, it's an expression statement
        let expr = self.parse_expr()?;
        
        // Check for semicolon
        let has_semi = if self.check(&TokenKind::Semicolon) {
            self.advance();
            true
        } else {
            false
        };
        
        let span = self.span_from(start);
        let stmt = Stmt {
            kind: StmtKind::Expr { expr, has_semi },
            span,
        };
        Ok(self.arena.alloc_stmt(stmt))
    }
    
    /// Parse a let statement
    fn parse_let_stmt(&mut self, start: aurora_ast::Span) -> ParseResult<u32> {
        self.expect(TokenKind::Let, "Expected 'let'")?;
        
        // Check for 'mut'
        let mutable = if self.check(&TokenKind::Mut) {
            self.advance();
            true
        } else {
            false
        };
        
        // Parse pattern
        let pattern = self.parse_pattern()?;
        
        // Optional type annotation
        let ty = if self.check(&TokenKind::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        
        // Optional initializer
        let init = if self.check(&TokenKind::Eq) {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };
        
        // Expect semicolon
        self.expect(TokenKind::Semicolon, "Expected ';' after let statement")?;
        
        let span = self.span_from(start);
        let stmt = Stmt {
            kind: StmtKind::Let { pattern, ty, init, mutable },
            span,
        };
        Ok(self.arena.alloc_stmt(stmt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    
    #[test]
    fn test_parse_let_stmt() {
        let source = "fn test() { let x = 42; }";
        let parser = Parser::new(source, "test.ax".to_string()).unwrap();
        let (_program, _arena) = parser.parse().unwrap();
    }
    
    #[test]
    fn test_parse_let_mut_stmt() {
        let source = "fn test() { let mut x = 42; }";
        let parser = Parser::new(source, "test.ax".to_string()).unwrap();
        let (_program, _arena) = parser.parse().unwrap();
    }
    
    #[test]
    fn test_parse_let_with_type() {
        let source = "fn test() { let x: i32 = 42; }";
        let parser = Parser::new(source, "test.ax".to_string()).unwrap();
        let (_program, _arena) = parser.parse().unwrap();
    }
    
    #[test]
    fn test_parse_expr_stmt() {
        let source = "fn test() { foo(); bar(); }";
        let parser = Parser::new(source, "test.ax".to_string()).unwrap();
        let (_program, _arena) = parser.parse().unwrap();
    }
}
