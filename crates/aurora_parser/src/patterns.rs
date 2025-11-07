//! Pattern parsing
//!
//! This module implements parsing for Aurora pattern expressions used in:
//! - Let bindings
//! - Function parameters
//! - Match arms
//! - For loop bindings

use aurora_ast::pattern::{FieldPattern, Pattern, PatternKind};
use aurora_ast::expr::{Literal, Path};
use aurora_lexer::TokenKind;
use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;

impl Parser {
    /// Parse a pattern expression
    pub(crate) fn parse_pattern(&mut self) -> ParseResult<u32> {
        let start = self.current().span;
        
        let kind = match self.peek() {
            // Wildcard pattern (_)
            TokenKind::Underscore => {
                self.advance();
                PatternKind::Wildcard
            }
            
            // Identifier pattern
            TokenKind::Ident => {
                let name = self.current().lexeme.clone();
                self.advance();
                
                // Check if it's a struct pattern
                if self.check(&TokenKind::LBrace) {
                    // This is a struct pattern
                    let path = Path {
                        segments: vec![name],
                        generics: vec![],
                    };
                    return self.parse_struct_pattern(path, start);
                }
                
                // Check for 'mut' prefix
                let mutable = if name == "mut" {
                    let ident_token = self.expect(TokenKind::Ident, "Expected identifier after 'mut'")?;
                    let actual_name = ident_token.lexeme.clone();
                    PatternKind::Ident { name: actual_name, mutable: true }
                } else {
                    PatternKind::Ident { name, mutable: false }
                };
                
                mutable
            }
            
            // Literal patterns
            TokenKind::IntLiteral(n) => {
                let n = *n;
                self.advance();
                PatternKind::Literal(Literal::Int(n))
            }
            TokenKind::FloatLiteral(f) => {
                let f = *f;
                self.advance();
                PatternKind::Literal(Literal::Float(f))
            }
            TokenKind::StringLiteral => {
                let s = self.current().lexeme.clone();
                self.advance();
                PatternKind::Literal(Literal::String(s))
            }
            TokenKind::CharLiteral => {
                let c = self.current().lexeme.chars().next().unwrap_or('?');
                self.advance();
                PatternKind::Literal(Literal::Char(c))
            }
            TokenKind::True => {
                self.advance();
                PatternKind::Literal(Literal::Bool(true))
            }
            TokenKind::False => {
                self.advance();
                PatternKind::Literal(Literal::Bool(false))
            }
            
            // Tuple pattern
            TokenKind::LParen => {
                self.advance();
                let mut patterns = Vec::new();
                
                if !self.check(&TokenKind::RParen) {
                    loop {
                        patterns.push(self.parse_pattern()?);
                        
                        if !self.check(&TokenKind::Comma) {
                            break;
                        }
                        self.advance();
                        
                        if self.check(&TokenKind::RParen) {
                            break;
                        }
                    }
                }
                
                self.expect(TokenKind::RParen, "Expected ')' after tuple pattern")?;
                PatternKind::Tuple(patterns)
            }
            
            _ => {
                return Err(ParseError::Expected {
                    expected: "pattern".to_string(),
                    found: format!("{:?}", self.peek()),
                    span: self.current().span,
                    message: "Expected a pattern expression".to_string(),
                });
            }
        };
        
        let span = self.span_from(start);
        let pattern = Pattern { kind, span };
        Ok(self.arena.alloc_pattern(pattern))
    }
    
    /// Parse a struct pattern
    fn parse_struct_pattern(&mut self, path: Path, start: aurora_ast::Span) -> ParseResult<u32> {
        self.expect(TokenKind::LBrace, "Expected '{'")?;
        
        let mut fields = Vec::new();
        
        if !self.check(&TokenKind::RBrace) {
            loop {
                let field_start = self.current().span;
                let field_name_token = self.expect(TokenKind::Ident, "Expected field name")?;
                let field_name = field_name_token.lexeme.clone();
                
                let pattern = if self.check(&TokenKind::Colon) {
                    self.advance();
                    self.parse_pattern()?
                } else {
                    // Shorthand: `{ x }` means `{ x: x }`
                    let ident_pattern = Pattern {
                        kind: PatternKind::Ident {
                            name: field_name.clone(),
                            mutable: false,
                        },
                        span: field_start,
                    };
                    self.arena.alloc_pattern(ident_pattern)
                };
                
                let field_span = self.span_from(field_start);
                fields.push(FieldPattern {
                    name: field_name,
                    pattern,
                    span: field_span,
                });
                
                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
                
                if self.check(&TokenKind::RBrace) {
                    break;
                }
            }
        }
        
        self.expect(TokenKind::RBrace, "Expected '}' after struct pattern")?;
        
        let span = self.span_from(start);
        let pattern = Pattern {
            kind: PatternKind::Struct { path, fields },
            span,
        };
        Ok(self.arena.alloc_pattern(pattern))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    
    #[test]
    fn test_parse_identifier_pattern() {
        let source = "fn test(x: i32) {}";
        let parser = Parser::new(source, "test.ax".to_string()).unwrap();
        let (_program, _arena) = parser.parse().unwrap();
    }
    
    #[test]
    fn test_parse_mut_pattern() {
        let source = "fn test(mut x: i32) {}";
        let parser = Parser::new(source, "test.ax".to_string()).unwrap();
        let (_program, _arena) = parser.parse().unwrap();
    }
    
    #[test]
    fn test_parse_wildcard_pattern() {
        let source = "fn test(_: i32) {}";
        let parser = Parser::new(source, "test.ax".to_string()).unwrap();
        let (_program, _arena) = parser.parse().unwrap();
    }
}
