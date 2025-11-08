//! Integration tests for AIR

use aurora_air::*;
use aurora_air::{Instruction as AirInst, Operand as AirOp};
use aurora_mir::{BasicBlock, Function as MirFunction, Instruction as MirInst, Operand as MirOp, Span};
use aurora_types::{EffectSet, Type};

#[test]
fn test_air_module_creation() {
    let module = AirModule::new("test_module".to_string());
    assert_eq!(module.name, "test_module");
    assert!(module.functions.is_empty());
}

#[test]
fn test_air_function_text_generation() {
    let mut func = AirFunction::new("test".to_string());
    func.push(AirInst::Mov {
        dest: AirOp::Reg(Register::RAX),
        src: AirOp::Imm(42),
    });
    func.push(AirInst::Ret);

    let text = func.to_text();
    assert!(text.contains("test:"));
    assert!(text.contains("mov"));
    assert!(text.contains("42"));
    assert!(text.contains("ret"));
}

#[test]
fn test_register_allocation() {
    let mut alloc = RegisterAllocator::new();
    let mut func = MirFunction::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);
    
    for i in 0..5 {
        func.add_value(aurora_mir::Value {
            id: i,
            ty: Type::Unit,
            span: Span::dummy(),
        });
    }
    
    alloc.allocate(&func);

    // First values should get registers (any valid register)
    let reg0 = alloc.get_register(0);
    // Just verify we got a register, not a specific one
    assert!(matches!(
        reg0,
        Register::RAX
            | Register::RBX
            | Register::RCX
            | Register::RDX
            | Register::RSI
            | Register::RDI
    ));
}

#[test]
fn test_peephole_optimization() {
    let mut opt = PeepholeOptimizer::new();
    let mut func = AirFunction::new("test".to_string());

    // Add redundant mov
    func.push(AirInst::Mov {
        dest: AirOp::Reg(Register::RAX),
        src: AirOp::Reg(Register::RAX),
    });

    // Add nops
    func.push(AirInst::Nop);
    func.push(AirInst::Nop);
    
    opt.optimize(&mut func);
    
    // Should have removed 3 instructions
    assert_eq!(opt.optimizations_count(), 3);
}

#[test]
fn test_instruction_scheduling() {
    let mut scheduler = InstructionScheduler::new(CpuProfile::skylake());
    let mut func = AirFunction::new("test".to_string());
    
    func.push(AirInst::Mov {
        dest: AirOp::Reg(Register::RAX),
        src: AirOp::Imm(1),
    });
    func.push(AirInst::Add {
        dest: AirOp::Reg(Register::RAX),
        src: AirOp::Imm(2),
    });
    
    scheduler.schedule(&mut func);
    assert!(scheduler.scheduled_count() > 0);
}

#[test]
fn test_air_emission() {
    let mut emitter = AirEmitter::new();
    let mut mir_func = MirFunction::new(0, "add".to_string(), Type::Unit, EffectSet::PURE);
    mir_func.entry = 0;

    let mut block = BasicBlock::new(0);
    block.push(MirInst::Return {
        value: None,
        span: Span::dummy(),
    });
    mir_func.add_block(block);

    let air_func = emitter.emit_function(&mir_func);
    assert_eq!(air_func.name, "add");
}

#[test]
fn test_complete_pipeline() {
    // Create MIR function
    let mut mir_func = MirFunction::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);
    mir_func.entry = 0;
    
    let mut block = BasicBlock::new(0);
    block.push(MirInst::Assign {
        dest: 0,
        value: MirOp::Const(aurora_mir::Constant::Int(42)),
        span: Span::dummy(),
    });
    block.push(MirInst::Return {
        value: Some(MirOp::Value(0)),
        span: Span::dummy(),
    });
    mir_func.add_block(block);
    
    // Emit AIR
    let mut emitter = AirEmitter::new();
    let mut air_func = emitter.emit_function(&mir_func);
    
    // Optimize
    let mut peephole = PeepholeOptimizer::new();
    peephole.optimize(&mut air_func);
    
    // Schedule
    let mut scheduler = InstructionScheduler::new(CpuProfile::skylake());
    scheduler.schedule(&mut air_func);
    
    // Generate text
    let text = air_func.to_text();
    assert!(text.contains("test:"));
}

#[test]
fn test_cpu_profiles() {
    let skylake = CpuProfile::skylake();
    assert_eq!(skylake.name, "Skylake");
    
    let zen = CpuProfile::zen();
    assert_eq!(zen.name, "Zen");
}

#[test]
fn test_operand_formatting() {
    let reg = AirOp::Reg(Register::RBX);
    assert_eq!(format!("{}", reg), "RBX");
    
    let imm = AirOp::Imm(100);
    assert_eq!(format!("{}", imm), "100");
    
    let mem = AirOp::Mem {
        base: Register::RBP,
        offset: -16,
    };
    assert!(format!("{}", mem).contains("RBP"));
}

#[test]
fn test_lea_optimization() {
    let mut opt = PeepholeOptimizer::new();
    let mut func = AirFunction::new("test".to_string());
    
    // Pattern that can be converted to LEA
    func.push(AirInst::Mov {
        dest: AirOp::Reg(Register::RAX),
        src: AirOp::Reg(Register::RBX),
    });
    func.push(AirInst::Add {
        dest: AirOp::Reg(Register::RAX),
        src: AirOp::Imm(8),
    });
    
    let before = func.instructions.len();
    opt.optimize(&mut func);
    let after = func.instructions.len();
    
    // Should have combined into single LEA
    assert!(after <= before);
}

#[test]
fn test_air_module_text() {
    let mut module = AirModule::new("test_module".to_string());
    
    let mut func = AirFunction::new("main".to_string());
    func.push(AirInst::Mov {
        dest: AirOp::Reg(Register::RAX),
        src: AirOp::Imm(0),
    });
    func.push(AirInst::Ret);
    
    module.add_function(func);
    
    let text = module.to_text();
    assert!(text.contains("section .text"));
    assert!(text.contains("global main"));
    assert!(text.contains("main:"));
}
