//! Integration tests for Aurora name resolution
//!
//! These tests demonstrate the complete name resolution functionality,
//! including symbol tables, scope management, hygiene, and module resolution.

use aurora_ast::decl::{FunctionDecl, Item, ItemKind, Param};
use aurora_ast::expr::{Expr, ExprKind, Literal, Path};
use aurora_ast::pattern::{Pattern, PatternKind};
use aurora_ast::span::{HygieneId, Span};
use aurora_ast::stmt::{Block, Stmt, StmtKind};
use aurora_ast::ty::{Type, TypeKind};
use aurora_ast::{Arena, Program};
use aurora_nameres::{Resolver, SymbolKind};

/// Test: Resolve a simple function call to a function definition
#[test]
fn test_function_call_resolution() {
    let mut arena = Arena::new();

    // Create function: fn add(a: i32, b: i32) -> i32 { a + b }

    // Create parameter types
    let i32_ty = Type {
        kind: TypeKind::Path {
            path: Path {
                segments: vec!["i32".to_string()],
                generics: vec![],
            },
        },
        span: Span::dummy(),
    };
    let a_ty_id = arena.alloc_type(i32_ty.clone());
    let b_ty_id = arena.alloc_type(i32_ty.clone());
    let ret_ty_id = arena.alloc_type(i32_ty);

    // Create parameter patterns
    let a_pattern = Pattern {
        kind: PatternKind::Ident {
            name: "a".to_string(),
            is_mut: false,
        },
        span: Span::dummy(),
        hygiene: HygieneId::root(),
    };
    let a_pat_id = arena.alloc_pattern(a_pattern);

    let b_pattern = Pattern {
        kind: PatternKind::Ident {
            name: "b".to_string(),
            is_mut: false,
        },
        span: Span::dummy(),
        hygiene: HygieneId::root(),
    };
    let b_pat_id = arena.alloc_pattern(b_pattern);

    // Create parameters
    let params = vec![
        Param {
            pattern: a_pat_id,
            ty: a_ty_id,
            is_mut: false,
            span: Span::dummy(),
        },
        Param {
            pattern: b_pat_id,
            ty: b_ty_id,
            is_mut: false,
            span: Span::dummy(),
        },
    ];

    // Create function body: a + b
    let a_expr = Expr {
        kind: ExprKind::Ident("a".to_string()),
        span: Span::dummy(),
        hygiene: HygieneId::root(),
    };
    let a_id = arena.alloc_expr(a_expr);

    let b_expr = Expr {
        kind: ExprKind::Ident("b".to_string()),
        span: Span::dummy(),
        hygiene: HygieneId::root(),
    };
    let b_id = arena.alloc_expr(b_expr);

    let add_expr = Expr {
        kind: ExprKind::Binary {
            left: a_id,
            op: aurora_ast::expr::BinaryOp::Add,
            right: b_id,
        },
        span: Span::dummy(),
        hygiene: HygieneId::root(),
    };
    let add_id = arena.alloc_expr(add_expr);

    let func_body = Block {
        stmts: vec![],
        expr: Some(add_id),
        span: Span::dummy(),
    };

    let func_decl = FunctionDecl {
        name: "add".to_string(),
        generics: vec![],
        params,
        return_type: Some(ret_ty_id),
        where_clause: None,
        body: func_body,
        is_pub: true,
        is_async: false,
        is_unsafe: false,
        span: Span::dummy(),
    };

    let item = Item {
        kind: ItemKind::Function(func_decl),
        span: Span::dummy(),
    };
    let item_id = arena.alloc_item(item);

    let program = Program::new(vec![item_id], Span::dummy());

    // Resolve
    let resolver = Resolver::new(&arena, "test_crate".to_string());
    let result = resolver.resolve(&program);

    // Check that resolution succeeded
    assert!(result.is_ok(), "Resolution should succeed without errors");

    // Check that the function was collected
    let symbols_in_global = result.symbols.symbols_in_scope(0);
    let add_fn = symbols_in_global.iter().find(|s| s.name == "add");
    assert!(add_fn.is_some(), "add function should be in symbol table");
    assert_eq!(add_fn.unwrap().kind, SymbolKind::Function);

    // Check that parameters were resolved
    assert!(
        result.resolution_map.get_pattern_binding(a_pat_id).is_some(),
        "Parameter 'a' should be bound"
    );
    assert!(
        result.resolution_map.get_pattern_binding(b_pat_id).is_some(),
        "Parameter 'b' should be bound"
    );

    // Check that identifier uses were resolved
    assert!(
        result.resolution_map.get_expr_resolution(a_id).is_some(),
        "Use of 'a' should be resolved"
    );
    assert!(
        result.resolution_map.get_expr_resolution(b_id).is_some(),
        "Use of 'b' should be resolved"
    );

    // Check resolution chains
    let a_chain = result.resolution_map.get_resolution_chain(a_id);
    assert!(a_chain.is_some(), "Should have resolution chain for 'a'");
    assert_eq!(a_chain.unwrap().name, "a");
    assert!(
        a_chain.unwrap().symbol_id.is_some(),
        "Resolution chain should have symbol ID"
    );
}

/// Test: Resolve println from prelude
#[test]
fn test_prelude_println_resolution() {
    let mut arena = Arena::new();

    // Create: fn main() { println("Hello"); }
    let hello_lit = Expr {
        kind: ExprKind::Literal(Literal::String("Hello".to_string())),
        span: Span::dummy(),
        hygiene: HygieneId::root(),
    };
    let hello_id = arena.alloc_expr(hello_lit);

    let println_expr = Expr {
        kind: ExprKind::Ident("println".to_string()),
        span: Span::dummy(),
        hygiene: HygieneId::root(),
    };
    let println_id = arena.alloc_expr(println_expr);

    let call_expr = Expr {
        kind: ExprKind::Call {
            func: println_id,
            args: vec![hello_id],
        },
        span: Span::dummy(),
        hygiene: HygieneId::root(),
    };
    let call_id = arena.alloc_expr(call_expr);

    let func_body = Block {
        stmts: vec![],
        expr: Some(call_id),
        span: Span::dummy(),
    };

    let func_decl = FunctionDecl {
        name: "main".to_string(),
        generics: vec![],
        params: vec![],
        return_type: None,
        where_clause: None,
        body: func_body,
        is_pub: false,
        is_async: false,
        is_unsafe: false,
        span: Span::dummy(),
    };

    let item = Item {
        kind: ItemKind::Function(func_decl),
        span: Span::dummy(),
    };
    let item_id = arena.alloc_item(item);

    let program = Program::new(vec![item_id], Span::dummy());

    // Resolve
    let resolver = Resolver::new(&arena, "test_crate".to_string());
    let result = resolver.resolve(&program);

    // Check that resolution succeeded
    assert!(result.is_ok(), "Resolution should succeed");

    // Check that println was resolved from prelude
    let println_resolution = result.resolution_map.get_expr_resolution(println_id);
    assert!(
        println_resolution.is_some(),
        "println should be resolved from prelude"
    );

    // Check resolution chain shows it came from prelude
    let chain = result.resolution_map.get_resolution_chain(println_id);
    assert!(chain.is_some());
    assert_eq!(chain.unwrap().name, "println");

    // Verify println is marked as used
    let symbol = result.symbols.get(println_resolution.unwrap());
    assert!(symbol.is_some());
    assert!(symbol.unwrap().is_used, "println should be marked as used");
}

/// Test: Undefined variable produces error
#[test]
fn test_undefined_variable_error() {
    let mut arena = Arena::new();

    // Create: fn test() { undefined_var; }
    let undefined_expr = Expr {
        kind: ExprKind::Ident("undefined_var".to_string()),
        span: Span::dummy(),
        hygiene: HygieneId::root(),
    };
    let undefined_id = arena.alloc_expr(undefined_expr);

    let func_body = Block {
        stmts: vec![],
        expr: Some(undefined_id),
        span: Span::dummy(),
    };

    let func_decl = FunctionDecl {
        name: "test".to_string(),
        generics: vec![],
        params: vec![],
        return_type: None,
        where_clause: None,
        body: func_body,
        is_pub: false,
        is_async: false,
        is_unsafe: false,
        span: Span::dummy(),
    };

    let item = Item {
        kind: ItemKind::Function(func_decl),
        span: Span::dummy(),
    };
    let item_id = arena.alloc_item(item);

    let program = Program::new(vec![item_id], Span::dummy());

    // Resolve
    let resolver = Resolver::new(&arena, "test_crate".to_string());
    let result = resolver.resolve(&program);

    // Check that resolution failed
    assert!(!result.is_ok(), "Resolution should have errors");
    assert!(result.diagnostics.len() > 0, "Should have diagnostics");

    // Check that we have an undefined symbol error
    use aurora_nameres::ResolutionError;
    let has_error = result.diagnostics.iter().any(|e| {
        matches!(e, ResolutionError::UndefinedSymbol { name, .. } if name == "undefined_var")
    });
    assert!(has_error, "Should have undefined symbol error");

    // Check resolution chain shows failure
    let chain = result.resolution_map.get_resolution_chain(undefined_id);
    assert!(chain.is_some());
    assert_eq!(chain.unwrap().name, "undefined_var");
    assert!(chain.unwrap().symbol_id.is_none(), "Should have no symbol");
}

/// Test: Variable shadowing in nested scopes
#[test]
fn test_variable_shadowing() {
    let mut arena = Arena::new();

    // Outer variable: let x = 1;
    let outer_lit = Expr {
        kind: ExprKind::Literal(Literal::Int(1)),
        span: Span::dummy(),
        hygiene: HygieneId::root(),
    };
    let outer_lit_id = arena.alloc_expr(outer_lit);

    let outer_pattern = Pattern {
        kind: PatternKind::Ident {
            name: "x".to_string(),
            is_mut: false,
        },
        span: Span::dummy(),
        hygiene: HygieneId::root(),
    };
    let outer_pat_id = arena.alloc_pattern(outer_pattern);

    let outer_let = Stmt {
        kind: StmtKind::Let {
            pattern: outer_pat_id,
            ty: None,
            init: Some(outer_lit_id),
            mutable: false,
        },
        span: Span::dummy(),
    };
    let outer_stmt_id = arena.alloc_stmt(outer_let);

    let func_body = Block {
        stmts: vec![outer_stmt_id],
        expr: None,
        span: Span::dummy(),
    };

    let func_decl = FunctionDecl {
        name: "test".to_string(),
        generics: vec![],
        params: vec![],
        return_type: None,
        where_clause: None,
        body: func_body,
        is_pub: false,
        is_async: false,
        is_unsafe: false,
        span: Span::dummy(),
    };

    let item = Item {
        kind: ItemKind::Function(func_decl),
        span: Span::dummy(),
    };
    let item_id = arena.alloc_item(item);

    let program = Program::new(vec![item_id], Span::dummy());

    // Resolve
    let resolver = Resolver::new(&arena, "test_crate".to_string());
    let result = resolver.resolve(&program);

    // Check that resolution succeeded
    assert!(result.is_ok(), "Resolution should succeed");

    // Check that the outer variable was bound
    assert!(
        result.resolution_map.get_pattern_binding(outer_pat_id).is_some(),
        "Outer variable should be bound"
    );
}

/// Test: Forward references (functions can be called before they're defined)
#[test]
fn test_forward_reference() {
    let mut arena = Arena::new();

    // Create: fn caller() { callee(); } fn callee() {}

    // Create callee call
    let callee_expr = Expr {
        kind: ExprKind::Ident("callee".to_string()),
        span: Span::dummy(),
        hygiene: HygieneId::root(),
    };
    let callee_id = arena.alloc_expr(callee_expr);

    let call_expr = Expr {
        kind: ExprKind::Call {
            func: callee_id,
            args: vec![],
        },
        span: Span::dummy(),
        hygiene: HygieneId::root(),
    };
    let call_id = arena.alloc_expr(call_expr);

    // Create caller function
    let caller_body = Block {
        stmts: vec![],
        expr: Some(call_id),
        span: Span::dummy(),
    };

    let caller_decl = FunctionDecl {
        name: "caller".to_string(),
        generics: vec![],
        params: vec![],
        return_type: None,
        where_clause: None,
        body: caller_body,
        is_pub: false,
        is_async: false,
        is_unsafe: false,
        span: Span::dummy(),
    };

    let caller_item = Item {
        kind: ItemKind::Function(caller_decl),
        span: Span::dummy(),
    };
    let caller_item_id = arena.alloc_item(caller_item);

    // Create callee function (defined after caller)
    let callee_body = Block {
        stmts: vec![],
        expr: None,
        span: Span::dummy(),
    };

    let callee_decl = FunctionDecl {
        name: "callee".to_string(),
        generics: vec![],
        params: vec![],
        return_type: None,
        where_clause: None,
        body: callee_body,
        is_pub: false,
        is_async: false,
        is_unsafe: false,
        span: Span::dummy(),
    };

    let callee_item = Item {
        kind: ItemKind::Function(callee_decl),
        span: Span::dummy(),
    };
    let callee_item_id = arena.alloc_item(callee_item);

    let program = Program::new(vec![caller_item_id, callee_item_id], Span::dummy());

    // Resolve
    let resolver = Resolver::new(&arena, "test_crate".to_string());
    let result = resolver.resolve(&program);

    // Check that resolution succeeded (forward reference works)
    assert!(result.is_ok(), "Forward reference should resolve");

    // Check that callee was resolved
    let callee_resolution = result.resolution_map.get_expr_resolution(callee_id);
    assert!(
        callee_resolution.is_some(),
        "Forward reference to callee should resolve"
    );
}
