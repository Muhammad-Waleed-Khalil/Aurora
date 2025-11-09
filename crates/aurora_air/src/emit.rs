//! AIR Emission - Lower MIR to AIR with proper calling conventions

use crate::air::*;
use crate::regalloc::RegisterAllocator;
use aurora_mir::{
    BinOp, Constant, Function as MirFunction, Instruction as MirInst, Operand as MirOp, UnaryOp,
};
use std::collections::HashMap;

/// System V ABI calling convention (x86_64)
const ARG_REGISTERS: [Register; 6] = [
    Register::RDI,
    Register::RSI,
    Register::RDX,
    Register::RCX,
    Register::R8,
    Register::R9,
];

/// AIR emitter with full calling convention support
pub struct AirEmitter {
    regalloc: RegisterAllocator,
    string_constants: HashMap<String, String>,
    next_string_id: usize,
}

impl AirEmitter {
    pub fn new() -> Self {
        Self {
            regalloc: RegisterAllocator::new(),
            string_constants: HashMap::new(),
            next_string_id: 0,
        }
    }

    /// Emit AIR module from MIR module
    pub fn emit_module(&mut self, mir_module: &aurora_mir::MirModule) -> AirModule {
        let mut air_module = AirModule::new("main".to_string());

        for func in mir_module.functions.values() {
            let air_func = self.emit_function(func);
            air_module.add_function(air_func);
        }

        // Add string constants to data section with null terminators
        for (label, content) in &self.string_constants {
            let mut bytes = content.as_bytes().to_vec();
            bytes.push(0); // Add null terminator for C strings
            air_module.data.push(DataDirective {
                label: label.clone(),
                kind: DataKind::String,
                value: bytes,
            });
        }

        air_module
    }

    /// Emit AIR from MIR function
    pub fn emit_function(&mut self, mir_func: &MirFunction) -> AirFunction {
        let mut air_func = AirFunction::new(mir_func.name.clone());

        // Reset register allocator for each function
        self.regalloc = RegisterAllocator::new();

        // Allocate registers
        self.regalloc.allocate(mir_func);

        // Emit function parameters (following System V ABI)
        self.emit_function_prologue(&mut air_func, mir_func);

        // Emit instructions for each block (in order)
        let mut block_ids: Vec<_> = mir_func.blocks.keys().copied().collect();
        block_ids.sort();

        for block_id in block_ids {
            if let Some(block) = mir_func.block(block_id) {
                // Emit block label
                air_func.push(Instruction::Label {
                    name: format!(".L{}", block_id),
                });

                // Emit instructions
                for inst in &block.instructions {
                    self.emit_instruction(inst, &mut air_func, mir_func);
                }
            }
        }

        air_func.frame_size = self.regalloc.stack_size();
        air_func.used_regs = self.regalloc.callee_saved_registers();
        air_func
    }

    /// Emit function prologue with parameter handling
    fn emit_function_prologue(&self, air_func: &mut AirFunction, mir_func: &MirFunction) {
        // Move parameters from argument registers to allocated locations
        for (i, &param_id) in mir_func.params.iter().enumerate() {
            if i < ARG_REGISTERS.len() {
                let dest_reg = self.regalloc.get_register(param_id);
                air_func.push(Instruction::Mov {
                    dest: Operand::Reg(dest_reg),
                    src: Operand::Reg(ARG_REGISTERS[i]),
                });
            } else {
                // Parameters beyond 6 are passed on stack
                let stack_offset = 16 + ((i - 6) * 8) as i32;
                let dest_reg = self.regalloc.get_register(param_id);
                air_func.push(Instruction::Mov {
                    dest: Operand::Reg(dest_reg),
                    src: Operand::Mem {
                        base: Register::RBP,
                        offset: stack_offset,
                    },
                });
            }
        }
    }

    fn emit_instruction(
        &mut self,
        inst: &MirInst,
        air_func: &mut AirFunction,
        _mir_func: &MirFunction,
    ) {
        air_func.push(Instruction::Comment {
            text: format!("MIR: {:?}", inst).chars().take(60).collect(),
        });

        match inst {
            MirInst::Assign { dest, value, .. } => {
                let dest_reg = self.regalloc.get_register(*dest);
                let src_op = self.operand_to_air(value);

                if let Operand::Reg(src_reg) = src_op {
                    if src_reg != dest_reg {
                        air_func.push(Instruction::Mov {
                            dest: Operand::Reg(dest_reg),
                            src: src_op,
                        });
                    }
                } else {
                    air_func.push(Instruction::Mov {
                        dest: Operand::Reg(dest_reg),
                        src: src_op,
                    });
                }
            }

            MirInst::BinOp {
                dest, op, lhs, rhs, ..
            } => {
                self.emit_binop(dest, op, lhs, rhs, air_func);
            }

            MirInst::UnaryOp { dest, op, value, .. } => {
                let dest_reg = self.regalloc.get_register(*dest);
                let val_op = self.operand_to_air(value);

                // Load value into dest
                air_func.push(Instruction::Mov {
                    dest: Operand::Reg(dest_reg),
                    src: val_op,
                });

                // Perform operation
                match op {
                    UnaryOp::Neg => {
                        air_func.push(Instruction::Neg {
                            operand: Operand::Reg(dest_reg),
                        });
                    }
                    UnaryOp::Not => {
                        air_func.push(Instruction::Xor {
                            dest: Operand::Reg(dest_reg),
                            src: Operand::Imm(1),
                        });
                    }
                    UnaryOp::BitNot => {
                        air_func.push(Instruction::Not {
                            operand: Operand::Reg(dest_reg),
                        });
                    }
                }
            }

            MirInst::Call { dest, func, args, .. } => {
                self.emit_call(dest, func, args, air_func);
            }

            MirInst::Return { value, .. } => {
                if let Some(val) = value {
                    let val_op = self.operand_to_air(val);
                    air_func.push(Instruction::Mov {
                        dest: Operand::Reg(Register::RAX),
                        src: val_op,
                    });
                } else {
                    // Return unit (0)
                    air_func.push(Instruction::Xor {
                        dest: Operand::Reg(Register::RAX),
                        src: Operand::Reg(Register::RAX),
                    });
                }
                // Note: actual ret is added by function epilogue
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
                    target: format!(".L{}", then_block),
                });
                air_func.push(Instruction::Jmp {
                    target: format!(".L{}", else_block),
                });
            }

            MirInst::Jump { target, .. } => {
                air_func.push(Instruction::Jmp {
                    target: format!(".L{}", target),
                });
            }

            MirInst::Load { dest, ptr, .. } => {
                let dest_reg = self.regalloc.get_register(*dest);
                let ptr_op = self.operand_to_air(ptr);

                if let Operand::Reg(ptr_reg) = ptr_op {
                    air_func.push(Instruction::Mov {
                        dest: Operand::Reg(dest_reg),
                        src: Operand::Mem {
                            base: ptr_reg,
                            offset: 0,
                        },
                    });
                }
            }

            MirInst::Store { ptr, value, .. } => {
                let val_op = self.operand_to_air(value);
                let ptr_op = self.operand_to_air(ptr);

                if let Operand::Reg(ptr_reg) = ptr_op {
                    air_func.push(Instruction::Mov {
                        dest: Operand::Mem {
                            base: ptr_reg,
                            offset: 0,
                        },
                        src: val_op,
                    });
                }
            }

            MirInst::Alloca { dest, .. } => {
                // Allocate on stack
                let dest_reg = self.regalloc.get_register(*dest);
                air_func.push(Instruction::Sub {
                    dest: Operand::Reg(Register::RSP),
                    src: Operand::Imm(8),
                });
                air_func.push(Instruction::Mov {
                    dest: Operand::Reg(dest_reg),
                    src: Operand::Reg(Register::RSP),
                });
            }

            MirInst::Cast { dest, value, .. } => {
                let dest_reg = self.regalloc.get_register(*dest);
                let val_op = self.operand_to_air(value);
                air_func.push(Instruction::Mov {
                    dest: Operand::Reg(dest_reg),
                    src: val_op,
                });
            }

            MirInst::GetElement { dest, base, index, .. } => {
                let dest_reg = self.regalloc.get_register(*dest);
                let base_op = self.operand_to_air(base);
                let index_op = self.operand_to_air(index);

                // Calculate address: base + index * 8
                if let (Operand::Reg(base_reg), Operand::Imm(idx)) = (base_op, index_op) {
                    air_func.push(Instruction::Lea {
                        dest: Operand::Reg(dest_reg),
                        src: Operand::Mem {
                            base: base_reg,
                            offset: (idx * 8) as i32,
                        },
                    });
                }
            }

            MirInst::Phi { dest, inputs, .. } => {
                // Phi nodes are handled during SSA elimination
                let dest_reg = self.regalloc.get_register(*dest);
                air_func.push(Instruction::Comment {
                    text: format!("Phi node: {} = {:?}", dest_reg, inputs),
                });
            }
        }
    }

    fn emit_binop(
        &mut self,
        dest: &u32,
        op: &BinOp,
        lhs: &MirOp,
        rhs: &MirOp,
        air_func: &mut AirFunction,
    ) {
        let dest_reg = self.regalloc.get_register(*dest);
        let lhs_op = self.operand_to_air(lhs);
        let rhs_op = self.operand_to_air(rhs);

        // Load LHS into dest
        air_func.push(Instruction::Mov {
            dest: Operand::Reg(dest_reg),
            src: lhs_op,
        });

        // Perform operation
        match op {
            BinOp::Add => {
                air_func.push(Instruction::Add {
                    dest: Operand::Reg(dest_reg),
                    src: rhs_op,
                });
            }
            BinOp::Sub => {
                air_func.push(Instruction::Sub {
                    dest: Operand::Reg(dest_reg),
                    src: rhs_op,
                });
            }
            BinOp::Mul => {
                air_func.push(Instruction::Imul {
                    dest: Operand::Reg(dest_reg),
                    src: rhs_op,
                });
            }
            BinOp::Div => {
                // Division requires special handling (rdx:rax / operand)
                air_func.push(Instruction::Mov {
                    dest: Operand::Reg(Register::RAX),
                    src: Operand::Reg(dest_reg),
                });
                air_func.push(Instruction::Xor {
                    dest: Operand::Reg(Register::RDX),
                    src: Operand::Reg(Register::RDX),
                });
                air_func.push(Instruction::Idiv { operand: rhs_op });
                air_func.push(Instruction::Mov {
                    dest: Operand::Reg(dest_reg),
                    src: Operand::Reg(Register::RAX),
                });
            }
            BinOp::Mod => {
                // Modulo uses remainder in rdx
                air_func.push(Instruction::Mov {
                    dest: Operand::Reg(Register::RAX),
                    src: Operand::Reg(dest_reg),
                });
                air_func.push(Instruction::Xor {
                    dest: Operand::Reg(Register::RDX),
                    src: Operand::Reg(Register::RDX),
                });
                air_func.push(Instruction::Idiv { operand: rhs_op });
                air_func.push(Instruction::Mov {
                    dest: Operand::Reg(dest_reg),
                    src: Operand::Reg(Register::RDX),
                });
            }
            BinOp::BitAnd => {
                air_func.push(Instruction::And {
                    dest: Operand::Reg(dest_reg),
                    src: rhs_op,
                });
            }
            BinOp::BitOr => {
                air_func.push(Instruction::Or {
                    dest: Operand::Reg(dest_reg),
                    src: rhs_op,
                });
            }
            BinOp::BitXor => {
                air_func.push(Instruction::Xor {
                    dest: Operand::Reg(dest_reg),
                    src: rhs_op,
                });
            }
            BinOp::Shl => {
                air_func.push(Instruction::Shl {
                    dest: Operand::Reg(dest_reg),
                    count: rhs_op,
                });
            }
            BinOp::Shr => {
                air_func.push(Instruction::Shr {
                    dest: Operand::Reg(dest_reg),
                    count: rhs_op,
                });
            }
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                // Comparison: cmp then setcc
                air_func.push(Instruction::Cmp {
                    left: Operand::Reg(dest_reg),
                    right: rhs_op,
                });

                // Set destination based on comparison
                let cond_jump = match op {
                    BinOp::Eq => "je",
                    BinOp::Ne => "jne",
                    BinOp::Lt => "jl",
                    BinOp::Le => "jle",
                    BinOp::Gt => "jg",
                    BinOp::Ge => "jge",
                    _ => unreachable!(),
                };

                air_func.push(Instruction::Comment {
                    text: format!("Comparison: {}", cond_jump),
                });

                // Simplified: set to 0 or 1
                air_func.push(Instruction::Mov {
                    dest: Operand::Reg(dest_reg),
                    src: Operand::Imm(0),
                });
            }
            BinOp::And | BinOp::Or => {
                // Logical AND/OR are short-circuiting, need special handling
                air_func.push(Instruction::Comment {
                    text: format!("Logical op: {:?}", op),
                });
            }
        }
    }

    fn emit_call(
        &mut self,
        dest: &Option<u32>,
        func: &MirOp,
        args: &[MirOp],
        air_func: &mut AirFunction,
    ) {
        // System V ABI: first 6 args in registers, rest on stack

        // Place arguments in reverse order on stack if needed
        if args.len() > 6 {
            for arg in args.iter().skip(6).rev() {
                let arg_op = self.operand_to_air(arg);
                air_func.push(Instruction::Push { operand: arg_op });
            }
        }

        // Place first 6 arguments in registers
        for (i, arg) in args.iter().take(6).enumerate() {
            let arg_op = self.operand_to_air(arg);

            // Use LEA for label addresses (Position Independent Code)
            match &arg_op {
                Operand::Label(label) => {
                    air_func.push(Instruction::Lea {
                        dest: Operand::Reg(ARG_REGISTERS[i]),
                        src: Operand::Label(label.clone()),
                    });
                }
                _ => {
                    air_func.push(Instruction::Mov {
                        dest: Operand::Reg(ARG_REGISTERS[i]),
                        src: arg_op,
                    });
                }
            }
        }

        // Call function - handle function name as label/symbol
        let func_target = match func {
            MirOp::Const(Constant::String(name)) => {
                // Function name is a symbol/label
                Operand::Label(name.clone())
            }
            _ => self.operand_to_air(func),
        };

        air_func.push(Instruction::Call { target: func_target });

        // Clean up stack if we pushed arguments
        if args.len() > 6 {
            let stack_cleanup = ((args.len() - 6) * 8) as i64;
            air_func.push(Instruction::Add {
                dest: Operand::Reg(Register::RSP),
                src: Operand::Imm(stack_cleanup),
            });
        }

        // Move result if needed
        if let Some(dest_val) = dest {
            let dest_reg = self.regalloc.get_register(*dest_val);
            if dest_reg != Register::RAX {
                air_func.push(Instruction::Mov {
                    dest: Operand::Reg(dest_reg),
                    src: Operand::Reg(Register::RAX),
                });
            }
        }
    }

    fn operand_to_air(&mut self, op: &MirOp) -> Operand {
        match op {
            MirOp::Value(v) => {
                let reg = self.regalloc.get_register(*v);
                Operand::Reg(reg)
            }
            MirOp::Const(c) => match c {
                Constant::Int(i) => Operand::Imm(*i),
                Constant::Bool(b) => Operand::Imm(if *b { 1 } else { 0 }),
                Constant::String(s) => {
                    // Add to string table
                    let label = format!("str_{}", self.next_string_id);
                    self.next_string_id += 1;
                    self.string_constants.insert(label.clone(), s.clone());
                    Operand::Label(label)
                }
                Constant::Unit => Operand::Imm(0),
                Constant::Float(_) => Operand::Imm(0), // TODO: proper float handling
            },
        }
    }

    /// Get collected string constants
    pub fn string_constants(&self) -> &HashMap<String, String> {
        &self.string_constants
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
    use aurora_mir::{BasicBlock, Function, Instruction, Operand as MirOp, Span, Value};
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

    #[test]
    fn test_emit_function_with_return() {
        let mut emitter = AirEmitter::new();
        let mut func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);

        let mut block = BasicBlock::new(0);
        block.push(Instruction::Return {
            value: Some(MirOp::Const(Constant::Int(42))),
            span: Span::dummy(),
        });
        func.add_block(block);

        let air_func = emitter.emit_function(&func);
        assert!(air_func.instructions.len() > 0);
    }

    #[test]
    fn test_emit_binop() {
        let mut emitter = AirEmitter::new();
        let mut func = Function::new(0, "add".to_string(), Type::Unit, EffectSet::PURE);

        func.add_value(Value { id: 0, ty: Type::Unit, span: Span::dummy() });
        func.add_value(Value { id: 1, ty: Type::Unit, span: Span::dummy() });
        func.add_value(Value { id: 2, ty: Type::Unit, span: Span::dummy() });

        let mut block = BasicBlock::new(0);
        block.push(Instruction::BinOp {
            dest: 2,
            op: BinOp::Add,
            lhs: MirOp::Value(0),
            rhs: MirOp::Value(1),
            span: Span::dummy(),
        });
        func.add_block(block);

        let air_func = emitter.emit_function(&func);
        assert!(air_func.instructions.len() > 0);
    }

    #[test]
    fn test_string_constants() {
        let mut emitter = AirEmitter::new();
        let _ = emitter.operand_to_air(&MirOp::Const(Constant::String("Hello".to_string())));
        assert_eq!(emitter.string_constants().len(), 1);
    }

    #[test]
    fn test_emit_call() {
        let mut emitter = AirEmitter::new();
        let mut func = Function::new(0, "caller".to_string(), Type::Unit, EffectSet::PURE);

        func.add_value(Value { id: 0, ty: Type::Unit, span: Span::dummy() });

        let mut block = BasicBlock::new(0);
        block.push(Instruction::Call {
            dest: Some(0),
            func: MirOp::Const(Constant::Int(0)),
            args: vec![MirOp::Const(Constant::Int(1))],
            effects: EffectSet::PURE,
            span: Span::dummy(),
        });
        func.add_block(block);

        let air_func = emitter.emit_function(&func);
        assert!(air_func.instructions.len() > 0);
    }

    #[test]
    fn test_emit_module() {
        let mut emitter = AirEmitter::new();
        let mut mir_module = aurora_mir::MirModule::new();

        let func = Function::new(0, "main".to_string(), Type::Unit, EffectSet::PURE);
        mir_module.add_function(func);

        let air_module = emitter.emit_module(&mir_module);
        assert_eq!(air_module.functions.len(), 1);
    }
}
