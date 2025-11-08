//! Aurora MIR (Mid-Level Intermediate Representation)
//!
//! This crate provides the MIR for Aurora compiler:
//! - SSA form representation
//! - Control Flow Graph (CFG)
//! - Dominance tree computation
//! - MIR lowering from typed AST
//! - Optimization passes
//! - MIR dumps and serialization

pub mod cfg;
pub mod dump;
pub mod lower;
pub mod mir;
pub mod opt;

pub use cfg::{DominatorTree, Loop, CFG};
pub use dump::MirDumper;
pub use lower::MirBuilder;
pub use mir::*;
pub use opt::*;

// Pipeline integration stubs
use aurora_ast::Ast;
use std::sync::Arc;
use std::collections::HashMap;

/// MIR Module (collection of functions)
#[derive(Debug, Clone)]
pub struct MirModule {
    /// Functions in this module
    pub functions: HashMap<FunctionId, Function>,
}

impl MirModule {
    /// Create new empty module
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    /// Add function to module
    pub fn add_function(&mut self, func: Function) {
        self.functions.insert(func.id, func);
    }

    /// Get function by ID
    pub fn get_function(&self, id: FunctionId) -> Option<&Function> {
        self.functions.get(&id)
    }

    /// Get number of functions
    pub fn function_count(&self) -> usize {
        self.functions.len()
    }

    /// Convert to string (for dumping)
    pub fn to_string(&self) -> String {
        let mut dumper = MirDumper::new();
        let mut output = String::new();

        output.push_str(&format!("// MIR Module with {} functions\n\n", self.function_count()));

        for func in self.functions.values() {
            output.push_str(&dumper.dump_function(func));
            output.push('\n');
        }

        output
    }
}

impl Default for MirModule {
    fn default() -> Self {
        Self::new()
    }
}

/// Lower AST to MIR
pub fn lower_ast_to_mir<D: Send + Sync + 'static>(
    ast: Ast,
    diagnostics: Arc<D>,
) -> MirModule {
    use aurora_types::TypeMap;

    // Create type map (would come from type checker in real implementation)
    let type_map = TypeMap::new();

    // Create lowering context
    let mut ctx = lower::LoweringContext::new(diagnostics, type_map);

    // Lower the AST
    ctx.lower(ast)
}

/// Optimize MIR module
pub fn optimize(mut mir: MirModule, opt_level: u8) -> MirModule {
    // Create optimization pipeline
    let level = opt::OptLevel::from_u8(opt_level);
    let mut pipeline = opt::OptPipeline::new(level);

    // Run optimization on each function
    for func in mir.functions.values_mut() {
        pipeline.run(func);
    }

    mir
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_types::{EffectSet, Type};

    #[test]
    fn test_mir_module_creation() {
        let module = MirModule::new();
        assert_eq!(module.function_count(), 0);
    }

    #[test]
    fn test_mir_module_add_function() {
        let mut module = MirModule::new();
        let func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);
        module.add_function(func);
        assert_eq!(module.function_count(), 1);
    }

    #[test]
    fn test_mir_module_get_function() {
        let mut module = MirModule::new();
        let func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);
        module.add_function(func);

        assert!(module.get_function(0).is_some());
        assert!(module.get_function(1).is_none());
    }

    #[test]
    fn test_mir_module_to_string() {
        let mut module = MirModule::new();
        let func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);
        module.add_function(func);

        let output = module.to_string();
        assert!(output.contains("MIR Module with 1 functions"));
        assert!(output.contains("fn test"));
    }

    #[test]
    fn test_lower_empty_ast() {
        let ast = Ast::empty();
        let diagnostics = Arc::new(());
        let mir = lower_ast_to_mir(ast, diagnostics);
        assert_eq!(mir.function_count(), 0);
    }

    #[test]
    fn test_optimize_level_0() {
        let module = MirModule::new();
        let optimized = optimize(module, 0);
        assert_eq!(optimized.function_count(), 0);
    }

    #[test]
    fn test_optimize_level_1() {
        let mut module = MirModule::new();
        let func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);
        module.add_function(func);

        let optimized = optimize(module, 1);
        assert_eq!(optimized.function_count(), 1);
    }

    #[test]
    fn test_optimize_level_2() {
        let mut module = MirModule::new();
        let func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);
        module.add_function(func);

        let optimized = optimize(module, 2);
        assert_eq!(optimized.function_count(), 1);
    }

    #[test]
    fn test_optimize_level_3() {
        let mut module = MirModule::new();
        let func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);
        module.add_function(func);

        let optimized = optimize(module, 3);
        assert_eq!(optimized.function_count(), 1);
    }

    #[test]
    fn test_mir_module_default() {
        let module = MirModule::default();
        assert_eq!(module.function_count(), 0);
    }

    #[test]
    fn test_function_with_params() {
        use aurora_types::PrimitiveType;

        let mut func = Function::new(0, "add".to_string(), Type::Primitive(PrimitiveType::I32), EffectSet::PURE);
        func.params = vec![0, 1];

        assert_eq!(func.params.len(), 2);
    }

    #[test]
    fn test_function_with_blocks() {
        let mut func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);
        let block = BasicBlock::new(0);
        func.add_block(block);

        assert!(func.block(0).is_some());
        assert!(func.block(1).is_none());
    }

    #[test]
    fn test_function_with_values() {
        let func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);
        assert_eq!(func.values.len(), 0);
    }

    #[test]
    fn test_mir_serialization() {
        let func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);
        let dumper = MirDumper::new();
        let json = dumper.to_json(&func);
        assert!(json.is_ok());
    }

    #[test]
    fn test_multiple_functions() {
        let mut module = MirModule::new();

        for i in 0..5 {
            let func = Function::new(i, format!("func{}", i), Type::Unit, EffectSet::PURE);
            module.add_function(func);
        }

        assert_eq!(module.function_count(), 5);
    }
}
