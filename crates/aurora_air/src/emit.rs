//! AIR Emission - Lower MIR to AIR

use crate::air::*;
use crate::regalloc::RegisterAllocator;
use aurora_mir::{BinOp, Function as MirFunction, Instruction as MirInst, Operand as MirOp};

/// AIR emitter
pub struct AirEmitter {
    regalloc: RegisterAllocator,
}

impl AirEmitter {
    pub fn new() -> Self {
        Self {
            regalloc: RegisterAllocator::new(),
        }
    }

    /// Emit AIR from MIR function
    pub fn emit_function(&mut self, mir_func: &MirFunction) -> AirFunction {
        let mut air_func = AirFunction::new(mir_func.name.clone());

        // Allocate registers
        self.regalloc.allocate(mir_func);

        // Emit instructions for each block
        for block_id in 0..mir_func.blocks.len() as u32 {
            if let Some(block) = mir_func.block(block_id) {
                // Emit block label
                air_func.push(Instruction::Label {
                    name: format!("bb{}", block_id),
                });

                // Emit instructions
                for inst in &block.instructions {
                    self.emit_instruction(inst, &mut air_func, mir_func);
                }
            }
        }

        air_func.frame_size = self.regalloc.stack_size();
        air_func.used_regs = self.regalloc.used_registers();
        air_func
    }

    fn emit_instruction(
        &self,
        inst: &MirInst,
        air_func: &mut AirFunction,
        _mir_func: &MirFunction,
    ) {
        match inst {
            MirInst::Assign { dest, value, .. } => {
                let dest_reg = self.regalloc.get_register(*dest);
                let src_op = self.operand_to_air(value);
                air_func.push(Instruction::Mov {
                    dest: Operand::Reg(dest_reg),
                    src: src_op,
                });
            }

            MirInst::BinOp {
                dest, op, lhs, rhs, ..
            } => {
                let dest_reg = self.regalloc.get_register(*dest);
                let lhs_op = self.operand_to_air(lhs);
                let rhs_op = self.operand_to_air(rhs);

                // Load LHS into dest
                air_func.push(Instruction::Mov {
                    dest: Operand::Reg(dest_reg),
                    src: lhs_op,
                });

                // Perform operation
                let air_inst = match op {
                    BinOp::Add => Instruction::Add {
                        dest: Operand::Reg(dest_reg),
                        src: rhs_op,
                    },
                    BinOp::Sub => Instruction::Sub {
                        dest: Operand::Reg(dest_reg),
                        src: rhs_op,
                    },
                    BinOp::Mul => Instruction::Imul {
                        dest: Operand::Reg(dest_reg),
                        src: rhs_op,
                    },
                    _ => Instruction::Comment {
                        text: format!("BinOp: {:?}", op),
                    },
                };
                air_func.push(air_inst);
            }

            MirInst::Call { dest, func, args, .. } => {
                // Push arguments (simplified - should follow calling convention)
                for arg in args.iter().rev() {
                    let arg_op = self.operand_to_air(arg);
                    air_func.push(Instruction::Push { operand: arg_op });
                }

                // Call function
                let func_op = self.operand_to_air(func);
                air_func.push(Instruction::Call { target: func_op });

                // Clean up stack
                if !args.is_empty() {
                    air_func.push(Instruction::Add {
                        dest: Operand::Reg(Register::RSP),
                        src: Operand::Imm((args.len() * 8) as i64),
                    });
                }

                // Move result if needed
                if let Some(dest_val) = dest {
                    let dest_reg = self.regalloc.get_register(*dest_val);
                    air_func.push(Instruction::Mov {
                        dest: Operand::Reg(dest_reg),
                        src: Operand::Reg(Register::RAX),
                    });
                }
            }

            MirInst::Return { value, .. } => {
                if let Some(val) = value {
                    let val_op = self.operand_to_air(val);
                    air_func.push(Instruction::Mov {
                        dest: Operand::Reg(Register::RAX),
                        src: val_op,
                    });
                }
                air_func.push(Instruction::Ret);
            }

            MirInst::Branch {
                cond,
                then_block,
                else_block,
                ..
            } => {
                let cond_op = self.operand_to_air(cond);
                air_func.push(Instruction::Test {
                    left: cond_op.clone(),
                    right: cond_op,
                });
                air_func.push(Instruction::Jne {
                    target: format!("bb{}", then_block),
                });
                air_func.push(Instruction::Jmp {
                    target: format!("bb{}", else_block),
                });
            }

            MirInst::Jump { target, .. } => {
                air_func.push(Instruction::Jmp {
                    target: format!("bb{}", target),
                });
            }

            _ => {
                air_func.push(Instruction::Comment {
                    text: format!("Unsupported: {:?}", inst),
                });
            }
        }
    }

    fn operand_to_air(&self, op: &MirOp) -> Operand {
        match op {
            MirOp::Value(v) => {
                let reg = self.regalloc.get_register(*v);
                Operand::Reg(reg)
            }
            MirOp::Const(c) => match c {
                aurora_mir::Constant::Int(i) => Operand::Imm(*i),
                aurora_mir::Constant::Bool(b) => Operand::Imm(if *b { 1 } else { 0 }),
                _ => Operand::Imm(0),
            },
        }
    }
}

impl Default for AirEmitter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_mir::{Function, Instruction, Operand as MirOp, Span};
    use aurora_types::{EffectSet, Type};

    #[test]
    fn test_emitter_creation() {
        let emitter = AirEmitter::new();
        assert_eq!(emitter.regalloc.stack_size(), 0);
    }

    #[test]
    fn test_emit_simple_function() {
        let mut emitter = AirEmitter::new();
        let func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);
        
        let air_func = emitter.emit_function(&func);
        assert_eq!(air_func.name, "test");
    }
}
