# Name
**Aurora**

# Tagline
*"Aurora – Systems brilliance with humane clarity."*

# Philosophy
Aurora aims to unite the mechanical sympathy of systems languages with the approachability of high-level scripting. Its philosophy rests on three pillars:
1. **Fearless performance** – match C/C++ and Rust on tight loops and concurrent workloads while keeping predictable latency.
2. **Explicit safety** – make correctness the default through ownership, effects, and exhaustive reasoning.
3. **Delightful ergonomics** – favor readable syntax, powerful inference, and best-in-class tooling so productivity scales with teams.

# Language Overview
- Paradigm: multi-paradigm (functional + imperative + data-oriented)
- Type system: static, globally inferred, Hindley–Milner core with traits/typeclasses and constrained multiple dispatch
- Memory: ownership-first with borrow checking, hybrid ARC/region fallback, optional precise GC for dynamic reflection
- Concurrency: structured async/await, goroutines + channels, actors with supervision, work-stealing runtime
- Interop: first-class C ABI, TypeScript structural typing bridge, JVM/.NET backends, WASM/WASI, N-API, Python HPy, GPU kernels
- Toolchain: `aur` CLI (Cargo/Zig-inspired), formatter, linter, doc generator, package registry, LSP
- Compilation: LLVM + Cranelift backends, SSA-based MIR, effect-aware optimizations, incremental builds
- Target domains: systems software, network services, data processing, ML/AI integration, embedded/WASM, cross-language SDKs

| Aspect | Aurora | Influences |
| --- | --- | --- |
| Type system | Algebraic data types, traits, structural adapters, optional higher-rank types | Haskell, OCaml, Rust, TypeScript |
| Memory | Ownership + borrow checker with ARC/regions fallback | Rust, Swift ARC, MLKit |
| Concurrency | Async/await, goroutines/channels, actors/supervisors | Rust, Go, Erlang/Elixir |
| Error handling | `Result`/`Option`, `?` operator, effect annotations | Rust, Haskell |
| Syntax | Expression-oriented, braces, pipelines, comprehensions | Python, Scala, Elixir |
| Metaprogramming | Hygienic macros, comptime execution | Racket, Rust, Zig |
| Tooling | `aur` CLI, fmt, lint, docs, test | Rust/Cargo, Zig |
| Interop | C ABI, WASM, JVM/.NET bridges | C, AssemblyScript, Kotlin |

# Feature-by-Feature Map
1. **Algebraic Data Types & Pattern Matching** – lifted from Haskell/OCaml: sum/product types, exhaustive `match`, pattern guards. Adapted with Rust-style `match` syntax and Pythonic readability.
2. **Type inference** – Hindley–Milner baseline (Haskell/OCaml) extended with pragmatic defaults (Rust style `impl Trait` hints). Optional rank-N types off by default for ergonomics.
3. **Typeclasses/Traits** – Haskell typeclasses blended with Rust traits: associated types, default methods, blanket impls, opt-in specialization via where-clauses.
4. **Null safety** – Swift/Kotlin style optionals via `Option<T>` and postfix `?` sugar; no implicit null.
5. **Multiple dispatch** – Julia-inspired but constrained by trait bounds to avoid ambiguity; compile-time selection only.
6. **Structural typing adapters** – TypeScript-like structural interfaces for FFI boundaries; default nominal typing to preserve clarity.
7. **Generics** – Rust/C++20 concepts for constraints, TypeScript ergonomics for defaulted type params; monomorphization default, reified generics opt-in.
8. **Ownership & Borrowing** – Rust-inspired lifetimes, mutable/immutable borrows, aliasing rules. Lifetimes inferred but annotatable.
9. **ARC + Regions** – Swift ARC with weak refs for cycles; MLKit-style region arenas for batch allocations. Compiler selects best strategy.
10. **Effect System** – Koka/Haskell influence: `effect` annotations for IO, alloc, parallelism, unsafe; effect polymorphism.
11. **Concurrency constructs** – Go goroutines/channels with Rust safety, Erlang actors/supervisors for resilience, Rust/Swift async/await for zero-cost state machines.
12. **Error handling** – Rust `Result` and `?`; effect-checked throws.
13. **Syntax** – Python readability, braces to avoid indentation semantics, Scala/Rust expression orientation, Elixir pipelines, Python comprehensions, Swift string interpolation.
14. **Metaprogramming** – Racket/Rust hygienic macros, Zig comptime execution, Rust const eval.
15. **Modules & Packages** – Cargo/Zig-inspired workspaces, lockfiles, reproducible builds; Go/Swift single-file scripting.
16. **Interop** – C ABI, WASM/WASI, Kotlin/Scala interop for JVM, C# for .NET, Node N-API, Python HPy, TypeScript structural typing.
17. **Standard library** – Python-like batteries, Rust iterators, Swift value semantics, Rust regex, Go crypto ethos.
18. **AI/DSL hooks** – Swift for TensorFlow & Julia for ML runtime integration, C# LINQ for query DSLs, Rust-CUDA/HIP for GPU kernels.

# Formal/Informal Spec
## Lexical Structure
- Unicode source, UTF-8.
- Identifiers: start with `_` or letter, continue with letters/digits/`_`. Backtick-quoted for raw identifiers.
- Keywords: `fn`, `let`, `mut`, `type`, `struct`, `enum`, `trait`, `impl`, `effect`, `async`, `await`, `spawn`, `actor`, `supervisor`, `match`, `if`, `else`, `for`, `while`, `loop`, `return`, `break`, `continue`, `comptime`, `macro`, `module`, `import`, `pub`, `use`, `as`, `where`, `region`, `arena`, `unsafe`, `defer`, `test`.
- Literals: integers (`123`, `0xFF`), floats (`1.0`, `1e-3`), booleans, chars, strings with interpolation `"Hello ${name}"`, byte strings `b""`.
- Comments: `//` line, `/* */` block, `///` doc comment, `//!` module doc.

## Syntax
Aurora uses braces for blocks; semicolons optional (inserted at line breaks). Expressions return last expression value.

### EBNF (excerpt)
```
program       = { module_item } ;
module_item   = function | type_decl | trait_decl | impl_block | const_decl | macro_decl | effect_decl ;
function      = attributes? "pub"? "fn" ident generics? param_list return_type? effect_clause? block ;
generics      = "<" generic_param { "," generic_param } ">" ;
param_list    = "(" [ param { "," param } ] ")" ;
param         = pattern ":" type_expr default_arg? ;
default_arg   = "=" expr ;
return_type   = "->" type_expr ;
effect_clause = "throws" effect_set ;
effect_set    = "[" effect_ident { "," effect_ident } "]" ;
block         = "{" { stmt } "}" ;
stmt          = let_stmt | expr_stmt | loop_stmt | match_stmt | block ;
let_stmt      = "let" ("mut")? pattern ":"? type_expr? "=" expr ;
expr_stmt     = expr ";"? ;
expr          = lambda | match_expr | if_expr | loop_expr | await_expr | assign_expr ;
assign_expr   = logic_or { assign_op logic_or } ;
match_expr    = "match" expr "{" { pattern "=>" expr guard? "," } "}" ;
guard         = "if" expr ;
pattern       = ident | "_" | tuple_pattern | struct_pattern | enum_pattern | literal_pattern ;
trait_decl    = "trait" ident generics? trait_body ;
impl_block    = "impl" generics? trait_impl? type_expr trait_where? "{" { impl_item } "}" ;
macro_decl    = "macro" ident "{" macro_rules "}" ;
```

### Basic Syntax Examples
```aurora
/// Hello world
fn main() {
    println("Hello, Aurora!");
}

fn variables() {
    let greeting = "hi";          // immutable
    let mut counter: i32 = 0;      // mutable with explicit type
    counter += 1;
    let greeting = greeting.to_upper(); // shadowing allowed
}

fn functions() {
    fn add<T: Add>(a: T, b: T) -> T { a + b }
    let inc = |x| add(x, 1);
    println("{}", inc(4));
}

fn defaults(name: Str = "world", *, excited: Bool = false) {
    let suffix = if excited { "!" } else { "." };
    println("Hello ${name}${suffix}");
}
```

### Lambdas & Closures
```aurora
fn closures() {
    let mut total = 0;
    let add = |x| {
        total += x;
        total
    };
    assert_eq(add(3), 3);
    assert_eq(add(2), 5);
}
```

### Generics & Constraints
```aurora
trait Summable {
    type Item;
    fn sum(self) -> Self::Item;
}

fn sum_all<T>(items: T) -> T::Item where T: IntoIterator + Summable {
    items.sum()
}
```

### ADTs & Pattern Matching
```aurora
enum Json {
    Null,
    Bool(Bool),
    Number(F64),
    String(Str),
    Array(Vec<Json>),
    Object(Map<Str, Json>),
}

fn decode_bool(value: Json) -> Result<Bool, Str> {
    match value {
        Json::Bool(b) => Ok(b),
        other => Err("Expected bool, found ${other.kind()}"),
    }
}

fn pattern_guards(x: Option<i32>) -> Str {
    match x {
        Some(v) if v % 2 == 0 => "even",
        Some(_) => "odd",
        None => "none",
    }
}
```

### Multiple Dispatch
```aurora
trait Area {
    fn area(self) -> F64;
}

struct Circle { radius: F64 }
struct Rect { width: F64, height: F64 }

@dispatch
fn intersect<A, B>(a: A, b: B) -> Bool where (A, B): Intersect {
    Intersect::intersects(a, b)
}
```

### Structural Typing Adapters
```aurora
struct JsPoint { x: F64, y: F64 }

interface PointLike { x: F64, y: F64 }

fn length(p: impl PointLike) -> F64 {
    (p.x * p.x + p.y * p.y).sqrt()
}
```

### Ownership & Borrowing
```aurora
fn ownership() {
    let v = Vec::new();
    let mut v2 = v;      // move
    v2.push(1);
    let ref = &v2;       // immutable borrow
    println("len ${ref.len()}");
    let ref_mut = &mut v2; // mutable borrow (exclusive)
    ref_mut.push(2);
}

fn lifetimes<'a>(input: &'a Str) -> &'a Str {
    input.trim()
}
```

### Regions & ARC
```aurora
fn use_region() {
    region r {
        let arena_vec = Vec::with_region(r);
        // allocations freed when region exits
    }
}

fn shared_arc() {
    let rc = Arc::new(Node::new());
    let weak = rc.downgrade();
}
```

### Effect System
```aurora
effect Async

effect fn fetch(url: Str) -> Result<Response, Error> throws [IO, Async] {
    let body = await http::get(url)?;
    Ok(body)
}
```

### Collections, Iterators, Comprehensions, Pipelines
```aurora
fn collections() {
    let nums = [1, 2, 3, 4];
    let evens = [n for n in nums if n % 2 == 0];
    let squares = nums.iter().map(|n| n * n).collect<Vec<_>>();
    let total = nums |> iter::filter(|n| n > 2) |> iter::sum();
}
```

### Concurrency
```aurora
async fn async_fetch(urls: Vec<Str>) -> Result<Vec<Response>, Error> {
    urls.into_iter()
        .map(|url| async move { http::get(url).await })
        .collect::<AsyncVec<_>>()
        .await?
}

fn goroutines() {
    let (tx, rx) = channel::<i32>();
    for i in 0..10 {
        spawn move || tx.send(i);
    }
    let total = rx.into_iter().sum();
    println("total ${total}");
}

actor Logger {
    fn start(self) {
        loop {
            match self.receive() {
                Msg::Log(line) => println(line),
            }
        }
    }
}

supervisor WebStack {
    children: [Logger, HttpServer],
    strategy: Strategy::OneForAll,
}
```

### Error Handling
```aurora
fn read_config(path: Path) -> Result<Config, Error> {
    let data = fs::read_to_string(path)?;
    serde::from_json(data)
}

fn handle_pattern(x: Option<i32>) -> i32 {
    match x {
        Some(v) => v,
        None => unreachable("config ensures presence"),
    }
}
```

### Modules & Packages
```aurora
// src/lib.aur
pub module net {
    pub fn connect(addr: Str) -> Result<Socket, Error> {
        // ...
    }
}

// src/main.aur
import { net } from "crate::net";

fn main() {
    let socket = net::connect("localhost:8080")?;
}
```

### FFI Examples
```aurora
// C interop
extern "C" fn qsort(data: &mut [i32]) {
    c::qsort(data.as_mut_ptr(), data.len(), sizeof(i32), comparator);
}

// Python HPy
extern "python" fn py_len(obj: PyObject) -> Result<i64, PyError> {
    obj.len()
}

// Node N-API
extern "node" fn napi_add(env: Env, info: CallbackInfo) -> napi::Result<Value> {
    let (a, b) = info.get_args::<(i32, i32)>(env)?;
    Ok(a + b)
}

// JVM interop
extern "jvm" fn make_list() -> jvm::List<String> {
    jvm::ArrayList::new()
}
```

### Macros & Comptime
```aurora
macro rules! make_vec {
    () => { Vec::new() };
    ($($x:expr),+ $(,)?) => {{
        let mut tmp = Vec::new();
        $(tmp.push($x);)+
        tmp
    }};
}

comptime fn perfect_hash(keys: &[Str]) -> PerfectHash {
    // run deterministic search at compile time
}
```

### Testing & Doc Tests
```aurora
fn add(a: i32, b: i32) -> i32 { a + b }

test "addition" {
    assert_eq(add(2, 3), 5);
}

/// Adds two numbers.
///
/// ```aurora
/// assert_eq(add(1, 2), 3);
/// ```
```

### Build & Run Commands
- `aur new app my_app`
- `aur build`
- `aur run`
- `aur test`
- `aur fmt`
- `aur lint`
- `aur doc`
- `aur cross --target wasm32-wasi`

### Style Guide
- **Naming:** `snake_case` for variables/functions/modules, `PascalCase` for types, `SCREAMING_SNAKE` for constants, macros lower-case with underscores.
- **Formatting:** enforced by `aur fmt`; 4-space indentation; trailing commas allowed; pipeline operator spaced `value |> func`.
- **Idioms:** prefer `match` for branching on enums; use `?` for error propagation; structure concurrency with `async`/`await`.
- **Lints:** `aur lint` defaults to deny unused mut, missing `await`, unreachable code, unsafe usage without justification.

# Memory & Effects
- **Ownership graph:** each value has a single owner; moves transfer ownership; types implement `Copy` when trivial.
- **Borrowing:** `&T` immutable, `&mut T` mutable, tracked by borrow checker; lifetimes inferred with explicit `'a` when ambiguity occurs.
- **Hybrid memory:** compiler chooses stack vs heap; polymorphic code defaults to ARC (Automatic Reference Counting); developers can mark `#[own]` to force ownership semantics.
- **Regions/arenas:** `region` blocks allocate in arenas; destructors run at region exit; arena handles `Drop` order deterministic.
- **Cycle handling:** `Weak` references break ARC cycles; optional `#[reflect]` modules enable precise mini-GC.
- **Effect system:** functions declare `throws [IO, Async]`; compiler infers minimal effect set; effect polymorphism via `effect fn f<E: EffectSet>(...) throws [E]`.
- **Unsafe:** `unsafe` blocks isolated; require justification comment; lints enforce documentation.

# Concurrency
- **Goroutines & channels:** `spawn` creates lightweight task; `channel<T>()` returns MPMC channel; borrow checker ensures sends/receives safe.
- **Async/await:** `async fn` compiles to state machines; `await` points inserted; futures pinned and send-safe by default.
- **Actors:** `actor` keyword defines mailbox-driven components; `self.receive()` yields typed messages; supervisors define restart strategies.
- **Work stealing scheduler:** double-ended deque per worker, victims steal from tail. Preemption via cooperative yields at await/block boundaries.
- **Structured concurrency:** `task::scope` ensures child tasks complete; cancellation tokens propagate.

# Error Handling
- `Result<T, E>` and `Option<T>` fundamental; `?` operator; typed effects permit `throws [Timeout]` etc.
- `unreachable!` requires justification string; `panic!` emits effect `Panic`; default policy aborts in release, unwinds in debug.
- Exhaustive `match` enforced; wildcard `_` allowed but warns if not last.

# Metaprogramming & comptime
- **Hygienic macros:** hygienic by default; `macro rules!` with pattern-matching expansions; `macro derive` for generated impls.
- **Comptime:** `comptime fn` executed during compilation with deterministic sandbox (no IO unless `#[allow(io)]`).
- **Const evaluation:** `const` expressions evaluated at compile time; CTFE engine shares interpreter with comptime.
- **Typed macros:** macros can specify output type and run type-checker on expansion.

# Modules/Packages/Builds
- Modules map to file paths; `module foo.bar;` corresponds to `src/foo/bar.aur`.
- Packages defined by `Aurora.toml`; workspaces support multiple crates with shared lockfile `Aurora.lock`.
- `aur` CLI handles dependency resolution, build caching, cross compilation, binary/lib/test targets.
- Build profiles: `dev`, `release`, `bench`, custom; each config in manifest.
- Single-file mode: `aur run script.aur` compiles to temp cache.

# Interop & Targets
- **C:** stable ABI; `extern "C"` functions; header generator `aur bindgen`.
- **Python:** HPy backend; `aur py-ext` builds wheel; structural typing ensures type hints.
- **Node:** N-API wrappers; `aur node-ext` outputs `.node` modules.
- **WASM/WASI:** `aur build --target wasm32-wasi`; generates WASI polyfill; optional component model.
- **JVM/.NET:** optional backends (Kotlin/Scala style) translate MIR to JVM bytecode/CLR IL; maintain trait mapping.
- **TypeScript bridge:** `structural interface` definitions generate `.d.ts`.
- **FFI safety:** `#[repr(C)]` ensures layout; lifetime annotations ensure safe boundaries.
- **Transpiler mode:** `aur transpile --to=rust` outputs Rust for incremental migration; `--to=c` for legacy.

# Standard Library Plan
- **Core:** `std::prelude`, `Option`, `Result`, iterators, smart pointers, ownership primitives.
- **Collections:** `Vec`, `Array`, `Map`, `Set`, `Deque`, `RingBuffer`, `SmallVec`.
- **Concurrency:** `std::task`, `std::channel`, `std::actor`, `std::sync` (Mutex, RwLock, Atomic).
- **Networking:** HTTP client/server, TLS (Go-like curated crypto).
- **Filesystem:** path utilities, watchers, temp dirs.
- **Serialization:** `serde`-like derive for JSON, YAML, binary.
- **Regex:** Rust's DFA/NFA hybrid; guaranteed linear time.
- **Math & Numerics:** big integers, SIMD vectors, complex numbers.
- **AI hooks:** `std::ml` bridging to tensor runtimes (ONNX, Torch, XLA), with trait-based dispatch.
- **Query DSL:** `std::linq` for relational operations, compile-time verified via effectful macros.
- **GPU kernels:** `std::gpu` with attribute `#[kernel]` generating SPIR-V/CUDA.

# Tooling (CLI, LSP, fmt, lint, doc, test)
- **CLI (`aur`):** commands: `new`, `init`, `add`, `update`, `remove`, `build`, `run`, `test`, `bench`, `fmt`, `lint`, `doc`, `publish`, `yank`, `vendor`, `cross`, `transpile`, `bindgen`, `package`.
- **Formatter:** AST-driven, idempotent; respects configuration `aur fmt --check`.
- **Linter:** `aur lint` similar to Rust Clippy; categories: correctness, style, perf, pedantic.
- **Doc generator:** extracts `///` comments into HTML+Markdown; supports literate tests.
- **Testing:** `aur test` runs unit, integration; property testing `#[quickcheck]`; benchmarking harness `aur bench` (criterion-like).
- **Language Server:** features: completions, go-to-definition, doc hover, type-on-hover, inline borrow checker hints, code actions, formatting, rename, refactor extract function, symbol search.
- **REPL:** `aur repl` uses JIT (Cranelift) for interactive prototyping.

# Compiler Architecture
## Front End
- **Lexer:** table-driven DFA; handles raw strings, nested block comments.
- **Parser:** Pratt parser for expressions, recursive descent for declarations; grammar unambiguous.
- **Macro expansion:** hygienic expansion in AST; resolves `macro_rules` before name resolution.
- **Name resolution:** multi-pass scope graph; modules produce symbol tables; handles re-exports.
- **Type inference:** constraint solver (Algorithm W variant) with extensions for traits and higher-ranked types; uses unification with occurs check; specialization resolved via coherence rules (Rust-style orphan rules adapted to allow structural adapters).
- **Borrow checker:** MIR-based dataflow similar to Rust's Polonius prototype; computes borrow regions using graph constraints; integrates with lifetime inference.
- **Effect checker:** effect lattice; tracks latent effects; effect polymorphism via row variables.
- **HIR:** desugared, type-annotated IR preserving spans for diagnostics; effects annotated; pattern matches compiled to decision trees.

## Middle End
- **MIR:** SSA form with explicit control flow, borrow state, effect flags.
- **Optimizations:**
  - Inlining guided by cost model.
  - Scalar replacement of aggregates (SROA).
  - Global value numbering (GVN) & sparse conditional constant prop.
  - Loop-invariant code motion (LICM).
  - Escape analysis to stack-allocate closures; decides ARC elision.
  - Devirtualization through trait resolution.
  - Effect-aware dead code elimination (DCE) prevents removal of IO.
  - Borrow-aware Named Return Value Optimization (NRVO).
  - Auto-vectorization via LLVM + pattern-driven rewrites.
  - Region inference for arenas to reduce ARC.
- **Generics:** monomorphization default; reified generics emit metadata tables; MIR caches to allow incremental recompile.
- **Comptime:** interpreter executes MIR subset; caching ensures determinism.
- **Incremental compilation:** dep graph hashed per item; similar to rustc's query system; cross-module inlining via ThinLTO.

## Back End
- **Primary backend:** LLVM for x86_64, ARM64, RISC-V; leverages LLD for linking.
- **Debug backend:** Cranelift for fast dev builds, powering REPL and JIT.
- **WASM:** MIR -> Wasm (via Cranelift or LLVM); generates WASI shims.
- **JVM/.NET:** optional lowering to bytecode/IL; type erasure rules ensure trait mapping.
- **ABI:** stable C ABI; name mangling scheme `AUR_{hash}`; metadata for reflection.
- **Debug info:** DWARF on native; source maps for WASM; `.aurdbg` files for advanced tooling.
- **Profiler:** `aur profile` uses sampling (perf) and instrumentation hooks; integrates with async runtime.

## Runtime
- **Scheduler:** work-stealing deques (Chase-Lev). Async runtime uses IOCP/epoll/kqueue depending on platform.
- **Memory:** `auralloc` hybrid allocator (Hoard-style front + bump alloc for arenas); deterministic destructors.
- **ARC/GC:** reference counting per object; cycle detection via trial deletion; optional precise GC triggered when reflection features used.
- **FFI support:** trampolines for calling conventions; bridging runtime ensures safety.

# Backends & Runtime
- Native (LLVM) targeting Linux, macOS, Windows.
- Embedded: bare-metal profiles with no_std subset; linkable to C RTOS.
- WASM/WASI for serverless, edge compute.
- GPU: SPIR-V/CUDA backend for `#[kernel]` functions; uses MLIR pipeline to generate GPU IR, then ptx/spirv.
- JVM/.NET optional modules.
- Tooling for cross compilation: `aur target add`, caches prebuilt stdlib per target.

# Security & Supply Chain
- Reproducible builds via deterministic codegen and pinned dependencies.
- Content-addressed cache for artifacts; `aur vendor` for offline builds.
- Package signing (Ed25519); `aur verify` ensures signatures.
- SBOM generated per build (CycloneDX).
- Comptime sandbox denies network/filesystem unless flagged `#[allow(io)]`.
- `unsafe` usage requires `#[allow(unsafe)]` in manifest with justification; CI lints enforce.

# Example Programs
1. **Hello world**
```aurora
fn main() {
    println("Hello, Aurora!");
}
```

2. **Fibonacci (iterative + tail recursion)**
```aurora
fn fib_iter(n: u32) -> u64 {
    let (mut a, mut b) = (0, 1);
    for _ in 0..n {
        let tmp = a + b;
        a = b;
        b = tmp;
    }
    a
}

effect fn fib_tail(n: u32, a: u64 = 0, b: u64 = 1) -> u64 {
    if n == 0 { a } else { fib_tail(n - 1, b, a + b) }
}
```

3. **HTTP fetch with async/await and Result**
```aurora
async fn main() -> Result<(), Error> {
    let resp = await http::get("https://example.com")?;
    println("status ${resp.status}");
    Ok(())
}
```

4. **Channel fan-in/fan-out**
```aurora
fn fan_in() {
    let (tx, rx) = channel::<i32>();
    for worker in 0..4 {
        let tx = tx.clone();
        spawn move || {
            for job in 0..25 {
                tx.send(worker * 25 + job);
            }
        };
    }
    drop(tx);
    let total = rx.into_iter().sum::<i32>();
    println("total ${total}");
}
```

5. **Actor supervisor tree**
```aurora
actor Worker {
    fn start(self) {
        loop {
            match self.receive() {
                Msg::Work(job) => process(job)?,
            }
        }
    }
}

supervisor Pool {
    strategy: Strategy::RestForOne,
    children: [Worker; 8],
}
```

6. **ADT + exhaustive match (JSON decoder)**
```aurora
fn decode_user(json: Json) -> Result<User, Error> {
    match json {
        Json::Object(obj) => {
            let name = obj.get("name").and_then(Json::as_str)?.to_owned();
            let age = obj.get("age").and_then(Json::as_i32)?;
            Ok(User { name, age })
        }
        _ => Err(Error::Invalid("Expected object")),
    }
}
```

7. **FFI to C qsort**
```aurora
extern "C" {
    fn qsort(base: *mut u8, nmemb: usize, size: usize, cmp: extern "C" fn(*const u8, *const u8) -> i32);
}

fn sort(slice: &mut [i32]) {
    unsafe {
        qsort(slice.as_mut_ptr() as *mut u8, slice.len(), sizeof::<i32>(), comparator);
    }
}
```

8. **Typed macro deriving Encode/Decode**
```aurora
#[derive(Encode, Decode)]
struct Packet { id: u32, payload: Vec<u8> }
```

9. **Comptime generating a perfect-hash map**
```aurora
const KEYMAP: PerfectHash = comptime::perfect_hash(["GET", "POST", "PUT", "DELETE"]);
```

10. **GPU kernel**
```aurora
#[kernel(target = "cuda")]
fn saxpy(a: F32, x: DeviceSlice<F32>, y: DeviceSliceMut<F32>) {
    let i = gpu::thread_index();
    if i < x.len() {
        y[i] = a * x[i] + y[i];
    }
}
```

# Comparison Table
| Language | Perf | Safety | Ergonomics | Concurrency | Interop | Tooling |
| --- | --- | --- | --- | --- | --- | --- |
| Aurora | C/Rust-level, effect-aware optimizations | Ownership + effects + ARC fallback | Python-like readability, type inference | Async, goroutines, actors | C, TS, JVM, .NET, WASM, Python, Node | Cargo/Zig-like `aur`, LSP, fmt |
| C/C++ | Peak perf | Manual memory, UB risk | Verbose templates | pthreads/manual | C ABI | Make/cmake |
| Rust | Peak perf | Ownership/borrow | Expressive but steeper learning | async/await, tokio | C, WASM | Cargo |
| Go | Good perf | GC, data races possible | Simple but limited generics | goroutines/channels | C via cgo | go tool |
| Python | Low perf | Dynamic | Highly readable | asyncio, GIL | C, JS | pip, tox |
| Swift | High perf | ARC, optionals | Clean syntax | async/await, actors | Obj-C, C | SwiftPM |
| TypeScript | JIT perf | Structural typing, dynamic | Familiar to JS devs | Promise-based | JS runtimes | npm |
| Haskell | High perf (compiled) | Pure, laziness | Advanced but niche | STM, async | C via FFI | Stack/cabal |
| Java | High perf JIT | GC, checked exceptions | Verbose but consistent | threads, virtual threads | JVM | Maven/Gradle |
| Kotlin | High perf JIT | Null safety | Modern syntax | coroutines | JVM/Native | Gradle |
| Julia | High perf JIT | GC | Multiple dispatch | Tasks, channels | C, Python | Pkg |

# Roadmap
## MVP (Months 0-9)
- Core language: ownership, borrow checker, ADTs, traits, generics, match, Result/Option.
- Basic stdlib: collections, IO, async runtime, channels.
- Compiler: LLVM backend, MIR optimizations subset, incremental builds minimal.
- Tooling: `aur` CLI (new/build/run/test/fmt), formatter, linter MVP, docs generator, basic LSP (diagnostics, completions).
- Interop: C FFI, WASM MVP.
- Testing: unit + integration harness, doc tests.
- Performance gates: within 10% of Rust on microbenchmarks (vector add, fib, sort), compile times within 1.2x rustc.

## Beta (Months 10-18)
- Effect system full, actor runtime, supervisor trees.
- ARC/region optimizer, escape analysis.
- Full LSP features, profiler integration, property testing.
- Package registry launch, reproducible builds, signature verification.
- JVM/.NET backend preview, Python/Node bridges.
- Performance gates: parity with Rust on SPEC-like subset, compile times <= rustc.

## 1.0 (Months 19-30)
- Self-hosting compiler (written in Aurora), full ThinLTO, GPU kernel pipeline, ML hooks.
- Stabilized ABI, semantic versioning, tooling telemetry (opt-in).
- Security audits, SBOM automation, compliance.
- Ergonomics: friendly diagnostics like Rust/Elm (suggestions, spans, fix-its), borrow checker explanations.
- Performance gates: match or beat Rust on majority of SPEC CPU subtests, async runtime throughput >= Go net/http baseline.

# Compiler Plan (MVP vs Full)
- **Bootstrap:** start with Rust implementation of compiler; emit Aurora subset for self-host stage.
- **MVP Frontend:** lexical/parsing, name resolution, HM inference w/o specialization, borrow checker subset.
- **MVP MIR:** SSA with basic optimizations (inlining, const-prop). LLVM backend only.
- **MVP Runtime:** work-stealing scheduler minimal, async runtime using epoll/kqueue.
- **Full Compiler:** add trait specialization, effect polymorphism, Polonius-style borrow, Cranelift backend, GPU/MLIR pipeline, incremental compilation query system.

# Testing Strategy
- Unit tests for parser/type checker using golden files.
- MIR equivalence tests using differential testing against Rust subset.
- Property tests for borrow checker invariants.
- Compile-fail suites (similar to Rust's `ui` tests) to ensure diagnostics quality.
- Performance benchmarks on SPEC-like kernels, async throughput, ARC overhead.
- Self-host compiler using staged bootstrap.

# Diagnostics Philosophy
- Borrow from Rust/Elm: colorized spans, suggestions, explainers (`help:`).
- Provide lifetime diagrams, effect flow traces, concurrency warnings.
- `aur doctor` analyzes project health, suggests fixes.

# Appendix
## Glossary
- **ADT:** Algebraic Data Type.
- **ARC:** Automatic Reference Counting.
- **HIR/MIR:** High/Mid-level Intermediate Representation.
- **ThinLTO:** Thin Link-Time Optimization.
- **SBOM:** Software Bill of Materials.

## References & Inspiration
- Haskell & OCaml (HM inference, ADTs, pattern matching).
- Rust (ownership, traits, tooling).
- Swift (ARC, optionals, async/await).
- Go (goroutines, stdlib ethos).
- Erlang/Elixir (actors, supervisors).
- Julia (multiple dispatch).
- TypeScript (structural typing).
- Zig (comptime, tooling).
- Koka (effect system).
- Swift for TensorFlow (ML integration).
- Rust-CUDA/HIP (GPU kernels).

# Bonus
## Logo Concept & Palette
- Logo: stylized aurora arc forming letter "A" with gradient.
- Palette: Polar Night (#2E3440), Aurora Green (#88C0D0), Sunrise Pink (#EBCB8B), Star White (#ECEFF4).

## Sample Manifest (`Aurora.toml`)
```toml
[package]
name = "aurora-app"
version = "0.1.0"
edition = "2025"
description = "Sample Aurora application"
license = "Apache-2.0"
authors = ["Aurora Team <team@example.com>"]

[dependencies]
std = "1.0"
http = "0.2"
serde = { version = "0.9", features = ["json"] }

[profiles.release]
opt-level = "z"
lto = "thin"

[target.wasm32-wasi]
panic = "abort"
```

## Sample LSP Capability Matrix
| Capability | MVP | Beta | 1.0 |
| --- | --- | --- | --- |
| Diagnostics | ✓ | ✓ | ✓ |
| Completions | ✓ | ✓ | ✓ |
| Semantic tokens | ✓ | ✓ | ✓ |
| Borrow checker hints | ✗ | ✓ | ✓ |
| Refactor extract fn | ✗ | ✓ | ✓ |
| Inline rename | ✓ | ✓ | ✓ |
| Code lens (tests) | ✗ | ✓ | ✓ |
| Live share | ✗ | ✗ | ✓ |

## Typed Macro Example Generating State Machine
```aurora
macro state_machine<State: Enum>(states: [State]) -> impl Trait {
    quote do {
        impl StateMachine for State {
            fn next(self, input: Event) -> State {
                match (self, input) {
                    $(for s in states => (State::$s, Event::${s}Start) => State::${s}Active,)
                    _ => self,
                }
            }
        }
    }
}
```

## GPU Kernel Lowering
- `#[kernel]` functions lowered to MLIR GPU dialect, optimized, then emitted as PTX/SPIR-V.
- Host launches use `gpu::launch(kernel, grid, block, args)`; borrow checker ensures device memory lifetimes.

