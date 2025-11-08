//! MIR Demo - Demonstrates complete MIR lowering and optimization

use aurora_mir::*;
use aurora_types::{EffectSet, PrimitiveType, Type};

fn main() {
    println!("=== Aurora MIR Demo ===\n");

    // Example 1: Simple hello world function
    demo_hello_world();

    // Example 2: Add function
    demo_add_function();

    // Example 3: Factorial with control flow
    demo_factorial();

    // Example 4: Optimization pipeline
    demo_optimization();
}

fn demo_hello_world() {
    println!("--- Example 1: Hello World ---");

    let mut builder = MirBuilder::new();
    builder.start_function(0, "main".to_string(), Type::Unit, EffectSet::IO);

    // Create string constant
    let msg = Operand::Const(Constant::String("Hello, World!".to_string()));

    // Call println with string
    builder.build_call(
        Operand::Const(Constant::Int(0)), // Function ID placeholder
        vec![msg],
        None,
        EffectSet::IO,
        Span::dummy(),
    );

    // Return unit
    builder.build_return(Some(Operand::Const(Constant::Unit)), Span::dummy());

    let func = builder.finish_function().unwrap();

    let mut dumper = MirDumper::new();
    println!("{}", dumper.dump_function(&func));
}

fn demo_add_function() {
    println!("--- Example 2: Add Function ---");

    let mut builder = MirBuilder::new();
    builder.start_function(
        1,
        "add".to_string(),
        Type::Primitive(PrimitiveType::I32),
        EffectSet::PURE,
    );

    // Create parameter values
    let a = builder.new_value(Type::Primitive(PrimitiveType::I32), Span::dummy());
    let b = builder.new_value(Type::Primitive(PrimitiveType::I32), Span::dummy());

    // Add them
    let result = builder.build_binop(
        BinOp::Add,
        Operand::Value(a),
        Operand::Value(b),
        Type::Primitive(PrimitiveType::I32),
        Span::dummy(),
    );

    // Return result
    builder.build_return(Some(Operand::Value(result)), Span::dummy());

    let func = builder.finish_function().unwrap();

    let mut dumper = MirDumper::new();
    println!("{}", dumper.dump_function(&func));
}

fn demo_factorial() {
    println!("--- Example 3: Factorial (with control flow) ---");

    let mut builder = MirBuilder::new();
    builder.start_function(
        2,
        "factorial".to_string(),
        Type::Primitive(PrimitiveType::I32),
        EffectSet::PURE,
    );

    // Parameter: n
    let n = builder.new_value(Type::Primitive(PrimitiveType::I32), Span::dummy());

    // Compare n <= 1
    let one = Operand::Const(Constant::Int(1));
    let cond = builder.build_binop(
        BinOp::Le,
        Operand::Value(n),
        one.clone(),
        Type::Primitive(PrimitiveType::Bool),
        Span::dummy(),
    );

    // Create blocks
    let then_block = builder.new_block();
    let else_block = builder.new_block();

    // Branch on condition
    builder.build_branch(Operand::Value(cond), then_block, else_block, Span::dummy());

    // Then block: return 1
    builder.add_block(BasicBlock::new(then_block));
    builder.set_block(then_block);
    builder.build_return(Some(one.clone()), Span::dummy());

    // Else block: return n * factorial(n - 1)
    builder.add_block(BasicBlock::new(else_block));
    builder.set_block(else_block);

    // n - 1
    let n_minus_1 = builder.build_binop(
        BinOp::Sub,
        Operand::Value(n),
        one,
        Type::Primitive(PrimitiveType::I32),
        Span::dummy(),
    );

    // Recursive call
    let rec_result = builder.build_call(
        Operand::Const(Constant::Int(2)), // factorial function ID
        vec![Operand::Value(n_minus_1)],
        Some(Type::Primitive(PrimitiveType::I32)),
        EffectSet::PURE,
        Span::dummy(),
    );

    // n * factorial(n - 1)
    let final_result = builder.build_binop(
        BinOp::Mul,
        Operand::Value(n),
        Operand::Value(rec_result.unwrap()),
        Type::Primitive(PrimitiveType::I32),
        Span::dummy(),
    );

    builder.build_return(Some(Operand::Value(final_result)), Span::dummy());

    let func = builder.finish_function().unwrap();

    let mut dumper = MirDumper::new();
    println!("{}", dumper.dump_function(&func));
}

fn demo_optimization() {
    println!("--- Example 4: Optimization Pipeline ---");

    // Create a function with redundant operations
    let mut builder = MirBuilder::new();
    builder.start_function(
        3,
        "optimizable".to_string(),
        Type::Primitive(PrimitiveType::I32),
        EffectSet::PURE,
    );

    // 2 + 3 (constant folding opportunity)
    let const_add = builder.build_binop(
        BinOp::Add,
        Operand::Const(Constant::Int(2)),
        Operand::Const(Constant::Int(3)),
        Type::Primitive(PrimitiveType::I32),
        Span::dummy(),
    );

    // Dead code: unused value
    let _unused = builder.build_binop(
        BinOp::Mul,
        Operand::Const(Constant::Int(10)),
        Operand::Const(Constant::Int(20)),
        Type::Primitive(PrimitiveType::I32),
        Span::dummy(),
    );

    // Return the result
    builder.build_return(Some(Operand::Value(const_add)), Span::dummy());

    let mut func = builder.finish_function().unwrap();

    println!("Before optimization:");
    let mut dumper = MirDumper::new();
    println!("{}", dumper.dump_function(&func));

    // Run optimization passes
    println!("\nApplying optimizations...");

    let mut pipeline = opt::OptPipeline::new(opt::OptLevel::O2);
    pipeline.run(&mut func);

    println!("\nAfter optimization:");
    println!("{}", dumper.dump_function(&func));

    // Show CFG information
    let cfg = CFG::build(&func);
    println!("\nCFG Info:");
    println!("  Entry block: {}", cfg.entry);
    println!("  Exit blocks: {:?}", cfg.exits);
    println!("  Reachable blocks: {}", cfg.reachable_blocks().len());

    // Show dominator tree
    let dom_tree = DominatorTree::compute(&cfg);
    println!("\nDominator Tree:");
    for (block, idom) in &dom_tree.idom {
        println!("  Block {} idom: {}", block, idom);
    }
}
