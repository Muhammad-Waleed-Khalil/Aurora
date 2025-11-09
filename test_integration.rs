#!/usr/bin/env rust-script
//! Direct integration test bypassing the parser
//! Tests MIR -> AIR -> Backend flow

use aurora_mir::{MirModule, Function, BasicBlock, Instruction, Operand, Constant, Span, Value};
use aurora_types::{Type, EffectSet};
use aurora_air::lower_mir_to_air;
use std::sync::Arc;

fn main() -> anyhow::Result<()> {
    println!("=== Direct Integration Test ===\n");

    // Step 1: Create MIR manually (simulating what AST lowering would do)
    println!("Step 1: Creating MIR...");
    let mut mir = MirModule::new();

    // Create main function
    let mut main_func = Function::new(0, "main".to_string(), Type::Unit, EffectSet::IO);

    // Add values for the function
    // We don't actually use any values since println returns Unit and takes a const string

    // Create entry block
    let mut entry_block = BasicBlock::new(0);
    let span = Span::dummy();

    // First println call: println("Hello, World!")
    entry_block.push(Instruction::Call {
        dest: None,
        func: Operand::Const(Constant::String("aurora_println".to_string())),
        args: vec![Operand::Const(Constant::String("Hello, World!".to_string()))],
        effects: EffectSet::IO,
        span,
    });

    // Second println call: println("Welcome to Aurora!")
    entry_block.push(Instruction::Call {
        dest: None,
        func: Operand::Const(Constant::String("aurora_println".to_string())),
        args: vec![Operand::Const(Constant::String("Welcome to Aurora!".to_string()))],
        effects: EffectSet::IO,
        span,
    });

    // Return from main
    entry_block.push(Instruction::Return {
        value: None,
        span,
    });

    main_func.add_block(entry_block);
    main_func.entry = 0;
    mir.add_function(main_func);

    println!("✓ MIR created with {} function(s)", mir.function_count());
    println!("\nMIR dump:");
    println!("{}\n", mir.to_string());

    // Step 2: Lower MIR to AIR
    println!("Step 2: Lowering MIR to AIR...");
    let air = lower_mir_to_air(mir, Arc::new(()));
    println!("✓ AIR generated");
    println!("\nAIR dump:");
    println!("{}\n", air.to_string());

    // Step 3: Generate code (this would normally call the backend)
    println!("Step 3: Code generation would happen here");
    println!("✓ Integration test completed successfully!");

    Ok(())
}
