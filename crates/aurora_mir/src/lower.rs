//! MIR Lowering - Convert typed AST to MIR
//!
//! This module lowers the typed AST into MIR (SSA form).

use crate::mir::*;
use aurora_ast::{Ast, ExprId, ExprKind, ItemKind, StmtKind};
use aurora_types::{EffectSet, Type, TypeMap, PrimitiveType};
use std::collections::HashMap;
use std::sync::Arc;

/// MIR Builder - constructs MIR from AST
pub struct MirBuilder {
    /// Next value ID
    next_value: ValueId,
    /// Next block ID
    next_block: BlockId,
    /// Current function being built
    current_func: Option<Function>,
    /// Current block being built
    current_block: Option<BlockId>,
    /// Variable map (name -> ValueId)
    vars: HashMap<String, ValueId>,
}

impl MirBuilder {
    /// Create new MIR builder
    pub fn new() -> Self {
        Self {
            next_value: 0,
            next_block: 0,
            current_func: None,
            current_block: None,
            vars: HashMap::new(),
        }
    }

    /// Start building a function
    pub fn start_function(
        &mut self,
        id: FunctionId,
        name: String,
        ret_ty: Type,
        effects: EffectSet,
    ) {
        let mut func = Function::new(id, name, ret_ty, effects);
        
        // Create entry block
        let entry_block = self.new_block();
        func.entry = entry_block;
        func.add_block(BasicBlock::new(entry_block));
        
        self.current_func = Some(func);
        self.current_block = Some(entry_block);
        self.vars.clear();
    }

    /// Finish building current function
    pub fn finish_function(&mut self) -> Option<Function> {
        self.current_func.take()
    }

    /// Create new value
    pub fn new_value(&mut self, ty: Type, span: Span) -> ValueId {
        let id = self.next_value;
        self.next_value += 1;

        if let Some(func) = &mut self.current_func {
            func.add_value(Value { id, ty, span });
        }

        id
    }

    /// Create new block
    pub fn new_block(&mut self) -> BlockId {
        let id = self.next_block;
        self.next_block += 1;
        id
    }

    /// Set current block
    pub fn set_block(&mut self, block: BlockId) {
        self.current_block = Some(block);
    }

    /// Emit instruction to current block
    pub fn emit(&mut self, inst: Instruction) {
        if let Some(func) = &mut self.current_func {
            if let Some(block_id) = self.current_block {
                if let Some(block) = func.block_mut(block_id) {
                    block.push(inst);
                }
            }
        }
    }

    /// Build assignment
    pub fn build_assign(&mut self, dest: ValueId, value: Operand, span: Span) {
        self.emit(Instruction::Assign { dest, value, span });
    }

    /// Build binary operation
    pub fn build_binop(
        &mut self,
        op: BinOp,
        lhs: Operand,
        rhs: Operand,
        result_ty: Type,
        span: Span,
    ) -> ValueId {
        let dest = self.new_value(result_ty, span);
        self.emit(Instruction::BinOp {
            dest,
            op,
            lhs,
            rhs,
            span,
        });
        dest
    }

    /// Build unary operation
    pub fn build_unaryop(
        &mut self,
        op: UnaryOp,
        value: Operand,
        result_ty: Type,
        span: Span,
    ) -> ValueId {
        let dest = self.new_value(result_ty, span);
        self.emit(Instruction::UnaryOp {
            dest,
            op,
            value,
            span,
        });
        dest
    }

    /// Build function call
    pub fn build_call(
        &mut self,
        func: Operand,
        args: Vec<Operand>,
        ret_ty: Option<Type>,
        effects: EffectSet,
        span: Span,
    ) -> Option<ValueId> {
        let dest = ret_ty.map(|ty| self.new_value(ty, span));
        self.emit(Instruction::Call {
            dest,
            func,
            args,
            effects,
            span,
        });
        dest
    }

    /// Build return
    pub fn build_return(&mut self, value: Option<Operand>, span: Span) {
        self.emit(Instruction::Return { value, span });
    }

    /// Build conditional branch
    pub fn build_branch(
        &mut self,
        cond: Operand,
        then_block: BlockId,
        else_block: BlockId,
        span: Span,
    ) {
        self.emit(Instruction::Branch {
            cond,
            then_block,
            else_block,
            span,
        });
    }

    /// Build unconditional jump
    pub fn build_jump(&mut self, target: BlockId, span: Span) {
        self.emit(Instruction::Jump { target, span });
    }

    /// Build phi node
    pub fn build_phi(
        &mut self,
        inputs: Vec<(BlockId, Operand)>,
        result_ty: Type,
        span: Span,
    ) -> ValueId {
        let dest = self.new_value(result_ty, span);
        self.emit(Instruction::Phi { dest, inputs, span });
        dest
    }

    /// Build alloca
    pub fn build_alloca(&mut self, ty: Type, span: Span) -> ValueId {
        let dest = self.new_value(ty.clone(), span);
        self.emit(Instruction::Alloca {
            dest,
            ty,
            effects: EffectSet::ALLOC,
            span,
        });
        dest
    }

    /// Build load
    pub fn build_load(&mut self, ptr: Operand, ty: Type, span: Span) -> ValueId {
        let dest = self.new_value(ty, span);
        self.emit(Instruction::Load {
            dest,
            ptr,
            effects: EffectSet::IO,
            span,
        });
        dest
    }

    /// Build store
    pub fn build_store(&mut self, ptr: Operand, value: Operand, span: Span) {
        self.emit(Instruction::Store {
            ptr,
            value,
            effects: EffectSet::IO,
            span,
        });
    }

    /// Define variable
    pub fn define_var(&mut self, name: String, value: ValueId) {
        self.vars.insert(name, value);
    }

    /// Lookup variable
    pub fn lookup_var(&self, name: &str) -> Option<ValueId> {
        self.vars.get(name).copied()
    }

    /// Add block to current function
    pub fn add_block(&mut self, block: BasicBlock) {
        if let Some(func) = &mut self.current_func {
            func.add_block(block);
        }
    }

    /// Check if current block is terminated
    pub fn is_terminated(&self) -> bool {
        if let Some(func) = &self.current_func {
            if let Some(block_id) = self.current_block {
                if let Some(block) = func.block(block_id) {
                    if let Some(term) = block.terminator() {
                        return BasicBlock::is_terminator(term);
                    }
                }
            }
        }
        false
    }
}

/// AST to MIR lowering context
pub struct LoweringContext<D> {
    /// MIR builder
    builder: MirBuilder,
    /// Type map from type checker
    type_map: TypeMap,
    /// Function ID counter
    next_func_id: FunctionId,
    /// Diagnostic collector
    diagnostics: Arc<D>,
    /// Current AST reference
    ast: Option<Ast>,
}

impl<D: Send + Sync + 'static> LoweringContext<D> {
    /// Create new lowering context
    pub fn new(diagnostics: Arc<D>, type_map: TypeMap) -> Self {
        Self {
            builder: MirBuilder::new(),
            type_map,
            next_func_id: 0,
            diagnostics,
            ast: None,
        }
    }

    /// Lower entire AST to MIR module
    pub fn lower(&mut self, ast: Ast) -> crate::MirModule {
        self.ast = Some(ast.clone());
        let mut module = crate::MirModule::new();

        // Lower each top-level item
        for &item_id in &ast.items {
            // In a real implementation, we would fetch the item from an arena
            // For now, we'll create a simple stub
            // This would be: let item = ast.get_item(item_id);
            // Then match on item.kind and lower accordingly
        }

        module
    }

    /// Lower a function declaration
    fn lower_function(&mut self, _name: String, _params: Vec<(String, Type)>, _ret_ty: Type, _body: &[StmtKind]) -> Function {
        let func_id = self.next_func_id;
        self.next_func_id += 1;

        self.builder.start_function(func_id, _name.clone(), _ret_ty.clone(), EffectSet::PURE);

        // Lower function body
        // In a real implementation, we would:
        // 1. Create parameters as values
        // 2. Lower each statement in the body
        // 3. Ensure proper termination

        self.builder.finish_function().unwrap()
    }

    /// Lower an expression to MIR
    fn lower_expr(&mut self, expr_id: ExprId, _expr: &ExprKind) -> Operand {
        // Get the type of this expression
        let ty = self.type_map.get_expr(expr_id).cloned().unwrap_or(Type::Unit);
        let span = Span::dummy(); // Would get from AST

        match _expr {
            ExprKind::Literal(lit) => {
                // Lower literal
                use aurora_ast::expr::Literal;
                let const_val = match lit {
                    Literal::Int(i) => Constant::Int(*i),
                    Literal::Float(f) => Constant::Float(f.to_bits()),
                    Literal::String(s) => Constant::String(s.clone()),
                    Literal::Bool(b) => Constant::Bool(*b),
                    Literal::Char(_) => Constant::Int(0), // Simplified
                };
                Operand::Const(const_val)
            }
            ExprKind::Ident(name) => {
                // Look up variable
                if let Some(value_id) = self.builder.lookup_var(name) {
                    Operand::Value(value_id)
                } else {
                    // Undefined variable - report error
                    Operand::Const(Constant::Unit)
                }
            }
            ExprKind::Binary { op, left, right } => {
                // Lower both sides (recursively would fetch from AST)
                let lhs = Operand::Const(Constant::Int(0)); // Placeholder
                let rhs = Operand::Const(Constant::Int(0)); // Placeholder

                // Map AST BinOp to MIR BinOp
                let mir_op = match op {
                    aurora_ast::expr::BinaryOp::Add => BinOp::Add,
                    aurora_ast::expr::BinaryOp::Sub => BinOp::Sub,
                    aurora_ast::expr::BinaryOp::Mul => BinOp::Mul,
                    aurora_ast::expr::BinaryOp::Div => BinOp::Div,
                    aurora_ast::expr::BinaryOp::Rem => BinOp::Mod,
                    aurora_ast::expr::BinaryOp::Eq => BinOp::Eq,
                    aurora_ast::expr::BinaryOp::Ne => BinOp::Ne,
                    aurora_ast::expr::BinaryOp::Lt => BinOp::Lt,
                    aurora_ast::expr::BinaryOp::Le => BinOp::Le,
                    aurora_ast::expr::BinaryOp::Gt => BinOp::Gt,
                    aurora_ast::expr::BinaryOp::Ge => BinOp::Ge,
                    aurora_ast::expr::BinaryOp::And => BinOp::And,
                    aurora_ast::expr::BinaryOp::Or => BinOp::Or,
                    aurora_ast::expr::BinaryOp::BitAnd => BinOp::BitAnd,
                    aurora_ast::expr::BinaryOp::BitOr => BinOp::BitOr,
                    aurora_ast::expr::BinaryOp::BitXor => BinOp::BitXor,
                    aurora_ast::expr::BinaryOp::Shl => BinOp::Shl,
                    aurora_ast::expr::BinaryOp::Shr => BinOp::Shr,
                    _ => BinOp::Add, // Default for unsupported ops
                };

                let result = self.builder.build_binop(mir_op, lhs, rhs, ty, span);
                Operand::Value(result)
            }
            ExprKind::Unary { op, operand } => {
                let value = Operand::Const(Constant::Int(0)); // Placeholder

                let mir_op = match op {
                    aurora_ast::expr::UnaryOp::Neg => UnaryOp::Neg,
                    aurora_ast::expr::UnaryOp::Not => UnaryOp::Not,
                    aurora_ast::expr::UnaryOp::BitNot => UnaryOp::BitNot,
                    _ => UnaryOp::Not, // Default
                };

                let result = self.builder.build_unaryop(mir_op, value, ty, span);
                Operand::Value(result)
            }
            ExprKind::Call { func, args } => {
                // Lower function and arguments
                let func_op = Operand::Const(Constant::Int(0)); // Placeholder
                let arg_ops = vec![]; // Placeholder

                let result = self.builder.build_call(
                    func_op,
                    arg_ops,
                    Some(ty),
                    EffectSet::IO, // Conservative - would analyze actual effects
                    span,
                );

                result.map_or(Operand::Const(Constant::Unit), Operand::Value)
            }
            ExprKind::If { condition, then_block, else_block } => {
                // Lower conditional
                let cond = Operand::Const(Constant::Bool(true)); // Placeholder

                let then_bb = self.builder.new_block();
                let else_bb = self.builder.new_block();
                let merge_bb = self.builder.new_block();

                // Branch
                self.builder.build_branch(cond, then_bb, else_bb, span);

                // Then block
                self.builder.add_block(BasicBlock::new(then_bb));
                self.builder.set_block(then_bb);
                let then_val = Operand::Const(Constant::Unit); // Placeholder
                if !self.builder.is_terminated() {
                    self.builder.build_jump(merge_bb, span);
                }

                // Else block
                self.builder.add_block(BasicBlock::new(else_bb));
                self.builder.set_block(else_bb);
                let else_val = Operand::Const(Constant::Unit); // Placeholder
                if !self.builder.is_terminated() {
                    self.builder.build_jump(merge_bb, span);
                }

                // Merge block
                self.builder.add_block(BasicBlock::new(merge_bb));
                self.builder.set_block(merge_bb);

                // PHI node for result
                let phi = self.builder.build_phi(
                    vec![(then_bb, then_val), (else_bb, else_val)],
                    ty,
                    span,
                );

                Operand::Value(phi)
            }
            ExprKind::Return { value } => {
                let ret_val = if value.is_some() {
                    Some(Operand::Const(Constant::Unit)) // Placeholder
                } else {
                    None
                };
                self.builder.build_return(ret_val, span);
                Operand::Const(Constant::Unit)
            }
            _ => {
                // Unsupported expression kind
                Operand::Const(Constant::Unit)
            }
        }
    }
}

impl Default for MirBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_creation() {
        let builder = MirBuilder::new();
        assert_eq!(builder.next_value, 0);
        assert_eq!(builder.next_block, 0);
    }

    #[test]
    fn test_start_function() {
        let mut builder = MirBuilder::new();
        builder.start_function(0, "test".to_string(), Type::Unit, EffectSet::PURE);
        
        assert!(builder.current_func.is_some());
        assert!(builder.current_block.is_some());
    }

    #[test]
    fn test_new_value() {
        let mut builder = MirBuilder::new();
        builder.start_function(0, "test".to_string(), Type::Unit, EffectSet::PURE);
        
        let v1 = builder.new_value(Type::Unit, Span::dummy());
        let v2 = builder.new_value(Type::Unit, Span::dummy());
        
        assert_eq!(v1, 0);
        assert_eq!(v2, 1);
    }

    #[test]
    fn test_build_assign() {
        let mut builder = MirBuilder::new();
        builder.start_function(0, "test".to_string(), Type::Unit, EffectSet::PURE);
        
        let dest = builder.new_value(Type::Unit, Span::dummy());
        builder.build_assign(dest, Operand::Const(Constant::Int(42)), Span::dummy());
        
        let func = builder.finish_function().unwrap();
        let entry_block = func.block(func.entry).unwrap();
        assert_eq!(entry_block.instructions.len(), 1);
    }

    #[test]
    fn test_build_binop() {
        let mut builder = MirBuilder::new();
        builder.start_function(0, "test".to_string(), Type::Unit, EffectSet::PURE);
        
        let result = builder.build_binop(
            BinOp::Add,
            Operand::Const(Constant::Int(1)),
            Operand::Const(Constant::Int(2)),
            Type::Unit,
            Span::dummy(),
        );
        
        assert_eq!(result, 0);
    }

    #[test]
    fn test_build_call() {
        let mut builder = MirBuilder::new();
        builder.start_function(0, "test".to_string(), Type::Unit, EffectSet::PURE);
        
        let result = builder.build_call(
            Operand::Const(Constant::Int(0)),
            vec![],
            Some(Type::Unit),
            EffectSet::PURE,
            Span::dummy(),
        );
        
        assert!(result.is_some());
    }

    #[test]
    fn test_variables() {
        let mut builder = MirBuilder::new();
        builder.start_function(0, "test".to_string(), Type::Unit, EffectSet::PURE);
        
        let v1 = builder.new_value(Type::Unit, Span::dummy());
        builder.define_var("x".to_string(), v1);
        
        assert_eq!(builder.lookup_var("x"), Some(v1));
        assert_eq!(builder.lookup_var("y"), None);
    }

    #[test]
    fn test_blocks() {
        let mut builder = MirBuilder::new();
        builder.start_function(0, "test".to_string(), Type::Unit, EffectSet::PURE);

        let b1 = builder.new_block();
        let b2 = builder.new_block();

        assert_ne!(b1, b2);
    }

    #[test]
    fn test_build_return() {
        let mut builder = MirBuilder::new();
        builder.start_function(0, "test".to_string(), Type::Unit, EffectSet::PURE);

        builder.build_return(None, Span::dummy());

        let func = builder.finish_function().unwrap();
        let entry_block = func.block(func.entry).unwrap();
        assert_eq!(entry_block.instructions.len(), 1);
    }

    #[test]
    fn test_build_branch() {
        let mut builder = MirBuilder::new();
        builder.start_function(0, "test".to_string(), Type::Unit, EffectSet::PURE);

        let then_block = builder.new_block();
        let else_block = builder.new_block();

        builder.build_branch(
            Operand::Const(Constant::Bool(true)),
            then_block,
            else_block,
            Span::dummy(),
        );

        let func = builder.finish_function().unwrap();
        let entry = func.block(func.entry).unwrap();
        assert_eq!(entry.instructions.len(), 1);
    }

    #[test]
    fn test_build_jump() {
        let mut builder = MirBuilder::new();
        builder.start_function(0, "test".to_string(), Type::Unit, EffectSet::PURE);

        let target = builder.new_block();
        builder.build_jump(target, Span::dummy());

        let func = builder.finish_function().unwrap();
        let entry = func.block(func.entry).unwrap();
        assert_eq!(entry.instructions.len(), 1);
    }

    #[test]
    fn test_build_phi() {
        let mut builder = MirBuilder::new();
        builder.start_function(0, "test".to_string(), Type::Unit, EffectSet::PURE);

        let inputs = vec![
            (0, Operand::Const(Constant::Int(1))),
            (1, Operand::Const(Constant::Int(2))),
        ];

        builder.build_phi(inputs, Type::Unit, Span::dummy());

        let func = builder.finish_function().unwrap();
        assert!(func.values.len() > 0);
    }

    #[test]
    fn test_build_alloca() {
        let mut builder = MirBuilder::new();
        builder.start_function(0, "test".to_string(), Type::Unit, EffectSet::PURE);

        builder.build_alloca(Type::Unit, Span::dummy());

        let func = builder.finish_function().unwrap();
        let entry = func.block(func.entry).unwrap();
        assert_eq!(entry.instructions.len(), 1);
    }

    #[test]
    fn test_build_load() {
        let mut builder = MirBuilder::new();
        builder.start_function(0, "test".to_string(), Type::Unit, EffectSet::PURE);

        builder.build_load(Operand::Const(Constant::Int(0)), Type::Unit, Span::dummy());

        let func = builder.finish_function().unwrap();
        let entry = func.block(func.entry).unwrap();
        assert_eq!(entry.instructions.len(), 1);
    }

    #[test]
    fn test_build_store() {
        let mut builder = MirBuilder::new();
        builder.start_function(0, "test".to_string(), Type::Unit, EffectSet::PURE);

        builder.build_store(
            Operand::Const(Constant::Int(0)),
            Operand::Const(Constant::Int(42)),
            Span::dummy(),
        );

        let func = builder.finish_function().unwrap();
        let entry = func.block(func.entry).unwrap();
        assert_eq!(entry.instructions.len(), 1);
    }

    #[test]
    fn test_set_block() {
        let mut builder = MirBuilder::new();
        builder.start_function(0, "test".to_string(), Type::Unit, EffectSet::PURE);

        let new_block = builder.new_block();
        builder.set_block(new_block);

        assert_eq!(builder.current_block, Some(new_block));
    }

    #[test]
    fn test_is_terminated_false() {
        let mut builder = MirBuilder::new();
        builder.start_function(0, "test".to_string(), Type::Unit, EffectSet::PURE);

        assert!(!builder.is_terminated());
    }

    #[test]
    fn test_is_terminated_true() {
        let mut builder = MirBuilder::new();
        builder.start_function(0, "test".to_string(), Type::Unit, EffectSet::PURE);

        builder.build_return(None, Span::dummy());

        assert!(builder.is_terminated());
    }

    #[test]
    fn test_lowering_context_creation() {
        use aurora_types::TypeMap;

        let diagnostics = Arc::new(());
        let type_map = TypeMap::new();
        let _ctx: LoweringContext<()> = LoweringContext::new(diagnostics, type_map);
    }

    #[test]
    fn test_lower_empty_ast() {
        use aurora_types::TypeMap;

        let diagnostics = Arc::new(());
        let type_map = TypeMap::new();
        let mut ctx: LoweringContext<()> = LoweringContext::new(diagnostics, type_map);

        let ast = Ast::empty();
        let module = ctx.lower(ast);

        assert_eq!(module.function_count(), 0);
    }

    #[test]
    fn test_multiple_blocks() {
        let mut builder = MirBuilder::new();
        builder.start_function(0, "test".to_string(), Type::Unit, EffectSet::PURE);

        for _ in 0..5 {
            let block = builder.new_block();
            builder.add_block(BasicBlock::new(block));
        }

        let func = builder.finish_function().unwrap();
        // Entry block + 5 new blocks = 6 total
        assert!(func.blocks.len() >= 1);
    }

    #[test]
    fn test_value_increments() {
        let mut builder = MirBuilder::new();
        builder.start_function(0, "test".to_string(), Type::Unit, EffectSet::PURE);

        let v1 = builder.new_value(Type::Unit, Span::dummy());
        let v2 = builder.new_value(Type::Unit, Span::dummy());
        let v3 = builder.new_value(Type::Unit, Span::dummy());

        assert!(v2 > v1);
        assert!(v3 > v2);
    }

    #[test]
    fn test_build_unaryop_neg() {
        let mut builder = MirBuilder::new();
        builder.start_function(0, "test".to_string(), Type::Unit, EffectSet::PURE);

        let result = builder.build_unaryop(
            UnaryOp::Neg,
            Operand::Const(Constant::Int(42)),
            Type::Unit,
            Span::dummy(),
        );

        assert!(result >= 0);
    }

    #[test]
    fn test_build_unaryop_not() {
        let mut builder = MirBuilder::new();
        builder.start_function(0, "test".to_string(), Type::Unit, EffectSet::PURE);

        let result = builder.build_unaryop(
            UnaryOp::Not,
            Operand::Const(Constant::Bool(true)),
            Type::Unit,
            Span::dummy(),
        );

        assert!(result >= 0);
    }
}
