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

// Pipeline integration stubs
use aurora_mir::MirModule;
use std::sync::Arc;

impl AirModule {
    /// Convert to string for dumping
    pub fn to_string(&self) -> String {
        self.to_text()
    }
}

/// Lower MIR to AIR
pub fn lower_mir_to_air<D: Send + Sync + 'static>(
    _mir: MirModule,
    _diagnostics: Arc<D>,
) -> AirModule {
    // TODO: Implement actual MIR → AIR lowering
    // For now, return empty module
    AirModule::new("main".to_string())
}
