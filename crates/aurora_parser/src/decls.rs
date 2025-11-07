//! Declaration parsing (functions, types, traits, impls, etc.)

use aurora_ast::decl::{
    ConstDecl, FunctionDecl, ImplDecl, Item, ItemKind, ModuleDecl, Param, TraitDecl, TypeDecl,
    UseDecl, UseTree, GenericParam, WhereClause,
};
use aurora_ast::{Block, Stmt, StmtKind};
use aurora_lexer::TokenKind;
use crate::error::{ParseError, ParseResult};
use crate::parser::Parser;

impl Parser {
    /// Parse a top-level item
    pub(crate) fn parse_item(&mut self) -> ParseResult<u32> {
        let start = self.current().span;

        // Check for visibility modifier
        let is_pub = if self.check(&TokenKind::Pub) {
            self.advance();
            true
        } else {
            false
        };

        // Parse based on keyword
        let kind = match self.peek() {
            TokenKind::Fn => self.parse_function(is_pub)?,
            TokenKind::Type => self.parse_type_decl(is_pub)?,
            TokenKind::Trait => self.parse_trait(is_pub)?,
            TokenKind::Impl => self.parse_impl()?,
            TokenKind::Const => self.parse_const(is_pub)?,
            TokenKind::Mod => self.parse_module(is_pub)?,
            TokenKind::Use => self.parse_use(is_pub)?,
            _ => {
                return Err(ParseError::Expected {
                    expected: "item declaration (fn, type, trait, impl, const, mod, use)".to_string(),
                    found: format!("{:?}", self.peek()),
                    span: self.current().span,
                    message: "Expected a top-level item".to_string(),
                });
            }
        };

        let span = self.span_from(start);
        let item = Item { kind, span };
        Ok(self.arena.alloc_item(item))
    }

    /// Parse function declaration
    fn parse_function(&mut self, is_pub: bool) -> ParseResult<ItemKind> {
        let start = self.current().span;

        // Check for async
        let is_async = if self.check(&TokenKind::Async) {
            self.advance();
            true
        } else {
            false
        };

        // 'fn' keyword
        self.expect(TokenKind::Fn, "Expected 'fn'")?;

        // Function name
        let name_token = self.expect(TokenKind::Identifier, "Expected function name")?;
        let name = name_token.lexeme.clone();

        // Generic parameters (optional)
        let generics = if self.check(&TokenKind::Lt) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };

        // Parameters
        self.expect(TokenKind::LParen, "Expected '(' after function name")?;
        let params = self.parse_param_list()?;
        self.expect(TokenKind::RParen, "Expected ')' after parameters")?;

        // Return type (optional)
        let return_type = if self.check(&TokenKind::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        // Where clause (optional)
        let where_clause = if self.check(&TokenKind::Where) {
            Some(self.parse_where_clause()?)
        } else {
            None
        };

        // Function body
        let body = self.parse_block()?;

        let span = self.span_from(start);

        Ok(ItemKind::Function(FunctionDecl {
            name,
            generics,
            params,
            return_type,
            where_clause,
            body,
            is_pub,
            is_async,
            is_unsafe: false, // TODO: handle unsafe
            span,
        }))
    }

    /// Parse function parameters
    fn parse_param_list(&mut self) -> ParseResult<Vec<Param>> {
        let mut params = Vec::new();

        if self.check(&TokenKind::RParen) {
            return Ok(params);
        }

        loop {
            let start = self.current().span;

            // Check for 'mut'
            let is_mut = if self.check(&TokenKind::Mut) {
                self.advance();
                true
            } else {
                false
            };

            // Parameter pattern (for now, just identifier)
            let pattern = self.parse_pattern()?;

            // ':'
            self.expect(TokenKind::Colon, "Expected ':' after parameter name")?;

            // Parameter type
            let ty = self.parse_type()?;

            let span = self.span_from(start);
            params.push(Param {
                pattern,
                ty,
                is_mut,
                span,
            });

            // Check for comma or end
            if !self.check(&TokenKind::Comma) {
                break;
            }
            self.advance(); // consume comma

            // Allow trailing comma
            if self.check(&TokenKind::RParen) {
                break;
            }
        }

        Ok(params)
    }

    /// Parse generic parameters
    fn parse_generic_params(&mut self) -> ParseResult<Vec<GenericParam>> {
        self.expect(TokenKind::Lt, "Expected '<'")?;

        let mut params = Vec::new();

        if !self.check(&TokenKind::Gt) {
            loop {
                let start = self.current().span;
                let name_token = self.expect(TokenKind::Identifier, "Expected generic parameter name")?;
                let name = name_token.lexeme.clone();

                // TODO: Parse bounds
                let bounds = Vec::new();

                let span = self.span_from(start);
                params.push(GenericParam { name, bounds, span });

                if !self.check(&TokenKind::Comma) {
                    break;
                }
                self.advance();
            }
        }

        self.expect(TokenKind::Gt, "Expected '>'")?;
        Ok(params)
    }

    /// Parse where clause
    fn parse_where_clause(&mut self) -> ParseResult<WhereClause> {
        let start = self.current().span;
        self.expect(TokenKind::Where, "Expected 'where'")?;

        // TODO: Parse predicates
        let predicates = Vec::new();

        let span = self.span_from(start);
        Ok(WhereClause { predicates, span })
    }

    /// Parse type declaration
    fn parse_type_decl(&mut self, is_pub: bool) -> ParseResult<ItemKind> {
        let start = self.current().span;

        self.expect(TokenKind::Type, "Expected 'type'")?;

        let name_token = self.expect(TokenKind::Identifier, "Expected type name")?;
        let name = name_token.lexeme.clone();

        // Generic parameters (optional)
        let generics = if self.check(&TokenKind::Lt) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };

        self.expect(TokenKind::Eq, "Expected '=' in type declaration")?;

        let ty = self.parse_type()?;

        self.expect(TokenKind::Semicolon, "Expected ';' after type declaration")?;

        let span = self.span_from(start);

        Ok(ItemKind::Type(TypeDecl {
            name,
            generics,
            ty,
            is_pub,
            span,
        }))
    }

    /// Parse trait declaration
    fn parse_trait(&mut self, is_pub: bool) -> ParseResult<ItemKind> {
        let start = self.current().span;

        self.expect(TokenKind::Trait, "Expected 'trait'")?;

        let name_token = self.expect(TokenKind::Identifier, "Expected trait name")?;
        let name = name_token.lexeme.clone();

        // Generic parameters (optional)
        let generics = if self.check(&TokenKind::Lt) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };

        // Where clause (optional)
        let where_clause = if self.check(&TokenKind::Where) {
            Some(self.parse_where_clause()?)
        } else {
            None
        };

        self.expect(TokenKind::LBrace, "Expected '{' after trait name")?;

        // Parse trait items
        let items = Vec::new(); // TODO: parse trait items

        self.expect(TokenKind::RBrace, "Expected '}' to close trait")?;

        let span = self.span_from(start);

        Ok(ItemKind::Trait(TraitDecl {
            name,
            generics,
            where_clause,
            items,
            is_pub,
            span,
        }))
    }

    /// Parse impl declaration
    fn parse_impl(&mut self) -> ParseResult<ItemKind> {
        let start = self.current().span;

        self.expect(TokenKind::Impl, "Expected 'impl'")?;

        // Generic parameters (optional)
        let generics = if self.check(&TokenKind::Lt) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };

        // Type being implemented
        let self_ty = self.parse_type()?;

        // Optional 'for' (trait impl)
        let trait_ref = if self.check(&TokenKind::For) {
            self.advance();
            // This is actually trait impl, swap the types
            // The first type we parsed is the trait
            None // TODO: handle trait impls properly
        } else {
            None
        };

        // Where clause (optional)
        let where_clause = if self.check(&TokenKind::Where) {
            Some(self.parse_where_clause()?)
        } else {
            None
        };

        self.expect(TokenKind::LBrace, "Expected '{' after impl signature")?;

        // Parse impl items
        let items = Vec::new(); // TODO: parse impl items

        self.expect(TokenKind::RBrace, "Expected '}' to close impl")?;

        let span = self.span_from(start);

        Ok(ItemKind::Impl(ImplDecl {
            generics,
            self_ty,
            trait_ref,
            where_clause,
            items,
            span,
        }))
    }

    /// Parse constant declaration
    fn parse_const(&mut self, is_pub: bool) -> ParseResult<ItemKind> {
        let start = self.current().span;

        self.expect(TokenKind::Const, "Expected 'const'")?;

        let name_token = self.expect(TokenKind::Identifier, "Expected constant name")?;
        let name = name_token.lexeme.clone();

        self.expect(TokenKind::Colon, "Expected ':' after constant name")?;

        let ty = self.parse_type()?;

        self.expect(TokenKind::Eq, "Expected '=' in const declaration")?;

        let value = self.parse_expr()?;

        self.expect(TokenKind::Semicolon, "Expected ';' after const value")?;

        let span = self.span_from(start);

        Ok(ItemKind::Const(ConstDecl {
            name,
            ty,
            value,
            is_pub,
            span,
        }))
    }

    /// Parse module declaration
    fn parse_module(&mut self, is_pub: bool) -> ParseResult<ItemKind> {
        let start = self.current().span;

        self.expect(TokenKind::Mod, "Expected 'mod'")?;

        let name_token = self.expect(TokenKind::Identifier, "Expected module name")?;
        let name = name_token.lexeme.clone();

        // Check if it's inline module or external
        let items = if self.check(&TokenKind::LBrace) {
            self.advance();
            let mut module_items = Vec::new();

            while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
                module_items.push(self.parse_item()?);
            }

            self.expect(TokenKind::RBrace, "Expected '}' to close module")?;
            Some(module_items)
        } else {
            self.expect(TokenKind::Semicolon, "Expected ';' after module name")?;
            None
        };

        let span = self.span_from(start);

        Ok(ItemKind::Module(ModuleDecl {
            name,
            items,
            is_pub,
            span,
        }))
    }

    /// Parse use declaration
    fn parse_use(&mut self, is_pub: bool) -> ParseResult<ItemKind> {
        let start = self.current().span;

        self.expect(TokenKind::Use, "Expected 'use'")?;

        let tree = self.parse_use_tree()?;

        self.expect(TokenKind::Semicolon, "Expected ';' after use statement")?;

        let span = self.span_from(start);

        Ok(ItemKind::Use(UseDecl { tree, is_pub, span }))
    }

    /// Parse use tree (import path)
    fn parse_use_tree(&mut self) -> ParseResult<UseTree> {
        // For now, just parse simple path
        let path = self.parse_path()?;

        Ok(UseTree::Path { path, alias: None })
    }

    /// Parse a block
    fn parse_block(&mut self) -> ParseResult<Block> {
        let start = self.current().span;

        self.expect(TokenKind::LBrace, "Expected '{'")?;

        let mut stmts = Vec::new();
        let mut trailing_expr = None;

        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            // Try to parse a statement
            if let Ok(stmt_id) = self.parse_stmt() {
                stmts.push(stmt_id);
            } else {
                // If statement fails, try expression without semicolon (trailing)
                if let Ok(expr_id) = self.parse_expr() {
                    // If there's no semicolon, this is a trailing expression
                    if !self.check(&TokenKind::Semicolon) {
                        trailing_expr = Some(expr_id);
                        break;
                    } else {
                        // With semicolon, it's an expression statement
                        self.advance();
                        let stmt = Stmt {
                            kind: StmtKind::Expr {
                                expr: expr_id,
                                has_semi: true,
                            },
                            span: self.previous().span,
                        };
                        stmts.push(self.arena.alloc_stmt(stmt));
                    }
                } else {
                    self.synchronize();
                }
            }
        }

        self.expect(TokenKind::RBrace, "Expected '}'")?;

        let span = self.span_from(start);

        Ok(Block {
            stmts,
            expr: trailing_expr,
            span,
        })
    }

    // Placeholder methods to be implemented in other modules
    fn parse_type(&mut self) -> ParseResult<u32> {
        // TODO: Implement in types.rs
        Ok(0)
    }

    fn parse_pattern(&mut self) -> ParseResult<u32> {
        // TODO: Implement in patterns.rs
        Ok(0)
    }

    fn parse_path(&mut self) -> ParseResult<aurora_ast::expr::Path> {
        // TODO: Implement path parsing
        Ok(aurora_ast::expr::Path {
            segments: vec!["placeholder".to_string()],
            generics: vec![],
        })
    }

    fn parse_expr(&mut self) -> ParseResult<u32> {
        // TODO: Implement in exprs.rs (Pratt parser)
        Ok(0)
    }

    fn parse_stmt(&mut self) -> ParseResult<u32> {
        // TODO: Implement in stmts.rs
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    #[test]
    fn test_parse_empty_function() {
        let source = "fn test() {}";
        let parser = Parser::new(source, "test.ax".to_string()).unwrap();
        let (program, _arena) = parser.parse().unwrap();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_pub_function() {
        let source = "pub fn test() {}";
        let parser = Parser::new(source, "test.ax".to_string()).unwrap();
        let (program, _arena) = parser.parse().unwrap();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_type_alias() {
        let source = "type MyInt = i64;";
        let parser = Parser::new(source, "test.ax".to_string()).unwrap();
        let (_program, _arena) = parser.parse().unwrap();
    }

    #[test]
    fn test_parse_module() {
        let source = "mod test;";
        let parser = Parser::new(source, "test.ax".to_string()).unwrap();
        let (program, _arena) = parser.parse().unwrap();
        assert_eq!(program.items.len(), 1);
    }
}
