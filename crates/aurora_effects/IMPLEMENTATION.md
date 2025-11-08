# Aurora Effects & Borrow Checker Implementation

This document describes the complete implementation of Aurora's effect system and borrow checker.

## Architecture Overview

The effect system and borrow checker is composed of five main modules:

1. **effects.rs** - Effect tracking, polymorphism, and subeffecting
2. **borrow.rs** - Borrow checking with dataflow analysis
3. **lifetimes.rs** - Lifetime tracking and constraints
4. **arc.rs** - Automatic reference counting insertion
5. **strict.rs** - Strict mode enforcement

## Effect System

### Effect Hierarchy

Aurora supports a partial order of effects:

```
PURE ⊆ IO ⊆ UNSAFE
PURE ⊆ ALLOC ⊆ UNSAFE
PURE ⊆ PARALLEL
```

### Effect Polymorphism

Functions can be polymorphic over effects using effect variables:

```rust
let mut ctx = EffectInferContext::new();
let effect_var = ctx.fresh_var();  // Creates effect variable

// Unify with concrete effect
ctx.unify(&effect_var, &Effect::Concrete(EffectSet::IO))?;
let result = ctx.apply(&effect_var);  // Resolves to IO
```

### Effect Composition

Effects compose through union:

```rust
let e1 = EffectSet::IO;
let e2 = EffectSet::ALLOC;
let composed = compose_effects(e1, e2);
// Result has both IO and ALLOC effects
```

### Subeffecting

The subeffecting relation allows pure functions to be used where effectful ones are expected:

```rust
assert!(is_subeffect(EffectSet::PURE, EffectSet::IO));  // true
assert!(is_subeffect(EffectSet::IO, EffectSet::UNSAFE));  // true
assert!(!is_subeffect(EffectSet::UNSAFE, EffectSet::IO));  // false
```

## Borrow Checker

### Advisory Mode (Default)

In advisory mode, borrow violations emit warnings but don't fail the build:

```rust
let mut checker = BorrowChecker::new();
let region = Region::static_region(true);

// Record two mutable borrows (conflict!)
checker.record_borrow(BorrowKind::Mutable, "x".to_string(), region.clone(), 1);
checker.record_borrow(BorrowKind::Mutable, "x".to_string(), region, 2);

// Generates advisory, but doesn't error
assert!(checker.has_advisories());
```

### Shared Borrows

Multiple shared borrows are allowed:

```rust
let mut checker = BorrowChecker::new();
let region = Region::static_region(false);

checker.record_borrow(BorrowKind::Shared, "x".to_string(), region.clone(), 1);
checker.record_borrow(BorrowKind::Shared, "x".to_string(), region, 2);

// No conflict
assert_eq!(checker.advisory_count(), 0);
```

### Mutable Borrows

Mutable borrows are exclusive:

```rust
let mut checker = BorrowChecker::new();
let region = Region::static_region(true);

checker.record_borrow(BorrowKind::Mutable, "x".to_string(), region.clone(), 1);
checker.record_borrow(BorrowKind::Shared, "x".to_string(), region, 2);

// Conflict: mutable borrow while borrowed
assert!(checker.has_advisories());
```

### Move Tracking

The borrow checker tracks moves and detects use-after-move:

```rust
let mut checker = BorrowChecker::new();

checker.record_move("x".to_string(), 1);

let region = Region::static_region(false);
checker.record_borrow(BorrowKind::Shared, "x".to_string(), region, 2);

// Advisory: use of moved value
assert!(checker.has_advisories());
```

### Dataflow Analysis

The borrow checker uses dataflow analysis to track live borrows:

```rust
let mut dataflow = BorrowDataflow::new();

let mut borrows = HashSet::new();
borrows.insert(1);
borrows.insert(2);

dataflow.set_borrows(10, borrows.clone());
assert_eq!(dataflow.live_borrows(10), borrows);
```

## Lifetime System

### Lifetime Inference

Lifetimes are automatically inferred using constraint-based analysis:

```rust
let mut ctx = LifetimeContext::new();

// Generate fresh anonymous lifetimes
let l1 = ctx.fresh();
let l2 = ctx.fresh();

// Add constraint: l1 outlives l2
ctx.add_outlives(l1.clone(), l2.clone());

// Check constraints are satisfiable
assert!(ctx.check_constraints().is_ok());
```

### Scope-Based Lifetimes

Lifetimes are tied to scopes:

```rust
let mut ctx = LifetimeContext::new();

let outer = ctx.fresh();
ctx.push_scope();
let inner = ctx.fresh();

// Outer lifetime outlives inner
assert!(ctx.outlives(&outer, &inner));

ctx.pop_scope();
// Inner lifetime is no longer valid
```

### Named Lifetimes

Support for explicit lifetime annotations:

```rust
let named = Lifetime::Named("a".to_string());
assert!(named.is_named());

let static_lt = Lifetime::Static;
assert!(static_lt.is_static());
```

## ARC Insertion

### Escape Analysis

The ARC system analyzes where values escape their scope:

```rust
let mut arc_ctx = ArcContext::new(false);

// Analyze different escape patterns
let kind = arc_ctx.analyze_escape("return_value".to_string(), 1);
assert_eq!(kind, EscapeKind::ReturnEscape);

let kind = arc_ctx.analyze_escape("heap_alloc".to_string(), 2);
assert_eq!(kind, EscapeKind::HeapEscape);

let kind = arc_ctx.analyze_escape("local_var".to_string(), 3);
assert_eq!(kind, EscapeKind::NoEscape);
```

### Automatic Insertion

ARC is automatically inserted at uncertain escape points:

```rust
let mut arc_ctx = ArcContext::new(false);

// Process an escape
arc_ctx.process_escape("heap_value".to_string(), 1)?;

// ARC sites are inserted
assert!(arc_ctx.sites().len() > 0);

// Check operations
let sites = arc_ctx.sites();
assert!(sites.iter().any(|s| s.operation == ArcOp::Retain));
assert!(sites.iter().any(|s| s.operation == ArcOp::Release));
```

### ARC Optimization

Redundant ARC operations are removed:

```rust
let mut optimizer = ArcOptimizer::new();
let mut sites = vec![
    ArcSite {
        path: "x".to_string(),
        operation: ArcOp::Retain,
        location: 1,
        reason: "test".to_string(),
    },
    ArcSite {
        path: "x".to_string(),
        operation: ArcOp::Release,
        location: 1,
        reason: "test".to_string(),
    },
];

optimizer.optimize(&mut sites);
// Redundant pair removed
assert_eq!(sites.len(), 0);
```

## Strict Mode

### Configuration

Strict mode can be configured with fine-grained control:

```rust
// Full strict mode
let strict = StrictConfig::strict();
assert!(strict.require_lifetimes);
assert!(strict.disallow_arc);
assert!(strict.advisories_are_errors);

// Permissive mode (default)
let permissive = StrictConfig::permissive();
assert!(!permissive.require_lifetimes);
assert!(!permissive.disallow_arc);
```

### Enforcement

Strict mode converts advisories to errors:

```rust
let mut checker = StrictChecker::strict();
let mut borrow_checker = BorrowChecker::new();

// Create violation
let region = Region::static_region(true);
borrow_checker.record_borrow(BorrowKind::Mutable, "x".to_string(), region.clone(), 1);
borrow_checker.record_borrow(BorrowKind::Mutable, "x".to_string(), region, 2);

// Check - will error in strict mode
let result = checker.check_borrow_checker(&borrow_checker);
assert!(result.is_err());
```

### Lifetime Requirements

Strict mode requires explicit lifetimes:

```rust
let mut checker = StrictChecker::strict();
let anon = Lifetime::Anon(0);

// Error: anonymous lifetime in strict mode
let result = checker.check_lifetime(&anon, "param");
assert!(result.is_err());

let named = Lifetime::Named("a".to_string());
let result = checker.check_lifetime(&named, "param");
assert!(result.is_ok());
```

### ARC Restrictions

Strict mode disallows automatic ARC insertion:

```rust
let mut arc_ctx = ArcContext::new(true);  // strict mode

let result = arc_ctx.insert_arc("x".to_string(), 1, "test".to_string());
assert!(result.is_err());  // ARC not allowed in strict mode
```

## Integration with Compiler Pipeline

### Effect Checker

The main integration point is the `EffectChecker`:

```rust
use aurora_effects::{EffectChecker, DiagnosticCollector, StrictConfig};

// Create diagnostic collector
struct MyDiagnostics;
impl DiagnosticCollector for MyDiagnostics {
    fn report_advisory(&self, message: String, location: usize, severity: u8) {
        println!("[{}] {}: {}", severity, location, message);
    }
}

// Create checker
let diag = Arc::new(MyDiagnostics);
let mut checker = EffectChecker::new(diag)
    .with_strict_mode(StrictConfig::permissive())
    .with_type_map(type_map);

// Check AST
let checked_ast = checker.check(ast);

// Access results
let advisories = checker.borrow_checker().advisories();
let arc_sites = checker.arc_context().sites();
```

### Diagnostic Reporting

Diagnostics are reported through a trait:

```rust
pub trait DiagnosticCollector: Send + Sync {
    fn report_advisory(&self, message: String, location: usize, severity: u8);
}
```

Severity levels:
- 0: Info
- 1: Warning
- 2: Error (strict mode only)

## Example Workflow

Complete example of checking a function:

```rust
use aurora_effects::*;

let diag = Arc::new(NullDiagnostics);
let mut checker = EffectChecker::new(diag);

// Enter function scope
checker.push_scope();

// Track effect
checker.effect_tracker.add_effect(EffectSet::IO);

// Record borrows
let region = Region::static_region(false);
checker.borrow_checker.record_borrow(
    BorrowKind::Shared,
    "x".to_string(),
    region,
    1,
);

// Process escapes
checker.arc_context.process_escape("value".to_string(), 2).ok();

// Exit scope
checker.pop_scope();

// Check AST
let ast = Ast::empty();
let result = checker.check(ast);

// Report advisories
checker.report_advisories();
```

## Testing

The implementation includes 70+ tests covering:

- Effect system (10 tests)
- Borrow checking (14 tests)
- Lifetime tracking (9 tests)
- ARC insertion (7 tests)
- Strict mode (10 tests)
- Integration (20 tests)

Run tests with:

```bash
cargo test -p aurora_effects
```

## Performance Characteristics

- **Effect checking**: O(n) where n is the number of expressions
- **Borrow checking**: O(n*m) where m is the average number of active borrows
- **Lifetime inference**: O(c^2) where c is the number of constraints
- **ARC optimization**: O(s^2) where s is the number of ARC sites

All operations use deterministic algorithms ensuring reproducible results.

## Future Enhancements

Potential improvements:

1. More sophisticated escape analysis
2. Region-based lifetime inference
3. Flow-sensitive borrow checking
4. Parallel effect tracking
5. Effect row polymorphism
6. Lifetime variance checking
7. Move path tracking with field granularity
8. ARC elision optimizations

## References

- Rust borrow checker: https://rustc-dev-guide.rust-lang.org/borrow_check.html
- Effect systems: https://www.eff-lang.org/
- ARC: https://en.wikipedia.org/wiki/Automatic_Reference_Counting
