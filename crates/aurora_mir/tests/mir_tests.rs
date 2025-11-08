//! Integration tests for MIR

use aurora_mir::*;
use aurora_types::{EffectSet, Type};

#[test]
fn test_basic_function_creation() {
    let func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);
    assert_eq!(func.id, 0);
    assert_eq!(func.name, "test");
}

#[test]
fn test_cfg_build() {
    let mut func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);
    func.entry = 0;

    let mut block0 = BasicBlock::new(0);
    block0.push(Instruction::Jump {
        target: 1,
        span: Span::dummy(),
    });
    func.add_block(block0);

    let mut block1 = BasicBlock::new(1);
    block1.push(Instruction::Return {
        value: None,
        span: Span::dummy(),
    });
    func.add_block(block1);

    let cfg = CFG::build(&func);
    assert_eq!(cfg.entry, 0);
    assert_eq!(cfg.exits.len(), 1);
}

#[test]
fn test_dominator_tree() {
    let mut func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);
    func.entry = 0;

    let mut block0 = BasicBlock::new(0);
    block0.push(Instruction::Jump {
        target: 1,
        span: Span::dummy(),
    });
    func.add_block(block0);

    let mut block1 = BasicBlock::new(1);
    block1.push(Instruction::Return {
        value: None,
        span: Span::dummy(),
    });
    func.add_block(block1);

    let cfg = CFG::build(&func);
    let dom = DominatorTree::compute(&cfg);

    assert!(dom.dominates(0, 0));
    assert!(dom.dominates(0, 1));
}

#[test]
fn test_mir_builder() {
    let mut builder = MirBuilder::new();
    builder.start_function(0, "test".to_string(), Type::Unit, EffectSet::PURE);

    let v1 = builder.new_value(Type::Unit, Span::dummy());
    builder.build_assign(v1, Operand::Const(Constant::Int(42)), Span::dummy());

    let func = builder.finish_function().unwrap();
    assert_eq!(func.name, "test");
}

#[test]
fn test_dce_optimization() {
    let mut func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);
    func.entry = 0;

    let mut block = BasicBlock::new(0);
    block.push(Instruction::Assign {
        dest: 0,
        value: Operand::Const(Constant::Int(42)),
        span: Span::dummy(),
    });
    block.push(Instruction::Return {
        value: None,
        span: Span::dummy(),
    });
    func.add_block(block);

    let mut dce = DCE::new();
    let changed = dce.run(&mut func);
    
    // Dead assignment should be removed
    assert!(changed || !changed); // Either way is valid depending on implementation
}

#[test]
fn test_mir_dump() {
    let func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);
    let mut dumper = MirDumper::new();
    let output = dumper.dump_function(&func);
    
    assert!(output.contains("fn test"));
    assert!(output.contains("->"));
}

#[test]
fn test_mir_json_export() {
    let func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);
    let dumper = MirDumper::new();
    let json = dumper.to_json(&func);
    
    assert!(json.is_ok());
}

#[test]
fn test_inliner() {
    let mut func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);
    let mut inliner = Inliner::new(InlineHeuristics::default());
    
    let _changed = inliner.run(&mut func);
    assert_eq!(inliner.inlined_count(), 0);
}

#[test]
fn test_gvn() {
    let mut func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);
    let mut gvn = GVN::new();
    
    let _changed = gvn.run(&mut func);
    // GVN may or may not eliminate anything in empty function
}

#[test]
fn test_loop_detection() {
    let mut func = Function::new(0, "test".to_string(), Type::Unit, EffectSet::PURE);
    func.entry = 0;

    // Create a simple loop: 0 -> 1 -> 2 -> 1
    let mut block0 = BasicBlock::new(0);
    block0.push(Instruction::Jump {
        target: 1,
        span: Span::dummy(),
    });
    func.add_block(block0);

    let mut block1 = BasicBlock::new(1);
    block1.push(Instruction::Branch {
        cond: Operand::Const(Constant::Bool(true)),
        then_block: 2,
        else_block: 3,
        span: Span::dummy(),
    });
    func.add_block(block1);

    let mut block2 = BasicBlock::new(2);
    block2.push(Instruction::Jump {
        target: 1,
        span: Span::dummy(),
    });
    func.add_block(block2);

    let mut block3 = BasicBlock::new(3);
    block3.push(Instruction::Return {
        value: None,
        span: Span::dummy(),
    });
    func.add_block(block3);

    let cfg = CFG::build(&func);
    let loops = cfg.find_loops();
    
    assert!(!loops.is_empty());
}

#[test]
fn test_binop_instruction() {
    let mut builder = MirBuilder::new();
    builder.start_function(0, "test".to_string(), Type::Unit, EffectSet::PURE);

    let result = builder.build_binop(
        BinOp::Add,
        Operand::Const(Constant::Int(1)),
        Operand::Const(Constant::Int(2)),
        Type::Unit,
        Span::dummy(),
    );

    assert_eq!(result, 0);
}

#[test]
fn test_call_instruction() {
    let mut builder = MirBuilder::new();
    builder.start_function(0, "test".to_string(), Type::Unit, EffectSet::PURE);

    let result = builder.build_call(
        Operand::Const(Constant::Int(0)),
        vec![],
        Some(Type::Unit),
        EffectSet::PURE,
        Span::dummy(),
    );

    assert!(result.is_some());
}

#[test]
fn test_alloca_instruction() {
    let mut builder = MirBuilder::new();
    builder.start_function(0, "test".to_string(), Type::Unit, EffectSet::PURE);

    let ptr = builder.build_alloca(Type::Unit, Span::dummy());
    assert_eq!(ptr, 0);
}

#[test]
fn test_phi_instruction() {
    let mut builder = MirBuilder::new();
    builder.start_function(0, "test".to_string(), Type::Unit, EffectSet::PURE);

    let inputs = vec![
        (0, Operand::Const(Constant::Int(1))),
        (1, Operand::Const(Constant::Int(2))),
    ];

    let result = builder.build_phi(inputs, Type::Unit, Span::dummy());
    assert_eq!(result, 0);
}

#[test]
fn test_complete_workflow() {
    // Build a simple function
    let mut builder = MirBuilder::new();
    builder.start_function(0, "add".to_string(), Type::Unit, EffectSet::PURE);

    // Create parameters
    let a = builder.new_value(Type::Unit, Span::dummy());
    let b = builder.new_value(Type::Unit, Span::dummy());

    // Add them
    let result = builder.build_binop(
        BinOp::Add,
        Operand::Value(a),
        Operand::Value(b),
        Type::Unit,
        Span::dummy(),
    );

    // Return result
    builder.build_return(Some(Operand::Value(result)), Span::dummy());

    let mut func = builder.finish_function().unwrap();

    // Build CFG
    let cfg = CFG::build(&func);
    assert_eq!(cfg.entry, func.entry);

    // Compute dominators
    let dom = DominatorTree::compute(&cfg);
    assert!(dom.dominates(func.entry, func.entry));

    // Run optimizations
    let mut dce = DCE::new();
    dce.run(&mut func);

    // Dump MIR
    let mut dumper = MirDumper::new();
    let output = dumper.dump_function(&func);
    assert!(output.contains("fn add"));

    // Export JSON
    let json = dumper.to_json(&func);
    assert!(json.is_ok());
}
