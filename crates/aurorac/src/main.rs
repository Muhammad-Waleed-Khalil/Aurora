//! Aurora Compiler Driver
//!
//! Main entry point for the Aurora compiler (aurorac).
//! Orchestrates all compiler phases and enforces agent boundaries.

use anyhow::Result;
use aurorac::{compile_file, check_file, CompilationOptions};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "aurorac")]
#[command(about = "Aurora Compiler - Compile Aurora (.ax) source files", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Input file to compile
    #[arg(value_name = "FILE")]
    input: Option<PathBuf>,

    /// Output file path
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,

    /// Emit MIR dump
    #[arg(long)]
    emit_mir: bool,

    /// Emit AIR dump
    #[arg(long)]
    emit_air: bool,

    /// Emit AST dump
    #[arg(long)]
    emit_ast: bool,

    /// Emit LLVM IR
    #[arg(long)]
    emit_llvm: bool,

    /// Optimization level (0-3)
    #[arg(short = 'O', default_value = "0")]
    opt_level: u8,

    /// Enable verbose output
    #[arg(short = 'v', long)]
    verbose: bool,

    /// Stop after type checking
    #[arg(long)]
    type_check_only: bool,

    /// Number of codegen units (parallel compilation)
    #[arg(long, default_value = "1")]
    codegen_units: usize,

    /// Emit debug information
    #[arg(long, default_value = "true")]
    debug_info: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Check syntax without compiling
    Check {
        /// File to check
        file: PathBuf,

        /// Verbose output
        #[arg(short = 'v', long)]
        verbose: bool,
    },
    /// Build the project
    Build {
        /// Release mode (optimization level 3)
        #[arg(long)]
        release: bool,

        /// Verbose output
        #[arg(short = 'v', long)]
        verbose: bool,
    },
    /// Show version information
    Version,
}

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Check { file, verbose }) => {
            println!("Checking: {}", file.display());

            let opts = CompilationOptions {
                input: file,
                verbose,
                ..CompilationOptions::new("dummy")
            };

            check_file(opts)?;
            println!("✓ Syntax check passed");
            Ok(())
        }
        Some(Commands::Build { release, verbose }) => {
            println!("Building in {} mode", if release { "release" } else { "debug" });

            // For now, assume we're building main.ax or src/main.ax
            let input = if PathBuf::from("main.ax").exists() {
                PathBuf::from("main.ax")
            } else if PathBuf::from("src/main.ax").exists() {
                PathBuf::from("src/main.ax")
            } else {
                anyhow::bail!("Could not find main.ax or src/main.ax");
            };

            let opts = CompilationOptions {
                input,
                opt_level: if release { 3 } else { 0 },
                verbose,
                debug_info: !release,
                ..CompilationOptions::new("dummy")
            };

            compile_file(opts)?;
            println!("✓ Build successful");
            Ok(())
        }
        Some(Commands::Version) => {
            println!("aurorac version {}", env!("CARGO_PKG_VERSION"));
            println!("Aurora Programming Language Compiler");
            Ok(())
        }
        None => {
            if let Some(input) = cli.input {
                println!("Compiling: {}", input.display());

                let opts = CompilationOptions {
                    input,
                    output: cli.output,
                    opt_level: cli.opt_level,
                    emit_mir: cli.emit_mir,
                    emit_air: cli.emit_air,
                    emit_ast: cli.emit_ast,
                    emit_llvm: cli.emit_llvm,
                    type_check_only: cli.type_check_only,
                    verbose: cli.verbose,
                    codegen_units: cli.codegen_units,
                    debug_info: cli.debug_info,
                };

                compile_file(opts)?;
                println!("✓ Compilation successful");
                Ok(())
            } else {
                println!("No input file specified. Use --help for usage information.");
                println!("\nUsage:");
                println!("  aurorac <FILE>           Compile a file");
                println!("  aurorac check <FILE>     Check syntax");
                println!("  aurorac build            Build project");
                println!("  aurorac --help           Show full help");
                Ok(())
            }
        }
    }
}
