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
                ExprKind::If { condition, then_block, else_block } => {
                    self.lower_if(*condition, *then_block, *else_block, ast)
                }
                ExprKind::While { condition, body } => {
                    self.lower_while(*condition, *body, ast);
                    Operand::Const(Constant::Unit)
                }
                ExprKind::Return { value } => {
                    let ret_val = value.map(|v| self.lower_expr_real(v, ast));
                    self.builder.build_return(ret_val, Span::dummy());
                    Operand::Const(Constant::Unit)
                }
                ExprKind::Block(block_id) => {
                    if let Some(AstNode::Block(block)) = ast.arena.get(*block_id) {
                        self.lower_block(block, ast);
                    }
                    Operand::Const(Constant::Unit)
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

    /// Lower an if expression
    fn lower_if(
        &mut self,
        condition: ExprId,
        then_block_id: u32,
        else_block_id: Option<u32>,
        ast: &Ast,
    ) -> Operand {
        eprintln!("[MIR] Lowering if expression");

        // Evaluate condition
        let cond_op = self.lower_expr_real(condition, ast);

        // Create blocks for then, else, and merge
        let then_bb = self.builder.new_block();
        let else_bb = self.builder.new_block();
        let merge_bb = self.builder.new_block();

        // Branch on condition
        self.builder.build_branch(cond_op, then_bb, else_bb, Span::dummy());

        // Lower then block
        self.builder.set_block(then_bb);
        if let Some(AstNode::Block(then_block)) = ast.arena.get(then_block_id) {
            self.lower_block(then_block, ast);
        }
        if !self.builder.is_terminated() {
            self.builder.build_jump(merge_bb, Span::dummy());
        }

        // Lower else block
        self.builder.set_block(else_bb);
        if let Some(else_id) = else_block_id {
            if let Some(AstNode::Block(else_block)) = ast.arena.get(else_id) {
                self.lower_block(else_block, ast);
            }
        }
        if !self.builder.is_terminated() {
            self.builder.build_jump(merge_bb, Span::dummy());
        }

        // Continue in merge block
        self.builder.set_block(merge_bb);
        Operand::Const(Constant::Unit)
    }

    /// Lower a while loop
    fn lower_while(&mut self, condition: ExprId, body_id: u32, ast: &Ast) {
        eprintln!("[MIR] Lowering while loop");

        // Create blocks for header, body, and exit
        let header_bb = self.builder.new_block();
        let body_bb = self.builder.new_block();
        let exit_bb = self.builder.new_block();

        // Jump to header
        self.builder.build_jump(header_bb, Span::dummy());

        // Header: evaluate condition and branch
        self.builder.set_block(header_bb);
        let cond_op = self.lower_expr_real(condition, ast);
        self.builder.build_branch(cond_op, body_bb, exit_bb, Span::dummy());

        // Body: execute loop body and jump back to header
        self.builder.set_block(body_bb);
        if let Some(AstNode::Block(body)) = ast.arena.get(body_id) {
            self.lower_block(body, ast);
        }
        if !self.builder.is_terminated() {
            self.builder.build_jump(header_bb, Span::dummy());
        }

        // Continue in exit block
        self.builder.set_block(exit_bb);
    }
}
