//! MIR Lowering - Convert typed AST to MIR
//!
//! This module lowers the typed AST into MIR (SSA form).

use crate::mir::*;
use aurora_types::{EffectSet, Type};
use std::collections::HashMap;

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
}
