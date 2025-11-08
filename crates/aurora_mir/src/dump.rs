//! MIR Dumping - Human-readable and JSON export

use crate::mir::*;
use serde_json;
use std::fmt::Write;

/// MIR Dumper
pub struct MirDumper {
    /// Include spans in output
    pub include_spans: bool,
    /// Indent level
    indent: usize,
}

impl MirDumper {
    pub fn new() -> Self {
        Self {
            include_spans: true,
            indent: 0,
        }
    }

    /// Dump function to string
    pub fn dump_function(&mut self, func: &Function) -> String {
        let mut output = String::new();
        
        writeln!(
            &mut output,
            "fn {}({}) -> {} {{",
            func.name,
            func.params
                .iter()
                .map(|v| format!("v{}", v))
                .collect::<Vec<_>>()
                .join(", "),
            self.type_to_string(&func.ret_ty)
        )
        .unwrap();

        self.indent += 2;

        // Dump blocks
        for block_id in 0..func.blocks.len() as BlockId {
            if let Some(block) = func.block(block_id) {
                self.dump_block(&mut output, block, func);
            }
        }

        self.indent -= 2;
        writeln!(&mut output, "}}").unwrap();

        output
    }

    fn dump_block(&self, output: &mut String, block: &BasicBlock, func: &Function) {
        writeln!(
            output,
            "{}bb{}: // preds: {:?}, succs: {:?}",
            self.indent_str(),
            block.id,
            block.predecessors,
            block.successors
        )
        .unwrap();

        for inst in &block.instructions {
            writeln!(output, "{}  {}", self.indent_str(), self.inst_to_string(inst, func))
                .unwrap();
        }
    }

    fn inst_to_string(&self, inst: &Instruction, func: &Function) -> String {
        match inst {
            Instruction::Assign { dest, value, .. } => {
                format!("v{} = {}", dest, self.operand_to_string(value, func))
            }
            Instruction::BinOp {
                dest, op, lhs, rhs, ..
            } => {
                format!(
                    "v{} = {} {:?} {}",
                    dest,
                    self.operand_to_string(lhs, func),
                    op,
                    self.operand_to_string(rhs, func)
                )
            }
            Instruction::UnaryOp { dest, op, value, .. } => {
                format!("v{} = {:?} {}", dest, op, self.operand_to_string(value, func))
            }
            Instruction::Call {
                dest, func: callee, args, effects, ..
            } => {
                let dest_str = dest.map_or("_".to_string(), |d| format!("v{}", d));
                format!(
                    "{} = call {} ({}) effects={:?}",
                    dest_str,
                    self.operand_to_string(callee, func),
                    args.iter()
                        .map(|a| self.operand_to_string(a, func))
                        .collect::<Vec<_>>()
                        .join(", "),
                    effects
                )
            }
            Instruction::Return { value, .. } => {
                format!(
                    "return {}",
                    value.as_ref().map_or("".to_string(), |v| self.operand_to_string(v, func))
                )
            }
            Instruction::Branch {
                cond,
                then_block,
                else_block,
                ..
            } => {
                format!(
                    "br {}, bb{}, bb{}",
                    self.operand_to_string(cond, func),
                    then_block,
                    else_block
                )
            }
            Instruction::Jump { target, .. } => format!("jump bb{}", target),
            Instruction::Phi { dest, inputs, .. } => {
                format!(
                    "v{} = phi [{}]",
                    dest,
                    inputs
                        .iter()
                        .map(|(b, v)| format!("bb{}: {}", b, self.operand_to_string(v, func)))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Instruction::Load { dest, ptr, .. } => {
                format!("v{} = load {}", dest, self.operand_to_string(ptr, func))
            }
            Instruction::Store { ptr, value, .. } => {
                format!(
                    "store {}, {}",
                    self.operand_to_string(ptr, func),
                    self.operand_to_string(value, func)
                )
            }
            Instruction::Alloca { dest, ty, .. } => {
                format!("v{} = alloca {}", dest, self.type_to_string(ty))
            }
            Instruction::Cast { dest, value, target_ty, .. } => {
                format!(
                    "v{} = cast {} to {}",
                    dest,
                    self.operand_to_string(value, func),
                    self.type_to_string(target_ty)
                )
            }
            Instruction::GetElement { dest, base, index, .. } => {
                format!(
                    "v{} = getelement {}, {}",
                    dest,
                    self.operand_to_string(base, func),
                    self.operand_to_string(index, func)
                )
            }
        }
    }

    fn operand_to_string(&self, op: &Operand, func: &Function) -> String {
        match op {
            Operand::Value(v) => {
                if let Some(value) = func.value(*v) {
                    format!("v{}: {}", v, self.type_to_string(&value.ty))
                } else {
                    format!("v{}", v)
                }
            }
            Operand::Const(c) => self.const_to_string(c),
        }
    }

    fn const_to_string(&self, c: &Constant) -> String {
        match c {
            Constant::Int(i) => i.to_string(),
            Constant::Float(f) => format!("{:?}", f64::from_bits(*f)),
            Constant::Bool(b) => b.to_string(),
            Constant::String(s) => format!("\"{}\"", s),
            Constant::Unit => "()".to_string(),
        }
    }

    fn type_to_string(&self, ty: &aurora_types::Type) -> String {
        format!("{:?}", ty)
    }

    fn indent_str(&self) -> String {
        " ".repeat(self.indent)
    }

    /// Export to JSON
    pub fn to_json(&self, func: &Function) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(func)
    }
}

impl Default for MirDumper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_types::{EffectSet, Type};

    #[test]
    fn test_dumper_creation() {
        let dumper = MirDumper::new();
        assert!(dumper.include_spans);
    }

    #[test]
    fn test_dump_function() {
        let func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);
        let mut dumper = MirDumper::new();
        let output = dumper.dump_function(&func);
        assert!(output.contains("fn test"));
    }

    #[test]
    fn test_json_export() {
        let func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);
        let dumper = MirDumper::new();
        let json = dumper.to_json(&func);
        assert!(json.is_ok());
    }
}
