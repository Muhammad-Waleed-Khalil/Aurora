//! Dead Code Elimination

use crate::mir::*;
use std::collections::HashSet;

pub struct DCE {
    eliminated_count: usize,
}

impl DCE {
    pub fn new() -> Self {
        Self { eliminated_count: 0 }
    }

    pub fn run(&mut self, func: &mut Function) -> bool {
        let mut live = HashSet::new();
        let mut changed = false;

        // Mark live values (simplified)
        for block in func.blocks.values() {
            for inst in &block.instructions {
                if self.has_side_effects(inst) {
                    if let Some(dest) = inst.dest() {
                        live.insert(dest);
                    }
                }
            }
        }

        // Remove dead instructions (simplified)
        for block in func.blocks.values_mut() {
            let original_len = block.instructions.len();
            block.instructions.retain(|inst| {
                inst.dest().map_or(true, |d| live.contains(&d)) || self.has_side_effects(inst)
            });
            if block.instructions.len() < original_len {
                changed = true;
                self.eliminated_count += original_len - block.instructions.len();
            }
        }

        changed
    }

    fn has_side_effects(&self, inst: &Instruction) -> bool {
        matches!(
            inst,
            Instruction::Call { .. }
                | Instruction::Store { .. }
                | Instruction::Return { .. }
                | Instruction::Branch { .. }
                | Instruction::Jump { .. }
        )
    }

    pub fn eliminated_count(&self) -> usize {
        self.eliminated_count
    }
}

impl Default for DCE {
    fn default() -> Self {
        Self::new()
    }
}
