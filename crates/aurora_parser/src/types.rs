//! Type parsing
//!
//! This module implements parsing for Aurora type expressions including:
//! - Primitives (i32, f64, bool, etc.)
//! - Paths (std::io::Read)
//! - Tuples ((i32, String))
//! - Arrays ([i32; 10], [i32])
//! - References (&T, &mut T)
//! - Function types (fn(i32) -> String)

use aurora_ast::ty::{FloatType, IntType, Type, TypeKind, UintType};
use aurora_ast::expr::Path;
use aurora_lexer::TokenKind;
use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;

impl Parser {
    /// Parse a type expression
    pub(crate) fn parse_type(&mut self) -> ParseResult<u32> {
        let start = self.token_to_span(self.current());
        
        let kind = match self.peek() {
            // Signed integer types
            TokenKind::I8 => {
                self.advance();
                TypeKind::Int(IntType::I8)
            }
            TokenKind::I16 => {
                self.advance();
                TypeKind::Int(IntType::I16)
            }
            TokenKind::I32 => {
                self.advance();
                TypeKind::Int(IntType::I32)
            }
            TokenKind::I64 => {
                self.advance();
                TypeKind::Int(IntType::I64)
            }
            
            // Unsigned integer types
            TokenKind::U8 => {
                self.advance();
                TypeKind::Uint(UintType::U8)
            }
            TokenKind::U16 => {
                self.advance();
                TypeKind::Uint(UintType::U16)
            }
            TokenKind::U32 => {
                self.advance();
                TypeKind::Uint(UintType::U32)
            }
            TokenKind::U64 => {
                self.advance();
                TypeKind::Uint(UintType::U64)
            }
            
            // Float types
            TokenKind::F32 => {
                self.advance();
                TypeKind::Float(FloatType::F32)
            }
            TokenKind::F64 => {
                self.advance();
                TypeKind::Float(FloatType::F64)
            }
            
            // Other primitives
            TokenKind::Bool => {
                self.advance();
                TypeKind::Bool
            }
            TokenKind::Char => {
                self.advance();
                TypeKind::Char
            }
            TokenKind::Str => {
                self.advance();
                TypeKind::Str
            }
            
            // Reference types
            TokenKind::And => {
                self.advance();
                let is_mut = if self.check(&TokenKind::Mut) {
                    self.advance();
                    true
                } else {
                    false
                };
                let inner = Box::new(self.parse_type()?);
                TypeKind::Reference { inner, is_mut }
            }
            
            // Tuple types
            TokenKind::LParen => {
                self.advance();
                let mut types = Vec::new();
                
                if !self.check(&TokenKind::RParen) {
                    loop {
                        types.push(self.parse_type()?);
                        
                        if !self.check(&TokenKind::Comma) {
                            break;
                        }
                        self.advance();
                        
                        if self.check(&TokenKind::RParen) {
                            break;
                        }
                    }
                }
                
                self.expect(TokenKind::RParen, "Expected ')' after tuple type")?;
                TypeKind::Tuple(types)
            }
            
            // Array/slice types
            TokenKind::LBracket => {
                self.advance();
                let element = Box::new(self.parse_type()?);
                
                if self.check(&TokenKind::Semicolon) {
                    // Array type with size
                    self.advance();
                    let length = self.parse_expr()?;
                    self.expect(TokenKind::RBracket, "Expected ']' after array type")?;
                    TypeKind::Array { element, length }
                } else {
                    // Slice type
                    self.expect(TokenKind::RBracket, "Expected ']' after slice type")?;
                    TypeKind::Slice { element }
                }
            }
            
            // Function types
            TokenKind::Fn => {
                self.advance();
                self.expect(TokenKind::LParen, "Expected '(' after 'fn'")?;
                
                let mut params = Vec::new();
                if !self.check(&TokenKind::RParen) {
                    loop {
                        params.push(self.parse_type()?);
                        
                        if !self.check(&TokenKind::Comma) {
                            break;
                        }
                        self.advance();
                        
                        if self.check(&TokenKind::RParen) {
                            break;
                        }
                    }
                }
                
                self.expect(TokenKind::RParen, "Expected ')' after function parameters")?;
                
                let return_type = if self.check(&TokenKind::RArrow) {
                    self.advance();
                    Some(Box::new(self.parse_type()?))
                } else {
                    None
                };
                
                TypeKind::Function { params, return_type }
            }
            
            // Inferred type
            TokenKind::Underscore => {
                self.advance();
                TypeKind::Infer
            }
            
            // Path types (e.g., String, Vec<T>, std::io::Read)
            TokenKind::Ident => {
                let path = self.parse_path()?;
                TypeKind::Path { path }
            }
            
            _ => {
                return Err(ParseError::Expected {
                    expected: "type".to_string(),
                    found: format!("{:?}", self.peek()),
                    span: self.token_to_span(self.current()),
                    message: "Expected a type expression".to_string(),
                });
            }
        };
        
        let span = self.span_from(start);
        let ty = Type { kind, span };
        Ok(self.arena.alloc_type(ty))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    
    #[test]
    fn test_parse_int_type() {
        let source = "fn test() -> i32 {}";
        let parser = Parser::new(source, "test.ax".to_string()).unwrap();
        let (_program, _arena) = parser.parse_program().unwrap();
    }

    #[test]
    fn test_parse_tuple_type() {
        let source = "fn test() -> (i32, f64) {}";
        let parser = Parser::new(source, "test.ax".to_string()).unwrap();
        let (_program, _arena) = parser.parse_program().unwrap();
    }

    #[test]
    fn test_parse_ref_type() {
        let source = "fn test(x: &i32, y: &mut String) {}";
        let parser = Parser::new(source, "test.ax".to_string()).unwrap();
        let (_program, _arena) = parser.parse_program().unwrap();
    }
}
