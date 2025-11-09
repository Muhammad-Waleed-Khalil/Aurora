// Direct MIR -> AIR integration test
// Compile with: rustc --edition 2021 -L target/release/deps test_mir_air_integration.rs

extern crate aurora_mir;
extern crate aurora_air;
extern crate aurora_types;

use aurora_mir::{MirModule, Function, BasicBlock, Instruction, Operand, Constant, Span};
use aurora_types::{Type, EffectSet};
use aurora_air::lower_mir_to_air;
use std::sync::Arc;

fn main() {
    println!("=== MIR -> AIR Integration Test ===\n");

    // Create MIR manually
    println!("Creating MIR...");
    let mut mir = MirModule::new();

    let mut main_func = Function::new(0, "main".to_string(), Type::Unit, EffectSet::IO);
    let mut entry_block = BasicBlock::new(0);
    let span = Span::dummy();

    // println("Hello, World!")
    entry_block.push(Instruction::Call {
        dest: None,
        func: Operand::Const(Constant::String("aurora_println".to_string())),
        args: vec![Operand::Const(Constant::String("Hello, World!".to_string()))],
        effects: EffectSet::IO,
        span,
    });

    // println("Welcome to Aurora!")
    entry_block.push(Instruction::Call {
        dest: None,
        func: Operand::Const(Constant::String("aurora_println".to_string())),
        args: vec![Operand::Const(Constant::String("Welcome to Aurora!".to_string()))],
        effects: EffectSet::IO,
        span,
    });

    // return
    entry_block.push(Instruction::Return { value: None, span });

    main_func.add_block(entry_block);
    main_func.entry = 0;
    mir.add_function(main_func);

    println!("✓ MIR: {} function(s)\n", mir.function_count());
    println!("{}", mir.to_string());

    // Lower to AIR
    println!("\nLowering to AIR...");
    let air = lower_mir_to_air(mir, Arc::new(()));
    println!("✓ AIR generated\n");
    println!("{}", air.to_string());

    println!("\n✓ Integration test passed!");
}
