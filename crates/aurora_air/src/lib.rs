//! Aurora AIR (Assembly Intermediate Representation)
//!
//! AIR is a NASM-like assembly format that sits between MIR and machine code.

pub mod air;
pub mod emit;
pub mod peephole;
pub mod regalloc;
pub mod schedule;

pub use air::{AirFunction, AirModule, Instruction, Operand, Register};
pub use emit::AirEmitter;
pub use peephole::PeepholeOptimizer;
pub use regalloc::RegisterAllocator;
pub use schedule::{CpuProfile, InstructionScheduler};
