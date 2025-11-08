# Aurora Effects & Borrow Checking

Complete implementation of Aurora's effect system and borrow checker with advisory and strict modes.

## Features

- **Effect System**: Track IO, Alloc, Parallel, and Unsafe effects with subeffecting partial order
- **Effect Polymorphism**: Generic functions can be polymorphic over effects
- **Borrow Checker**: Advisory mode (warnings) and strict mode (errors)
- **Dataflow Analysis**: Track live borrows and detect conflicts
- **Lifetime Inference**: Automatic lifetime tracking with constraint solving
- **ARC Insertion**: Automatic reference counting at uncertain escape points
- **Strict Mode**: Convert all advisories to errors, require explicit lifetimes

## Advisory Mode (Default)

By default, the effect checker operates in advisory mode where borrow violations and effect issues generate warnings but don't block compilation:

```rust
use aurora_effects::*;

let diag = Arc::new(NullDiagnostics);
let mut checker = EffectChecker::new(diag);

// Borrow violations generate warnings
let region = Region::static_region(true);
checker.borrow_checker.record_borrow(
    BorrowKind::Mutable,
    "x".to_string(),
    region.clone(),
    1,
);
checker.borrow_checker.record_borrow(
    BorrowKind::Mutable,
    "x".to_string(),
    region,
    2,
);

// Has advisories but doesn't fail
assert!(checker.borrow_checker().has_advisories());
```

## Strict Mode

Strict mode enforces all rules and converts advisories to errors:

```rust
use aurora_effects::*;

let diag = Arc::new(NullDiagnostics);
let mut checker = EffectChecker::new(diag)
    .with_strict_mode(StrictConfig::strict());

// Same code now produces errors
```

## Effect Tracking

Track effects through your program:

```rust
use aurora_effects::*;

let mut tracker = EffectTracker::new();

// Start with pure
assert_eq!(tracker.current(), EffectSet::PURE);

// Add IO effect
tracker.add_effect(EffectSet::IO);
assert_eq!(tracker.current(), EffectSet::IO);

// Check if effect is allowed
tracker.check_allowed(EffectSet::PURE).unwrap();  // OK
tracker.check_allowed(EffectSet::UNSAFE).is_err();  // Error
```

## Borrow Checking

Detect borrow conflicts:

```rust
use aurora_effects::*;

let mut checker = BorrowChecker::new();
let region = Region::static_region(false);

// Shared borrows are fine
checker.record_borrow(BorrowKind::Shared, "x".to_string(), region.clone(), 1);
checker.record_borrow(BorrowKind::Shared, "x".to_string(), region, 2);
assert_eq!(checker.advisory_count(), 0);

// Mutable borrows conflict
let region = Region::static_region(true);
checker.record_borrow(BorrowKind::Mutable, "y".to_string(), region.clone(), 3);
checker.record_borrow(BorrowKind::Mutable, "y".to_string(), region, 4);
assert!(checker.has_advisories());
```

## Lifetime Tracking

Automatic lifetime inference:

```rust
use aurora_effects::*;

let mut ctx = LifetimeContext::new();

// Generate lifetimes
let outer = ctx.fresh();
ctx.push_scope();
let inner = ctx.fresh();

// Check outlives relation
assert!(ctx.outlives(&outer, &inner));

ctx.pop_scope();
```

## ARC Insertion

Automatic reference counting at escape points:

```rust
use aurora_effects::*;

let mut arc = ArcContext::new(false);

// Analyze escapes
let kind = arc.analyze_escape("return_value".to_string(), 1);
assert_eq!(kind, EscapeKind::ReturnEscape);

// Process escape - inserts ARC
arc.process_escape("heap_value".to_string(), 2).ok();
assert!(arc.sites().len() > 0);
```

## Integration with Compiler

```rust
use aurora_effects::*;
use std::sync::Arc;

// Implement diagnostic collector
struct MyDiagnostics;
impl DiagnosticCollector for MyDiagnostics {
    fn report_advisory(&self, message: String, location: usize, severity: u8) {
        match severity {
            0 => println!("INFO: {}", message),
            1 => println!("WARNING: {}", message),
            2 => eprintln!("ERROR: {}", message),
            _ => {}
        }
    }
}

// Create checker
let diag = Arc::new(MyDiagnostics);
let mut checker = EffectChecker::new(diag);

// Check AST
let ast = get_ast();
let checked_ast = checker.check(ast);
```

## Advisory Examples

The borrow checker provides helpful suggestions:

```
WARNING: Cannot borrow x as mutable because it is already borrowed
Suggestion: Consider using a different lifetime or scope

WARNING: Use of moved value: x
Suggestion: Consider cloning the value or using a reference

WARNING: Cannot borrow x as mutable more than once
Suggestion: Use separate scopes or refactor to avoid simultaneous borrows

INFO: ARC inserted for 'value': Value escapes (HeapEscape)
```

## Strict Mode Configuration

Fine-grained control over strict mode:

```rust
use aurora_effects::*;

let config = StrictConfig {
    require_lifetimes: true,          // Require explicit lifetimes
    disallow_arc: true,               // No automatic ARC insertion
    advisories_are_errors: true,      // Convert warnings to errors
    disallow_implicit_conversions: true,
    require_unsafe_blocks: true,
};

let checker = EffectChecker::new(diag).with_strict_mode(config);
```

## Testing

Run the comprehensive test suite:

```bash
cargo test -p aurora_effects
```

70+ tests covering all functionality.

## Performance

- Deterministic checking (same input = same output)
- O(n) effect checking
- O(n*m) borrow checking with m average active borrows
- Efficient caching of computed effects

## Documentation

See [IMPLEMENTATION.md](./IMPLEMENTATION.md) for detailed implementation notes and examples.

## License

Part of the Aurora programming language project.
