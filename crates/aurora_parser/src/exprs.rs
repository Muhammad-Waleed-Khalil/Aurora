//! Expression parsing using Pratt parser
//!
//! This module implements a Pratt (precedence climbing) parser for expressions.
//! The parser handles all Aurora operators with correct precedence and associativity.

use aurora_ast::expr::{
    BinaryOp, Expr, ExprKind, FieldInit, GenericArg, Literal, MatchArm, Path, UnaryOp,
};
use aurora_ast::Span;
use aurora_lexer::TokenKind;
use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;

/// Precedence levels (higher number = higher precedence)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum Precedence {
    None = 0,
    Assignment = 1,      // = += -= *= /= etc.
    Propagation = 2,     // ?
    Pipeline = 3,        // |> <|
    Range = 4,           // .. ..=
    LogicalOr = 5,       // ||
    LogicalAnd = 6,      // &&
    Comparison = 7,      // == != < <= > >=
    BitwiseOr = 8,       // |
    BitwiseXor = 9,      // ^
    BitwiseAnd = 10,     // &
    Shift = 11,          // << >>
    Additive = 12,       // + -
    Multiplicative = 13, // * / %
    Exponentiation = 14, // **
    Unary = 15,          // - ! ~ * & &mut
    Call = 16,           // . :: () []
}

impl Parser {
    /// Parse an expression with given minimum precedence
    pub(crate) fn parse_expr(&mut self) -> ParseResult<u32> {
        self.parse_expr_with_precedence(Precedence::None)
    }
    
    /// Parse expression with precedence climbing
    fn parse_expr_with_precedence(&mut self, min_prec: Precedence) -> ParseResult<u32> {
        let start = self.token_to_span(self.current());
        
        // Parse prefix/primary expression
        let mut left = self.parse_prefix_expr()?;
        
        // Parse infix/postfix operators
        while !self.is_at_end() {
            let prec = self.get_infix_precedence();
            
            if prec < min_prec {
                break;
            }
            
            left = self.parse_infix_expr(left, prec, start)?;
        }
        
        Ok(left)
    }
    
    /// Parse prefix/primary expression
    fn parse_prefix_expr(&mut self) -> ParseResult<u32> {
        let start = self.token_to_span(self.current());
        
        let kind = match self.peek() {
            // Literals
            TokenKind::IntLiteral => {
                let n = self.current().lexeme.parse::<i64>().unwrap_or(0);
                self.advance();
                ExprKind::Literal(Literal::Int(n))
            }
            TokenKind::FloatLiteral => {
                let f = self.current().lexeme.parse::<f64>().unwrap_or(0.0);
                self.advance();
                ExprKind::Literal(Literal::Float(f))
            }
            TokenKind::StringLiteral => {
                let s = self.current().lexeme.clone();
                self.advance();
                ExprKind::Literal(Literal::String(s))
            }
            TokenKind::CharLiteral => {
                let c = self.current().lexeme.chars().next().unwrap_or('?');
                self.advance();
                ExprKind::Literal(Literal::Char(c))
            }
            TokenKind::True => {
                self.advance();
                ExprKind::Literal(Literal::Bool(true))
            }
            TokenKind::False => {
                self.advance();
                ExprKind::Literal(Literal::Bool(false))
            }
            
            // Identifiers and paths
            TokenKind::Ident => {
                let name = self.current().lexeme.clone();
                self.advance();
                
                // Check if it's a path or struct literal
                if self.check(&TokenKind::ColonColon) {
                    // Path
                    let path = self.parse_path_from_segment(name)?;
                    
                    // Check for struct literal
                    if self.check(&TokenKind::LBrace) {
                        return self.parse_struct_literal(path, start);
                    }
                    
                    ExprKind::Path(path)
                } else if self.check(&TokenKind::LBrace) {
                    // Struct literal with simple name
                    let path = Path {
                        segments: vec![name],
                        generics: vec![],
                    };
                    return self.parse_struct_literal(path, start);
                } else {
                    // Simple identifier
                    ExprKind::Ident(name)
                }
            }
            
            // Unary operators
            TokenKind::Minus => {
                self.advance();
                let operand = self.parse_expr_with_precedence(Precedence::Unary)?;
                ExprKind::Unary { op: UnaryOp::Neg, operand }
            }
            TokenKind::Not => {
                self.advance();
                let operand = self.parse_expr_with_precedence(Precedence::Unary)?;
                ExprKind::Unary { op: UnaryOp::Not, operand }
            }
            TokenKind::Tilde => {
                self.advance();
                let operand = self.parse_expr_with_precedence(Precedence::Unary)?;
                ExprKind::Unary { op: UnaryOp::BitNot, operand }
            }
            TokenKind::Star => {
                self.advance();
                let operand = self.parse_expr_with_precedence(Precedence::Unary)?;
                ExprKind::Unary { op: UnaryOp::Deref, operand }
            }
            TokenKind::And => {
                self.advance();
                let op = if self.check(&TokenKind::Mut) {
                    self.advance();
                    UnaryOp::RefMut
                } else {
                    UnaryOp::Ref
                };
                let operand = self.parse_expr_with_precedence(Precedence::Unary)?;
                ExprKind::Unary { op, operand }
            }
            
            // Parenthesized expressions and tuples
            TokenKind::LParen => {
                self.advance();
                
                if self.check(&TokenKind::RParen) {
                    // Unit literal ()
                    self.advance();
                    return Ok(self.alloc_expr(ExprKind::Tuple(vec![]), start));
                }
                
                let first_expr = self.parse_expr()?;
                
                if self.check(&TokenKind::Comma) {
                    // Tuple
                    let mut exprs = vec![first_expr];
                    
                    while self.check(&TokenKind::Comma) {
                        self.advance();
                        if self.check(&TokenKind::RParen) {
                            break;
                        }
                        exprs.push(self.parse_expr()?);
                    }
                    
                    self.expect(TokenKind::RParen, "Expected ')' after tuple")?;
                    ExprKind::Tuple(exprs)
                } else {
                    // Parenthesized expression
                    self.expect(TokenKind::RParen, "Expected ')'")?;
                    return Ok(first_expr);
                }
            }
            
            // Array literals
            TokenKind::LBracket => {
                self.advance();
                let mut exprs = Vec::new();
                
                if !self.check(&TokenKind::RBracket) {
                    loop {
                        exprs.push(self.parse_expr()?);
                        
                        if !self.check(&TokenKind::Comma) {
                            break;
                        }
                        self.advance();
                        
                        if self.check(&TokenKind::RBracket) {
                            break;
                        }
                    }
                }
                
                self.expect(TokenKind::RBracket, "Expected ']' after array")?;
                ExprKind::Array(exprs)
            }
            
            // Block expressions
            TokenKind::LBrace => {
                let block = self.parse_block()?;
                let block_id = self.arena.alloc(aurora_ast::nodes::AstNode::Block(block));
                ExprKind::Block(block_id)
            }
            
            // If expressions
            TokenKind::If => {
                return self.parse_if_expr(start);
            }
            
            // Match expressions
            TokenKind::Match => {
                return self.parse_match_expr(start);
            }
            
            // Loop expressions
            TokenKind::Loop => {
                self.advance();
                let body_block = self.parse_block()?;
                let body = self.arena.alloc(aurora_ast::nodes::AstNode::Block(body_block));
                ExprKind::Loop { body }
            }
            
            // While loops
            TokenKind::While => {
                self.advance();
                let condition = self.parse_expr()?;
                let body_block = self.parse_block()?;
                let body = self.arena.alloc(aurora_ast::nodes::AstNode::Block(body_block));
                ExprKind::While { condition, body }
            }
            
            // For loops
            TokenKind::For => {
                self.advance();
                let pattern = self.parse_pattern()?;
                self.expect(TokenKind::In, "Expected 'in' after for loop pattern")?;
                let iterator = self.parse_expr()?;
                let body_block = self.parse_block()?;
                let body = self.arena.alloc(aurora_ast::nodes::AstNode::Block(body_block));
                ExprKind::For { pattern, iterator, body }
            }
            
            // Return
            TokenKind::Return => {
                self.advance();
                let value = if self.check(&TokenKind::Semicolon) || self.check(&TokenKind::RBrace) {
                    None
                } else {
                    Some(self.parse_expr()?)
                };
                ExprKind::Return { value }
            }
            
            // Break
            TokenKind::Break => {
                self.advance();
                let value = if self.check(&TokenKind::Semicolon) || self.check(&TokenKind::RBrace) {
                    None
                } else {
                    Some(self.parse_expr()?)
                };
                ExprKind::Break { value }
            }
            
            // Continue
            TokenKind::Continue => {
                self.advance();
                ExprKind::Continue
            }
            
            // Unsafe blocks
            TokenKind::Unsafe => {
                self.advance();
                let block_node = self.parse_block()?;
                let block = self.arena.alloc(aurora_ast::nodes::AstNode::Block(block_node));
                ExprKind::Unsafe { block }
            }
            
            _ => {
                return Err(ParseError::Expected {
                    expected: "expression".to_string(),
                    found: format!("{:?}", self.peek()),
                    span: self.token_to_span(self.current()),
                    message: "Expected an expression".to_string(),
                });
            }
        };
        
        Ok(self.alloc_expr(kind, start))
    }
    
    /// Parse infix/postfix expression
    fn parse_infix_expr(&mut self, left: u32, prec: Precedence, start: Span) -> ParseResult<u32> {
        match self.peek() {
            // Binary operators
            TokenKind::Plus | TokenKind::Minus | TokenKind::Star | TokenKind::Slash
            | TokenKind::Percent | TokenKind::StarStar
            | TokenKind::EqEq | TokenKind::NotEq
            | TokenKind::Lt | TokenKind::LtEq | TokenKind::Gt | TokenKind::GtEq
            | TokenKind::AndAnd | TokenKind::OrOr
            | TokenKind::And | TokenKind::Or | TokenKind::Caret
            | TokenKind::LtLt | TokenKind::GtGt => {
                let op = self.token_to_binary_op()?;
                self.advance();
                let right = self.parse_expr_with_precedence(prec)?;
                let kind = ExprKind::Binary { op, left, right };
                Ok(self.alloc_expr(kind, start))
            }
            
            // Assignment operators
            TokenKind::Eq | TokenKind::PlusEq | TokenKind::MinusEq
            | TokenKind::StarEq | TokenKind::SlashEq | TokenKind::PercentEq
            | TokenKind::AndEq | TokenKind::OrEq | TokenKind::CaretEq
            | TokenKind::LtLtEq | TokenKind::GtGtEq => {
                let op = self.token_to_binary_op()?;
                self.advance();
                let right = self.parse_expr_with_precedence(Precedence::Assignment)?;
                let kind = ExprKind::Binary { op, left, right };
                Ok(self.alloc_expr(kind, start))
            }
            
            // Range operators
            TokenKind::DotDot => {
                self.advance();
                let inclusive = if self.check(&TokenKind::Eq) {
                    self.advance();
                    true
                } else {
                    false
                };
                
                let end = if self.check(&TokenKind::Comma) || self.check(&TokenKind::RBracket)
                    || self.check(&TokenKind::RParen) || self.check(&TokenKind::Semicolon) {
                    None
                } else {
                    Some(self.parse_expr_with_precedence(Precedence::Range)?)
                };
                
                let kind = ExprKind::Range {
                    start: Some(left),
                    end,
                    inclusive,
                };
                Ok(self.alloc_expr(kind, start))
            }
            
            // Field access
            TokenKind::Dot => {
                self.advance();
                
                // Check for method call vs field access
                let field_token = self.expect(TokenKind::Ident, "Expected field or method name")?;
                let name = field_token.lexeme.clone();
                
                if self.check(&TokenKind::LParen) {
                    // Method call
                    self.advance();
                    let args = self.parse_call_args()?;
                    self.expect(TokenKind::RParen, "Expected ')' after method arguments")?;
                    let kind = ExprKind::MethodCall {
                        receiver: left,
                        method: name,
                        args,
                    };
                    Ok(self.alloc_expr(kind, start))
                } else {
                    // Field access
                    let kind = ExprKind::Field { object: left, field: name };
                    Ok(self.alloc_expr(kind, start))
                }
            }
            
            // Function call
            TokenKind::LParen => {
                self.advance();
                let args = self.parse_call_args()?;
                self.expect(TokenKind::RParen, "Expected ')' after arguments")?;
                let kind = ExprKind::Call { func: left, args };
                Ok(self.alloc_expr(kind, start))
            }
            
            // Index access
            TokenKind::LBracket => {
                self.advance();
                let index = self.parse_expr()?;
                self.expect(TokenKind::RBracket, "Expected ']' after index")?;
                let kind = ExprKind::Index { collection: left, index };
                Ok(self.alloc_expr(kind, start))
            }
            
            // Try operator (?)
            TokenKind::Question => {
                self.advance();
                let kind = ExprKind::Try { expr: left };
                Ok(self.alloc_expr(kind, start))
            }
            
            _ => Ok(left),
        }
    }
    
    /// Get precedence for infix operator at current position
    fn get_infix_precedence(&self) -> Precedence {
        match self.peek() {
            TokenKind::Eq | TokenKind::PlusEq | TokenKind::MinusEq
            | TokenKind::StarEq | TokenKind::SlashEq | TokenKind::PercentEq
            | TokenKind::AndEq | TokenKind::OrEq | TokenKind::CaretEq
            | TokenKind::LtLtEq | TokenKind::GtGtEq => Precedence::Assignment,
            
            TokenKind::Question => Precedence::Propagation,
            TokenKind::DotDot => Precedence::Range,
            TokenKind::OrOr => Precedence::LogicalOr,
            TokenKind::AndAnd => Precedence::LogicalAnd,
            
            TokenKind::EqEq | TokenKind::NotEq
            | TokenKind::Lt | TokenKind::LtEq | TokenKind::Gt | TokenKind::GtEq => Precedence::Comparison,
            
            TokenKind::Or => Precedence::BitwiseOr,
            TokenKind::Caret => Precedence::BitwiseXor,
            TokenKind::And => Precedence::BitwiseAnd,
            TokenKind::LtLt | TokenKind::GtGt => Precedence::Shift,
            TokenKind::Plus | TokenKind::Minus => Precedence::Additive,
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Precedence::Multiplicative,
            TokenKind::StarStar => Precedence::Exponentiation,
            
            TokenKind::Dot | TokenKind::ColonColon | TokenKind::LParen | TokenKind::LBracket => Precedence::Call,
            
            _ => Precedence::None,
        }
    }
    
    /// Convert token to binary operator
    fn token_to_binary_op(&self) -> ParseResult<BinaryOp> {
        match self.peek() {
            TokenKind::Plus => Ok(BinaryOp::Add),
            TokenKind::Minus => Ok(BinaryOp::Sub),
            TokenKind::Star => Ok(BinaryOp::Mul),
            TokenKind::Slash => Ok(BinaryOp::Div),
            TokenKind::Percent => Ok(BinaryOp::Rem),
            TokenKind::StarStar => Ok(BinaryOp::Pow),
            TokenKind::EqEq => Ok(BinaryOp::Eq),
            TokenKind::NotEq => Ok(BinaryOp::Ne),
            TokenKind::Lt => Ok(BinaryOp::Lt),
            TokenKind::LtEq => Ok(BinaryOp::Le),
            TokenKind::Gt => Ok(BinaryOp::Gt),
            TokenKind::GtEq => Ok(BinaryOp::Ge),
            TokenKind::AndAnd => Ok(BinaryOp::And),
            TokenKind::OrOr => Ok(BinaryOp::Or),
            TokenKind::And => Ok(BinaryOp::BitAnd),
            TokenKind::Or => Ok(BinaryOp::BitOr),
            TokenKind::Caret => Ok(BinaryOp::BitXor),
            TokenKind::LtLt => Ok(BinaryOp::Shl),
            TokenKind::GtGt => Ok(BinaryOp::Shr),
            TokenKind::Eq => Ok(BinaryOp::Assign),
            TokenKind::PlusEq => Ok(BinaryOp::AddAssign),
            TokenKind::MinusEq => Ok(BinaryOp::SubAssign),
            TokenKind::StarEq => Ok(BinaryOp::MulAssign),
            TokenKind::SlashEq => Ok(BinaryOp::DivAssign),
            TokenKind::PercentEq => Ok(BinaryOp::RemAssign),
            TokenKind::AndEq => Ok(BinaryOp::BitAndAssign),
            TokenKind::OrEq => Ok(BinaryOp::BitOrAssign),
            TokenKind::CaretEq => Ok(BinaryOp::BitXorAssign),
            TokenKind::LtLtEq => Ok(BinaryOp::ShlAssign),
            TokenKind::GtGtEq => Ok(BinaryOp::ShrAssign),
            _ => Err(ParseError::Expected {
                expected: "binary operator".to_string(),
                found: format!("{:?}", self.peek()),
                span: self.token_to_span(self.current()),
                message: "Expected a binary operator".to_string(),
            }),
        }
    }
    
    /// Parse if expression
    fn parse_if_expr(&mut self, start: Span) -> ParseResult<u32> {
        self.expect(TokenKind::If, "Expected 'if'")?;
        let condition = self.parse_expr()?;
        let then_block_node = self.parse_block()?;
        let then_block = self.arena.alloc(aurora_ast::nodes::AstNode::Block(then_block_node));
        
        let else_block = if self.check(&TokenKind::Else) {
            self.advance();
            
            if self.check(&TokenKind::If) {
                // else if
                let else_if_expr_id = self.parse_if_expr(self.token_to_span(self.current()))?;
                // Wrap in a block
                let else_if_block = aurora_ast::Block {
                    stmts: vec![],
                    expr: Some(else_if_expr_id),
                    span: self.token_to_span(self.previous()),
                };
                Some(self.arena.alloc(aurora_ast::nodes::AstNode::Block(else_if_block)))
            } else {
                // else block
                let else_block_node = self.parse_block()?;
                Some(self.arena.alloc(aurora_ast::nodes::AstNode::Block(else_block_node)))
            }
        } else {
            None
        };
        
        let kind = ExprKind::If { condition, then_block, else_block };
        Ok(self.alloc_expr(kind, start))
    }
    
    /// Parse match expression
    fn parse_match_expr(&mut self, start: Span) -> ParseResult<u32> {
        self.expect(TokenKind::Match, "Expected 'match'")?;
        let scrutinee = self.parse_expr()?;
        
        self.expect(TokenKind::LBrace, "Expected '{' after match scrutinee")?;
        
        let mut arms = Vec::new();
        
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let arm_start = self.token_to_span(self.current());
            let pattern = self.parse_pattern()?;
            
            let guard = if self.check(&TokenKind::If) {
                self.advance();
                Some(self.parse_expr()?)
            } else {
                None
            };
            
            self.expect(TokenKind::FatArrow, "Expected '=>' after match pattern")?;
            let body = self.parse_expr()?;
            
            // Optional comma
            if self.check(&TokenKind::Comma) {
                self.advance();
            }
            
            let arm_span = self.span_from(arm_start);
            arms.push(MatchArm { pattern, guard, body, span: arm_span });
        }
        
        self.expect(TokenKind::RBrace, "Expected '}' after match arms")?;
        
        let kind = ExprKind::Match { scrutinee, arms };
        Ok(self.alloc_expr(kind, start))
    }
    
    /// Parse struct literal
    fn parse_struct_literal(&mut self, path: Path, start: Span) -> ParseResult<u32> {
        self.expect(TokenKind::LBrace, "Expected '{'")?;

        let mut fields = Vec::new();

        if !self.check(&TokenKind::RBrace) {
            loop {
                let field_start = self.token_to_span(self.current());
                let field_name_token = self.expect(TokenKind::Ident, "Expected field name")?;
                let field_name = field_name_token.lexeme.clone();

                let value = if self.check(&TokenKind::Colon) {
                    // Explicit field value: `x: expr`
                    self.advance();
                    self.parse_expr()?
                } else {
                    // Shorthand: `x` means `x: x`
                    let ident_expr = ExprKind::Ident(field_name.clone());
                    self.alloc_expr(ident_expr, field_start)
                };

                let field_span = self.span_from(field_start);
                fields.push(FieldInit { name: field_name, value, span: field_span });

                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();

                if self.check(&TokenKind::RBrace) {
                    break;
                }
            }
        }

        self.expect(TokenKind::RBrace, "Expected '}' after struct fields")?;

        let kind = ExprKind::Struct { path, fields };
        Ok(self.alloc_expr(kind, start))
    }
    
    /// Parse function call arguments
    fn parse_call_args(&mut self) -> ParseResult<Vec<u32>> {
        let mut args = Vec::new();
        
        if self.check(&TokenKind::RParen) {
            return Ok(args);
        }
        
        loop {
            args.push(self.parse_expr()?);
            
            if !self.check(&TokenKind::Comma) {
                break;
            }
            self.advance();
            
            if self.check(&TokenKind::RParen) {
                break;
            }
        }
        
        Ok(args)
    }
    
    /// Parse path from first segment
    pub(crate) fn parse_path_from_segment(&mut self, first: String) -> ParseResult<Path> {
        let mut segments = vec![first];
        
        while self.check(&TokenKind::ColonColon) {
            self.advance();
            let segment_token = self.expect(TokenKind::Ident, "Expected path segment")?;
            segments.push(segment_token.lexeme.clone());
        }
        
        // TODO: Parse generic arguments
        let generics = vec![];
        
        Ok(Path { segments, generics })
    }
    
    /// Parse a simple path
    pub(crate) fn parse_path(&mut self) -> ParseResult<Path> {
        let first_token = self.expect(TokenKind::Ident, "Expected path")?;
        let first = first_token.lexeme.clone();
        self.parse_path_from_segment(first)
    }
    
    /// Helper to allocate an expression
    fn alloc_expr(&mut self, kind: ExprKind, start: Span) -> u32 {
        let span = self.span_from(start);
        let expr = Expr {
            kind,
            span,
            hygiene: Default::default(),
        };
        self.arena.alloc_expr(expr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    
    #[test]
    fn test_parse_literal_expr() {
        let source = "fn test() { 42; }";
        let parser = Parser::new(source, "test.ax".to_string()).unwrap();
        let (_program, _arena) = parser.parse_program().unwrap();
    }

    #[test]
    fn test_parse_binary_expr() {
        let source = "fn test() { 1 + 2 * 3; }";
        let parser = Parser::new(source, "test.ax".to_string()).unwrap();
        let (_program, _arena) = parser.parse_program().unwrap();
    }

    #[test]
    fn test_parse_if_expr() {
        let source = "fn test() { if true { 1 } else { 2 } }";
        let parser = Parser::new(source, "test.ax".to_string()).unwrap();
        let (_program, _arena) = parser.parse_program().unwrap();
    }

    #[test]
    fn test_parse_function_call() {
        let source = "fn test() { foo(1, 2, 3); }";
        let parser = Parser::new(source, "test.ax".to_string()).unwrap();
        let (_program, _arena) = parser.parse_program().unwrap();
    }
}
