# Aurora Language Specification

**Version**: 1.0.0
**Status**: Foundation Phase
**Target**: MVP Compiler Implementation

---

## 1. Executive Summary

Aurora is a systems programming language designed for autonomous coding agents, infrastructure teams, and systems programmers. It combines Python-level syntax ergonomics with C-level performance, targeting d1.10× of optimized C on compute kernels and d1.25× on mixed I/O workloads after release builds and architecture tuning.

### Key Differentiators
- **Agent-Operable**: Deterministic compilation, unambiguous grammar, structured diagnostics
- **Python-Level Ease**: Minimal sigils, obvious keywords, braces for stable diffs
- **Non-Intrusive Safety**: Lints by default, strict mode opt-in, no build-blocking walls
- **Assembly-Level Optimization**: NASM-like AIR with CPU-specific peephole and scheduling
- **Universal Interop**: C ABI, HPy (Python), N-API (Node), WASM/WASI, PE/ELF/Mach-O

---

## 2. User Profiles & Use Cases

### Primary Users
1. **Systems Programmers**: Need low-overhead binaries with precise memory and concurrency control
2. **Infrastructure Teams**: Require cross-platform tooling with deterministic builds
3. **AI Coding Agents**: Demand unambiguous parsing, deterministic compilation, and structured introspection
4. **Performance Engineers**: Need assembly-level visibility and CPU-specific tuning

### Core Use Cases
- High-performance compute kernels (FFT, GEMM, sorting, compression)
- System utilities and CLI tools with minimal runtime overhead
- Native extensions for Python and Node.js
- Cross-platform services requiring C interop
- WebAssembly modules for serverless environments
- Real-time systems requiring predictable performance

---

## 3. Language Philosophy

### Design Tenets
1. **Clarity Over Cleverness**: Readable code is maintainable code
2. **Explicit Over Implicit**: No hidden costs or magic behavior
3. **Safety Without Walls**: Advisories guide, strict modes enforce
4. **Performance Transparency**: Zero-cost abstractions with visible escape hatches
5. **Functional-First**: Expressions, ADTs, pattern matching, monads without ceremony

### Syntax Philosophy
- Minimal punctuation: `fn` not `function`, `let` not `var/const`, `use` not `import/require`
- No sigils for types: `Option<T>` not `T?`, `Result<T, E>` not throwing functions
- Braces for blocks: Stable git diffs and no indentation fragility
- Single propagation operator `?` for Result/Option unwrapping
- No implicit nulls: Option type is explicit and exhaustive

---

## 4. Lexical Structure

### 4.1 Character Set & Encoding
- **Source Encoding**: UTF-8 mandatory
- **Identifiers**: Unicode XID categories (XID_Start, XID_Continue)
- **Leading Underscore**: Allowed for private/unused bindings
- **Reserved Prefixes**: `__` reserved for compiler internals

### 4.2 Tokens

#### Keywords (Reserved)
```
fn      let     mut     const   static  type    trait   impl
use     mod     pub     as      self    Self    super   crate
if      else    match   for     while   loop    break   continue
return  yield   async   await   defer   unsafe  comptime
true    false   Some    None    Ok      Err     unreachable
```

#### Operators & Delimiters
```
Arithmetic:  +  -  *  /  %  **
Comparison:  ==  !=  <  <=  >  >=
Logical:     &&  ||  !
Bitwise:     &  |  ^  <<  >>  ~
Assignment:  =  +=  -=  *=  /=  %=  &=  |=  ^=  <<=  >>=
Access:      .  ::  ->  =>  ?  ??
Delimiters:  (  )  {  }  [  ]  ,  ;  :
Pipes:       |>  <|
```

#### Literals
- **Integers**: `42`, `0x2A`, `0b101010`, `1_000_000`
- **Floats**: `3.14`, `1.0e-5`, `0x1.2p3`
- **Strings**:
  - Raw: `r"no\nescape"`
  - Cooked: `"hello\nworld"`
  - Interpolated: `"value: {x}"`
- **Characters**: `'a'`, `'\n'`, `'\u{1F600}'`
- **Booleans**: `true`, `false`

#### Comments
- Line: `// comment`
- Block: `/* comment */` (nestable)
- Doc: `/// outer`, `//! inner`

### 4.3 Lexer Implementation
- **Strategy**: Table-driven NFA with explicit state graph
- **Disambiguation**: Maximal-munch with reserved-word priority
- **Ambiguity**: Zero tolerance; lexer must emit unambiguous token stream
- **Deliverable**: Machine-readable token catalog

---

## 5. Grammar & Syntax

### 5.1 Grammar Strategy
- **Declarations**: LL-style recursive descent
- **Expressions**: Pratt operator-precedence parser
- **Conflicts**: Zero shift-reduce or reduce-reduce conflicts
- **Deliverable**: Published CFG with precedence table

### 5.2 Operator Precedence (Highest to Lowest)
```
1.  Field access, method call:  .  ::  ()  []
2.  Unary:                       -  !  ~  *  &  &mut
3.  Exponentiation:              **
4.  Multiplicative:              *  /  %
5.  Additive:                    +  -
6.  Shift:                       <<  >>
7.  Bitwise AND:                 &
8.  Bitwise XOR:                 ^
9.  Bitwise OR:                  |
10. Comparison:                  ==  !=  <  <=  >  >=
11. Logical AND:                 &&
12. Logical OR:                  ||
13. Range:                       ..  ..=
14. Pipeline:                    |>  <|
15. Assignment:                  =  +=  -=  *=  /=  %=  &=  |=  ^=  <<=  >>=
16. Propagation:                 ?
```

### 5.3 Program Structure
```
program         ::= item*
item            ::= function | type_decl | trait_decl | impl_block | const_decl | mod_decl | use_decl
```

### 5.4 Functions
```aurora
fn add(x: i32, y: i32) -> i32 {
    x + y
}

fn generic<T: Display>(item: T) -> String {
    item.to_string()
}

async fn fetch(url: String) -> Result<String, Error> {
    http::get(url).await
}
```

### 5.5 Types
```aurora
type Point = struct { x: f64, y: f64 }

type Color = enum {
    Red,
    Green,
    Blue,
    RGB(u8, u8, u8),
}

type Result<T, E> = enum {
    Ok(T),
    Err(E),
}
```

### 5.6 Pattern Matching
```aurora
match result {
    Ok(value) => println("Success: {value}"),
    Err(error) => println("Error: {error}"),
}

let Point { x, y } = point;
```

### 5.7 Control Flow
```aurora
if condition {
    // ...
} else if other {
    // ...
} else {
    // ...
}

for item in collection {
    // ...
}

while condition {
    // ...
}

loop {
    if done { break }
}
```

---

## 6. Type System

### 6.1 Core Types
- **Integers**: `i8`, `i16`, `i32`, `i64`, `i128`, `u8`, `u16`, `u32`, `u64`, `u128`, `isize`, `usize`
- **Floats**: `f32`, `f64`
- **Boolean**: `bool`
- **Character**: `char` (Unicode scalar value)
- **String**: `str` (UTF-8 view), `String` (owned UTF-8)
- **Unit**: `()` (zero-sized type)
- **Never**: `!` (uninhabited type)

### 6.2 Compound Types
- **Tuples**: `(i32, f64, String)`
- **Arrays**: `[i32; 10]` (fixed size)
- **Slices**: `[i32]` (view into contiguous sequence)
- **Structs**: Named and tuple structs
- **Enums**: Algebraic data types with variants

### 6.3 Type Inference
- **Strategy**: Hindley-Milner with principal types
- **Occurs Check**: Enforced to prevent infinite types
- **Bidirectional**: Function signatures guide expression inference
- **Local**: Variables inferred within function scope
- **No Backtracking**: Inference must be deterministic and efficient

### 6.4 Generics
```aurora
fn identity<T>(x: T) -> T { x }

type Container<T> = struct {
    items: Vec<T>,
}

impl<T: Display> Container<T> {
    fn print_all(self) {
        for item in self.items {
            println("{item}")
        }
    }
}
```

### 6.5 Typeclasses (Traits)
```aurora
trait Display {
    fn to_string(self) -> String;
}

trait Add<Rhs = Self> {
    type Output;
    fn add(self, rhs: Rhs) -> Self::Output;
}

impl Add for i32 {
    type Output = i32;
    fn add(self, rhs: i32) -> i32 {
        self + rhs
    }
}
```

### 6.6 Null Safety
- **No Implicit Null**: No `null` or `undefined` values
- **Option Type**: Explicit nullable values
```aurora
type Option<T> = enum {
    Some(T),
    None,
}

fn find(list: [i32], target: i32) -> Option<usize> {
    for (i, item) in list.enumerate() {
        if item == target {
            return Some(i)
        }
    }
    None
}
```

---

## 7. Memory Model

### 7.1 Ownership
- **Default**: Values are owned by their binding
- **Move Semantics**: Assignment transfers ownership
- **Copy Trait**: Opt-in bitwise copying for simple types
```aurora
let s1 = String::from("hello");
let s2 = s1;  // s1 moved to s2, s1 invalid after this
```

### 7.2 Borrowing
- **Shared Borrow**: `&T` (immutable, multiple allowed)
- **Unique Borrow**: `&mut T` (mutable, exclusive)
- **Lifetime Elision**: Compiler infers lifetimes in common cases
```aurora
fn len(s: &String) -> usize {
    s.len()
}

fn append(s: &mut String, text: &str) {
    s.push_str(text)
}
```

### 7.3 Advisory Mode (Default)
- Borrow checker emits **advisories** not errors
- ARC (Atomic Reference Counting) auto-inserted at uncertain escape points
- Developer can audit ARC insertion sites via MIR dumps

### 7.4 Strict Mode (Opt-In)
```aurora
#![strict(borrow)]

fn example(x: &i32) -> &i32 {
    x  // Error: lifetime must be explicit in strict mode
}
```

### 7.5 Regions/Arenas
```aurora
use std::mem::Region;

fn process_tree(data: &[u8]) {
    let region = Region::new();
    let tree = parse_into(&region, data);
    // ... work with tree
    // region automatically freed at scope end
}
```

---

## 8. Effects System

### 8.1 Effect Annotations
```aurora
fn read_file(path: &str) -> String throws IO {
    fs::read_to_string(path)?
}

fn compute(x: i32) -> i32 pure {
    x * 2
}

fn allocate<T>(size: usize) -> Vec<T> alloc {
    Vec::with_capacity(size)
}
```

### 8.2 Effect Rows
- Functions carry effect rows: `IO`, `Alloc`, `Parallel`, `Unsafe`
- Subeffecting forms partial order: `Pure < Alloc < IO`
- Effect polymorphism: `fn work<E: IO>() -> Result<(), E>`

### 8.3 Advisory vs Strict
- **Advisory Mode**: Effects produce lints
- **Strict Mode**: Effects checked at type-check time
```aurora
#![strict(effects)]
```

---

## 9. Concurrency

### 9.1 Goroutines
```aurora
spawn {
    println("Running in separate goroutine")
}

let handle = spawn {
    expensive_computation()
};
let result = handle.join();
```

### 9.2 Channels
```aurora
let (tx, rx) = channel::<i32>();

spawn move {
    tx.send(42);
};

let value = rx.recv();
```

### 9.3 Async/Await
```aurora
async fn fetch_data(url: &str) -> Result<String, Error> {
    let response = http::get(url).await?;
    let body = response.text().await?;
    Ok(body)
}

async fn main() {
    let data = fetch_data("https://api.example.com").await?;
    println("Data: {data}")
}
```

### 9.4 Actors
```aurora
actor Counter {
    count: i32,

    fn increment(&mut self) {
        self.count += 1
    }

    fn get(&self) -> i32 {
        self.count
    }
}

let counter = Counter::spawn(0);
counter.send(Counter::increment);
let value = counter.ask(Counter::get).await;
```

---

## 10. Error Handling

### 10.1 Result Type
```aurora
type Result<T, E> = enum {
    Ok(T),
    Err(E),
}

fn divide(a: i32, b: i32) -> Result<i32, &str> {
    if b == 0 {
        Err("division by zero")
    } else {
        Ok(a / b)
    }
}
```

### 10.2 Propagation Operator
```aurora
fn compute() -> Result<i32, Error> {
    let x = read_input()?;
    let y = parse_number(x)?;
    let z = process(y)?;
    Ok(z)
}
```

### 10.3 Pattern Matching
```aurora
match divide(10, 2) {
    Ok(result) => println("Result: {result}"),
    Err(error) => println("Error: {error}"),
}
```

---

## 11. Metaprogramming

### 11.1 Hygienic Macros
```aurora
macro unless($cond:expr, $body:block) {
    if !($cond) $body
}

unless(is_valid, {
    println("Invalid!")
})
```

### 11.2 Comptime
```aurora
const TABLE: [u32; 256] = comptime {
    let mut table = [0u32; 256];
    for i in 0..256 {
        table[i] = compute_crc(i);
    }
    table
};
```

### 11.3 Derives
```aurora
#[derive(Debug, Clone, PartialEq)]
type Point = struct { x: i32, y: i32 };
```

---

## 12. Interoperability

### 12.1 C ABI
```aurora
#[export]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

extern "C" {
    fn strlen(s: *const u8) -> usize;
}
```

### 12.2 Python (HPy)
```aurora
#[hpy_export]
fn process(data: &str) -> String {
    data.to_uppercase()
}
```

### 12.3 Node.js (N-API)
```aurora
#[napi_export]
fn hash(input: String) -> u64 {
    compute_hash(&input)
}
```

### 12.4 WebAssembly
```aurora
#[wasm_export]
fn fibonacci(n: u32) -> u32 {
    if n <= 1 { n } else { fibonacci(n-1) + fibonacci(n-2) }
}
```

---

## 13. Standard Library

### 13.1 Core Modules
- `std::io` - File I/O, stdin/stdout, buffering
- `std::fs` - Filesystem operations
- `std::net` - TCP/UDP networking, HTTP client
- `std::collections` - Vec, HashMap, HashSet, BTreeMap, etc.
- `std::string` - String manipulation, UTF-8 validation
- `std::process` - Subprocess spawning and management
- `std::sync` - Synchronization primitives (Mutex, RwLock, Atomic)
- `std::time` - Time measurement and duration
- `std::regex` - Regular expression engine
- `std::json` - JSON parsing and serialization
- `std::env` - Environment variables and command-line args

### 13.2 Design Principles
- Simple, obvious names (no cryptic abbreviations)
- Unsafe APIs behind explicit imports or `unsafe` blocks
- Consistent error handling via Result
- Zero-cost abstractions where possible
- Batteries included for common tasks

---

## 14. Tooling

### 14.1 CLI (`aurora`)
```bash
ax init <name>          # Create new project
ax build                # Compile project
ax run                  # Build and execute
ax test                 # Run test suite
ax bench                # Run benchmarks
ax fmt                  # Format code
ax lint                 # Run linter
ax doc                  # Generate documentation
ax cross --target=...   # Cross-compile
ax objdump             # Disassemble AIR/machine code
```

### 14.2 Build Profiles
```toml
[package]
name = "myproject"
version = "0.1.0"

[profile.dev]
opt_level = 0
debug = true

[profile.release]
opt_level = 3
lto = "thin"
cpu = "native"
```

### 14.3 LSP Features
- Completions (context-aware)
- Go-to-definition
- Find references
- Type hover
- Diagnostics with fix-its
- Rename refactoring
- Macro expansion view
- Inline MIR/AIR preview
- Code actions

### 14.4 Diagnostics
```json
{
  "id": "E0501",
  "category": "borrow_checker",
  "severity": "error",
  "message": "cannot borrow `x` as mutable because it is also borrowed as immutable",
  "spans": [
    {"file": "main.ax", "line": 10, "column": 5, "length": 1, "label": "mutable borrow occurs here"},
    {"file": "main.ax", "line": 8, "column": 5, "length": 1, "label": "immutable borrow occurs here"}
  ],
  "fix_its": [
    {"description": "Consider cloning the value", "edits": [...]}
  ],
  "doc_url": "https://docs.auroralang.org/errors/E0501",
  "confidence": 0.95
}
```

---

## 15. IR & Backend

### 15.1 MIR (Mid-Level IR)
- **Form**: SSA (Single Static Assignment)
- **Features**: Explicit effect edges, source span preservation
- **Optimizations**: Inlining, SROA, GVN, LICM, DCE, NRVO, devirtualization, loop-SIMD
- **Dumps**: Human-readable text format, JSON export

### 15.2 AIR (Assembly-Level IR)
- **Style**: NASM-like textual assembly
- **Dialects**: x86_64, AArch64, RISC-V
- **Optimizations**: Peephole (mov collapse, LEA patterns), instruction scheduling
- **CPU Profiles**: Skylake, Ice Lake, Zen, Neoverse
- **Round-Trip**: AIR ’ parse ’ AIR must be stable

### 15.3 Backends
- **Primary**: LLVM (x86_64, AArch64, RISC-V)
- **Fast Debug**: Cranelift
- **Direct**: Optional AIR ’ obj emitter for specific targets

### 15.4 Linking
- **Linker**: LLD (LLVM linker)
- **Formats**: PE/COFF (Windows), ELF (Linux), Mach-O (macOS)
- **Debug Info**: DWARF, PDB, SEH (Windows)

---

## 16. Performance Targets

### 16.1 Compile Time
- Dev builds: Within 1.2× of Go compiler speed
- Incremental: <1s for typical single-file changes
- Release builds: ThinLTO acceptable for production

### 16.2 Runtime Performance
- Compute kernels: d1.10× of optimized C
- Mixed I/O workloads: d1.25× of optimized C
- Memory overhead: d1.1× of Rust for comparable tasks

### 16.3 Binary Size
- Minimal runtime overhead (no large runtime or GC)
- Link-time optimization removes unused code
- Optional stripping of debug info

---

## 17. Testing Strategy

### 17.1 Test Types
- **Unit Tests**: Per-function verification
- **Property Tests**: QuickCheck-style generative testing
- **Golden Tests**: AST/MIR/AIR snapshot comparison
- **Differential Tests**: Semantics verification vs C implementations
- **Integration Tests**: End-to-end compiler pipeline
- **Stress Tests**: Concurrency races with reproducible seeds

### 17.2 Cross-Architecture CI
- Native: Linux x86_64, Windows x86_64, macOS ARM64
- Emulated: AArch64, RISC-V, MIPS, SPARC (via QEMU)

### 17.3 Reproducibility
- Byte-for-byte identical binaries with fixed seeds
- Build fingerprinting for toolchain verification
- Content-addressed artifact caching

---

## 18. Security & Supply Chain

### 18.1 Reproducible Builds
- Lockfile for dependencies (content-addressed)
- Fixed compiler version and flags
- Deterministic codegen (no timestamps in artifacts)

### 18.2 SBOM (Software Bill of Materials)
- Auto-generated dependency graph
- License tracking
- Vulnerability scanning integration

### 18.3 Vendoring
- Offline builds with vendored dependencies
- Cryptographic signature verification
- Supply chain policy enforcement

### 18.4 Safety Policies
```toml
[policy]
unsafe = "deny"          # Disallow unsafe blocks
reflection = "deny"      # Disallow runtime reflection
gc = "deny"              # Disallow garbage collection
comptime_fs = "allow"    # Allow comptime filesystem access
comptime_net = "deny"    # Disallow comptime network access
```

---

## 19. Roadmap

### MVP (Foundation Phase)
**Goal**: Self-hosted compiler subset, basic tooling, PE/ELF output

**Components**:
- Lexer (NFA, maximal-munch)
- Parser (Pratt + CFG)
- AST (arena allocation, spans, hygiene)
- HM type inference
- Basic typeclasses (no specialization)
- Borrow checker (advisory mode)
- MIR (SSA, basic optimizations)
- AIR (x86_64, peephole)
- LLVM backend (COFF/ELF)
- CLI (build, run, test, fmt)
- LSP (completions, diagnostics)
- JSON diagnostics
- Unit/golden tests

**Acceptance Criteria**:
- Compiles itself
- Zero grammar ambiguities
- Deterministic builds
- Passes conformance test suite

### Beta (Robustness Phase)
**Goal**: Production-ready compiler, cross-platform support, ecosystem bridges

**Components**:
- Effects system (strict mode)
- Actors/supervisors
- AArch64/RISC-V backends
- WASI support
- Python HPy bridge
- Node N-API bridge
- Derive macros
- ThinLTO
- Profiler integration
- Content-addressed cache
- SBOM generation

**Acceptance Criteria**:
- Cross-arch CI green
- Interop tests pass for C/Python/Node
- Performance within 1.2× targets

### 1.0 (Production Release)
**Goal**: Stable ABI, frozen spec, ecosystem maturity

**Components**:
- Specialization with coherence
- Optional mini-GC for plugins
- Debugger polish (DWARF/PDB)
- Supply chain policies enforced
- Performance targets fully met
- Documentation complete
- Ecosystem bridges GA

**Acceptance Criteria**:
- ABI stability report published
- Language spec frozen
- All KPIs met

---

## 20. Success Metrics

### Correctness
-  Zero undefined behavior in safe code
-  Grammar ambiguity report empty
-  Principal types for all expressions
-  AIR round-trips successfully

### Performance
-  Compute kernels d1.10× of C
-  I/O workloads d1.25× of C
-  Dev builds within 1.2× of Go
-  Memory overhead d1.1× of Rust

### Determinism
-  Identical binaries from identical inputs
-  Cross-machine reproducibility
-  Stable MIR/AIR across patches

### Quality
-  100% coverage on critical paths
-  Cross-arch tests pass
-  Concurrency stress tests pass
-  Documentation complete and accurate

---

## 21. Non-Goals

We explicitly **reject**:
- L Mandatory borrow annotations
- L Global exceptions
- L Indentation-as-syntax
- L Dynamic dispatch without static constraints
- L Hidden garbage collection
- L Context-dependent grammar
- L Implicit nulls
- L Safety that blocks builds by default

---

## 22. Open Questions

### For MVP Resolution
1. **Macro Syntax**: Procedural vs pattern-based macro invocation syntax?
2. **Async Runtime**: Bundled runtime vs bring-your-own?
3. **Module System**: Explicit exports vs public-by-default?
4. **Numeric Literals**: Suffix-based typing (`42i32`) vs inference-based?

### For Beta Resolution
1. **Specialization**: Min-overlap or full coherence checking?
2. **GC Integration**: Opt-in per-crate or per-function?
3. **ABI Versioning**: Semver-based or hash-based?

---

## 23. References

- **Lexer/Parser**: Dragon Book (Compilers: Principles, Techniques, and Tools)
- **Type Inference**: Principal Type Schemes for Functional Programs (Damas & Milner)
- **Borrow Checking**: RustBelt: Securing the Foundations of the Rust Programming Language
- **Effect Systems**: Algebraic Effects and Handlers (Plotkin & Pretnar)
- **LLVM**: LLVM: A Compilation Framework for Lifelong Program Analysis & Transformation
- **Assembly Optimization**: Intel 64 and IA-32 Architectures Optimization Reference Manual
- **Supply Chain**: SLSA Framework (Supply-chain Levels for Software Artifacts)

---

**Document Status**: Living specification, subject to refinement during MVP phase.
**Last Updated**: 2025-11-04
**Next Review**: Upon completion of Lexer + Parser implementation
