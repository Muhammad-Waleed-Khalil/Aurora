//! Mid-Level Intermediate Representation (MIR) for Aurora
//!
//! MIR is a lower-level representation in SSA form that serves as the target
//! for type checking and the source for optimizations and codegen.
//!
//! # SSA Form
//!
//! MIR uses Static Single Assignment (SSA) form where each variable is assigned
//! exactly once. This simplifies many optimizations and analyses.
//!
//! # Structure
//!
//! - Function → BasicBlocks → Instructions
//! - Each instruction has a unique ID
//! - Control flow represented via CFG edges
//! - Effect tracking on all instructions

use aurora_types::{EffectSet, Type};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for MIR values (SSA variables)
pub type ValueId = u32;

/// Unique identifier for basic blocks
pub type BlockId = u32;

/// Unique identifier for functions
pub type FunctionId = u32;

/// MIR Value (SSA variable)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Value {
    /// Unique ID
    pub id: ValueId,
    /// Type of this value
    pub ty: Type,
    /// Source span (for diagnostics)
    pub span: Span,
}

/// Source span for diagnostics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    /// Start byte offset
    pub start: usize,
    /// End byte offset
    pub end: usize,
    /// Source file ID
    pub file_id: usize,
}

impl Span {
    /// Create a new span
    pub fn new(start: usize, end: usize, file_id: usize) -> Self {
        Self {
            start,
            end,
            file_id,
        }
    }

    /// Create a dummy span for testing
    pub fn dummy() -> Self {
        Self {
            start: 0,
            end: 0,
            file_id: 0,
        }
    }
}

/// MIR Instruction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Instruction {
    /// Assign: dest = value
    Assign {
        dest: ValueId,
        value: Operand,
        span: Span,
    },

    /// Binary operation: dest = lhs op rhs
    BinOp {
        dest: ValueId,
        op: BinOp,
        lhs: Operand,
        rhs: Operand,
        span: Span,
    },

    /// Unary operation: dest = op value
    UnaryOp {
        dest: ValueId,
        op: UnaryOp,
        value: Operand,
        span: Span,
    },

    /// Function call: dest = func(args...)
    Call {
        dest: Option<ValueId>,
        func: Operand,
        args: Vec<Operand>,
        effects: EffectSet,
        span: Span,
    },

    /// Return from function
    Return {
        value: Option<Operand>,
        span: Span,
    },

    /// Conditional branch
    Branch {
        cond: Operand,
        then_block: BlockId,
        else_block: BlockId,
        span: Span,
    },

    /// Unconditional jump
    Jump {
        target: BlockId,
        span: Span,
    },

    /// Phi node (SSA merge)
    Phi {
        dest: ValueId,
        inputs: Vec<(BlockId, Operand)>,
        span: Span,
    },

    /// Load from memory
    Load {
        dest: ValueId,
        ptr: Operand,
        effects: EffectSet,
        span: Span,
    },

    /// Store to memory
    Store {
        ptr: Operand,
        value: Operand,
        effects: EffectSet,
        span: Span,
    },

    /// Alloca (stack allocation)
    Alloca {
        dest: ValueId,
        ty: Type,
        effects: EffectSet,
        span: Span,
    },

    /// Cast
    Cast {
        dest: ValueId,
        value: Operand,
        target_ty: Type,
        span: Span,
    },

    /// Get field/element
    GetElement {
        dest: ValueId,
        base: Operand,
        index: Operand,
        span: Span,
    },
}

impl Instruction {
    /// Get the destination value ID (if any)
    pub fn dest(&self) -> Option<ValueId> {
        match self {
            Instruction::Assign { dest, .. }
            | Instruction::BinOp { dest, .. }
            | Instruction::UnaryOp { dest, .. }
            | Instruction::Phi { dest, .. }
            | Instruction::Load { dest, .. }
            | Instruction::Alloca { dest, .. }
            | Instruction::Cast { dest, .. }
            | Instruction::GetElement { dest, .. } => Some(*dest),
            Instruction::Call { dest, .. } => *dest,
            _ => None,
        }
    }

    /// Get span for diagnostics
    pub fn span(&self) -> Span {
        match self {
            Instruction::Assign { span, .. }
            | Instruction::BinOp { span, .. }
            | Instruction::UnaryOp { span, .. }
            | Instruction::Call { span, .. }
            | Instruction::Return { span, .. }
            | Instruction::Branch { span, .. }
            | Instruction::Jump { span, .. }
            | Instruction::Phi { span, .. }
            | Instruction::Load { span, .. }
            | Instruction::Store { span, .. }
            | Instruction::Alloca { span, .. }
            | Instruction::Cast { span, .. }
            | Instruction::GetElement { span, .. } => *span,
        }
    }

    /// Get effects of this instruction
    pub fn effects(&self) -> EffectSet {
        match self {
            Instruction::Call { effects, .. }
            | Instruction::Load { effects, .. }
            | Instruction::Store { effects, .. }
            | Instruction::Alloca { effects, .. } => *effects,
            _ => EffectSet::PURE,
        }
    }
}

/// Binary operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    // Logical
    And,
    Or,
    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

/// Unary operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    Neg,
    Not,
    BitNot,
}

/// Operand (SSA value reference or constant)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operand {
    /// SSA value
    Value(ValueId),
    /// Constant
    Const(Constant),
}

/// Constant value
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Constant {
    Int(i64),
    Float(u64), // Stored as bits
    Bool(bool),
    String(String),
    Unit,
}

/// Basic block in CFG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicBlock {
    /// Unique ID
    pub id: BlockId,
    /// Instructions in this block
    pub instructions: Vec<Instruction>,
    /// Predecessor blocks
    pub predecessors: Vec<BlockId>,
    /// Successor blocks
    pub successors: Vec<BlockId>,
}

impl BasicBlock {
    /// Create new basic block
    pub fn new(id: BlockId) -> Self {
        Self {
            id,
            instructions: Vec::new(),
            predecessors: Vec::new(),
            successors: Vec::new(),
        }
    }

    /// Add instruction to block
    pub fn push(&mut self, inst: Instruction) {
        self.instructions.push(inst);
    }

    /// Get terminator instruction (last instruction)
    pub fn terminator(&self) -> Option<&Instruction> {
        self.instructions.last()
    }

    /// Check if this is a terminator instruction
    pub fn is_terminator(inst: &Instruction) -> bool {
        matches!(
            inst,
            Instruction::Return { .. } | Instruction::Branch { .. } | Instruction::Jump { .. }
        )
    }
}

/// MIR Function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    /// Function ID
    pub id: FunctionId,
    /// Function name
    pub name: String,
    /// Parameter values
    pub params: Vec<ValueId>,
    /// Return type
    pub ret_ty: Type,
    /// Basic blocks
    pub blocks: HashMap<BlockId, BasicBlock>,
    /// Entry block ID
    pub entry: BlockId,
    /// All values in this function
    pub values: HashMap<ValueId, Value>,
    /// Effect signature
    pub effects: EffectSet,
}

impl Function {
    /// Create new function
    pub fn new(id: FunctionId, name: String, ret_ty: Type, effects: EffectSet) -> Self {
        Self {
            id,
            name,
            params: Vec::new(),
            ret_ty,
            blocks: HashMap::new(),
            entry: 0,
            values: HashMap::new(),
            effects,
        }
    }

    /// Add basic block
    pub fn add_block(&mut self, block: BasicBlock) {
        self.blocks.insert(block.id, block);
    }

    /// Get block
    pub fn block(&self, id: BlockId) -> Option<&BasicBlock> {
        self.blocks.get(&id)
    }

    /// Get mutable block
    pub fn block_mut(&mut self, id: BlockId) -> Option<&mut BasicBlock> {
        self.blocks.get_mut(&id)
    }

    /// Add value
    pub fn add_value(&mut self, value: Value) {
        self.values.insert(value.id, value);
    }

    /// Get value
    pub fn value(&self, id: ValueId) -> Option<&Value> {
        self.values.get(&id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_block_creation() {
        let block = BasicBlock::new(0);
        assert_eq!(block.id, 0);
        assert!(block.instructions.is_empty());
    }

    #[test]
    fn test_instruction_dest() {
        let inst = Instruction::Assign {
            dest: 1,
            value: Operand::Const(Constant::Int(42)),
            span: Span::dummy(),
        };
        assert_eq!(inst.dest(), Some(1));

        let ret = Instruction::Return {
            value: None,
            span: Span::dummy(),
        };
        assert_eq!(ret.dest(), None);
    }

    #[test]
    fn test_function_creation() {
        let func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);
        assert_eq!(func.id, 0);
        assert_eq!(func.name, "test");
        assert!(func.blocks.is_empty());
    }

    #[test]
    fn test_operand() {
        let op1 = Operand::Value(42);
        let op2 = Operand::Const(Constant::Bool(true));

        assert!(matches!(op1, Operand::Value(42)));
        assert!(matches!(op2, Operand::Const(Constant::Bool(true))));
    }

    #[test]
    fn test_span() {
        let span = Span::new(10, 20, 0);
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 20);
        assert_eq!(span.file_id, 0);
    }
}
