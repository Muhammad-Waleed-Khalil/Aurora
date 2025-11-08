# Aurora Name Resolution - Usage Guide

## Quick Start

```rust
use aurora_ast::{Arena, Program};
use aurora_nameres::{Resolver, NameResolver};

// Method 1: Direct resolution (with Arena access)
let arena = Arena::new();
let program = Program::empty(); // Your parsed AST

let resolver = Resolver::new(&arena, "my_crate".to_string());
let result = resolver.resolve(&program);

if result.is_ok() {
    // Resolution succeeded
    println!("Resolved {} symbols", result.symbols.len());
} else {
    // Handle errors
    for error in result.diagnostics() {
        eprintln!("Error: {:?}", error);
    }
}

// Method 2: Pipeline integration (for compiler driver)
let mut name_resolver = NameResolver::new(diagnostics_arc);
let result = name_resolver.resolve_with_arena(&program, &arena, "my_crate".to_string());

// Access results
if let Some(symbols) = name_resolver.symbols() {
    println!("Symbol table has {} entries", symbols.len());
}
```

## Working with Symbols

```rust
use aurora_nameres::{SymbolTable, Symbol, SymbolKind, Visibility};

let mut symbols = SymbolTable::new();

// Insert a symbol
let symbol = Symbol::new(
    0,                          // ID (auto-assigned)
    "my_function".to_string(),  // Name
    SymbolKind::Function,       // Kind
    Visibility::Public,         // Visibility
    span,                       // Source span
    scope_id,                   // Scope
);

if let Some(symbol_id) = symbols.insert(symbol) {
    println!("Symbol inserted with ID: {}", symbol_id);
}

// Look up a symbol
if let Some(symbol_id) = symbols.lookup(scope_id, "my_function") {
    if let Some(symbol) = symbols.get(symbol_id) {
        println!("Found: {} ({:?})", symbol.name, symbol.kind);
    }
}

// Mark symbol as used
symbols.mark_used(symbol_id);

// Find unused symbols
for symbol in symbols.unused_symbols() {
    println!("Warning: unused symbol '{}'", symbol.name);
}
```

## Working with Scopes

```rust
use aurora_nameres::{ScopeTree, ScopeKind};

let mut scopes = ScopeTree::new();

// Push a new scope
let func_scope = scopes.push_named_scope(
    ScopeKind::Function,
    span,
    "my_function".to_string(),
);

// Create nested block scope
let block_scope = scopes.push_scope(ScopeKind::Block, span);

// Pop back to parent scope
scopes.pop_scope();

// Get parent chain (for lookups)
let chain = scopes.parent_chain(scopes.current_scope());
println!("Searching scopes: {:?}", chain);

// Find enclosing loop (for break/continue)
if let Some(loop_scope) = scopes.nearest_loop_scope(scopes.current_scope()) {
    println!("Inside loop scope: {}", loop_scope);
}
```

## Resolution Chains

Resolution chains explain WHY a name resolved (or failed):

```rust
// After resolution
let result = resolver.resolve(&program);

// Get resolution chain for an identifier
if let Some(chain) = result.resolution_map.get_resolution_chain(expr_id) {
    println!("Name: {}", chain.name);
    println!("Scopes searched: {:?}", chain.scopes_searched);

    if let Some(symbol_id) = chain.symbol_id {
        println!("Resolved to symbol: {}", symbol_id);
        println!("Found in scope: {:?}", chain.found_in_scope);
    } else {
        println!("Resolution failed: {}", chain.reason);
    }
}
```

## Handling Errors

```rust
use aurora_nameres::ResolutionError;

for error in result.diagnostics() {
    match error {
        ResolutionError::UndefinedSymbol { name, span } => {
            eprintln!("{}:{}: undefined symbol '{}'",
                span.line, span.column, name);
        }
        ResolutionError::DuplicateDefinition { name, first_span, second_span } => {
            eprintln!("{}:{}: duplicate definition of '{}'",
                second_span.line, second_span.column, name);
            eprintln!("{}:{}: first defined here",
                first_span.line, first_span.column);
        }
        ResolutionError::ModuleError(module_err) => {
            eprintln!("Module error: {:?}", module_err);
        }
    }
}
```

## Module Graph

```rust
use aurora_nameres::ModuleGraph;

let mut modules = ModuleGraph::new("my_crate".to_string());

// Add a module
let module_id = modules.add_module(&module_decl, parent_id)?;

// Check for cycles
if let Err(err) = modules.detect_cycles() {
    eprintln!("Module dependency cycle detected!");
}

// Get topological order (for compilation)
if let Some(order) = modules.topological_sort() {
    for module_id in order {
        println!("Compile module: {}", module_id);
    }
}
```

## Hygiene System

For macro expansion (future use):

```rust
use aurora_nameres::{HygieneContext, HygieneResolver};

let mut hygiene = HygieneContext::new();

// Enter macro expansion
hygiene.enter_expansion("my_macro".to_string());

// Generate fresh hygiene ID for macro-generated identifier
let fresh_id = hygiene.fresh_hygiene_id();

// Exit macro expansion
hygiene.exit_expansion();

// Check if binding is allowed
let resolver = HygieneResolver::new();
if resolver.can_bind(use_hygiene, def_hygiene) {
    println!("Binding allowed");
}
```

## Prelude

The following symbols are automatically available:

```rust
// I/O functions
println("text");
print("text");
eprintln("error");
eprint("error");
dbg!(value);

// Types
String, Vec, Option, Result, Box, Arc, Rc

// Variants
Some(value)    // Option::Some
None           // Option::None
Ok(value)      // Result::Ok
Err(error)     // Result::Err

// Traits
Clone, Copy, Debug, Display, Default
Iterator, IntoIterator
From, Into
```

## Example: Complete Resolution

```rust
use aurora_ast::{Arena, Program};
use aurora_nameres::Resolver;

// 1. Parse code into AST (done by ParserAgent)
let mut arena = Arena::new();
let program = parse_program(&mut arena, source_code);

// 2. Run name resolution
let resolver = Resolver::new(&arena, "my_crate".to_string());
let result = resolver.resolve(&program);

// 3. Check for errors
if !result.is_ok() {
    for error in result.diagnostics() {
        report_error(error);
    }
    return Err("Name resolution failed");
}

// 4. Access resolved information
println!("Symbols: {}", result.symbols.len());
println!("Scopes: {}", result.scopes.len());
println!("Modules: {}", result.modules.len());
println!("Resolved expressions: {}", result.resolution_map.expr_count());

// 5. Use resolution map
for expr_id in all_identifiers {
    if let Some(symbol_id) = result.resolution_map.get_expr_resolution(expr_id) {
        if let Some(symbol) = result.symbols.get(symbol_id) {
            println!("Identifier {} -> {:?} '{}'",
                expr_id, symbol.kind, symbol.name);
        }
    }
}

// 6. Pass to next phase (TypeSystemAgent)
type_checker.check(&arena, &program, &result);
```

## Performance Tips

1. **Reuse symbol tables**: For incremental compilation, cache symbol tables per module
2. **Scope depth**: Keep scope nesting shallow for faster lookups
3. **Batch resolution**: Resolve entire modules at once instead of per-function
4. **Hash map efficiency**: Symbol table uses hash maps - O(1) average lookup

## Testing

See `tests/integration_test.rs` for comprehensive examples of:
- Function call resolution
- Prelude resolution
- Error handling
- Variable shadowing
- Forward references

Run tests:
```bash
cargo test --package aurora_nameres
```

## Limitations

Name resolution does NOT handle (these are handled by later phases):

- Type inference (TypeSystemAgent)
- Type checking (TypeSystemAgent)
- Borrow checking (EffectsBorrowAgent)
- Effect system (EffectsBorrowAgent)
- Trait resolution (TypeSystemAgent)
- Generic monomorphization (TypeSystemAgent)

## Further Reading

- [Implementation Documentation](../../NAMERES_IMPLEMENTATION.md)
- [API Documentation](https://docs.rs/aurora_nameres)
- [Aurora Language Specification](../../docs/spec.md)
