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
        let start = self.token_to_span(self.current());

        // Check for visibility modifier
        let is_pub = if self.check(&TokenKind::Pub) {
            self.advance();
            true
        } else {
            false
        };

        // Parse based on keyword
        let kind = match self.peek() {
            TokenKind::Fn | TokenKind::Fun => self.parse_function(is_pub)?,
            TokenKind::Type => self.parse_type_decl(is_pub)?,
            TokenKind::Trait => self.parse_trait(is_pub)?,
            TokenKind::Impl => self.parse_impl()?,
            TokenKind::Const => self.parse_const(is_pub)?,
            TokenKind::Mod => self.parse_module(is_pub)?,
            TokenKind::Use => self.parse_use(is_pub)?,
            // Note: struct and enum are parsed via 'type' for now
            // They'll be added when we implement full struct/enum syntax
            _ => {
                return Err(ParseError::Expected {
                    expected: "item declaration (fn/fun, type, trait, impl, const, mod, use)".to_string(),
                    found: format!("{:?}", self.peek()),
                    span: self.token_to_span(self.current()),
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
        let start = self.token_to_span(self.current());

        // Check for async
        let is_async = if self.check(&TokenKind::Async) {
            self.advance();
            true
        } else {
            false
        };

        // 'fn' or 'fun' keyword
        if !self.check(&TokenKind::Fn) && !self.check(&TokenKind::Fun) {
            return Err(ParseError::Expected {
                expected: "'fn' or 'fun'".to_string(),
                found: format!("{:?}", self.peek()),
                span: self.token_to_span(self.current()),
                message: "Expected function keyword".to_string(),
            });
        }
        self.advance();

        // Function name
        let name_token = self.expect(TokenKind::Ident, "Expected function name")?;
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
        let return_type = if self.check(&TokenKind::RArrow) {
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
            let start = self.token_to_span(self.current());

            // Check for 'mut' or 'var' (simplified syntax)
            let is_mut = if self.check(&TokenKind::Mut) || self.check(&TokenKind::Var) {
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
                let start = self.token_to_span(self.current());
                let name_token = self.expect(TokenKind::Ident, "Expected generic parameter name")?;
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
        let start = self.token_to_span(self.current());
        self.expect(TokenKind::Where, "Expected 'where'")?;

        // TODO: Parse predicates
        let predicates = Vec::new();

        let span = self.span_from(start);
        Ok(WhereClause { predicates, span })
    }

    /// Parse type declaration
    fn parse_type_decl(&mut self, is_pub: bool) -> ParseResult<ItemKind> {
        let start = self.token_to_span(self.current());

        self.expect(TokenKind::Type, "Expected 'type'")?;

        let name_token = self.expect(TokenKind::Ident, "Expected type name")?;
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
        let start = self.token_to_span(self.current());

        self.expect(TokenKind::Trait, "Expected 'trait'")?;

        let name_token = self.expect(TokenKind::Ident, "Expected trait name")?;
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
        let items = self.parse_trait_items()?;

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
        let start = self.token_to_span(self.current());

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
        let items = self.parse_impl_items()?;

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
        let start = self.token_to_span(self.current());

        self.expect(TokenKind::Const, "Expected 'const'")?;

        let name_token = self.expect(TokenKind::Ident, "Expected constant name")?;
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
        let start = self.token_to_span(self.current());

        self.expect(TokenKind::Mod, "Expected 'mod'")?;

        let name_token = self.expect(TokenKind::Ident, "Expected module name")?;
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
        let start = self.token_to_span(self.current());

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

    /// Parse trait items
    fn parse_trait_items(&mut self) -> ParseResult<Vec<aurora_ast::decl::TraitItem>> {
        let mut items = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            // For now, just skip trait items
            // Full implementation would parse fn signatures, associated types, etc.
            if self.check(&TokenKind::Fn) {
                // Skip function signature for now
                self.advance();
                while !self.check(&TokenKind::Semicolon) && !self.check(&TokenKind::LBrace) && !self.is_at_end() {
                    self.advance();
                }
                if self.check(&TokenKind::Semicolon) {
                    self.advance();
                } else if self.check(&TokenKind::LBrace) {
                    // Skip body
                    let mut depth = 1;
                    self.advance();
                    while depth > 0 && !self.is_at_end() {
                        if self.check(&TokenKind::LBrace) {
                            depth += 1;
                        } else if self.check(&TokenKind::RBrace) {
                            depth -= 1;
                        }
                        self.advance();
                    }
                }
            } else {
                self.advance();
            }
        }

        Ok(items)
    }

    /// Parse impl items
    fn parse_impl_items(&mut self) -> ParseResult<Vec<aurora_ast::decl::ImplItem>> {
        use aurora_ast::decl::ImplItem;
        let mut items = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            let start = self.token_to_span(self.current());

            // Check for pub
            let is_pub = if self.check(&TokenKind::Pub) {
                self.advance();
                true
            } else {
                false
            };

            match self.peek() {
                TokenKind::Fn => {
                    let func = self.parse_function(is_pub)?;
                    if let ItemKind::Function(func_decl) = func {
                        items.push(ImplItem::Function(func_decl));
                    }
                }
                TokenKind::Const => {
                    let const_item = self.parse_const(is_pub)?;
                    if let ItemKind::Const(const_decl) = const_item {
                        items.push(ImplItem::Const(const_decl));
                    }
                }
                TokenKind::Type => {
                    let type_item = self.parse_type_decl(is_pub)?;
                    if let ItemKind::Type(type_decl) = type_item {
                        items.push(ImplItem::Type(type_decl));
                    }
                }
                _ => {
                    return Err(ParseError::Expected {
                        expected: "impl item (fn, const, type)".to_string(),
                        found: format!("{:?}", self.peek()),
                        span: self.token_to_span(self.current()),
                        message: "Expected an impl item".to_string(),
                    });
                }
            }
        }

        Ok(items)
    }

    /// Parse a block
    pub(crate) fn parse_block(&mut self) -> ParseResult<Block> {
        let start = self.token_to_span(self.current());

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
                            span: self.token_to_span(self.previous()),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    #[test]
    fn test_parse_empty_function() {
        let source = "fn test() {}";
        let parser = Parser::new(source, "test.ax".to_string()).unwrap();
        let (program, _arena) = parser.parse_program().unwrap();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_pub_function() {
        let source = "pub fn test() {}";
        let parser = Parser::new(source, "test.ax".to_string()).unwrap();
        let (program, _arena) = parser.parse_program().unwrap();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parse_type_alias() {
        let source = "type MyInt = i64;";
        let parser = Parser::new(source, "test.ax".to_string()).unwrap();
        let (_program, _arena) = parser.parse_program().unwrap();
    }

    #[test]
    fn test_parse_module() {
        let source = "mod test;";
        let parser = Parser::new(source, "test.ax".to_string()).unwrap();
        let (program, _arena) = parser.parse_program().unwrap();
        assert_eq!(program.items.len(), 1);
    }
}
