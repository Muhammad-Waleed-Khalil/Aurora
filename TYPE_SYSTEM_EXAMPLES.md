# Aurora Type System Examples

## Overview

This document demonstrates the capabilities of the Aurora type system through practical examples.

## Basic Type Inference

### Example 1: Simple Variable Inference

```aurora
let x = 42;        // Infers: x: i32
let y = 3.14;      // Infers: y: f64
let z = true;      // Infers: z: bool
let s = "hello";   // Infers: s: str
```

**Type Inference Process:**
1. Literal 42 has default type i32
2. Variable x is bound to i32
3. Type is stored in environment: Γ ⊢ x: i32

### Example 2: Function Type Inference

```aurora
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

**Type**: `(i32, i32) -> i32`

**Inference Steps:**
1. Parse function signature: params=[i32, i32], ret=i32
2. Type check body: a: i32, b: i32
3. Check `a + b`: requires both operands i32, returns i32
4. Unify body type with return type: i32 ~ i32 ✓
5. Final type: `(i32, i32) -> i32`

### Example 3: Type Inference with Operators

```aurora
fn compare(x: i32, y: i32) -> bool {
    x < y
}
```

**Type**: `(i32, i32) -> bool`

**Inference:**
- `x < y`: comparison operator
- Requires: x: i32, y: i32
- Returns: bool
- Body type: bool
- Unify with return type: bool ~ bool ✓

## Polymorphic Functions

### Example 4: Identity Function

```aurora
fn identity<T>(x: T) -> T {
    x
}

// Usage
let a = identity(42);      // T = i32
let b = identity(true);    // T = bool
let c = identity("hi");    // T = str
```

**Type Scheme**: `∀T. (T) -> T`

**Monomorphization:**
- Instance 1: `identity[i32]: (i32) -> i32`
- Instance 2: `identity[bool]: (bool) -> bool`
- Instance 3: `identity[str]: (str) -> str`

**Tracked by MonoTracker:**
```rust
MonoInstance { generic_name: "identity", type_args: [i32] }
MonoInstance { generic_name: "identity", type_args: [bool] }
MonoInstance { generic_name: "identity", type_args: [str] }
```

### Example 5: Generic Container

```aurora
struct Vec<T> {
    data: [T],
    len: usize,
}

fn vec_new<T>() -> Vec<T> {
    Vec { data: [], len: 0 }
}

let v1: Vec<i32> = vec_new();     // T = i32
let v2: Vec<bool> = vec_new();    // T = bool
```

**Type Scheme**: `∀T. () -> Vec<T>`

### Example 6: Generic with Multiple Parameters

```aurora
fn pair<A, B>(a: A, b: B) -> (A, B) {
    (a, b)
}

let p1 = pair(1, true);         // A=i32, B=bool → (i32, bool)
let p2 = pair("x", 3.14);       // A=str, B=f64 → (str, f64)
```

**Type Scheme**: `∀A,B. (A, B) -> (A, B)`

## Let-Polymorphism

### Example 7: Polymorphic Let Binding

```aurora
let id = |x| x;           // Infers: ∀T. (T) -> T

let a = id(42);           // T instantiated to i32
let b = id("hi");         // T instantiated to str (reusable!)
```

**Generalization:**
1. Infer type of `|x| x`: let α = fresh var
2. Type is: `(α) -> α`
3. Generalize free vars: `∀α. (α) -> α`
4. Bind in environment: `id: ∀α. (α) -> α`

**Instantiation:**
1. Lookup `id`: get scheme `∀α. (α) -> α`
2. Call `id(42)`: instantiate α with fresh β
3. Unify `(β) -> β` with `(i32) -> ?`
4. Solve: β = i32, result = i32

### Example 8: Complex Let-Polymorphism

```aurora
let compose = |f, g, x| f(g(x));
// Infers: ∀A,B,C. ((B)->C, (A)->B, A) -> C

let inc = |x| x + 1;              // (i32) -> i32
let double = |x| x * 2;           // (i32) -> i32

let inc_then_double = compose(double, inc);
// Type: (i32) -> i32
```

## Option Types & Null Safety

### Example 9: Option Type

```aurora
fn divide(a: i32, b: i32) -> Option<i32> {
    if b == 0 {
        None              // Type: Option<i32>
    } else {
        Some(a / b)       // Type: Option<i32>
    }
}
```

**Type**: `(i32, i32) -> Option<i32>`

**Type Checking:**
1. `None`: infer type Option<α>
2. `Some(a / b)`: infer Option<i32>
3. Unify branch types: Option<α> ~ Option<i32>
4. Solve: α = i32
5. Both branches: Option<i32> ✓

### Example 10: Exhaustiveness Checking

```aurora
match divide(10, 2) {
    Some(x) => x,
    None => 0,
}
// ✓ Exhaustive (all cases covered)

match divide(10, 2) {
    Some(x) => x,
}
// ✗ Error: Non-exhaustive match, missing pattern: None
```

**Pattern Matrix:**
```
| Some(x) |
| None    |
```
Exhaustive: ✓ (covers all Option constructors)

### Example 11: Option Chaining

```aurora
fn parse(s: str) -> Option<i32> { /* ... */ }

fn process(s: str) -> Option<i32> {
    parse(s).map(|x| x * 2)
}
```

**Type**: `(str) -> Option<i32>`

## Result Types & Error Handling

### Example 12: Result Type

```aurora
fn parse_int(s: str) -> Result<i32, str> {
    if s == "42" {
        Ok(42)
    } else {
        Err("Invalid number")
    }
}
```

**Type**: `(str) -> Result<i32, str>`

**Type Checking:**
- `Ok(42)`: Result<i32, β>
- `Err("Invalid")`: Result<α, str>
- Unify: Result<i32, β> ~ Result<α, str>
- Solve: α = i32, β = str
- Result: Result<i32, str> ✓

### Example 13: Result Exhaustiveness

```aurora
match parse_int("42") {
    Ok(n) => n,
    Err(e) => 0,
}
// ✓ Exhaustive

match parse_int("42") {
    Ok(n) => n,
}
// ✗ Error: Non-exhaustive match, missing pattern: Err(_)
```

**Pattern Matrix:**
```
| Ok(n)  |
| Err(e) |
```
Exhaustive: ✓ (covers all Result constructors)

## Recursive Functions

### Example 14: Factorial

```aurora
fn factorial(n: i32) -> i32 {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}
```

**Type**: `(i32) -> i32`

**Type Checking:**
1. Assume factorial: (i32) -> i32 in environment
2. Check condition: `n <= 1`: bool ✓
3. Check then branch: `1`: i32 ✓
4. Check else branch: `n * factorial(n - 1)`
   - `factorial(n - 1)`: (i32) -> i32 with arg i32 → i32
   - `n * i32`: i32 ✓
5. Unify branches: i32 ~ i32 ✓
6. Return type: i32 ✓

### Example 15: Mutual Recursion

```aurora
fn is_even(n: i32) -> bool {
    if n == 0 {
        true
    } else {
        is_odd(n - 1)
    }
}

fn is_odd(n: i32) -> bool {
    if n == 0 {
        false
    } else {
        is_even(n - 1)
    }
}
```

**Types**:
- `is_even: (i32) -> bool`
- `is_odd: (i32) -> bool`

**Type Checking:**
1. Add both signatures to environment
2. Check is_even body with is_odd: (i32) -> bool available
3. Check is_odd body with is_even: (i32) -> bool available
4. Both check successfully ✓

## Typeclasses / Traits

### Example 16: Trait Definition

```aurora
trait Display {
    fn to_string(&self) -> str;
}
```

**Trait**: Display with method to_string

### Example 17: Trait Implementation

```aurora
struct Point {
    x: i32,
    y: i32,
}

impl Display for Point {
    fn to_string(&self) -> str {
        // ...
    }
}
```

**Registration:**
```rust
TraitImpl {
    trait_id: Display,
    self_type: Point,
    ...
}
```

**Coherence**: Only one impl of Display for Point allowed ✓

### Example 18: Trait Bounds

```aurora
fn print<T: Display>(x: T) {
    println(x.to_string())
}
```

**Type Scheme**: `∀T where T: Display. (T) -> ()`

**Bound Checking:**
1. Call `print(point)` where point: Point
2. Check: Point implements Display?
3. Lookup in trait registry: ✓ found impl
4. Call succeeds ✓

## Effect System

### Example 19: Pure Functions

```aurora
fn add(a: i32, b: i32) -> i32 {
    a + b
}
// Effects: PURE (no side effects)
```

**Type**: `(i32, i32) -> i32 + {}`

### Example 20: IO Functions

```aurora
fn main() {
    println("Hello, World!");
}
// Effects: IO (prints to console)
```

**Type**: `() -> () + {IO}`

### Example 21: Effect Subtyping

```aurora
fn pure_fn() -> i32 { 42 }
// Type: () -> i32 + {}

fn io_fn() -> i32 {
    println("Computing...");
    42
}
// Type: () -> i32 + {IO}

fn call_pure(f: () -> i32) -> i32 {
    f()
}

call_pure(pure_fn);  // ✓ {} ⊆ {}
call_pure(io_fn);    // ✗ {IO} ⊄ {}
```

**Subeffecting:**
- PURE ⊆ IO ✓
- IO ⊄ PURE ✗

## Unification Examples

### Example 22: Variable Unification

```rust
// Type inference for: let x = if cond { 1 } else { 2 }

// Step 1: Create fresh var for x
let α = fresh_var();

// Step 2: Type check branches
let then_ty = infer(1);    // i32
let else_ty = infer(2);    // i32

// Step 3: Unify branches
unify(then_ty, else_ty);   // i32 ~ i32 ✓

// Step 4: Unify with x
unify(α, i32);             // α → i32

// Result: x: i32
```

### Example 23: Function Unification

```rust
// Type inference for: let f = |x| x + 1

// Step 1: Create fresh vars
let α = fresh_var();  // param type
let β = fresh_var();  // return type

// Step 2: Assume f: (α) -> β
// Step 3: Check body: x + 1
//   - x: α
//   - 1: i32
//   - +: (i32, i32) -> i32
//   - Unify α with i32 → α = i32
//   - Result: i32
//   - Unify β with i32 → β = i32

// Result: f: (i32) -> i32
```

### Example 24: Occurs Check

```rust
// Invalid: let x = [x]

let α = fresh_var();  // type of x
let list_α = List<α>; // type of [x]

unify(α, list_α);
// ✗ Error: Occurs check failed: α occurs in List<α>
// Would create infinite type: α = List<List<List<...>>>
```

## Type Display

### Example 25: Pretty Printing

```rust
// Type: ∀T. (T) -> Option<T>
println!("{}", ty);
// Output: "forall 'T. (T) -> Option<T>"

// Type: (i32, i32) -> Result<i32, str>
println!("{}", ty);
// Output: "(i32, i32) -> Result<i32, str>"

// Type: fn(i32) -> i32 + {IO}
println!("{}", ty);
// Output: "fn(i32) -> i32 + IO"
```

## Edge Cases

### Example 26: Unit Type

```aurora
fn do_nothing() {
    // no return
}
// Type: () -> ()
```

### Example 27: Never Type

```aurora
fn panic() -> ! {
    loop {}
}
// Type: () -> !
// Never returns
```

**Subtyping:**
```aurora
let x: i32 = panic();  // ✓ ! is subtype of i32
let y: bool = panic(); // ✓ ! is subtype of bool
```

### Example 28: Tuple Types

```aurora
fn make_pair(x: i32, y: bool) -> (i32, bool) {
    (x, y)
}
// Type: (i32, bool) -> (i32, bool)
```

### Example 29: Array Types

```aurora
let arr: [i32; 5] = [1, 2, 3, 4, 5];
// Type: [i32; 5] (fixed-size array)

let slice: [i32] = &arr;
// Type: [i32] (slice, size unknown)
```

## Advanced Features

### Example 30: Higher-Order Functions

```aurora
fn map<A, B>(f: (A) -> B, xs: [A]) -> [B] {
    // ...
}

let doubled = map(|x| x * 2, [1, 2, 3]);
// Type inference:
//   A = i32
//   B = i32
//   Result: [i32]
```

### Example 31: Nested Generics

```aurora
let x: Option<Result<i32, str>> = Some(Ok(42));
// Type: Option<Result<i32, str>>

match x {
    Some(Ok(n)) => n,
    Some(Err(e)) => 0,
    None => 0,
}
// ✓ Exhaustive (all 3 cases)
```

## Performance

### Example 32: Efficient Inference

```rust
// Linear time type inference
let n = 1000;
for i in 0..n {
    let x = i;  // Each variable: O(1) inference
}
// Total: O(n) time

// No exponential blowup!
```

### Example 33: Monomorphization Deduplication

```rust
// Generic function called twice with same types
fn id<T>(x: T) -> T { x }

let a = id(1);
let b = id(2);

// MonoTracker:
// Instance 1: id[i32] ← added
// Instance 2: id[i32] ← deduplicated!
// Only generates code once ✓
```

## Summary

The Aurora type system successfully handles:

✓ Basic type inference
✓ Polymorphic functions
✓ Let-polymorphism
✓ Option and Result types
✓ Exhaustiveness checking
✓ Recursive functions
✓ Trait constraints
✓ Effect tracking
✓ Generic instantiation
✓ Monomorphization
✓ Principal types
✓ Deterministic inference
✓ Occurs check
✓ Subtyping
✓ Higher-order functions
✓ Complex nested types

All with:
- No exponential backtracking
- Linear time complexity
- Deterministic results
- Complete error messages
- Efficient representation
