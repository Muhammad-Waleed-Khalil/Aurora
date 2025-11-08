//! Peephole Optimizations for AIR

use crate::air::{AirFunction, Instruction, Operand, Register};

/// Peephole optimizer
pub struct PeepholeOptimizer {
    optimizations_applied: usize,
}

impl PeepholeOptimizer {
    pub fn new() -> Self {
        Self {
            optimizations_applied: 0,
        }
    }

    /// Run peephole optimizations
    pub fn optimize(&mut self, func: &mut AirFunction) {
        self.mov_collapse(func);
        self.lea_patterns(func);
        self.remove_nops(func);
    }

    /// Remove redundant mov instructions (mov rax, rax)
    fn mov_collapse(&mut self, func: &mut AirFunction) {
        let mut to_remove = Vec::new();

        for (i, inst) in func.instructions.iter().enumerate() {
            if let Instruction::Mov { dest, src } = inst {
                if dest == src {
                    to_remove.push(i);
                    self.optimizations_applied += 1;
                }
            }
        }

        // Remove in reverse to maintain indices
        for &i in to_remove.iter().rev() {
            func.instructions.remove(i);
        }
    }

    /// Use LEA for address arithmetic
    fn lea_patterns(&mut self, func: &mut AirFunction) {
        let mut replacements = Vec::new();

        for (i, window) in func.instructions.windows(2).enumerate() {
            // Pattern: mov reg, base; add reg, offset
            // Replace with: lea reg, [base + offset]
            if let [Instruction::Mov { dest: dest1, src }, Instruction::Add { dest: dest2, src: Operand::Imm(offset) }] =
                window
            {
                if dest1 == dest2 {
                    if let (Operand::Reg(dest_reg), Operand::Reg(src_reg)) = (dest1, src) {
                        replacements.push((
                            i,
                            Instruction::Lea {
                                dest: Operand::Reg(*dest_reg),
                                src: Operand::Mem {
                                    base: *src_reg,
                                    offset: *offset as i32,
                                },
                            },
                        ));
                        self.optimizations_applied += 1;
                    }
                }
            }
        }

        // Apply replacements
        for (i, replacement) in replacements.iter().rev() {
            func.instructions[*i] = replacement.clone();
            func.instructions.remove(i + 1);
        }
    }

    /// Remove nop instructions
    fn remove_nops(&mut self, func: &mut AirFunction) {
        let original_len = func.instructions.len();
        func.instructions.retain(|inst| !matches!(inst, Instruction::Nop));
        self.optimizations_applied += original_len - func.instructions.len();
    }

    /// Get count of optimizations applied
    pub fn optimizations_count(&self) -> usize {
        self.optimizations_applied
    }
}

impl Default for PeepholeOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimizer_creation() {
        let opt = PeepholeOptimizer::new();
        assert_eq!(opt.optimizations_count(), 0);
    }

    #[test]
    fn test_mov_collapse() {
        let mut opt = PeepholeOptimizer::new();
        let mut func = AirFunction::new("test".to_string());
        
        func.push(Instruction::Mov {
            dest: Operand::Reg(Register::RAX),
            src: Operand::Reg(Register::RAX),
        });
        
        opt.optimize(&mut func);
        assert!(opt.optimizations_count() > 0);
    }

    #[test]
    fn test_nop_removal() {
        let mut opt = PeepholeOptimizer::new();
        let mut func = AirFunction::new("test".to_string());
        
        func.push(Instruction::Nop);
        func.push(Instruction::Nop);
        
        opt.optimize(&mut func);
        assert_eq!(opt.optimizations_count(), 2);
    }
}
