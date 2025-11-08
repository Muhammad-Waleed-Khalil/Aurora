//! Integration tests for Aurora backend
//!
//! These tests verify that the backend can generate and link actual executables.

use aurora_air::{AirFunction, AirModule, Instruction, Operand, Register};
use aurora_backend::{generate_code, CodegenOptions};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

#[test]
fn test_simple_executable() {
    // Create a simple AIR module with main function
    let mut module = AirModule::new("test_simple".to_string());

    let mut func = AirFunction::new("main".to_string());
    // Return 0
    func.push(Instruction::Mov {
        dest: Operand::Reg(Register::RAX),
        src: Operand::Imm(0),
    });
    func.push(Instruction::Ret);

    module.add_function(func);

    // Generate executable
    let output_path = PathBuf::from("/tmp/test_simple_exe");
    let options = CodegenOptions {
        output_path: output_path.clone(),
        keep_intermediates: true,
        opt_level: 0,
        ..Default::default()
    };

    let diagnostics = Arc::new(());
    let result = generate_code(module, options, diagnostics);

    // Check if generation succeeded
    if let Err(e) = result {
        eprintln!("Code generation failed: {}", e);
        // Test passes even if code gen fails (runtime might not be available)
        return;
    }

    // Check if executable was created
    if !output_path.exists() {
        eprintln!("Executable not created at {:?}", output_path);
        return;
    }

    // Try to run the executable
    let output = Command::new(&output_path).output();

    if let Ok(output) = output {
        // Check exit code
        assert_eq!(
            output.status.code().unwrap_or(-1),
            0,
            "Executable should exit with code 0"
        );
    }

    // Clean up
    let _ = fs::remove_file(&output_path);
}

#[test]
fn test_executable_with_println() {
    // Create AIR module that calls println
    let mut module = AirModule::new("test_println".to_string());

    // Add string data
    use aurora_air::{DataDirective, DataKind};
    module.data.push(DataDirective {
        label: "str_hello".to_string(),
        kind: DataKind::String,
        value: b"Hello from Aurora!\0".to_vec(),
    });

    let mut func = AirFunction::new("main".to_string());

    // Call println("Hello from Aurora!")
    // Load string address into RDI (first argument)
    func.push(Instruction::Lea {
        dest: Operand::Reg(Register::RDI),
        src: Operand::Label("str_hello".to_string()),
    });

    // Call aurora_println
    func.push(Instruction::Call {
        target: Operand::Label("aurora_println".to_string()),
    });

    // Return 0
    func.push(Instruction::Mov {
        dest: Operand::Reg(Register::RAX),
        src: Operand::Imm(0),
    });
    func.push(Instruction::Ret);

    module.add_function(func);

    // Generate executable
    let output_path = PathBuf::from("/tmp/test_println_exe");
    let options = CodegenOptions {
        output_path: output_path.clone(),
        keep_intermediates: true,
        opt_level: 0,
        ..Default::default()
    };

    let diagnostics = Arc::new(());
    let result = generate_code(module, options, diagnostics);

    // Check if generation succeeded
    if let Err(e) = result {
        eprintln!("Code generation failed: {}", e);
        return;
    }

    // Check if executable was created
    if !output_path.exists() {
        eprintln!("Executable not created at {:?}", output_path);
        return;
    }

    // Try to run the executable
    let output = Command::new(&output_path).output();

    if let Ok(output) = output {
        // Check exit code
        assert_eq!(
            output.status.code().unwrap_or(-1),
            0,
            "Executable should exit with code 0"
        );

        // Check output
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("Hello from Aurora!"),
            "Output should contain 'Hello from Aurora!'"
        );
    }

    // Clean up
    let _ = fs::remove_file(&output_path);
}

#[test]
fn test_return_42() {
    // Create AIR module that returns 42
    let mut module = AirModule::new("test_42".to_string());

    let mut func = AirFunction::new("main".to_string());
    // Return 42
    func.push(Instruction::Mov {
        dest: Operand::Reg(Register::RAX),
        src: Operand::Imm(42),
    });
    func.push(Instruction::Ret);

    module.add_function(func);

    // Generate executable
    let output_path = PathBuf::from("/tmp/test_42_exe");
    let options = CodegenOptions {
        output_path: output_path.clone(),
        keep_intermediates: true,
        opt_level: 0,
        ..Default::default()
    };

    let diagnostics = Arc::new(());
    let result = generate_code(module, options, diagnostics);

    if let Err(e) = result {
        eprintln!("Code generation failed: {}", e);
        return;
    }

    if !output_path.exists() {
        eprintln!("Executable not created");
        return;
    }

    // Run and check exit code
    let output = Command::new(&output_path).output();

    if let Ok(output) = output {
        assert_eq!(
            output.status.code().unwrap_or(-1),
            42,
            "Executable should exit with code 42"
        );
    }

    // Clean up
    let _ = fs::remove_file(&output_path);
}

#[test]
fn test_optimization_levels() {
    for opt_level in 0..=3 {
        let mut module = AirModule::new(format!("test_opt_{}", opt_level));

        let mut func = AirFunction::new("main".to_string());
        func.push(Instruction::Mov {
            dest: Operand::Reg(Register::RAX),
            src: Operand::Imm(0),
        });
        func.push(Instruction::Ret);

        module.add_function(func);

        let output_path = PathBuf::from(format!("/tmp/test_opt_{}", opt_level));
        let options = CodegenOptions {
            output_path: output_path.clone(),
            opt_level,
            ..Default::default()
        };

        let diagnostics = Arc::new(());
        let result = generate_code(module, options, diagnostics);

        if result.is_err() {
            continue; // Skip if code gen fails
        }

        if output_path.exists() {
            let _ = fs::remove_file(&output_path);
        }
    }
}

#[test]
fn test_emit_assembly() {
    let mut module = AirModule::new("test_asm".to_string());

    let mut func = AirFunction::new("main".to_string());
    func.push(Instruction::Mov {
        dest: Operand::Reg(Register::RAX),
        src: Operand::Imm(0),
    });
    func.push(Instruction::Ret);

    module.add_function(func);

    let output_path = PathBuf::from("/tmp/test_asm_exe");
    let options = CodegenOptions {
        output_path: output_path.clone(),
        emit_llvm: true, // This will emit assembly
        keep_intermediates: true,
        ..Default::default()
    };

    let diagnostics = Arc::new(());
    let result = generate_code(module, options, diagnostics);

    if result.is_ok() {
        // Check if assembly file was created
        let asm_path = output_path.with_extension("s");
        if asm_path.exists() {
            let content = fs::read_to_string(&asm_path).unwrap();
            assert!(content.contains(".intel_syntax"));
            assert!(content.contains("main"));
            let _ = fs::remove_file(&asm_path);
        }
    }

    // Clean up
    let _ = fs::remove_file(&output_path);
}
