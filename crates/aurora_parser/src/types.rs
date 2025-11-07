//! Type parsing
//!
//! This module implements parsing for Aurora type expressions including:
//! - Primitives (i32, f64, bool, etc.)
//! - Paths (std::io::Read)
//! - Tuples ((i32, String))
//! - Arrays ([i32; 10], [i32])
//! - References (&T, &mut T)
//! - Function types (fn(i32) -> String)

use aurora_ast::ty::{PrimitiveType, Type, TypeKind};
use aurora_ast::expr::Path;
use aurora_lexer::TokenKind;
use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;

impl Parser {
    /// Parse a type expression
    pub(crate) fn parse_type(&mut self) -> ParseResult<u32> {
        let start = self.current().span;
        
        let kind = match self.peek() {
            // Primitive types
            TokenKind::I8 => {
                self.advance();
                TypeKind::Primitive(PrimitiveType::I8)
            }
            TokenKind::I16 => {
                self.advance();
                TypeKind::Primitive(PrimitiveType::I16)
            }
            TokenKind::I32 => {
                self.advance();
                TypeKind::Primitive(PrimitiveType::I32)
            }
            TokenKind::I64 => {
                self.advance();
                TypeKind::Primitive(PrimitiveType::I64)
            }
            TokenKind::I128 => {
                self.advance();
                TypeKind::Primitive(PrimitiveType::I128)
            }
            TokenKind::U8 => {
                self.advance();
                TypeKind::Primitive(PrimitiveType::U8)
            }
            TokenKind::U16 => {
                self.advance();
                TypeKind::Primitive(PrimitiveType::U16)
            }
            TokenKind::U32 => {
                self.advance();
                TypeKind::Primitive(PrimitiveType::U32)
            }
            TokenKind::U64 => {
                self.advance();
                TypeKind::Primitive(PrimitiveType::U64)
            }
            TokenKind::U128 => {
                self.advance();
                TypeKind::Primitive(PrimitiveType::U128)
            }
            TokenKind::F32 => {
                self.advance();
                TypeKind::Primitive(PrimitiveType::F32)
            }
            TokenKind::F64 => {
                self.advance();
                TypeKind::Primitive(PrimitiveType::F64)
            }
            TokenKind::Bool => {
                self.advance();
                TypeKind::Primitive(PrimitiveType::Bool)
            }
            TokenKind::Char => {
                self.advance();
                TypeKind::Primitive(PrimitiveType::Char)
            }
            TokenKind::Str => {
                self.advance();
                TypeKind::Primitive(PrimitiveType::Str)
            }
            
            // Reference types
            TokenKind::And => {
                self.advance();
                let mutable = if self.check(&TokenKind::Mut) {
                    self.advance();
                    true
                } else {
                    false
                };
                let inner = self.parse_type()?;
                TypeKind::Ref { mutable, inner }
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
                
                // Empty tuple is unit type
                if types.is_empty() {
                    TypeKind::Primitive(PrimitiveType::Unit)
                } else {
                    TypeKind::Tuple(types)
                }
            }
            
            // Array/slice types
            TokenKind::LBracket => {
                self.advance();
                let element = self.parse_type()?;
                
                let size = if self.check(&TokenKind::Semicolon) {
                    self.advance();
                    // Parse array size (for now, just expect a number)
                    if let TokenKind::IntLiteral(n) = self.peek() {
                        let size = *n as usize;
                        self.advance();
                        Some(size)
                    } else {
                        return Err(ParseError::Expected {
                            expected: "array size".to_string(),
                            found: format!("{:?}", self.peek()),
                            span: self.current().span,
                            message: "Expected array size after ';'".to_string(),
                        });
                    }
                } else {
                    None // Slice type
                };
                
                self.expect(TokenKind::RBracket, "Expected ']' after array type")?;
                TypeKind::Array { element, size }
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
                    self.parse_type()?
                } else {
                    // Unit type by default
                    let unit = Type {
                        kind: TypeKind::Primitive(PrimitiveType::Unit),
                        span: self.current().span,
                    };
                    self.arena.alloc_type(unit)
                };
                
                TypeKind::Fn { params, return_type }
            }
            
            // Path types (e.g., String, Vec<T>, std::io::Read)
            TokenKind::Ident => {
                let path = self.parse_path()?;
                TypeKind::Path(path)
            }
            
            _ => {
                return Err(ParseError::Expected {
                    expected: "type".to_string(),
                    found: format!("{:?}", self.peek()),
                    span: self.current().span,
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
    fn test_parse_primitive_types() {
        let types = vec!["i32", "f64", "bool", "char", "str"];
        
        for type_str in types {
            let parser = Parser::new(type_str, "test.ax".to_string()).unwrap();
            // We'd need to expose parse_type publicly or test through declarations
        }
    }
    
    #[test]
    fn test_parse_tuple_type() {
        let source = "fn test() -> (i32, f64) {}";
        let parser = Parser::new(source, "test.ax".to_string()).unwrap();
        let (_program, _arena) = parser.parse().unwrap();
    }
    
    #[test]
    fn test_parse_array_type() {
        let source = "fn test() -> [i32; 10] {}";
        let parser = Parser::new(source, "test.ax".to_string()).unwrap();
        let (_program, _arena) = parser.parse().unwrap();
    }
    
    #[test]
    fn test_parse_ref_type() {
        let source = "fn test(x: &i32, y: &mut String) {}";
        let parser = Parser::new(source, "test.ax".to_string()).unwrap();
        let (_program, _arena) = parser.parse().unwrap();
    }
}
