//! Aurora AIR (Assembly Intermediate Representation)
//!
//! AIR is a NASM-like assembly format that sits between MIR and machine code.
//!
//! # Pipeline
//!
//! 1. **Emission**: Lower MIR to AIR with calling conventions
//! 2. **Register Allocation**: Linear scan with liveness analysis
//! 3. **Peephole Optimization**: Pattern-based local optimizations
//! 4. **Instruction Scheduling**: CPU-aware instruction reordering
//!
//! # Example
//!
//! ```ignore
//! use aurora_air::{lower_mir_to_air, CpuProfile};
//!
//! let air_module = lower_mir_to_air(mir_module, diagnostics);
//! println!("{}", air_module.to_string());
//! ```

pub mod air;
pub mod emit;
pub mod peephole;
pub mod regalloc;
pub mod schedule;

pub use air::{AirFunction, AirModule, Instruction, Operand, Register, DataDirective, DataKind};
pub use emit::AirEmitter;
pub use peephole::PeepholeOptimizer;
pub use regalloc::RegisterAllocator;
pub use schedule::{CpuProfile, InstructionScheduler};

// Pipeline integration
use aurora_mir::MirModule;
use std::sync::Arc;

impl AirModule {
    /// Convert to string for dumping
    pub fn to_string(&self) -> String {
        self.to_text()
    }
}

/// AIR lowering options
#[derive(Debug, Clone)]
pub struct AirOptions {
    /// CPU profile for scheduling
    pub cpu_profile: CpuProfile,
    /// Enable peephole optimizations
    pub enable_peephole: bool,
    /// Enable instruction scheduling
    pub enable_scheduling: bool,
    /// Optimization level (0-3)
    pub opt_level: u8,
}

impl Default for AirOptions {
    fn default() -> Self {
        Self {
            cpu_profile: CpuProfile::generic(),
            enable_peephole: true,
            enable_scheduling: true,
            opt_level: 2,
        }
    }
}

impl AirOptions {
    /// Create options for specific CPU
    pub fn for_cpu(cpu: &str) -> Self {
        let profile = match cpu {
            "skylake" => CpuProfile::skylake(),
            "zen" => CpuProfile::zen(),
            _ => CpuProfile::generic(),
        };

        Self {
            cpu_profile: profile,
            enable_peephole: true,
            enable_scheduling: true,
            opt_level: 2,
        }
    }

    /// Disable optimizations
    pub fn no_opt() -> Self {
        Self {
            cpu_profile: CpuProfile::generic(),
            enable_peephole: false,
            enable_scheduling: false,
            opt_level: 0,
        }
    }
}

/// Lower MIR to AIR with full optimization pipeline
pub fn lower_mir_to_air<D: Send + Sync + 'static>(
    mir: MirModule,
    _diagnostics: Arc<D>,
) -> AirModule {
    lower_mir_to_air_with_options(mir, _diagnostics, AirOptions::default())
}

/// Lower MIR to AIR with custom options
pub fn lower_mir_to_air_with_options<D: Send + Sync + 'static>(
    mir: MirModule,
    _diagnostics: Arc<D>,
    options: AirOptions,
) -> AirModule {
    // Step 1: Emit AIR from MIR
    let mut emitter = AirEmitter::new();
    let mut air_module = emitter.emit_module(&mir);

    // Step 2: Apply optimizations per function
    for func in &mut air_module.functions {
        // Peephole optimizations
        if options.enable_peephole && options.opt_level > 0 {
            let mut peephole = PeepholeOptimizer::new();
            peephole.optimize(func);
        }

        // Instruction scheduling
        if options.enable_scheduling && options.opt_level > 1 {
            let mut scheduler = InstructionScheduler::new(options.cpu_profile.clone());
            scheduler.schedule(func);
        }
    }

    air_module
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_mir::{BasicBlock, Constant, Function, Instruction, Operand, Span, Value};
    use aurora_types::{EffectSet, Type};

    #[test]
    fn test_air_options_default() {
        let opts = AirOptions::default();
        assert_eq!(opts.opt_level, 2);
        assert!(opts.enable_peephole);
        assert!(opts.enable_scheduling);
    }

    #[test]
    fn test_air_options_for_cpu() {
        let opts = AirOptions::for_cpu("skylake");
        assert_eq!(opts.cpu_profile.name, "Skylake");

        let opts = AirOptions::for_cpu("zen");
        assert_eq!(opts.cpu_profile.name, "Zen");
    }

    #[test]
    fn test_air_options_no_opt() {
        let opts = AirOptions::no_opt();
        assert_eq!(opts.opt_level, 0);
        assert!(!opts.enable_peephole);
        assert!(!opts.enable_scheduling);
    }

    #[test]
    fn test_lower_empty_module() {
        let mir = MirModule::new();
        let diagnostics = Arc::new(());

        let air = lower_mir_to_air(mir, diagnostics);
        assert_eq!(air.functions.len(), 0);
    }

    #[test]
    fn test_lower_simple_function() {
        let mut mir = MirModule::new();

        let func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);
        mir.add_function(func);

        let diagnostics = Arc::new(());
        let air = lower_mir_to_air(mir, diagnostics);

        assert_eq!(air.functions.len(), 1);
        assert_eq!(air.functions[0].name, "test");
    }

    #[test]
    fn test_lower_with_instructions() {
        let mut mir = MirModule::new();

        let mut func = Function::new(0, "add".to_string(), Type::Unit, EffectSet::PURE);

        // Add values
        func.add_value(Value { id: 0, ty: Type::Unit, span: Span::dummy() });
        func.add_value(Value { id: 1, ty: Type::Unit, span: Span::dummy() });
        func.add_value(Value { id: 2, ty: Type::Unit, span: Span::dummy() });

        // Create block with instructions
        let mut block = BasicBlock::new(0);
        block.push(Instruction::Assign {
            dest: 0,
            value: Operand::Const(Constant::Int(1)),
            span: Span::dummy(),
        });
        block.push(Instruction::Assign {
            dest: 1,
            value: Operand::Const(Constant::Int(2)),
            span: Span::dummy(),
        });
        block.push(Instruction::BinOp {
            dest: 2,
            op: aurora_mir::BinOp::Add,
            lhs: Operand::Value(0),
            rhs: Operand::Value(1),
            span: Span::dummy(),
        });
        func.add_block(block);

        mir.add_function(func);

        let diagnostics = Arc::new(());
        let air = lower_mir_to_air(mir, diagnostics);

        assert_eq!(air.functions.len(), 1);
        assert!(!air.functions[0].instructions.is_empty());
    }

    #[test]
    fn test_lower_with_optimizations() {
        let mut mir = MirModule::new();
        let func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);
        mir.add_function(func);

        let diagnostics = Arc::new(());
        let options = AirOptions::for_cpu("skylake");

        let air = lower_mir_to_air_with_options(mir, diagnostics, options);
        assert_eq!(air.functions.len(), 1);
    }

    #[test]
    fn test_lower_without_optimizations() {
        let mut mir = MirModule::new();
        let func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);
        mir.add_function(func);

        let diagnostics = Arc::new(());
        let options = AirOptions::no_opt();

        let air = lower_mir_to_air_with_options(mir, diagnostics, options);
        assert_eq!(air.functions.len(), 1);
    }

    #[test]
    fn test_air_module_to_string() {
        let mut air = AirModule::new("test".to_string());
        let func = AirFunction::new("main".to_string());
        air.add_function(func);

        let output = air.to_string();
        assert!(output.contains("test"));
        assert!(output.contains("main"));
    }

    #[test]
    fn test_lower_with_string_constants() {
        let mut mir = MirModule::new();

        let mut func = Function::new(0, "main".to_string(), Type::Unit, EffectSet::PURE);
        func.add_value(Value { id: 0, ty: Type::Unit, span: Span::dummy() });

        let mut block = BasicBlock::new(0);
        block.push(Instruction::Assign {
            dest: 0,
            value: Operand::Const(Constant::String("Hello, World!".to_string())),
            span: Span::dummy(),
        });
        func.add_block(block);

        mir.add_function(func);

        let diagnostics = Arc::new(());
        let air = lower_mir_to_air(mir, diagnostics);

        // Should have string constant in data section
        assert!(air.data.len() > 0);
    }

    #[test]
    fn test_multiple_functions() {
        let mut mir = MirModule::new();

        for i in 0..3 {
            let func = Function::new(i, format!("func{}", i), Type::Unit, EffectSet::PURE);
            mir.add_function(func);
        }

        let diagnostics = Arc::new(());
        let air = lower_mir_to_air(mir, diagnostics);

        assert_eq!(air.functions.len(), 3);
    }
}
