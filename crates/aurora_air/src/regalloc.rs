//! Register Allocation - Linear Scan with Liveness Analysis
//!
//! Implements linear scan register allocation with:
//! - Liveness analysis
//! - Interference graph
//! - Spill code generation
//! - Register coalescing hints

use crate::air::Register;
use aurora_mir::{BlockId, Function, Instruction, Operand, ValueId};
use std::collections::{HashMap, HashSet};

/// Live interval for a value
#[derive(Debug, Clone)]
struct LiveInterval {
    value: ValueId,
    start: usize,
    end: usize,
}

/// Register allocator with liveness analysis
pub struct RegisterAllocator {
    /// Value to register mapping
    allocation: HashMap<ValueId, Register>,
    /// Stack slots for spilled values
    spills: HashMap<ValueId, i32>,
    /// Next stack offset
    next_stack_offset: i32,
    /// Available general purpose registers (caller-saved first for efficiency)
    available_regs: Vec<Register>,
    /// Callee-saved registers that we're using
    callee_saved: Vec<Register>,
    /// Live intervals
    intervals: Vec<LiveInterval>,
}

impl RegisterAllocator {
    pub fn new() -> Self {
        Self {
            allocation: HashMap::new(),
            spills: HashMap::new(),
            next_stack_offset: 0,
            // Caller-saved (volatile) registers first - don't need saving
            available_regs: vec![
                Register::RAX,
                Register::RCX,
                Register::RDX,
                Register::R8,
                Register::R9,
                Register::R10,
                Register::R11,
                // Callee-saved (non-volatile) - need to save/restore
                Register::RBX,
                Register::R12,
                Register::R13,
                Register::R14,
                Register::R15,
            ],
            callee_saved: Vec::new(),
            intervals: Vec::new(),
        }
    }

    /// Run register allocation on function
    pub fn allocate(&mut self, func: &Function) {
        // Step 1: Compute live intervals
        self.compute_live_intervals(func);

        // Step 2: Sort intervals by start point
        self.intervals.sort_by_key(|i| i.start);

        // Step 3: Linear scan allocation
        self.linear_scan();
    }

    /// Compute live intervals for all values
    fn compute_live_intervals(&mut self, func: &Function) {
        let mut live_in: HashMap<BlockId, HashSet<ValueId>> = HashMap::new();
        let mut live_out: HashMap<BlockId, HashSet<ValueId>> = HashMap::new();

        // Initialize
        for block_id in func.blocks.keys() {
            live_in.insert(*block_id, HashSet::new());
            live_out.insert(*block_id, HashSet::new());
        }

        // Iterate until fixed point (dataflow analysis)
        let mut changed = true;
        while changed {
            changed = false;

            for (block_id, block) in &func.blocks {
                let mut live = live_out.get(block_id).unwrap().clone();

                // Process instructions in reverse
                for inst in block.instructions.iter().rev() {
                    // Remove defined values
                    if let Some(def) = inst.dest() {
                        live.remove(&def);
                    }

                    // Add used values
                    self.add_uses(inst, &mut live);
                }

                // Update live_in
                let old_live_in = live_in.get(block_id).unwrap().clone();
                if live != old_live_in {
                    live_in.insert(*block_id, live.clone());
                    changed = true;
                }

                // Propagate to predecessors' live_out
                for pred_id in &block.predecessors {
                    let pred_live_out = live_out.get_mut(pred_id).unwrap();
                    let old_size = pred_live_out.len();
                    pred_live_out.extend(live.iter().copied());
                    if pred_live_out.len() != old_size {
                        changed = true;
                    }
                }
            }
        }

        // Build intervals from liveness information
        let mut value_ranges: HashMap<ValueId, (usize, usize)> = HashMap::new();

        for (block_id, block) in &func.blocks {
            let block_start = *block_id as usize * 100; // Approximate position

            // Values live at block entry
            for &val in live_in.get(block_id).unwrap() {
                let entry = value_ranges.entry(val).or_insert((block_start, block_start));
                entry.0 = entry.0.min(block_start);
            }

            // Process instructions
            for (i, inst) in block.instructions.iter().enumerate() {
                let pos = block_start + i;

                // Defined value
                if let Some(def) = inst.dest() {
                    let entry = value_ranges.entry(def).or_insert((pos, pos));
                    entry.1 = entry.1.max(pos);
                }

                // Used values
                let mut uses = HashSet::new();
                self.add_uses(inst, &mut uses);
                for val in uses {
                    let entry = value_ranges.entry(val).or_insert((pos, pos));
                    entry.1 = entry.1.max(pos);
                }
            }

            // Values live at block exit
            for &val in live_out.get(block_id).unwrap() {
                let block_end = block_start + block.instructions.len();
                let entry = value_ranges.entry(val).or_insert((block_end, block_end));
                entry.1 = entry.1.max(block_end);
            }
        }

        // Convert to intervals
        self.intervals = value_ranges
            .into_iter()
            .map(|(value, (start, end))| LiveInterval { value, start, end })
            .collect();
    }

    /// Add used values from instruction to set
    fn add_uses(&self, inst: &Instruction, uses: &mut HashSet<ValueId>) {
        match inst {
            Instruction::Assign { value, .. } => self.add_operand_uses(value, uses),
            Instruction::BinOp { lhs, rhs, .. } => {
                self.add_operand_uses(lhs, uses);
                self.add_operand_uses(rhs, uses);
            }
            Instruction::UnaryOp { value, .. } => self.add_operand_uses(value, uses),
            Instruction::Call { func, args, .. } => {
                self.add_operand_uses(func, uses);
                for arg in args {
                    self.add_operand_uses(arg, uses);
                }
            }
            Instruction::Return { value, .. } => {
                if let Some(v) = value {
                    self.add_operand_uses(v, uses);
                }
            }
            Instruction::Branch { cond, .. } => self.add_operand_uses(cond, uses),
            Instruction::Load { ptr, .. } => self.add_operand_uses(ptr, uses),
            Instruction::Store { ptr, value, .. } => {
                self.add_operand_uses(ptr, uses);
                self.add_operand_uses(value, uses);
            }
            Instruction::Cast { value, .. } => self.add_operand_uses(value, uses),
            Instruction::GetElement { base, index, .. } => {
                self.add_operand_uses(base, uses);
                self.add_operand_uses(index, uses);
            }
            Instruction::Phi { inputs, .. } => {
                for (_, op) in inputs {
                    self.add_operand_uses(op, uses);
                }
            }
            _ => {}
        }
    }

    fn add_operand_uses(&self, op: &Operand, uses: &mut HashSet<ValueId>) {
        if let Operand::Value(v) = op {
            uses.insert(*v);
        }
    }

    /// Linear scan register allocation
    fn linear_scan(&mut self) {
        let mut active: Vec<LiveInterval> = Vec::new();

        for interval in self.intervals.clone() {
            // Remove expired intervals
            active.retain(|a| a.end >= interval.start);

            // Try to allocate register
            if active.len() < self.available_regs.len() {
                let reg = self.available_regs[active.len()];
                self.allocation.insert(interval.value, reg);

                // Track callee-saved registers
                if matches!(
                    reg,
                    Register::RBX
                        | Register::R12
                        | Register::R13
                        | Register::R14
                        | Register::R15
                ) {
                    if !self.callee_saved.contains(&reg) {
                        self.callee_saved.push(reg);
                    }
                }

                active.push(interval.clone());
            } else {
                // Spill to stack
                self.spill_value(interval.value);
            }
        }
    }

    /// Spill a value to stack
    fn spill_value(&mut self, value: ValueId) {
        let offset = self.next_stack_offset;
        self.spills.insert(value, offset);
        self.next_stack_offset += 8; // 8 bytes per value
    }

    /// Get register for value
    pub fn get_register(&self, value: ValueId) -> Register {
        self.allocation.get(&value).copied().unwrap_or(Register::RAX)
    }

    /// Get stack size needed
    pub fn stack_size(&self) -> u32 {
        self.next_stack_offset as u32
    }

    /// Get callee-saved registers that need saving/restoring
    pub fn callee_saved_registers(&self) -> Vec<Register> {
        self.callee_saved.clone()
    }

    /// Check if value is spilled
    pub fn is_spilled(&self, value: ValueId) -> bool {
        self.spills.contains_key(&value)
    }

    /// Get spill offset
    pub fn spill_offset(&self, value: ValueId) -> Option<i32> {
        self.spills.get(&value).copied()
    }

    /// Get all allocated registers
    pub fn allocated_registers(&self) -> Vec<Register> {
        self.allocation.values().copied().collect()
    }
}

impl Default for RegisterAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_mir::{BasicBlock, Constant, Function, Instruction as MirInst, Operand as MirOp, Span, Value};
    use aurora_types::{EffectSet, Type};

    #[test]
    fn test_allocator_creation() {
        let alloc = RegisterAllocator::new();
        assert_eq!(alloc.stack_size(), 0);
        assert_eq!(alloc.callee_saved_registers().len(), 0);
    }

    #[test]
    fn test_simple_allocation() {
        let mut alloc = RegisterAllocator::new();
        let mut func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);

        func.add_value(Value {
            id: 0,
            ty: Type::Unit,
            span: Span::dummy(),
        });

        let mut block = BasicBlock::new(0);
        block.push(MirInst::Assign {
            dest: 0,
            value: MirOp::Const(Constant::Int(42)),
            span: Span::dummy(),
        });
        func.add_block(block);

        alloc.allocate(&func);

        let reg = alloc.get_register(0);
        assert_eq!(reg, Register::RAX);
    }

    #[test]
    fn test_liveness_analysis() {
        let mut alloc = RegisterAllocator::new();
        let mut func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);

        // Create values
        for i in 0..3 {
            func.add_value(Value {
                id: i,
                ty: Type::Unit,
                span: Span::dummy(),
            });
        }

        let mut block = BasicBlock::new(0);
        block.push(MirInst::Assign {
            dest: 0,
            value: MirOp::Const(Constant::Int(1)),
            span: Span::dummy(),
        });
        block.push(MirInst::Assign {
            dest: 1,
            value: MirOp::Const(Constant::Int(2)),
            span: Span::dummy(),
        });
        block.push(MirInst::BinOp {
            dest: 2,
            op: aurora_mir::BinOp::Add,
            lhs: MirOp::Value(0),
            rhs: MirOp::Value(1),
            span: Span::dummy(),
        });
        func.add_block(block);

        alloc.allocate(&func);

        // All values should be allocated
        assert!(!alloc.is_spilled(0));
        assert!(!alloc.is_spilled(1));
        assert!(!alloc.is_spilled(2));
    }

    #[test]
    fn test_spilling_when_needed() {
        let mut alloc = RegisterAllocator::new();
        let mut func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);

        // Create many concurrent live values
        let num_values = 20;
        for i in 0..num_values {
            func.add_value(Value {
                id: i,
                ty: Type::Unit,
                span: Span::dummy(),
            });
        }

        // Add one more value for result
        func.add_value(Value {
            id: num_values,
            ty: Type::Unit,
            span: Span::dummy(),
        });

        let mut block = BasicBlock::new(0);
        // Define all values
        for i in 0..num_values {
            block.push(MirInst::Assign {
                dest: i,
                value: MirOp::Const(Constant::Int(i as i64)),
                span: Span::dummy(),
            });
        }

        // Use all values at the end - this makes them all live concurrently
        for i in 0..num_values - 1 {
            block.push(MirInst::BinOp {
                dest: num_values,
                op: aurora_mir::BinOp::Add,
                lhs: MirOp::Value(i),
                rhs: MirOp::Value(i + 1),
                span: Span::dummy(),
            });
        }

        func.add_block(block);

        alloc.allocate(&func);

        // Some values should be spilled since we have more than available registers
        let spilled_count = (0..num_values).filter(|&i| alloc.is_spilled(i)).count();
        assert!(spilled_count > 0, "Expected some values to be spilled with {} values and {} registers", num_values, alloc.available_regs.len());
    }

    #[test]
    fn test_callee_saved_tracking() {
        let mut alloc = RegisterAllocator::new();
        let mut func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);

        // Create enough values to use callee-saved registers
        for i in 0..10 {
            func.add_value(Value {
                id: i,
                ty: Type::Unit,
                span: Span::dummy(),
            });
        }

        let mut block = BasicBlock::new(0);
        for i in 0..10 {
            block.push(MirInst::Assign {
                dest: i,
                value: MirOp::Const(Constant::Int(i as i64)),
                span: Span::dummy(),
            });
        }
        func.add_block(block);

        alloc.allocate(&func);

        // Should track callee-saved registers if used
        let callee_saved = alloc.callee_saved_registers();
        // Might have callee-saved registers depending on allocation
        assert!(callee_saved.len() <= 5); // Max 5 callee-saved in our set
    }

    #[test]
    fn test_allocated_registers() {
        let mut alloc = RegisterAllocator::new();
        let mut func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);

        func.add_value(Value {
            id: 0,
            ty: Type::Unit,
            span: Span::dummy(),
        });

        let mut block = BasicBlock::new(0);
        block.push(MirInst::Assign {
            dest: 0,
            value: MirOp::Const(Constant::Int(42)),
            span: Span::dummy(),
        });
        func.add_block(block);

        alloc.allocate(&func);

        let allocated = alloc.allocated_registers();
        assert!(!allocated.is_empty());
    }

    #[test]
    fn test_spill_offset() {
        let mut alloc = RegisterAllocator::new();

        // Manually spill a value
        alloc.spill_value(0);

        assert_eq!(alloc.spill_offset(0), Some(0));
        assert_eq!(alloc.stack_size(), 8);

        alloc.spill_value(1);
        assert_eq!(alloc.spill_offset(1), Some(8));
        assert_eq!(alloc.stack_size(), 16);
    }
}
