//! Compilation Pipeline
//!
//! Orchestrates the complete compilation flow:
//! Source → Lexer → Parser → AST → Name Resolution → Type Checking →
//! Effects → MIR → Optimization → AIR → Code Generation → Linking

use crate::session::{CompilationSession, PhaseResult};
use anyhow::{Context, Result};
use aurora_air::AirModule;
use aurora_ast::Ast;
use aurora_backend::CodegenOptions;
use aurora_diagnostics::{Diagnostic, DiagnosticLevel};
use aurora_effects::EffectChecker;
use aurora_lexer::Lexer;
use aurora_mir::MirModule;
use aurora_nameres::NameResolver;
use aurora_parser::Parser;
use aurora_types::TypeChecker;
use std::fs;
use tracing::{debug, info, warn};

/// The main compilation pipeline
pub struct Pipeline<'sess> {
    session: &'sess mut CompilationSession,
}

impl<'sess> Pipeline<'sess> {
    /// Create a new pipeline for the given session
    pub fn new(session: &'sess mut CompilationSession) -> Self {
        Self { session }
    }

    /// Run the complete compilation pipeline
    pub fn compile(&mut self) -> Result<()> {
        info!("Starting compilation of {}", self.session.source_name());

        // Phase 1: Lexical Analysis
        let tokens = self.lex()?;

        // Phase 2: Parsing
        let ast = self.parse(tokens)?;

        // Phase 3: Name Resolution
        let resolved_ast = self.resolve_names(ast)?;

        // Phase 4: Type Checking
        let typed_ast = self.type_check(resolved_ast)?;

        // Phase 5: Effect Checking
        let checked_ast = self.check_effects(typed_ast)?;

        // Check for errors before continuing to backend
        self.session.check_errors()?;

        // Phase 6: MIR Generation and Optimization
        let mir = self.lower_to_mir(checked_ast)?;

        // Phase 7: AIR Generation
        let air = self.lower_to_air(mir)?;

        // Phase 8: Code Generation
        self.codegen(air)?;

        info!("Compilation successful");
        Ok(())
    }

    /// Phase 1: Lexical analysis
    fn lex(&mut self) -> Result<Vec<aurora_lexer::Token>> {
        info!("Phase 1: Lexical analysis");

        let lexer = Lexer::new(&self.session.source, self.session.diagnostics.clone());
        let tokens = lexer.tokenize();

        if self.session.options.verbose {
            debug!("Lexed {} tokens", tokens.len());
        }

        self.session.check_errors()?;
        Ok(tokens)
    }

    /// Phase 2: Parsing
    fn parse(&mut self, tokens: Vec<aurora_lexer::Token>) -> Result<Ast> {
        info!("Phase 2: Parsing");

        let parser = Parser::new(tokens, self.session.diagnostics.clone());
        let ast = parser.parse();

        if self.session.options.verbose {
            debug!("Parsed AST with {} nodes", ast.node_count());
        }

        if self.session.options.emit_ast {
            self.dump_ast(&ast)?;
        }

        self.session.check_errors()?;
        Ok(ast)
    }

    /// Phase 3: Name resolution
    fn resolve_names(&mut self, ast: Ast) -> Result<Ast> {
        info!("Phase 3: Name resolution");

        let mut resolver = NameResolver::new(self.session.diagnostics.clone());
        let resolved = resolver.resolve(ast);

        if self.session.options.verbose {
            debug!("Resolved {} symbols", resolver.symbol_count());
        }

        self.session.check_errors()?;
        Ok(resolved)
    }

    /// Phase 4: Type checking
    fn type_check(&mut self, ast: Ast) -> Result<Ast> {
        info!("Phase 4: Type checking");

        let mut checker = TypeChecker::new(self.session.diagnostics.clone());
        let typed = checker.check(ast);

        if self.session.options.verbose {
            debug!("Type checked successfully");
        }

        self.session.check_errors()?;

        if self.session.options.type_check_only {
            info!("Stopping after type checking (--type-check-only)");
            std::process::exit(0);
        }

        Ok(typed)
    }

    /// Phase 5: Effect checking
    fn check_effects(&mut self, ast: Ast) -> Result<Ast> {
        info!("Phase 5: Effect checking");

        let mut checker = EffectChecker::new(self.session.diagnostics.clone());
        let checked = checker.check(ast);

        if self.session.options.verbose {
            debug!("Effect checking complete");
        }

        self.session.check_errors()?;
        Ok(checked)
    }

    /// Phase 6: Lower to MIR and optimize
    fn lower_to_mir(&mut self, ast: Ast) -> Result<MirModule> {
        info!("Phase 6: MIR lowering and optimization");

        let mir = aurora_mir::lower_ast_to_mir(ast, self.session.diagnostics.clone());

        if self.session.options.verbose {
            debug!("Generated MIR with {} functions", mir.function_count());
        }

        // Optimize MIR based on opt level
        let optimized = if self.session.options.opt_level > 0 {
            info!("Running MIR optimizations (level {})", self.session.options.opt_level);
            aurora_mir::optimize(mir, self.session.options.opt_level)
        } else {
            mir
        };

        if self.session.options.emit_mir {
            self.dump_mir(&optimized)?;
        }

        self.session.check_errors()?;
        Ok(optimized)
    }

    /// Phase 7: Lower to AIR
    fn lower_to_air(&mut self, mir: MirModule) -> Result<AirModule> {
        info!("Phase 7: AIR lowering");

        let air = aurora_air::lower_mir_to_air(mir, self.session.diagnostics.clone());

        if self.session.options.verbose {
            debug!("Generated AIR");
        }

        if self.session.options.emit_air {
            self.dump_air(&air)?;
        }

        self.session.check_errors()?;
        Ok(air)
    }

    /// Phase 8: Code generation
    fn codegen(&mut self, air: AirModule) -> Result<()> {
        info!("Phase 8: Code generation");

        let codegen_opts = CodegenOptions {
            opt_level: self.session.options.opt_level,
            debug_info: self.session.options.debug_info,
            emit_llvm: self.session.options.emit_llvm,
            codegen_units: self.session.options.codegen_units,
            output_path: self.session.options.output_path(),
        };

        aurora_backend::generate_code(
            air,
            codegen_opts,
            self.session.diagnostics.clone(),
        )?;

        if self.session.options.emit_llvm {
            let llvm_path = self.session.options.llvm_dump_path();
            info!("LLVM IR written to {}", llvm_path.display());
        }

        info!("Binary written to {}", self.session.options.output_path().display());

        self.session.check_errors()?;
        Ok(())
    }

    /// Dump AST to file
    fn dump_ast(&self, ast: &Ast) -> Result<()> {
        let path = self.session.options.ast_dump_path();
        let dump = format!("{:#?}", ast);
        fs::write(&path, dump)
            .with_context(|| format!("Failed to write AST dump to {}", path.display()))?;
        info!("AST dump written to {}", path.display());
        Ok(())
    }

    /// Dump MIR to file
    fn dump_mir(&self, mir: &MirModule) -> Result<()> {
        let path = self.session.options.mir_dump_path();
        let dump = mir.to_string();
        fs::write(&path, dump)
            .with_context(|| format!("Failed to write MIR dump to {}", path.display()))?;
        info!("MIR dump written to {}", path.display());
        Ok(())
    }

    /// Dump AIR to file
    fn dump_air(&self, air: &AirModule) -> Result<()> {
        let path = self.session.options.air_dump_path();
        let dump = air.to_string();
        fs::write(&path, dump)
            .with_context(|| format!("Failed to write AIR dump to {}", path.display()))?;
        info!("AIR dump written to {}", path.display());
        Ok(())
    }
}

/// Run a quick syntax check without full compilation
pub fn check_syntax(session: &mut CompilationSession) -> Result<()> {
    info!("Running syntax check on {}", session.source_name());

    // Just lex and parse
    let lexer = Lexer::new(&session.source, session.diagnostics.clone());
    let tokens = lexer.tokenize();

    session.check_errors()?;

    let parser = Parser::new(tokens, session.diagnostics.clone());
    let _ast = parser.parse();

    session.check_errors()?;

    info!("Syntax check passed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::CompilationOptions;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_file(content: &str) -> Result<NamedTempFile> {
        let mut file = NamedTempFile::new()?;
        file.write_all(content.as_bytes())?;
        file.flush()?;
        Ok(file)
    }

    #[test]
    fn test_pipeline_creation() -> Result<()> {
        let file = create_test_file("fn main() {}")?;
        let opts = CompilationOptions::new(file.path());
        let mut session = CompilationSession::new(opts)?;
        let _pipeline = Pipeline::new(&mut session);
        Ok(())
    }

    #[test]
    fn test_lex_phase() -> Result<()> {
        let file = create_test_file("fn main() { let x = 42; }")?;
        let opts = CompilationOptions::new(file.path());
        let mut session = CompilationSession::new(opts)?;
        let mut pipeline = Pipeline::new(&mut session);

        let tokens = pipeline.lex()?;
        assert!(tokens.len() > 0);

        Ok(())
    }
}
