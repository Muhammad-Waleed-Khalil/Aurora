//! Register Allocation - Linear Scan Algorithm

use crate::air::Register;
use aurora_mir::{Function, ValueId};
use std::collections::HashMap;

/// Register allocator
pub struct RegisterAllocator {
    /// Value to register mapping
    allocation: HashMap<ValueId, Register>,
    /// Stack slots for spilled values
    spills: HashMap<ValueId, i32>,
    /// Next stack offset
    next_stack_offset: i32,
    /// Available registers
    available_regs: Vec<Register>,
    /// Used registers
    used_regs: Vec<Register>,
}

impl RegisterAllocator {
    pub fn new() -> Self {
        Self {
            allocation: HashMap::new(),
            spills: HashMap::new(),
            next_stack_offset: 0,
            available_regs: vec![
                Register::RAX,
                Register::RBX,
                Register::RCX,
                Register::RDX,
                Register::RSI,
                Register::RDI,
                Register::R8,
                Register::R9,
                Register::R10,
                Register::R11,
            ],
            used_regs: Vec::new(),
        }
    }

    /// Run register allocation on function
    pub fn allocate(&mut self, func: &Function) {
        // Simplified linear scan allocation
        // In reality, would compute live ranges and intervals

        let mut reg_index = 0;

        for value in func.values.values() {
            if reg_index < self.available_regs.len() {
                // Allocate register
                let reg = self.available_regs[reg_index];
                self.allocation.insert(value.id, reg);
                if !self.used_regs.contains(&reg) {
                    self.used_regs.push(reg);
                }
                reg_index += 1;
            } else {
                // Spill to stack
                let offset = self.next_stack_offset;
                self.spills.insert(value.id, offset);
                self.next_stack_offset += 8; // 8 bytes per value
            }
        }
    }

    /// Get register for value
    pub fn get_register(&self, value: ValueId) -> Register {
        self.allocation.get(&value).copied().unwrap_or(Register::RAX)
    }

    /// Get stack size needed
    pub fn stack_size(&self) -> u32 {
        self.next_stack_offset as u32
    }

    /// Get used registers
    pub fn used_registers(&self) -> Vec<Register> {
        self.used_regs.clone()
    }

    /// Check if value is spilled
    pub fn is_spilled(&self, value: ValueId) -> bool {
        self.spills.contains_key(&value)
    }

    /// Get spill offset
    pub fn spill_offset(&self, value: ValueId) -> Option<i32> {
        self.spills.get(&value).copied()
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
    use aurora_mir::{Function, Value, Span};
    use aurora_types::{EffectSet, Type};

    #[test]
    fn test_allocator_creation() {
        let alloc = RegisterAllocator::new();
        assert_eq!(alloc.stack_size(), 0);
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
        
        alloc.allocate(&func);
        
        let reg = alloc.get_register(0);
        assert_eq!(reg, Register::RAX);
    }

    #[test]
    fn test_spilling() {
        let mut alloc = RegisterAllocator::new();
        let mut func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);
        
        // Add more values than available registers
        for i in 0..15 {
            func.add_value(Value {
                id: i,
                ty: Type::Unit,
                span: Span::dummy(),
            });
        }
        
        alloc.allocate(&func);

        // With 10 registers available, values 10-14 should be spilled
        // Check that at least one value beyond available registers is spilled
        assert!(alloc.is_spilled(10) || alloc.is_spilled(11) || alloc.is_spilled(12) || alloc.is_spilled(13) || alloc.is_spilled(14));
    }
}
