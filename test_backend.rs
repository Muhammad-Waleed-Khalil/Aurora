use aurora_air::{AirFunction, AirModule, Instruction, Operand, Register};
use aurora_backend::{generate_code, CodegenOptions};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

fn main() {
    println!("Creating simple AIR module...");

    // Create a simple AIR module with main function that returns 0
    let mut module = AirModule::new("test".to_string());

    let mut func = AirFunction::new("main".to_string());
    // Return 0
    func.push(Instruction::Mov {
        dest: Operand::Reg(Register::RAX),
        src: Operand::Imm(0),
    });
    func.push(Instruction::Ret);

    module.add_function(func);

    // Print AIR
    println!("\n=== Generated AIR ===");
    println!("{}", module.to_text());

    // Generate executable
    let output_path = PathBuf::from("/tmp/test_backend_main");
    let options = CodegenOptions {
        output_path: output_path.clone(),
        keep_intermediates: true,
        emit_llvm: true, // Also emit assembly for inspection
        opt_level: 0,
        ..Default::default()
    };

    let diagnostics = Arc::new(());

    println!("\n=== Generating code ===");
    match generate_code(module, options, diagnostics) {
        Ok(()) => println!("Code generation succeeded!"),
        Err(e) => {
            eprintln!("Code generation failed: {}", e);
            std::process::exit(1);
        }
    }

    // Check if assembly was generated
    let asm_path = output_path.with_extension("s");
    if asm_path.exists() {
        println!("\n=== Generated Assembly ===");
        match std::fs::read_to_string(&asm_path) {
            Ok(content) => println!("{}", content),
            Err(e) => eprintln!("Failed to read assembly: {}", e),
        }
    }

    // Check if executable was created
    if !output_path.exists() {
        eprintln!("Executable not created at {:?}", output_path);
        std::process::exit(1);
    }

    println!("\n=== Running executable ===");
    // Try to run the executable
    match Command::new(&output_path).output() {
        Ok(output) => {
            println!("Exit code: {}", output.status.code().unwrap_or(-1));
            println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
            println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
        Err(e) => {
            eprintln!("Failed to run executable: {}", e);
            std::process::exit(1);
        }
    }

    println!("\n=== Success! ===");
}
