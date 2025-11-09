// Complete pipeline test: MIR -> AIR -> Assembly -> Linking -> Executable
extern crate aurora_mir;
extern crate aurora_air;
extern crate aurora_backend;
extern crate aurora_types;

use aurora_mir::{MirModule, Function, BasicBlock, Instruction, Operand, Constant, Span};
use aurora_types::{Type, EffectSet};
use aurora_air::lower_mir_to_air;
use aurora_backend::{generate_code, CodegenOptions};
use std::sync::Arc;
use std::path::PathBuf;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Complete Aurora Compilation Pipeline Test ===\n");

    // Step 1: Create MIR (simulating what AST->MIR lowering would do)
    println!("Step 1: Creating MIR...");
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

    println!("✓ MIR created with {} function(s)\n", mir.function_count());

    // Step 2: Lower MIR to AIR
    println!("Step 2: Lowering MIR to AIR...");
    let air = lower_mir_to_air(mir, Arc::new(()));
    println!("✓ AIR generated\n");

    // Step 3: Generate code and link
    println!("Step 3: Generating machine code and linking...");
    let output_path = PathBuf::from("hello_world_complete");

    let options = CodegenOptions {
        output_path: output_path.clone(),
        opt_level: 0,
        debug_info: false,
        emit_llvm: true,  // Also write assembly file
        keep_intermediates: true,
        codegen_units: 1,
        target_triple: Some("x86_64-unknown-linux-gnu".to_string()),
    };

    generate_code(air, options, Arc::new(()))?;
    println!("✓ Executable generated: {}\n", output_path.display());

    // Step 4: Run the executable
    println!("Step 4: Running the executable...");
    println!("--- Output ---");
    let output = Command::new(format!("./{}", output_path.display()))
        .output()?;

    print!("{}", String::from_utf8_lossy(&output.stdout));
    print!("{}", String::from_utf8_lossy(&output.stderr));
    println!("--- End Output ---\n");

    if output.status.success() {
        println!("✓ Program executed successfully!");
        println!("\n=== PIPELINE TEST PASSED ===");
        Ok(())
    } else {
        return Err(format!("Program failed with exit code: {:?}", output.status.code()).into());
    }
}
