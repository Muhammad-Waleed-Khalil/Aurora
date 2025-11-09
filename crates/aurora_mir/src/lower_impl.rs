//! MIR Lowering Implementation - Actual AST traversal

use super::lower::{LoweringContext, MirBuilder};
use crate::mir::*;
use aurora_ast::nodes::AstNode;
use aurora_ast::{Ast, ExprId, ExprKind, ItemKind, StmtKind};
use aurora_types::{EffectSet, Type, PrimitiveType};

impl<D: Send + Sync + 'static> LoweringContext<D> {
    /// Lower entire AST to MIR module (real implementation)
    pub fn lower_ast_real(&mut self, ast: Ast) -> crate::MirModule {
        let mut module = crate::MirModule::new();

        // Iterate through top-level items
        for &item_id in &ast.items {
            if let Some(AstNode::Item(item)) = ast.arena.get(item_id) {
                match &item.kind {
                    ItemKind::Function(func_decl) => {
                        let function = self.lower_function_real(func_decl, &ast);
                        module.add_function(function);
                    }
                    _ => {}
                }
            }
        }

        module
    }

    /// Lower a function declaration to MIR
    fn lower_function_real(
        &mut self,
        func_decl: &aurora_ast::decl::FunctionDecl,
        ast: &Ast,
    ) -> Function {
        let func_id = self.next_func_id;
        self.next_func_id += 1;

        let ret_ty = Type::Unit;
        self.builder.start_function(func_id, func_decl.name.clone(), ret_ty, EffectSet::IO);
        self.lower_block(&func_decl.body, ast);

        if !self.builder.is_terminated() {
            self.builder.build_return(None, Span::dummy());
        }

        self.builder.finish_function().unwrap()
    }

    /// Lower a block
    fn lower_block(&mut self, block: &aurora_ast::stmt::Block, ast: &Ast) {
        eprintln!("[MIR] Lowering block with {} statements", block.stmts.len());
        for &stmt_id in &block.stmts {
            eprintln!("[MIR] Lowering statement ID: {}", stmt_id);
            if let Some(AstNode::Stmt(stmt)) = ast.arena.get(stmt_id) {
                eprintln!("[MIR] Found statement: {:?}", stmt.kind);
                self.lower_stmt(&stmt.kind, ast, Span::dummy());
            } else {
                eprintln!("[MIR] Statement ID {} not found in arena!", stmt_id);
            }
        }

        if let Some(expr_id) = block.expr {
            eprintln!("[MIR] Lowering trailing expression ID: {}", expr_id);
            let _value = self.lower_expr_real(expr_id, ast);
        }
    }

    /// Lower a statement
    fn lower_stmt(&mut self, stmt: &StmtKind, ast: &Ast, span: Span) {
        match stmt {
            StmtKind::Let { pattern, init, .. } => {
                if let Some(AstNode::Pattern(pat)) = ast.arena.get(*pattern) {
                    use aurora_ast::pattern::PatternKind;
                    if let PatternKind::Ident { name, .. } = &pat.kind {
                        if let Some(init_expr) = init {
                            let value_op = self.lower_expr_real(*init_expr, ast);
                            let ty = Type::Primitive(PrimitiveType::I64);
                            let value_id = self.builder.new_value(ty, span);
                            self.builder.build_assign(value_id, value_op, span);
                            self.builder.define_var(name.clone(), value_id);
                        }
                    }
                }
            }
            StmtKind::Expr { expr, .. } => {
                self.lower_expr_real(*expr, ast);
            }
            StmtKind::Item(_) => {}
        }
    }

    /// Lower an expression to an operand
    fn lower_expr_real(&mut self, expr_id: ExprId, ast: &Ast) -> Operand {
        if let Some(AstNode::Expr(expr)) = ast.arena.get(expr_id) {
            match &expr.kind {
                ExprKind::Literal(lit) => {
                    use aurora_ast::expr::Literal;
                    let const_val = match lit {
                        Literal::Int(i) => Constant::Int(*i),
                        Literal::Float(f) => Constant::Float(f.to_bits()),
                        Literal::String(s) => Constant::String(s.clone()),
                        Literal::Bool(b) => Constant::Bool(*b),
                        Literal::Char(c) => Constant::Int(*c as i64),
                    };
                    Operand::Const(const_val)
                }
                ExprKind::Ident(name) => {
                    if let Some(value_id) = self.builder.lookup_var(name) {
                        Operand::Value(value_id)
                    } else {
                        Operand::Const(Constant::String(name.clone()))
                    }
                }
                ExprKind::Call { func, args } => {
                    let func_op = self.lower_expr_real(*func, ast);
                    let arg_ops: Vec<Operand> = args.iter().map(|&arg| self.lower_expr_real(arg, ast)).collect();

                    self.builder.build_call(func_op, arg_ops, None, EffectSet::IO, Span::dummy());
                    Operand::Const(Constant::Unit)
                }
                ExprKind::Binary { op, left, right } => {
                    let lhs = self.lower_expr_real(*left, ast);
                    let rhs = self.lower_expr_real(*right, ast);
                    let mir_op = self.convert_binop(op);
                    let result_ty = Type::Primitive(PrimitiveType::I64);
                    let value_id = self.builder.build_binop(mir_op, lhs, rhs, result_ty, Span::dummy());
                    Operand::Value(value_id)
                }
                ExprKind::Unary { op, operand } => {
                    let val = self.lower_expr_real(*operand, ast);
                    let mir_op = self.convert_unaryop(op);
                    let result_ty = Type::Primitive(PrimitiveType::I64);
                    let value_id = self.builder.build_unaryop(mir_op, val, result_ty, Span::dummy());
                    Operand::Value(value_id)
                }
                _ => Operand::Const(Constant::Unit)
            }
        } else {
            Operand::Const(Constant::Unit)
        }
    }

    /// Convert AST BinaryOp to MIR BinOp
    fn convert_binop(&self, op: &aurora_ast::expr::BinaryOp) -> BinOp {
        use aurora_ast::expr::BinaryOp;
        match op {
            BinaryOp::Add => BinOp::Add,
            BinaryOp::Sub => BinOp::Sub,
            BinaryOp::Mul => BinOp::Mul,
            BinaryOp::Div => BinOp::Div,
            BinaryOp::Rem => BinOp::Mod,
            BinaryOp::Eq => BinOp::Eq,
            BinaryOp::Ne => BinOp::Ne,
            BinaryOp::Lt => BinOp::Lt,
            BinaryOp::Le => BinOp::Le,
            BinaryOp::Gt => BinOp::Gt,
            BinaryOp::Ge => BinOp::Ge,
            BinaryOp::And => BinOp::And,
            BinaryOp::Or => BinOp::Or,
            BinaryOp::BitAnd => BinOp::BitAnd,
            BinaryOp::BitOr => BinOp::BitOr,
            BinaryOp::BitXor => BinOp::BitXor,
            BinaryOp::Shl => BinOp::Shl,
            BinaryOp::Shr => BinOp::Shr,
            _ => BinOp::Add,
        }
    }

    /// Convert AST UnaryOp to MIR UnaryOp
    fn convert_unaryop(&self, op: &aurora_ast::expr::UnaryOp) -> UnaryOp {
        use aurora_ast::expr::UnaryOp as AstUnaryOp;
        match op {
            AstUnaryOp::Neg => UnaryOp::Neg,
            AstUnaryOp::Not => UnaryOp::Not,
            AstUnaryOp::BitNot => UnaryOp::BitNot,
            _ => UnaryOp::Not,
        }
    }
}
