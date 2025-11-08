//! Peephole Optimizations for AIR
//!
//! Implements various local optimizations on AIR instruction sequences:
//! - Dead code elimination
//! - Mov collapse
//! - LEA patterns
//! - Algebraic simplifications
//! - Strength reduction

use crate::air::{AirFunction, Instruction, Operand, Register};

/// Peephole optimizer with comprehensive pattern matching
pub struct PeepholeOptimizer {
    optimizations_applied: usize,
}

impl PeepholeOptimizer {
    pub fn new() -> Self {
        Self {
            optimizations_applied: 0,
        }
    }

    /// Run all peephole optimizations (multiple passes)
    pub fn optimize(&mut self, func: &mut AirFunction) {
        let mut changed = true;
        let mut passes = 0;
        const MAX_PASSES: usize = 5;

        while changed && passes < MAX_PASSES {
            let before = self.optimizations_applied;

            self.dead_mov_elimination(func);
            self.mov_propagation(func);
            self.lea_patterns(func);
            self.algebraic_simplifications(func);
            self.strength_reduction(func);
            self.redundant_load_store(func);
            self.branch_simplification(func);
            self.remove_nops(func);

            changed = self.optimizations_applied > before;
            passes += 1;
        }
    }

    /// Remove dead mov instructions (mov rax, rax)
    fn dead_mov_elimination(&mut self, func: &mut AirFunction) {
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

    /// Propagate mov chains: mov rax, rbx; mov rcx, rax → mov rax, rbx; mov rcx, rbx
    fn mov_propagation(&mut self, func: &mut AirFunction) {
        let mut replacements = Vec::new();

        for i in 0..func.instructions.len().saturating_sub(1) {
            if let [Instruction::Mov { dest: dest1, src: src1 }, Instruction::Mov { dest: dest2, src: src2 }] =
                &func.instructions[i..=i + 1]
            {
                // If src2 == dest1, replace src2 with src1
                if src2 == dest1 && src1 != dest2 {
                    if let Operand::Reg(_) = src1 {
                        replacements.push((i + 1, Instruction::Mov {
                            dest: dest2.clone(),
                            src: src1.clone(),
                        }));
                        self.optimizations_applied += 1;
                    }
                }
            }
        }

        for (i, replacement) in replacements.iter().rev() {
            func.instructions[*i] = replacement.clone();
        }
    }

    /// Use LEA for address arithmetic
    fn lea_patterns(&mut self, func: &mut AirFunction) {
        let mut replacements = Vec::new();

        for i in 0..func.instructions.len().saturating_sub(1) {
            match &func.instructions[i..=i + 1] {
                // Pattern: mov reg, base; add reg, offset → lea reg, [base + offset]
                [Instruction::Mov { dest: dest1, src }, Instruction::Add { dest: dest2, src: Operand::Imm(offset) }] => {
                    if dest1 == dest2 {
                        if let (Operand::Reg(dest_reg), Operand::Reg(src_reg)) = (dest1, src) {
                            replacements.push((
                                i,
                                vec![Instruction::Lea {
                                    dest: Operand::Reg(*dest_reg),
                                    src: Operand::Mem {
                                        base: *src_reg,
                                        offset: *offset as i32,
                                    },
                                }],
                            ));
                            self.optimizations_applied += 1;
                        }
                    }
                }
                // Pattern: add reg, imm; add reg, imm → add reg, imm+imm
                [Instruction::Add { dest: dest1, src: Operand::Imm(imm1) }, Instruction::Add { dest: dest2, src: Operand::Imm(imm2) }] => {
                    if dest1 == dest2 {
                        replacements.push((
                            i,
                            vec![Instruction::Add {
                                dest: dest1.clone(),
                                src: Operand::Imm(imm1 + imm2),
                            }],
                        ));
                        self.optimizations_applied += 1;
                    }
                }
                _ => {}
            }
        }

        // Apply replacements (remove 2nd instruction, replace 1st)
        for (i, replacement) in replacements.iter().rev() {
            func.instructions[*i] = replacement[0].clone();
            if *i + 1 < func.instructions.len() {
                func.instructions.remove(i + 1);
            }
        }
    }

    /// Algebraic simplifications
    fn algebraic_simplifications(&mut self, func: &mut AirFunction) {
        let mut to_remove = Vec::new();

        for (i, inst) in func.instructions.iter().enumerate() {
            let should_remove = match inst {
                // add rax, 0 → nop
                Instruction::Add { src: Operand::Imm(0), .. } => true,
                // sub rax, 0 → nop
                Instruction::Sub { src: Operand::Imm(0), .. } => true,
                // imul rax, 1 → nop
                Instruction::Imul { src: Operand::Imm(1), .. } => true,
                // or rax, 0 → nop
                Instruction::Or { src: Operand::Imm(0), .. } => true,
                // and rax, -1 → nop
                Instruction::And { src: Operand::Imm(-1), .. } => true,
                // xor rax, 0 → nop
                Instruction::Xor { src: Operand::Imm(0), .. } => true,
                // shl rax, 0 → nop
                Instruction::Shl { count: Operand::Imm(0), .. } => true,
                // shr rax, 0 → nop
                Instruction::Shr { count: Operand::Imm(0), .. } => true,
                _ => false,
            };

            if should_remove {
                to_remove.push(i);
                self.optimizations_applied += 1;
            }
        }

        for &i in to_remove.iter().rev() {
            func.instructions.remove(i);
        }

        // Handle special cases that need replacement
        let mut replacements = Vec::new();
        for (i, inst) in func.instructions.iter().enumerate() {
            match inst {
                // xor rax, rax → mov rax, 0 (but xor is better!)
                Instruction::Xor { dest, src } if dest == src => {
                    // Actually xor rax, rax is optimal for zeroing, keep it
                    continue;
                }
                // imul rax, 0 → mov rax, 0
                Instruction::Imul { dest, src: Operand::Imm(0) } => {
                    replacements.push((i, Instruction::Xor {
                        dest: dest.clone(),
                        src: dest.clone(),
                    }));
                    self.optimizations_applied += 1;
                }
                _ => {}
            }
        }

        for (i, replacement) in replacements.iter().rev() {
            func.instructions[*i] = replacement.clone();
        }
    }

    /// Strength reduction
    fn strength_reduction(&mut self, func: &mut AirFunction) {
        let mut replacements = Vec::new();

        for (i, inst) in func.instructions.iter().enumerate() {
            match inst {
                // imul rax, 2 → add rax, rax
                Instruction::Imul { dest, src: Operand::Imm(2) } => {
                    replacements.push((i, Instruction::Add {
                        dest: dest.clone(),
                        src: dest.clone(),
                    }));
                    self.optimizations_applied += 1;
                }
                // imul rax, power-of-2 → shl rax, log2
                Instruction::Imul { dest, src: Operand::Imm(n) } if *n > 0 && ((*n as u64) & ((*n as u64) - 1)) == 0 => {
                    let shift = (*n as u64).trailing_zeros() as i64;
                    replacements.push((i, Instruction::Shl {
                        dest: dest.clone(),
                        count: Operand::Imm(shift),
                    }));
                    self.optimizations_applied += 1;
                }
                _ => {}
            }
        }

        for (i, replacement) in replacements.iter().rev() {
            func.instructions[*i] = replacement.clone();
        }
    }

    /// Remove redundant load/store sequences
    fn redundant_load_store(&mut self, func: &mut AirFunction) {
        let mut to_remove = Vec::new();

        for i in 0..func.instructions.len().saturating_sub(1) {
            if let [inst1, inst2] = &func.instructions[i..=i + 1] {
                // Pattern: mov [mem], rax; mov rax, [mem] → mov [mem], rax
                if let (
                    Instruction::Mov { dest: Operand::Mem { .. }, src: Operand::Reg(r1) },
                    Instruction::Mov { dest: Operand::Reg(r2), src: Operand::Mem { .. } }
                ) = (inst1, inst2) {
                    if r1 == r2 {
                        to_remove.push(i + 1);
                        self.optimizations_applied += 1;
                    }
                }
            }
        }

        for &i in to_remove.iter().rev() {
            func.instructions.remove(i);
        }
    }

    /// Simplify branch patterns
    fn branch_simplification(&mut self, func: &mut AirFunction) {
        let mut to_remove = Vec::new();

        for i in 0..func.instructions.len().saturating_sub(2) {
            if let [inst1, inst2, inst3] = &func.instructions[i..=i + 2] {
                // Pattern: test rax, rax; jne L1; jmp L1 → jmp L1
                if let (
                    Instruction::Test { .. },
                    Instruction::Jne { target: t1 },
                    Instruction::Jmp { target: t2 }
                ) = (inst1, inst2, inst3) {
                    if t1 == t2 {
                        to_remove.push(i);     // Remove test
                        to_remove.push(i + 1); // Remove jne
                        self.optimizations_applied += 1;
                    }
                }
            }
        }

        for &i in to_remove.iter().rev() {
            if i < func.instructions.len() {
                func.instructions.remove(i);
            }
        }
    }

    /// Remove nop instructions and comments (optional)
    fn remove_nops(&mut self, func: &mut AirFunction) {
        let original_len = func.instructions.len();
        func.instructions.retain(|inst| !matches!(inst, Instruction::Nop));
        self.optimizations_applied += original_len - func.instructions.len();
    }

    /// Get count of optimizations applied
    pub fn optimizations_count(&self) -> usize {
        self.optimizations_applied
    }

    /// Reset optimization counter
    pub fn reset_counter(&mut self) {
        self.optimizations_applied = 0;
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
    fn test_dead_mov_elimination() {
        let mut opt = PeepholeOptimizer::new();
        let mut func = AirFunction::new("test".to_string());

        func.push(Instruction::Mov {
            dest: Operand::Reg(Register::RAX),
            src: Operand::Reg(Register::RAX),
        });

        let before = func.instructions.len();
        opt.optimize(&mut func);
        assert!(func.instructions.len() < before);
        assert!(opt.optimizations_count() > 0);
    }

    #[test]
    fn test_nop_removal() {
        let mut opt = PeepholeOptimizer::new();
        let mut func = AirFunction::new("test".to_string());

        func.push(Instruction::Nop);
        func.push(Instruction::Nop);

        opt.optimize(&mut func);
        assert_eq!(func.instructions.len(), 0);
        assert_eq!(opt.optimizations_count(), 2);
    }

    #[test]
    fn test_lea_pattern() {
        let mut opt = PeepholeOptimizer::new();
        let mut func = AirFunction::new("test".to_string());

        func.push(Instruction::Mov {
            dest: Operand::Reg(Register::RAX),
            src: Operand::Reg(Register::RBX),
        });
        func.push(Instruction::Add {
            dest: Operand::Reg(Register::RAX),
            src: Operand::Imm(8),
        });

        opt.optimize(&mut func);
        assert_eq!(func.instructions.len(), 1);
        assert!(matches!(func.instructions[0], Instruction::Lea { .. }));
    }

    #[test]
    fn test_algebraic_add_zero() {
        let mut opt = PeepholeOptimizer::new();
        let mut func = AirFunction::new("test".to_string());

        func.push(Instruction::Add {
            dest: Operand::Reg(Register::RAX),
            src: Operand::Imm(0),
        });

        opt.optimize(&mut func);
        assert_eq!(func.instructions.len(), 0);
    }

    #[test]
    fn test_strength_reduction_mul_2() {
        let mut opt = PeepholeOptimizer::new();
        let mut func = AirFunction::new("test".to_string());

        func.push(Instruction::Imul {
            dest: Operand::Reg(Register::RAX),
            src: Operand::Imm(2),
        });

        opt.optimize(&mut func);
        assert!(matches!(func.instructions[0], Instruction::Add { .. }));
    }

    #[test]
    fn test_strength_reduction_mul_power_of_2() {
        let mut opt = PeepholeOptimizer::new();
        let mut func = AirFunction::new("test".to_string());

        func.push(Instruction::Imul {
            dest: Operand::Reg(Register::RAX),
            src: Operand::Imm(8),
        });

        opt.optimize(&mut func);
        assert!(matches!(func.instructions[0], Instruction::Shl { .. }));
    }

    #[test]
    fn test_mov_propagation() {
        let mut opt = PeepholeOptimizer::new();
        let mut func = AirFunction::new("test".to_string());

        func.push(Instruction::Mov {
            dest: Operand::Reg(Register::RAX),
            src: Operand::Reg(Register::RBX),
        });
        func.push(Instruction::Mov {
            dest: Operand::Reg(Register::RCX),
            src: Operand::Reg(Register::RAX),
        });

        opt.optimize(&mut func);
        // Should propagate RBX to second mov
        if let Instruction::Mov { src, .. } = &func.instructions[1] {
            assert_eq!(src, &Operand::Reg(Register::RBX));
        }
    }

    #[test]
    fn test_combine_add_immediates() {
        let mut opt = PeepholeOptimizer::new();
        let mut func = AirFunction::new("test".to_string());

        func.push(Instruction::Add {
            dest: Operand::Reg(Register::RAX),
            src: Operand::Imm(5),
        });
        func.push(Instruction::Add {
            dest: Operand::Reg(Register::RAX),
            src: Operand::Imm(3),
        });

        opt.optimize(&mut func);
        assert_eq!(func.instructions.len(), 1);
        if let Instruction::Add { src, .. } = &func.instructions[0] {
            assert_eq!(src, &Operand::Imm(8));
        }
    }

    #[test]
    fn test_reset_counter() {
        let mut opt = PeepholeOptimizer::new();
        let mut func = AirFunction::new("test".to_string());

        func.push(Instruction::Nop);
        opt.optimize(&mut func);
        assert!(opt.optimizations_count() > 0);

        opt.reset_counter();
        assert_eq!(opt.optimizations_count(), 0);
    }

    #[test]
    fn test_multiple_passes() {
        let mut opt = PeepholeOptimizer::new();
        let mut func = AirFunction::new("test".to_string());

        // Create pattern that requires multiple passes
        func.push(Instruction::Mov {
            dest: Operand::Reg(Register::RAX),
            src: Operand::Reg(Register::RBX),
        });
        func.push(Instruction::Add {
            dest: Operand::Reg(Register::RAX),
            src: Operand::Imm(0),
        });
        func.push(Instruction::Nop);

        opt.optimize(&mut func);
        assert!(func.instructions.len() <= 1);
    }

    #[test]
    fn test_no_optimization_needed() {
        let mut opt = PeepholeOptimizer::new();
        let mut func = AirFunction::new("test".to_string());

        func.push(Instruction::Mov {
            dest: Operand::Reg(Register::RAX),
            src: Operand::Imm(42),
        });
        func.push(Instruction::Ret);

        let before = func.instructions.len();
        opt.optimize(&mut func);
        assert_eq!(func.instructions.len(), before);
    }
}
