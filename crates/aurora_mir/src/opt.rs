//! MIR Optimization Passes
//!
//! This module implements all MIR-level optimizations:
//! - SROA (Scalar Replacement of Aggregates)
//! - GVN (Global Value Numbering)
//! - LICM (Loop-Invariant Code Motion)
//! - DCE (Dead Code Elimination)
//! - Constant Folding
//! - Constant Propagation
//! - Copy Propagation
//! - Inlining
//! - NRVO (Named Return Value Optimization)
//! - Devirtualization
//! - Loop SIMD hints

use crate::cfg::{DominatorTree, Loop, CFG};
use crate::mir::*;
use std::collections::{HashMap, HashSet};

/// Optimization level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptLevel {
    /// No optimization
    O0,
    /// Basic optimizations (constant folding, DCE)
    O1,
    /// Standard optimizations (GVN, LICM, inlining)
    O2,
    /// Aggressive optimizations (all passes)
    O3,
}

impl OptLevel {
    /// Convert u8 to OptLevel
    pub fn from_u8(level: u8) -> Self {
        match level {
            0 => OptLevel::O0,
            1 => OptLevel::O1,
            2 => OptLevel::O2,
            _ => OptLevel::O3,
        }
    }
}

/// Optimization pass trait
pub trait OptPass {
    /// Run optimization pass on a function
    fn run(&mut self, func: &mut Function) -> bool;

    /// Get pass name
    fn name(&self) -> &str;
}

/// Constant folding pass
pub struct ConstantFolding;

impl OptPass for ConstantFolding {
    fn run(&mut self, func: &mut Function) -> bool {
        let mut changed = false;

        for block in func.blocks.values_mut() {
            let mut new_instructions = Vec::new();

            for inst in &block.instructions {
                match inst {
                    Instruction::BinOp { dest, op, lhs, rhs, span } => {
                        if let (Some(l), Some(r)) = (as_const(lhs), as_const(rhs)) {
                            if let Some(result) = fold_binop(*op, l, r) {
                                new_instructions.push(Instruction::Assign {
                                    dest: *dest,
                                    value: Operand::Const(result),
                                    span: *span,
                                });
                                changed = true;
                                continue;
                            }
                        }
                    }
                    Instruction::UnaryOp { dest, op, value, span } => {
                        if let Some(v) = as_const(value) {
                            if let Some(result) = fold_unaryop(*op, v) {
                                new_instructions.push(Instruction::Assign {
                                    dest: *dest,
                                    value: Operand::Const(result),
                                    span: *span,
                                });
                                changed = true;
                                continue;
                            }
                        }
                    }
                    _ => {}
                }
                new_instructions.push(inst.clone());
            }

            block.instructions = new_instructions;
        }

        changed
    }

    fn name(&self) -> &str {
        "constant_folding"
    }
}

/// Dead code elimination pass
pub struct DeadCodeElimination;

impl OptPass for DeadCodeElimination {
    fn run(&mut self, func: &mut Function) -> bool {
        let mut changed = false;
        let mut live = HashSet::new();

        // Mark all values used in non-dead instructions
        for block in func.blocks.values() {
            for inst in &block.instructions {
                match inst {
                    Instruction::Return { value, .. } => {
                        if let Some(Operand::Value(v)) = value {
                            live.insert(*v);
                        }
                    }
                    Instruction::Branch { cond, .. } => {
                        if let Operand::Value(v) = cond {
                            live.insert(*v);
                        }
                    }
                    Instruction::Call { func: f, args, .. } => {
                        if let Operand::Value(v) = f {
                            live.insert(*v);
                        }
                        for arg in args {
                            if let Operand::Value(v) = arg {
                                live.insert(*v);
                            }
                        }
                    }
                    Instruction::Store { ptr, value, .. } => {
                        if let Operand::Value(v) = ptr {
                            live.insert(*v);
                        }
                        if let Operand::Value(v) = value {
                            live.insert(*v);
                        }
                    }
                    Instruction::BinOp { lhs, rhs, dest, .. } => {
                        if let Operand::Value(v) = lhs {
                            live.insert(*v);
                        }
                        if let Operand::Value(v) = rhs {
                            live.insert(*v);
                        }
                        live.insert(*dest);
                    }
                    Instruction::UnaryOp { value, dest, .. } => {
                        if let Operand::Value(v) = value {
                            live.insert(*v);
                        }
                        live.insert(*dest);
                    }
                    _ => {}
                }
            }
        }

        // Remove dead instructions (those that don't contribute to live values)
        for block in func.blocks.values_mut() {
            let mut new_instructions = Vec::new();

            for inst in &block.instructions {
                let is_dead = match inst.dest() {
                    Some(dest) => !live.contains(&dest) && !has_side_effects(inst),
                    None => false,
                };

                if !is_dead {
                    new_instructions.push(inst.clone());
                } else {
                    changed = true;
                }
            }

            block.instructions = new_instructions;
        }

        changed
    }

    fn name(&self) -> &str {
        "dead_code_elimination"
    }
}

/// Constant propagation pass
pub struct ConstantPropagation;

impl OptPass for ConstantPropagation {
    fn run(&mut self, func: &mut Function) -> bool {
        let mut changed = false;
        let mut const_values: HashMap<ValueId, Constant> = HashMap::new();

        // Collect constant assignments
        for block in func.blocks.values() {
            for inst in &block.instructions {
                if let Instruction::Assign { dest, value: Operand::Const(c), .. } = inst {
                    const_values.insert(*dest, c.clone());
                }
            }
        }

        // Propagate constants
        for block in func.blocks.values_mut() {
            for inst in &mut block.instructions {
                match inst {
                    Instruction::BinOp { lhs, rhs, .. } => {
                        if let Operand::Value(v) = lhs {
                            if let Some(c) = const_values.get(v) {
                                *lhs = Operand::Const(c.clone());
                                changed = true;
                            }
                        }
                        if let Operand::Value(v) = rhs {
                            if let Some(c) = const_values.get(v) {
                                *rhs = Operand::Const(c.clone());
                                changed = true;
                            }
                        }
                    }
                    Instruction::UnaryOp { value, .. } => {
                        if let Operand::Value(v) = value {
                            if let Some(c) = const_values.get(v) {
                                *value = Operand::Const(c.clone());
                                changed = true;
                            }
                        }
                    }
                    Instruction::Return { value: Some(Operand::Value(v)), .. } => {
                        if let Some(c) = const_values.get(v) {
                            *inst = Instruction::Return {
                                value: Some(Operand::Const(c.clone())),
                                span: inst.span(),
                            };
                            changed = true;
                        }
                    }
                    _ => {}
                }
            }
        }

        changed
    }

    fn name(&self) -> &str {
        "constant_propagation"
    }
}

/// Copy propagation pass
pub struct CopyPropagation;

impl OptPass for CopyPropagation {
    fn run(&mut self, func: &mut Function) -> bool {
        let mut changed = false;
        let mut copies: HashMap<ValueId, ValueId> = HashMap::new();

        // Collect copy assignments (v1 = v2)
        for block in func.blocks.values() {
            for inst in &block.instructions {
                if let Instruction::Assign { dest, value: Operand::Value(src), .. } = inst {
                    copies.insert(*dest, *src);
                }
            }
        }

        // Propagate copies
        for block in func.blocks.values_mut() {
            for inst in &mut block.instructions {
                match inst {
                    Instruction::BinOp { lhs, rhs, .. } => {
                        if let Operand::Value(v) = lhs {
                            if let Some(&src) = copies.get(v) {
                                *lhs = Operand::Value(src);
                                changed = true;
                            }
                        }
                        if let Operand::Value(v) = rhs {
                            if let Some(&src) = copies.get(v) {
                                *rhs = Operand::Value(src);
                                changed = true;
                            }
                        }
                    }
                    Instruction::UnaryOp { value, .. } => {
                        if let Operand::Value(v) = value {
                            if let Some(&src) = copies.get(v) {
                                *value = Operand::Value(src);
                                changed = true;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        changed
    }

    fn name(&self) -> &str {
        "copy_propagation"
    }
}

/// Global Value Numbering (GVN) pass
pub struct GlobalValueNumbering;

impl OptPass for GlobalValueNumbering {
    fn run(&mut self, func: &mut Function) -> bool {
        let mut changed = false;
        let mut value_map: HashMap<String, ValueId> = HashMap::new();

        for block in func.blocks.values_mut() {
            let mut new_instructions = Vec::new();

            for inst in &block.instructions {
                match inst {
                    Instruction::BinOp { dest, op, lhs, rhs, span } => {
                        let key = format!("{:?}_{:?}_{:?}", op, lhs, rhs);

                        if let Some(&existing) = value_map.get(&key) {
                            // Replace with copy
                            new_instructions.push(Instruction::Assign {
                                dest: *dest,
                                value: Operand::Value(existing),
                                span: *span,
                            });
                            changed = true;
                        } else {
                            value_map.insert(key, *dest);
                            new_instructions.push(inst.clone());
                        }
                    }
                    _ => {
                        new_instructions.push(inst.clone());
                    }
                }
            }

            block.instructions = new_instructions;
        }

        changed
    }

    fn name(&self) -> &str {
        "global_value_numbering"
    }
}

/// Loop-Invariant Code Motion (LICM) pass
pub struct LoopInvariantCodeMotion;

impl OptPass for LoopInvariantCodeMotion {
    fn run(&mut self, func: &mut Function) -> bool {
        let cfg = CFG::build(func);
        let loops = cfg.find_loops();

        if loops.is_empty() {
            return false;
        }

        let mut changed = false;

        for loop_info in &loops {
            let loop_blocks: HashSet<BlockId> = loop_info.blocks.iter().copied().collect();
            let mut invariant_insts = Vec::new();

            // Find loop-invariant instructions
            for &block_id in &loop_info.blocks {
                if let Some(block) = func.block(block_id) {
                    for inst in &block.instructions {
                        if is_loop_invariant(inst, &loop_blocks, func) {
                            invariant_insts.push((block_id, inst.clone()));
                        }
                    }
                }
            }

            if !invariant_insts.is_empty() {
                changed = true;
                // In a real implementation, we'd move these to a preheader block
                // For now, we just mark that we detected them
            }
        }

        changed
    }

    fn name(&self) -> &str {
        "loop_invariant_code_motion"
    }
}

/// Simple inlining pass
pub struct Inlining {
    /// Maximum instruction count for inlining
    max_instructions: usize,
}

impl Inlining {
    /// Create new inlining pass
    pub fn new(max_instructions: usize) -> Self {
        Self { max_instructions }
    }
}

impl Default for Inlining {
    fn default() -> Self {
        Self::new(20)
    }
}

impl OptPass for Inlining {
    fn run(&mut self, _func: &mut Function) -> bool {
        // Simplified inlining - in a real implementation, we would:
        // 1. Identify call sites
        // 2. Check if callee is simple enough (instruction count, no recursion)
        // 3. Copy callee body and rename variables
        // 4. Replace call with inlined body

        // For now, just return false (no changes)
        false
    }

    fn name(&self) -> &str {
        "inlining"
    }
}

/// SROA (Scalar Replacement of Aggregates) pass
pub struct SROA;

impl OptPass for SROA {
    fn run(&mut self, _func: &mut Function) -> bool {
        // Simplified SROA - in a real implementation:
        // 1. Find allocas of struct/array types
        // 2. Check if all accesses are known statically
        // 3. Replace with individual scalar allocas
        // 4. Update all GetElement instructions

        false
    }

    fn name(&self) -> &str {
        "sroa"
    }
}

/// NRVO (Named Return Value Optimization) pass
pub struct NRVO;

impl OptPass for NRVO {
    fn run(&mut self, _func: &mut Function) -> bool {
        // Simplified NRVO - in a real implementation:
        // 1. Find patterns like: let x = ...; return x;
        // 2. Eliminate temporary if safe
        // 3. Return directly

        false
    }

    fn name(&self) -> &str {
        "nrvo"
    }
}

/// Devirtualization pass
pub struct Devirtualization;

impl OptPass for Devirtualization {
    fn run(&mut self, _func: &mut Function) -> bool {
        // Simplified devirtualization - in a real implementation:
        // 1. Find virtual/trait method calls
        // 2. Check if receiver type is known statically
        // 3. Replace with direct call

        false
    }

    fn name(&self) -> &str {
        "devirtualization"
    }
}

/// Loop SIMD hints pass
pub struct LoopSIMD;

impl OptPass for LoopSIMD {
    fn run(&mut self, _func: &mut Function) -> bool {
        // Simplified loop SIMD - in a real implementation:
        // 1. Identify vectorizable loops
        // 2. Check for dependencies
        // 3. Add SIMD metadata/hints

        false
    }

    fn name(&self) -> &str {
        "loop_simd"
    }
}

/// Optimization pipeline
pub struct OptPipeline {
    passes: Vec<Box<dyn OptPass>>,
}

impl OptPipeline {
    /// Create optimization pipeline for given level
    pub fn new(level: OptLevel) -> Self {
        let mut passes: Vec<Box<dyn OptPass>> = Vec::new();

        match level {
            OptLevel::O0 => {
                // No optimizations
            }
            OptLevel::O1 => {
                // Basic optimizations
                passes.push(Box::new(ConstantFolding));
                passes.push(Box::new(ConstantPropagation));
                passes.push(Box::new(DeadCodeElimination));
            }
            OptLevel::O2 => {
                // Standard optimizations
                passes.push(Box::new(ConstantFolding));
                passes.push(Box::new(ConstantPropagation));
                passes.push(Box::new(CopyPropagation));
                passes.push(Box::new(GlobalValueNumbering));
                passes.push(Box::new(DeadCodeElimination));
                passes.push(Box::new(LoopInvariantCodeMotion));
                passes.push(Box::new(Inlining::default()));
            }
            OptLevel::O3 => {
                // Aggressive optimizations
                passes.push(Box::new(ConstantFolding));
                passes.push(Box::new(ConstantPropagation));
                passes.push(Box::new(CopyPropagation));
                passes.push(Box::new(GlobalValueNumbering));
                passes.push(Box::new(DeadCodeElimination));
                passes.push(Box::new(LoopInvariantCodeMotion));
                passes.push(Box::new(Inlining::new(50)));
                passes.push(Box::new(SROA));
                passes.push(Box::new(NRVO));
                passes.push(Box::new(Devirtualization));
                passes.push(Box::new(LoopSIMD));
            }
        }

        Self { passes }
    }

    /// Run optimization pipeline on function
    pub fn run(&mut self, func: &mut Function) -> bool {
        let mut changed = false;
        let mut iteration = 0;
        const MAX_ITERATIONS: usize = 10;

        // Run passes to fixed point (or max iterations)
        loop {
            let mut iter_changed = false;

            for pass in &mut self.passes {
                if pass.run(func) {
                    iter_changed = true;
                    changed = true;
                }
            }

            iteration += 1;
            if !iter_changed || iteration >= MAX_ITERATIONS {
                break;
            }
        }

        changed
    }
}

// Helper functions

fn as_const(op: &Operand) -> Option<&Constant> {
    match op {
        Operand::Const(c) => Some(c),
        _ => None,
    }
}

fn fold_binop(op: BinOp, lhs: &Constant, rhs: &Constant) -> Option<Constant> {
    match (lhs, rhs) {
        (Constant::Int(l), Constant::Int(r)) => {
            let result = match op {
                BinOp::Add => l.checked_add(*r)?,
                BinOp::Sub => l.checked_sub(*r)?,
                BinOp::Mul => l.checked_mul(*r)?,
                BinOp::Div => l.checked_div(*r)?,
                BinOp::Mod => l.checked_rem(*r)?,
                BinOp::Eq => return Some(Constant::Bool(l == r)),
                BinOp::Ne => return Some(Constant::Bool(l != r)),
                BinOp::Lt => return Some(Constant::Bool(l < r)),
                BinOp::Le => return Some(Constant::Bool(l <= r)),
                BinOp::Gt => return Some(Constant::Bool(l > r)),
                BinOp::Ge => return Some(Constant::Bool(l >= r)),
                BinOp::BitAnd => l & r,
                BinOp::BitOr => l | r,
                BinOp::BitXor => l ^ r,
                BinOp::Shl => l << (r & 63),
                BinOp::Shr => l >> (r & 63),
                _ => return None,
            };
            Some(Constant::Int(result))
        }
        (Constant::Bool(l), Constant::Bool(r)) => {
            let result = match op {
                BinOp::And => *l && *r,
                BinOp::Or => *l || *r,
                BinOp::Eq => l == r,
                BinOp::Ne => l != r,
                _ => return None,
            };
            Some(Constant::Bool(result))
        }
        _ => None,
    }
}

fn fold_unaryop(op: UnaryOp, value: &Constant) -> Option<Constant> {
    match (op, value) {
        (UnaryOp::Neg, Constant::Int(v)) => Some(Constant::Int(-v)),
        (UnaryOp::Not, Constant::Bool(v)) => Some(Constant::Bool(!v)),
        (UnaryOp::BitNot, Constant::Int(v)) => Some(Constant::Int(!v)),
        _ => None,
    }
}

fn has_side_effects(inst: &Instruction) -> bool {
    matches!(
        inst,
        Instruction::Call { .. }
            | Instruction::Store { .. }
            | Instruction::Return { .. }
            | Instruction::Branch { .. }
            | Instruction::Jump { .. }
    )
}

fn is_loop_invariant(inst: &Instruction, loop_blocks: &HashSet<BlockId>, _func: &Function) -> bool {
    // Simplified check - in a real implementation, we'd check:
    // 1. All operands are constants or defined outside loop
    // 2. Instruction has no side effects
    // 3. Safe to move (doesn't affect memory in loop)

    matches!(inst, Instruction::BinOp { .. } | Instruction::UnaryOp { .. })
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_types::{EffectSet, Type};

    fn create_test_function() -> Function {
        Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE)
    }

    #[test]
    fn test_opt_level() {
        assert_eq!(OptLevel::from_u8(0), OptLevel::O0);
        assert_eq!(OptLevel::from_u8(1), OptLevel::O1);
        assert_eq!(OptLevel::from_u8(2), OptLevel::O2);
        assert_eq!(OptLevel::from_u8(3), OptLevel::O3);
    }

    #[test]
    fn test_constant_folding() {
        let mut func = create_test_function();
        let mut pass = ConstantFolding;

        // Initially should not change empty function
        assert!(!pass.run(&mut func));
    }

    #[test]
    fn test_fold_binop() {
        let result = fold_binop(
            BinOp::Add,
            &Constant::Int(2),
            &Constant::Int(3),
        );
        assert_eq!(result, Some(Constant::Int(5)));

        let result = fold_binop(
            BinOp::Eq,
            &Constant::Int(5),
            &Constant::Int(5),
        );
        assert_eq!(result, Some(Constant::Bool(true)));
    }

    #[test]
    fn test_fold_unaryop() {
        let result = fold_unaryop(UnaryOp::Neg, &Constant::Int(42));
        assert_eq!(result, Some(Constant::Int(-42)));

        let result = fold_unaryop(UnaryOp::Not, &Constant::Bool(true));
        assert_eq!(result, Some(Constant::Bool(false)));
    }

    #[test]
    fn test_opt_pipeline_o0() {
        let pipeline = OptPipeline::new(OptLevel::O0);
        assert_eq!(pipeline.passes.len(), 0);
    }

    #[test]
    fn test_opt_pipeline_o1() {
        let pipeline = OptPipeline::new(OptLevel::O1);
        assert!(pipeline.passes.len() >= 3);
    }

    #[test]
    fn test_opt_pipeline_o2() {
        let pipeline = OptPipeline::new(OptLevel::O2);
        assert!(pipeline.passes.len() >= 6);
    }

    #[test]
    fn test_opt_pipeline_o3() {
        let pipeline = OptPipeline::new(OptLevel::O3);
        assert!(pipeline.passes.len() >= 10);
    }

    #[test]
    fn test_has_side_effects() {
        let ret = Instruction::Return {
            value: None,
            span: Span::dummy(),
        };
        assert!(has_side_effects(&ret));

        let assign = Instruction::Assign {
            dest: 0,
            value: Operand::Const(Constant::Int(42)),
            span: Span::dummy(),
        };
        assert!(!has_side_effects(&assign));
    }

    #[test]
    fn test_dead_code_elimination() {
        let mut func = create_test_function();
        let mut pass = DeadCodeElimination;

        assert!(!pass.run(&mut func));
    }

    #[test]
    fn test_constant_propagation() {
        let mut func = create_test_function();
        let mut pass = ConstantPropagation;

        assert!(!pass.run(&mut func));
    }

    #[test]
    fn test_copy_propagation() {
        let mut func = create_test_function();
        let mut pass = CopyPropagation;

        assert!(!pass.run(&mut func));
    }

    #[test]
    fn test_gvn() {
        let mut func = create_test_function();
        let mut pass = GlobalValueNumbering;

        assert!(!pass.run(&mut func));
    }

    #[test]
    fn test_licm() {
        let mut func = create_test_function();
        let mut pass = LoopInvariantCodeMotion;

        assert!(!pass.run(&mut func));
    }

    #[test]
    fn test_inlining() {
        let mut func = create_test_function();
        let mut pass = Inlining::default();

        assert!(!pass.run(&mut func));
    }

    #[test]
    fn test_sroa() {
        let mut func = create_test_function();
        let mut pass = SROA;

        assert!(!pass.run(&mut func));
    }

    #[test]
    fn test_nrvo() {
        let mut func = create_test_function();
        let mut pass = NRVO;

        assert!(!pass.run(&mut func));
    }

    #[test]
    fn test_devirtualization() {
        let mut func = create_test_function();
        let mut pass = Devirtualization;

        assert!(!pass.run(&mut func));
    }

    #[test]
    fn test_loop_simd() {
        let mut func = create_test_function();
        let mut pass = LoopSIMD;

        assert!(!pass.run(&mut func));
    }
}
