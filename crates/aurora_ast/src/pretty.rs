//! Pretty-printer for AST debugging
//!
//! This module provides human-readable AST output for debugging and
//! development purposes.

use crate::arena::Arena;
use crate::nodes::AstNode;
use crate::{Expr, Item, Pattern, Stmt, Type};
use std::fmt::Write;

/// Pretty-print configuration
#[derive(Debug, Clone)]
pub struct PrettyConfig {
    /// Indentation string (e.g., "  " for 2 spaces)
    pub indent: String,
    /// Show node IDs
    pub show_ids: bool,
    /// Show spans
    pub show_spans: bool,
    /// Maximum depth to print (None = unlimited)
    pub max_depth: Option<usize>,
}

impl Default for PrettyConfig {
    fn default() -> Self {
        Self {
            indent: "  ".to_string(),
            show_ids: true,
            show_spans: false,
            max_depth: None,
        }
    }
}

/// Pretty-printer for AST nodes
pub struct PrettyPrinter {
    config: PrettyConfig,
    output: String,
    depth: usize,
}

impl PrettyPrinter {
    /// Create a new pretty-printer with default config
    pub fn new() -> Self {
        Self::with_config(PrettyConfig::default())
    }

    /// Create a new pretty-printer with custom config
    pub fn with_config(config: PrettyConfig) -> Self {
        Self {
            config,
            output: String::new(),
            depth: 0,
        }
    }

    /// Pretty-print a node from the arena
    pub fn print_node(&mut self, arena: &Arena, node_id: u32) -> String {
        self.output.clear();
        self.depth = 0;
        self.print_node_internal(arena, node_id);
        self.output.clone()
    }

    /// Print the entire AST starting from root
    pub fn print_tree(&mut self, arena: &Arena) -> String {
        self.output.clear();
        self.depth = 0;

        if let Some(root) = arena.root() {
            self.print_node_internal(arena, root);
        } else {
            writeln!(&mut self.output, "(empty tree)").unwrap();
        }

        self.output.clone()
    }

    fn print_node_internal(&mut self, arena: &Arena, node_id: u32) {
        if let Some(max_depth) = self.config.max_depth {
            if self.depth >= max_depth {
                self.write_line("...");
                return;
            }
        }

        if let Some(node) = arena.get(node_id) {
            match node {
                AstNode::Expr(expr) => self.print_expr(arena, node_id, expr),
                AstNode::Stmt(stmt) => self.print_stmt(arena, node_id, stmt),
                AstNode::Block(block) => {
                    self.write_line("Block {");
                    self.depth += 1;
                    for &stmt_id in &block.stmts {
                        self.print_node_internal(arena, stmt_id);
                    }
                    if let Some(expr_id) = block.expr {
                        self.write_line("trailing_expr:");
                        self.depth += 1;
                        self.print_node_internal(arena, expr_id);
                        self.depth -= 1;
                    }
                    self.depth -= 1;
                    self.write_line("}");
                }
                AstNode::Item(item) => self.print_item(arena, node_id, item),
                AstNode::Type(ty) => self.print_type(arena, node_id, ty),
                AstNode::Pattern(pat) => self.print_pattern(arena, node_id, pat),
            }
        } else {
            self.write_line(&format!("(invalid node id: {})", node_id));
        }
    }

    fn print_expr(&mut self, arena: &Arena, node_id: u32, expr: &Expr) {
        use crate::expr::ExprKind::*;

        let id_str = if self.config.show_ids {
            format!(" [id={}]", node_id)
        } else {
            String::new()
        };

        match &expr.kind {
            Literal(lit) => {
                self.write_line(&format!("Literal({:?}){}", lit, id_str));
            }
            Ident(name) => {
                self.write_line(&format!("Ident({}){}", name, id_str));
            }
            Path(path) => {
                self.write_line(&format!("Path({}){}", path.segments.join("::"), id_str));
            }
            Unary { op, operand } => {
                self.write_line(&format!("Unary({:?}){}", op, id_str));
                self.depth += 1;
                self.print_node_internal(arena, *operand);
                self.depth -= 1;
            }
            Binary { op, left, right } => {
                self.write_line(&format!("Binary({:?}){}", op, id_str));
                self.depth += 1;
                self.write_line("left:");
                self.depth += 1;
                self.print_node_internal(arena, *left);
                self.depth -= 1;
                self.write_line("right:");
                self.depth += 1;
                self.print_node_internal(arena, *right);
                self.depth -= 1;
                self.depth -= 1;
            }
            Call { func, args, .. } => {
                self.write_line(&format!("Call{}", id_str));
                self.depth += 1;
                self.write_line("func:");
                self.depth += 1;
                self.print_node_internal(arena, *func);
                self.depth -= 1;
                if !args.is_empty() {
                    self.write_line("args:");
                    self.depth += 1;
                    for &arg in args {
                        self.print_node_internal(arena, arg);
                    }
                    self.depth -= 1;
                }
                self.depth -= 1;
            }
            Tuple(exprs) => {
                self.write_line(&format!("Tuple{}", id_str));
                self.depth += 1;
                for &expr_id in exprs {
                    self.print_node_internal(arena, expr_id);
                }
                self.depth -= 1;
            }
            Array(exprs) => {
                self.write_line(&format!("Array{}", id_str));
                self.depth += 1;
                for &expr_id in exprs {
                    self.print_node_internal(arena, expr_id);
                }
                self.depth -= 1;
            }
            If { condition, .. } => {
                self.write_line(&format!("If{}", id_str));
                self.depth += 1;
                self.write_line("condition:");
                self.depth += 1;
                self.print_node_internal(arena, *condition);
                self.depth -= 1;
                self.depth -= 1;
            }
            Return { value } => {
                self.write_line(&format!("Return{}", id_str));
                if let Some(v) = value {
                    self.depth += 1;
                    self.print_node_internal(arena, *v);
                    self.depth -= 1;
                }
            }
            _ => {
                self.write_line(&format!("Expr({:?}){}", expr.kind, id_str));
            }
        }
    }

    fn print_stmt(&mut self, arena: &Arena, node_id: u32, stmt: &Stmt) {
        use crate::stmt::StmtKind::*;

        let id_str = if self.config.show_ids {
            format!(" [id={}]", node_id)
        } else {
            String::new()
        };

        match &stmt.kind {
            Let { init, mutable, .. } => {
                let mut_str = if *mutable { "mut " } else { "" };
                self.write_line(&format!("Let({}){}", mut_str, id_str));
                if let Some(init_id) = init {
                    self.depth += 1;
                    self.print_node_internal(arena, *init_id);
                    self.depth -= 1;
                }
            }
            Expr { expr, has_semi } => {
                let semi = if *has_semi { ";" } else { "" };
                self.write_line(&format!("ExprStmt{}{}", semi, id_str));
                self.depth += 1;
                self.print_node_internal(arena, *expr);
                self.depth -= 1;
            }
            Item(item_id) => {
                self.write_line(&format!("ItemStmt{}", id_str));
                self.depth += 1;
                self.print_node_internal(arena, *item_id);
                self.depth -= 1;
            }
        }
    }

    fn print_item(&mut self, _arena: &Arena, node_id: u32, item: &Item) {
        use crate::decl::ItemKind::*;

        let id_str = if self.config.show_ids {
            format!(" [id={}]", node_id)
        } else {
            String::new()
        };

        match &item.kind {
            Function(func) => {
                let pub_str = if func.is_pub { "pub " } else { "" };
                let async_str = if func.is_async { "async " } else { "" };
                self.write_line(&format!(
                    "{}{}fn {}(...){}",
                    pub_str, async_str, func.name, id_str
                ));
            }
            Type(ty_decl) => {
                let pub_str = if ty_decl.is_pub { "pub " } else { "" };
                self.write_line(&format!("{}type {}{}",pub_str, ty_decl.name, id_str));
            }
            Const(const_decl) => {
                let pub_str = if const_decl.is_pub { "pub " } else { "" };
                self.write_line(&format!(
                    "{}const {}{}",
                    pub_str, const_decl.name, id_str
                ));
            }
            Module(mod_decl) => {
                let pub_str = if mod_decl.is_pub { "pub " } else { "" };
                self.write_line(&format!("{}mod {}{}", pub_str, mod_decl.name, id_str));
            }
            _ => {
                self.write_line(&format!("Item(...){}", id_str));
            }
        }
    }

    fn print_type(&mut self, _arena: &Arena, node_id: u32, ty: &Type) {
        use crate::ty::TypeKind::*;

        let id_str = if self.config.show_ids {
            format!(" [id={}]", node_id)
        } else {
            String::new()
        };

        match &ty.kind {
            Int(int_ty) => {
                self.write_line(&format!("Type({:?}){}", int_ty, id_str));
            }
            Bool => {
                self.write_line(&format!("Type(bool){}", id_str));
            }
            Path { path } => {
                self.write_line(&format!("Type({}){}", path.segments.join("::"), id_str));
            }
            Tuple(types) => {
                self.write_line(&format!("Type(tuple<{}>){}", types.len(), id_str));
            }
            _ => {
                self.write_line(&format!("Type(...){}", id_str));
            }
        }
    }

    fn print_pattern(&mut self, _arena: &Arena, node_id: u32, pattern: &Pattern) {
        use crate::pattern::PatternKind::*;

        let id_str = if self.config.show_ids {
            format!(" [id={}]", node_id)
        } else {
            String::new()
        };

        match &pattern.kind {
            Wildcard => {
                self.write_line(&format!("Pattern(_){}", id_str));
            }
            Ident { name, is_mut } => {
                let mut_str = if *is_mut { "mut " } else { "" };
                self.write_line(&format!("Pattern({}{}){}", mut_str, name, id_str));
            }
            Literal(lit) => {
                self.write_line(&format!("Pattern({:?}){}", lit, id_str));
            }
            _ => {
                self.write_line(&format!("Pattern(...){}", id_str));
            }
        }
    }

    fn write_line(&mut self, s: &str) {
        for _ in 0..self.depth {
            write!(&mut self.output, "{}", self.config.indent).unwrap();
        }
        writeln!(&mut self.output, "{}", s).unwrap();
    }
}

impl Default for PrettyPrinter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{BinaryOp, Expr, ExprKind, Literal};
    use crate::span::Span;

    #[test]
    fn test_pretty_print_simple() {
        let mut arena = Arena::new();

        let expr = arena.alloc_expr(Expr {
            kind: ExprKind::Literal(Literal::Int(42)),
            span: Span::dummy(),
            hygiene: Default::default(),
        });

        let mut printer = PrettyPrinter::new();
        let output = printer.print_node(&arena, expr);

        assert!(output.contains("Literal(Int(42))"));
        assert!(output.contains("[id=0]"));
    }

    #[test]
    fn test_pretty_print_binary() {
        let mut arena = Arena::new();

        let left = arena.alloc_expr(Expr {
            kind: ExprKind::Literal(Literal::Int(1)),
            span: Span::dummy(),
            hygiene: Default::default(),
        });

        let right = arena.alloc_expr(Expr {
            kind: ExprKind::Literal(Literal::Int(2)),
            span: Span::dummy(),
            hygiene: Default::default(),
        });

        let binary = arena.alloc_expr(Expr {
            kind: ExprKind::Binary {
                op: BinaryOp::Add,
                left,
                right,
            },
            span: Span::dummy(),
            hygiene: Default::default(),
        });

        let mut printer = PrettyPrinter::new();
        let output = printer.print_node(&arena, binary);

        assert!(output.contains("Binary(Add)"));
        assert!(output.contains("left:"));
        assert!(output.contains("right:"));
        assert!(output.contains("Literal(Int(1))"));
        assert!(output.contains("Literal(Int(2))"));
    }

    #[test]
    fn test_pretty_print_config() {
        let mut arena = Arena::new();

        let expr = arena.alloc_expr(Expr {
            kind: ExprKind::Ident("x".to_string()),
            span: Span::dummy(),
            hygiene: Default::default(),
        });

        let mut config = PrettyConfig::default();
        config.show_ids = false;

        let mut printer = PrettyPrinter::with_config(config);
        let output = printer.print_node(&arena, expr);

        assert!(output.contains("Ident(x)"));
        assert!(!output.contains("[id="));
    }
}
