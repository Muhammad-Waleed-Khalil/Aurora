//! Aurora Intermediate Representation (AIR)
//!
//! AIR is a NASM-like textual assembly format that sits between MIR and machine code.
//! It supports x86_64 instruction set and includes debug info and unwind directives.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Physical register
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Register {
    // 64-bit general purpose
    RAX, RBX, RCX, RDX,
    RSI, RDI, RBP, RSP,
    R8, R9, R10, R11, R12, R13, R14, R15,
    
    // 32-bit
    EAX, EBX, ECX, EDX,
    ESI, EDI, EBP, ESP,
    
    // XMM registers (SIMD)
    XMM0, XMM1, XMM2, XMM3, XMM4, XMM5, XMM6, XMM7,
    XMM8, XMM9, XMM10, XMM11, XMM12, XMM13, XMM14, XMM15,
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            Register::RAX => "rax",
            Register::RBX => "rbx",
            Register::RCX => "rcx",
            Register::RDX => "rdx",
            Register::RSI => "rsi",
            Register::RDI => "rdi",
            Register::RBP => "rbp",
            Register::RSP => "rsp",
            Register::R8 => "r8",
            Register::R9 => "r9",
            Register::R10 => "r10",
            Register::R11 => "r11",
            Register::R12 => "r12",
            Register::R13 => "r13",
            Register::R14 => "r14",
            Register::R15 => "r15",
            Register::EAX => "eax",
            Register::EBX => "ebx",
            Register::ECX => "ecx",
            Register::EDX => "edx",
            Register::ESI => "esi",
            Register::EDI => "edi",
            Register::EBP => "ebp",
            Register::ESP => "esp",
            Register::XMM0 => "xmm0",
            Register::XMM1 => "xmm1",
            Register::XMM2 => "xmm2",
            Register::XMM3 => "xmm3",
            Register::XMM4 => "xmm4",
            Register::XMM5 => "xmm5",
            Register::XMM6 => "xmm6",
            Register::XMM7 => "xmm7",
            Register::XMM8 => "xmm8",
            Register::XMM9 => "xmm9",
            Register::XMM10 => "xmm10",
            Register::XMM11 => "xmm11",
            Register::XMM12 => "xmm12",
            Register::XMM13 => "xmm13",
            Register::XMM14 => "xmm14",
            Register::XMM15 => "xmm15",
        };
        write!(f, "{}", name)
    }
}

/// AIR operand
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operand {
    /// Register
    Reg(Register),
    /// Immediate value
    Imm(i64),
    /// Memory address [base + offset]
    Mem { base: Register, offset: i32 },
    /// Memory address [base + index * scale + offset]
    MemComplex {
        base: Option<Register>,
        index: Option<Register>,
        scale: u8, // 1, 2, 4, or 8
        offset: i32,
    },
    /// Label reference
    Label(String),
}

/// AIR instruction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Instruction {
    // Data movement
    Mov { dest: Operand, src: Operand },
    Movzx { dest: Operand, src: Operand },
    Movsx { dest: Operand, src: Operand },
    Lea { dest: Operand, src: Operand },
    
    // Arithmetic
    Add { dest: Operand, src: Operand },
    Sub { dest: Operand, src: Operand },
    Imul { dest: Operand, src: Operand },
    Idiv { operand: Operand },
    Inc { operand: Operand },
    Dec { operand: Operand },
    Neg { operand: Operand },
    
    // Logical
    And { dest: Operand, src: Operand },
    Or { dest: Operand, src: Operand },
    Xor { dest: Operand, src: Operand },
    Not { operand: Operand },
    Shl { dest: Operand, count: Operand },
    Shr { dest: Operand, count: Operand },
    Sar { dest: Operand, count: Operand },
    
    // Comparison
    Cmp { left: Operand, right: Operand },
    Test { left: Operand, right: Operand },
    
    // Control flow
    Jmp { target: String },
    Je { target: String },
    Jne { target: String },
    Jl { target: String },
    Jle { target: String },
    Jg { target: String },
    Jge { target: String },
    Call { target: Operand },
    Ret,
    
    // Stack
    Push { operand: Operand },
    Pop { operand: Operand },
    
    // SIMD (SSE/AVX)
    Movaps { dest: Operand, src: Operand },
    Addps { dest: Operand, src: Operand },
    Mulps { dest: Operand, src: Operand },
    
    // Special
    Nop,
    Label { name: String },
    Comment { text: String },
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // Data movement
            Instruction::Mov { dest, src } => write!(f, "    mov {}, {}", dest, src),
            Instruction::Movzx { dest, src } => write!(f, "    movzx {}, {}", dest, src),
            Instruction::Movsx { dest, src } => write!(f, "    movsx {}, {}", dest, src),
            Instruction::Lea { dest, src } => write!(f, "    lea {}, {}", dest, src),

            // Arithmetic
            Instruction::Add { dest, src } => write!(f, "    add {}, {}", dest, src),
            Instruction::Sub { dest, src } => write!(f, "    sub {}, {}", dest, src),
            Instruction::Imul { dest, src } => write!(f, "    imul {}, {}", dest, src),
            Instruction::Idiv { operand } => write!(f, "    idiv {}", operand),
            Instruction::Inc { operand } => write!(f, "    inc {}", operand),
            Instruction::Dec { operand } => write!(f, "    dec {}", operand),
            Instruction::Neg { operand } => write!(f, "    neg {}", operand),

            // Logical
            Instruction::And { dest, src } => write!(f, "    and {}, {}", dest, src),
            Instruction::Or { dest, src } => write!(f, "    or {}, {}", dest, src),
            Instruction::Xor { dest, src } => write!(f, "    xor {}, {}", dest, src),
            Instruction::Not { operand } => write!(f, "    not {}", operand),
            Instruction::Shl { dest, count } => write!(f, "    shl {}, {}", dest, count),
            Instruction::Shr { dest, count } => write!(f, "    shr {}, {}", dest, count),
            Instruction::Sar { dest, count } => write!(f, "    sar {}, {}", dest, count),

            // Comparison
            Instruction::Cmp { left, right } => write!(f, "    cmp {}, {}", left, right),
            Instruction::Test { left, right } => write!(f, "    test {}, {}", left, right),

            // Control flow
            Instruction::Jmp { target } => write!(f, "    jmp {}", target),
            Instruction::Je { target } => write!(f, "    je {}", target),
            Instruction::Jne { target } => write!(f, "    jne {}", target),
            Instruction::Jl { target } => write!(f, "    jl {}", target),
            Instruction::Jle { target } => write!(f, "    jle {}", target),
            Instruction::Jg { target } => write!(f, "    jg {}", target),
            Instruction::Jge { target } => write!(f, "    jge {}", target),
            Instruction::Call { target } => write!(f, "    call {}", target),
            Instruction::Ret => write!(f, "    ret"),

            // Stack
            Instruction::Push { operand } => write!(f, "    push {}", operand),
            Instruction::Pop { operand } => write!(f, "    pop {}", operand),

            // SIMD
            Instruction::Movaps { dest, src } => write!(f, "    movaps {}, {}", dest, src),
            Instruction::Addps { dest, src } => write!(f, "    addps {}, {}", dest, src),
            Instruction::Mulps { dest, src } => write!(f, "    mulps {}, {}", dest, src),

            // Special
            Instruction::Nop => write!(f, "    nop"),
            Instruction::Label { name } => write!(f, "{}:", name),
            Instruction::Comment { text } => write!(f, "    ; {}", text),
        }
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Operand::Reg(r) => write!(f, "{}", r),
            Operand::Imm(i) => write!(f, "{}", i),
            Operand::Mem { base, offset } => {
                if *offset == 0 {
                    write!(f, "[{}]", base)
                } else {
                    write!(f, "[{} + {}]", base, offset)
                }
            }
            Operand::MemComplex { base, index, scale, offset } => {
                let mut parts = Vec::new();
                if let Some(b) = base {
                    parts.push(format!("{}", b));
                }
                if let Some(idx) = index {
                    if *scale != 1 {
                        parts.push(format!("{} * {}", idx, scale));
                    } else {
                        parts.push(format!("{}", idx));
                    }
                }
                let addr = parts.join(" + ");
                if *offset != 0 {
                    write!(f, "[{} + {}]", addr, offset)
                } else {
                    write!(f, "[{}]", addr)
                }
            }
            Operand::Label(l) => write!(f, "{}", l),
        }
    }
}

/// AIR function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirFunction {
    /// Function name
    pub name: String,
    /// Instructions
    pub instructions: Vec<Instruction>,
    /// Stack frame size
    pub frame_size: u32,
    /// Used registers (for saving/restoring)
    pub used_regs: Vec<Register>,
}

impl AirFunction {
    pub fn new(name: String) -> Self {
        Self {
            name,
            instructions: Vec::new(),
            frame_size: 0,
            used_regs: Vec::new(),
        }
    }

    pub fn push(&mut self, inst: Instruction) {
        self.instructions.push(inst);
    }

    /// Generate AIR text
    pub fn to_text(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("{}:\n", self.name));
        
        // Prologue
        if self.frame_size > 0 {
            output.push_str("    push rbp\n");
            output.push_str("    mov rbp, rsp\n");
            output.push_str(&format!("    sub rsp, {}\n", self.frame_size));
        }
        
        // Save used registers
        for reg in &self.used_regs {
            output.push_str(&format!("    push {}\n", reg));
        }
        
        // Body
        for inst in &self.instructions {
            output.push_str(&format!("{}\n", inst));
        }
        
        // Epilogue
        for reg in self.used_regs.iter().rev() {
            output.push_str(&format!("    pop {}\n", reg));
        }
        
        if self.frame_size > 0 {
            output.push_str("    mov rsp, rbp\n");
            output.push_str("    pop rbp\n");
        }
        
        output.push_str("    ret\n");
        output
    }
}

/// AIR module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirModule {
    /// Module name
    pub name: String,
    /// Functions
    pub functions: Vec<AirFunction>,
    /// Global data
    pub data: Vec<DataDirective>,
}

impl AirModule {
    pub fn new(name: String) -> Self {
        Self {
            name,
            functions: Vec::new(),
            data: Vec::new(),
        }
    }

    pub fn add_function(&mut self, func: AirFunction) {
        self.functions.push(func);
    }

    /// Generate complete AIR text
    pub fn to_text(&self) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("; AIR for module: {}\n\n", self.name));
        output.push_str("section .text\n");
        output.push_str("global main\n\n");
        
        for func in &self.functions {
            output.push_str(&func.to_text());
            output.push('\n');
        }
        
        if !self.data.is_empty() {
            output.push_str("\nsection .data\n");
            for data in &self.data {
                output.push_str(&format!("{}\n", data));
            }
        }
        
        output
    }
}

/// Data directive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataDirective {
    pub label: String,
    pub kind: DataKind,
    pub value: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataKind {
    Byte,
    Word,
    Dword,
    Qword,
    String,
}

impl fmt::Display for DataDirective {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            DataKind::String => {
                // For strings, output as db directive with bytes
                write!(f, "{}:  db ", self.label)?;
                for (i, &byte) in self.value.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", byte)?;
                }
                Ok(())
            }
            DataKind::Byte => {
                write!(f, "{}:  db ", self.label)?;
                for (i, &byte) in self.value.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", byte)?;
                }
                Ok(())
            }
            DataKind::Word => write!(f, "{}:  dw {}", self.label, self.value[0]),
            DataKind::Dword => write!(f, "{}:  dd {}", self.label, self.value[0]),
            DataKind::Qword => write!(f, "{}:  dq {}", self.label, self.value[0]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_display() {
        assert_eq!(format!("{}", Register::RAX), "RAX");
        assert_eq!(format!("{}", Register::RBX), "RBX");
    }

    #[test]
    fn test_operand_display() {
        let reg = Operand::Reg(Register::RAX);
        assert_eq!(format!("{}", reg), "RAX");

        let imm = Operand::Imm(42);
        assert_eq!(format!("{}", imm), "42");

        let mem = Operand::Mem { base: Register::RBP, offset: -8 };
        assert_eq!(format!("{}", mem), "[RBP + -8]");
    }

    #[test]
    fn test_instruction_display() {
        let mov = Instruction::Mov {
            dest: Operand::Reg(Register::RAX),
            src: Operand::Imm(42),
        };
        assert!(format!("{}", mov).contains("mov"));
    }

    #[test]
    fn test_air_function() {
        let mut func = AirFunction::new("test".to_string());
        func.push(Instruction::Mov {
            dest: Operand::Reg(Register::RAX),
            src: Operand::Imm(42),
        });
        func.push(Instruction::Ret);

        let text = func.to_text();
        assert!(text.contains("test:"));
        assert!(text.contains("mov"));
        assert!(text.contains("ret"));
    }

    #[test]
    fn test_air_module() {
        let mut module = AirModule::new("test_module".to_string());
        let func = AirFunction::new("main".to_string());
        module.add_function(func);

        let text = module.to_text();
        assert!(text.contains("test_module"));
        assert!(text.contains("section .text"));
    }
}
