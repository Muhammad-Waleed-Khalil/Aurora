# Aurora Constitution

## Core Principles

### I. Agent-First Design
Every compiler operation must be deterministic, reproducible, and introspectable. Grammar must be unambiguous with zero shift-reduce conflicts. All diagnostics must be machine-consumable JSON with fix-its and confidence scores. LSP, structured queries, and auto-repair hooks are first-class concerns, not afterthoughts.

### II. Clarity Over Cleverness
Python-level readability is non-negotiable: minimal sigils, obvious keywords, words over symbols. No hidden complexity or magic behavior. Braces for blocks ensure stable diffs and avoid indentation-as-syntax fragility. Expression-oriented semantics reduce boilerplate without sacrificing readability.

### III. Non-Intrusive Safety (NON-NEGOTIABLE)
Safety features produce lints and advisories by default, NOT build failures. Strict enforcement is opt-in only. No implicit null; Option type is explicit. Ownership is inferred; borrow checker advises; ARC inserted only at proven escape points. No global exceptions; Result/Option with single propagation operator.

### IV. Performance Without Compromise
Target: ≤1.10× of optimized C on compute kernels, ≤1.25× on mixed I/O workloads. Assembly-level optimization via NASM-like AIR (Aurora IR) with peephole and scheduling passes. CPU-aware tuning for Skylake, Zen, Neoverse, etc. Zero-cost abstractions through monomorphization, inlining, devirtualization.

### V. Architectural Rigor
Each compiler phase owns its domain exclusively. AST schema frozen per minor version. Type system CANNOT compensate for lexer ambiguity. Grammar depends on lexer; parser depends on both. No phase contamination. Iterative traversal with stackless visitors and arena allocation for cache locality.

### VI. Interoperability First
C ABI stability with stable mangling, header generation, safety shims. Python HPy bridge with explicit GIL policy. Node N-API modules across LTS releases. WASM/WASI with deterministic builds and capability manifests. Support PE/COFF (Windows), ELF (Linux), Mach-O (macOS).

### VII. Developer Experience
Batteries-included standard library (files, net, JSON, subprocess, regex). Single `aurora` CLI for all operations. Zero-configuration idempotent formatter. Fast dev builds with incremental compilation. Quality documentation with runnable examples and architectural explanations.

### VIII. Supply Chain Security
Reproducible builds: byte-for-byte identical with locked manifests. Content-addressed cache with cryptographic integrity. Automatic SBOM generation. Vendoring mode with signature verification. Policy gates control reflection, GC, unsafe, comptime capabilities.

### IX. Concurrency & Parallelism
M:N work-stealing scheduler with cooperative yields. Channels (bounded/unbounded) with zero-copy fast paths. Actors with supervisor strategies (permanent, transient, temporary). Async/await as zero-cost state machines with structured cancellation. Deterministic semantics; no data races.

### X. Metaprogramming Power
Hygienic macros expand to AST with tracked scopes. Comptime is deterministic and sandboxed. Derives generate typed trait implementations via macros. LSP and tooling can inspect macro expansion. Lisp-class power without ceremony or cryptic syntax.

## Quality Standards

### Correctness Requirements
- Zero tolerance for undefined behavior in safe code
- Exhaustive pattern matching or explicit unreachable with justification
- Borrow/effect checker produces no false positives in advisory mode
- Grammar ambiguity report must be empty
- Principal types for all unannotated expressions

### Performance Requirements
- Compute kernels: within 1.10× of optimized C
- Mixed I/O workloads: within 1.25× of optimized C
- Dev build compile speed: within 1.2× of Go
- Memory overhead: within 1.1× of Rust for comparable tasks

### Determinism Requirements
- Identical binaries from identical inputs (source + flags + seed)
- Reproducible across machines with same toolchain fingerprint
- MIR/AIR output stable across patch releases
- All compiler phases produce memoizable outputs

### Testing Requirements
- 100% coverage on critical compiler paths
- Unit, property, golden, and differential tests mandatory
- Cross-arch CI: x86_64, ARM64, RISC-V, MIPS, SPARC
- Concurrency stress testing with reproducible seeds

### Documentation Requirements
- Every public API has examples
- Architectural decisions documented with rationale
- Diagnostic messages include fix-its and doc URLs
- "Why" explanations available via introspection endpoints

## Agent Boundaries (STRICT ENFORCEMENT)

### Orchestrator
**Authority**: Absolute over architecture, task decomposition, quality gates, merge approval
**Scope**: Backlog decomposition, agent assignment, acceptance criteria enforcement, architectural integrity
**KPIs**: Zero spec regressions, no agent overlap, ≤5% rework rate
**FORBIDDEN**: Writing implementation code directly, performing another agent's work

### LexerAgent
**Scope**: Token definitions, NFA state machine, maximal-munch, UTF-8, XID identifiers
**Deliverables**: Token catalog, zero-backtracking lexer, empty ambiguity report
**FORBIDDEN**: Grammar, AST, parser, type system

### GrammarAgent
**Scope**: Operator precedence, CFG rules, associativity, grammar conformance tests
**Deliverables**: Published grammar spec, zero conflicts
**FORBIDDEN**: Parser implementation, type rules in grammar

### ParserAgent
**Scope**: LL-style parser, Pratt expressions, AST with spans & hygiene anchors
**Deliverables**: Deterministic AST, zero ambiguity, consistent span mapping
**FORBIDDEN**: Type inference, name resolution, macro expansion

### ASTAgent
**Scope**: Node schema, arena layout, traversal mechanisms, precomputed indices
**Deliverables**: Frozen schema per minor version, zero breaking field changes
**FORBIDDEN**: Parsing, type inference, codegen

### NameResAgent
**Scope**: Hygiene, scopes, imports/exports, symbol tables, binding resolution
**Deliverables**: Deterministic symbol graph, zero accidental capture
**FORBIDDEN**: Type inference, borrow logic

### TypeSystemAgent
**Scope**: HM inference, typeclasses, generics, monomorphization, null-safety
**Deliverables**: Principal types, zero inference explosions
**FORBIDDEN**: Borrow/effect logic, MIR lowering

### EffectsBorrowAgent
**Scope**: Effect rows, subeffect ordering, borrow checker dataflow, ARC advisories
**Deliverables**: Advisory + strict modes, structured diagnostics
**FORBIDDEN**: Type inference, MIR optimizations

### MIRAgent
**Scope**: SSA form, dominance, CFG, inlining, SROA, GVN, LICM, DCE, NRVO, devirt, loop-SIMD
**Deliverables**: MIR dumps with spans, proven pass correctness
**FORBIDDEN**: Assembly-level optimizations, AIR

### AIRAgent
**Scope**: NASM-like IR emission, peepholes (mov collapse, LEA patterns), scheduling
**Deliverables**: Round-trip AIR, CPU-profiled patterns
**FORBIDDEN**: MIR ownership, LLVM code

### BackendAgent
**Scope**: LLVM/Cranelift pipelines, LLD linking, debug info (DWARF/PDB/SEH)
**Deliverables**: Reproducible binaries, verified symbol maps
**FORBIDDEN**: AIR peephole logic, optimizer logic

### InteropAgent
**Scope**: C ABI, HPy, N-API, WASM/WASI, header/codegen shims
**Deliverables**: Interop conformance suite
**FORBIDDEN**: IR or type system changes to "make interop easier"

### ConcurrencyAgent
**Scope**: Scheduler, goroutines, channels, actors, async/await, cancellation
**Deliverables**: Deterministic cancellation rules
**FORBIDDEN**: Type system or borrow system changes

### OptimizerAgent
**Scope**: Micro-arch tuning profiles, perf gates, benchmark enforcement, PGO
**Deliverables**: Verified perf gains vs C targets
**FORBIDDEN**: Semantic changes that hide slow code

### BuildAgent
**Scope**: `aurora` CLI, workspace management, build profiles, cross-compilation
**Deliverables**: Deterministic build graph
**FORBIDDEN**: Compiler internals

### DiagnosticsAgent
**Scope**: JSON diagnostics, fix-its, LSP (completions, hovers, actions, rename)
**Deliverables**: Zero-crash LSP with inline MIR/AIR previews
**FORBIDDEN**: Changing semantics to "fix UX"

### TestingAgent
**Scope**: Unit/property/golden tests, differential C checks, QEMU cross-arch, determinism audits
**Deliverables**: 100% coverage on critical paths
**FORBIDDEN**: Implementing features

### SecurityAgent
**Scope**: SBOM, vendoring, signature verification, GC/reflection policy gates
**Deliverables**: Hardened build + supply chain integrity
**FORBIDDEN**: Feature creep into language

### DocumentationAgent
**Scope**: Language reference, CLI docs, API references, interop manuals, spec addenda
**Deliverables**: Docs matching current spec and behavior
**FORBIDDEN**: Redefining language philosophy or semantics

## Development Workflow

### Compiler Pipeline (Sequential)
1. Lexical Analysis → Token stream (NFA maximal-munch)
2. Parsing → AST (Pratt + CFG)
3. Macro Expansion → Hygiene-preserving AST transformation
4. Name Resolution → Symbol tables and import/export sealing
5. Type Checking → HM inference, typeclass resolution, monomorphization
6. Borrow/Effect Analysis → Ownership verification with advisories
7. MIR Generation → SSA lowering with effect edges
8. MIR Optimization → Inlining, SROA, GVN, LICM, DCE, NRVO, devirt, loop-SIMD
9. AIR Emission → Assembly-like IR with peephole optimization
10. Backend → Machine code generation and linking (LLD)

### Commit Standards (ALL REQUIRED)
- Lexer ambiguity checks pass
- Grammar conflict report empty
- Type inference produces principal types
- AIR round-trips (emit → parse)
- Cross-arch test matrix green
- No performance regressions
- No determinism violations

### Merge Requirements (ORCHESTRATOR GATES)
- Orchestrator approval required
- All acceptance criteria met
- Documentation updated if public APIs changed
- Changelog entry for user-visible changes
- No agent boundary violations

## Non-Goals (EXPLICIT REJECTIONS)

We reject:
- Mandatory borrow annotations in general code
- Global exceptions or implicit exception propagation
- Indentation-as-syntax
- Dynamic multiple dispatch without static constraints
- Hidden garbage collection (GC is opt-in only)
- Context-dependent grammar
- Implicit nulls or undefined by default
- Safety that blocks builds by default

## Milestone Success Criteria

### MVP (Foundation)
- Self-hosted compiler subset compiles
- Unit/property/golden test suite passes
- PE and ELF binaries pass conformance tests
- Basic LSP with diagnostics and completion
- Lexer (NFA), parser (Pratt + CFG), HM inference complete
- ADTs, typeclasses (no specialization), Result/Option working
- Channels/goroutines, basic async, C FFI operational
- AIR x86_64, COFF/ELF, DCE + peephole functional

### Beta (Robustness)
- Cross-arch matrix green
- Profiler and ThinLTO integrated
- Effects enforced in strict mode
- Regions/arenas stable
- AArch64/RISC-V backends operational
- WASI support stable
- Derive macros, content-addressed cache, SBOM complete

### 1.0 (Production)
- ABI stability report published
- Specification frozen
- Ecosystem bridges (C/Python/Node) GA
- Debugger polish (DWARF/PDB)
- Supply chain policies enforced
- Performance targets met on full benchmark suite
- Specialization with coherence, optional mini-GC for plugins

## Governance

This constitution supersedes all other practices and guidelines. Agent boundary violations are defects. The Orchestrator has absolute authority to:
- Block merges that violate principles
- Eject agents that cross boundaries
- Demand refactors that restore architectural integrity
- Halt development if quality decays below standards

### Amendment Process
Amendments require:
1. Documented architectural necessity
2. Orchestrator consensus on impact analysis
3. Update of all dependent documentation
4. Migration path for existing code
5. Community notification of breaking changes

**Version**: 1.0.0 | **Ratified**: 2025-11-04 | **Last Amended**: 2025-11-04
