# Aurora Quality Checklist

**Version**: 1.0.0
**Purpose**: Validate requirements completeness, clarity, consistency, and implementation quality
**Last Updated**: 2025-11-04

---

## How to Use This Checklist

This checklist is organized into sections corresponding to each compiler phase and cross-cutting concerns. Use it to:

1. **Before Phase Start**: Validate that prerequisites are met
2. **During Implementation**: Ensure adherence to principles
3. **Before Phase Completion**: Verify acceptance criteria
4. **At Merge Time**: Orchestrator gates all merges with this checklist

**Status Indicators**:
-  Verified and passing
- � Needs attention or clarification
- L Failing, must fix before merge
-  Not yet applicable

---

## Section 1: Constitution Compliance

### 1.1 Agent Boundaries
- [ ] Agent operates only within its designated domain
- [ ] No direct calls to other agents (only via orchestrator)
- [ ] Outputs are pure functions: Input � Result<Output, Diagnostics>
- [ ] No global state or side effects beyond designated scope

### 1.2 Determinism
- [ ] Identical inputs produce identical outputs
- [ ] No dependence on timestamps, randomness, or external state
- [ ] All operations are memoizable
- [ ] Build artifacts are byte-for-byte reproducible

### 1.3 Quality Standards
- [ ] No undefined behavior in safe code
- [ ] Grammar ambiguity report is empty (lexer/parser)
- [ ] Type inference produces principal types (type system)
- [ ] AIR round-trips successfully (backend)
- [ ] Performance targets met or justified deviation documented

---

## Section 2: Lexer Phase Checklist

### 2.1 Token Catalog
- [X] All Aurora tokens defined in `TokenKind` enum
- [X] Keywords, operators, delimiters, literals complete
- [X] Machine-readable JSON schema exported
- [X] Token catalog versioned and documented

### 2.2 NFA Implementation
- [X] Table-driven NFA state machine implemented
- [X] Maximal-munch tokenization enforced
- [X] Reserved-word priority correctly applied
- [X] UTF-8 validation rejects invalid sequences
- [X] XID identifier rules enforced

### 2.3 Lexer Driver
- [X] `Lexer::new(source)` and `Lexer::next_token()` implemented
- [X] Spans tracked (file, line, column, length)
- [X] Line comments, block comments, doc comments handled
- [X] Error messages include precise location

### 2.4 Ambiguity Validation
- [X] All token types have test cases
- [X] Ambiguity checker implemented and run
- [X] Ambiguity report is empty (14 comprehensive edge case tests)
- [X] Edge cases tested (e.g., `..`, `...`, `..=`, `1..2`)

### 2.5 Performance
- [X] Lexer throughput e100 MB/s on x86_64
- [X] Benchmarks exist and run in CI (7 comprehensive benchmarks)
- [X] Hot paths profiled and optimized

---

## Section 3: Grammar Phase Checklist

### 3.1 Precedence Table
- [X] 16 precedence levels defined
- [X] Associativity (left, right, none) assigned to all operators
- [X] `PrecedenceTable` struct implemented
- [X] Machine-readable format (JSON/YAML) exported (precedence.json)

### 3.2 CFG Rules
- [X] BNF notation for all constructs (fn, type, trait, impl, const, mod, use)
- [X] Statement grammar complete (if, for, while, loop, match, let, return)
- [X] Expression grammar delegated to Pratt parser
- [X] Pattern matching grammar defined
- [X] Type grammar defined

### 3.3 Conflict Analysis
- [X] LL(1) conflict detector implemented
- [X] FIRST/FOLLOW sets computed correctly
- [⚠️] Conflict report: 6 epsilon conflicts (acceptable for MVP, expressions use Pratt)
- [X] Pratt precedence validated

### 3.4 Documentation
- [X] Grammar published as machine-readable artifact (grammar.json, grammar.bnf)
- [⏳] Railroad diagrams generated (future enhancement)
- [X] Grammar rationale documented (docs/grammar.md)

---

## Section 4: Parser Phase Checklist

### 4.1 AST Schema
- [ ] All node kinds defined (`Expr`, `Stmt`, `Decl`, `Type`, `Pattern`)
- [ ] Nodes carry spans for source mapping
- [ ] Hygiene IDs prepared for macro expansion
- [ ] Schema frozen for MVP and versioned

### 4.2 Arena Allocator
- [ ] Bump allocator for AST nodes implemented
- [ ] Parent links precomputed
- [ ] Preorder/postorder index arrays implemented
- [ ] Cache locality optimized

### 4.3 Traversal
- [ ] Iterative (stackless) visitor pattern implemented
- [ ] Preorder and postorder traversal work
- [ ] Subtree slicing via indices works
- [ ] Pretty-printer for debugging exists

### 4.4 LL Parser
- [ ] Recursive descent for top-level items implemented
- [ ] Functions, types, traits, impls parse correctly
- [ ] Modules and use statements parse correctly
- [ ] Error recovery at synchronization points

### 4.5 Pratt Parser
- [ ] Table-driven expression parsing implemented
- [ ] Prefix, infix, postfix operators handled
- [ ] Precedence and associativity correct
- [ ] Complex expressions nest properly

### 4.6 Statement Parsing
- [ ] `let`, `if`/`else`, `match` parse correctly
- [ ] Loops (`for`, `while`, `loop`) parse correctly
- [ ] `return`, `break`, `continue`, `yield` parse correctly

### 4.7 Error Recovery
- [ ] Synchronization points identified (semicolons, braces)
- [ ] Panic-mode recovery implemented
- [ ] Multiple errors collected per parse
- [ ] Partial AST generated on error

### 4.8 Tests
- [ ] Golden tests for all constructs (AST snapshots)
- [ ] Error recovery tests pass
- [ ] Property tests (fuzzing) find no crashes
- [ ] Differential tests vs grammar spec pass

---

## Section 5: Name Resolution Phase Checklist

### 5.1 Symbol Table
- [ ] `Symbol` type defined (name, kind, visibility, span)
- [ ] Scope hierarchy implemented (module, function, block)
- [ ] Symbol insertion and lookup work
- [ ] Shadowing and visibility rules enforced

### 5.2 Hygiene System
- [ ] Hygiene IDs assigned to identifiers
- [ ] Scope rebinding after macro expansion works
- [ ] Accidental capture prevented

### 5.3 Module Graph
- [ ] Module dependency graph built
- [ ] Cycles detected and reported
- [ ] Import/export tables sealed

### 5.4 Resolution Pass
- [ ] All identifiers resolve to symbols or emit errors
- [ ] "Why" chains (provenance tracking) generated
- [ ] Diagnostics for undefined names actionable
- [ ] Diagnostics for ambiguous names actionable

### 5.5 Tests
- [ ] Shadowing, visibility, imports tested
- [ ] Error cases (undefined, ambiguous) tested
- [ ] Hygiene (no accidental capture) tested

---

## Section 6: Type System Phase Checklist

### 6.1 Type Representation
- [ ] `Type` enum complete (primitives, compounds, generics, typeclasses)
- [ ] Type equality implemented correctly
- [ ] Subtyping rules implemented
- [ ] Occurs check prevents infinite types

### 6.2 Hindley-Milner Inference
- [ ] Constraint generation from AST implemented
- [ ] Unification algorithm implemented
- [ ] Principal types computed for all expressions
- [ ] Bidirectional type checking works
- [ ] Inference is deterministic

### 6.3 Typeclasses (Traits)
- [ ] Typeclass representation defined
- [ ] Typeclass resolution (find impls) implemented
- [ ] Coherence enforced (single impl per (trait, type) pair)
- [ ] Associated types supported

### 6.4 Generics & Monomorphization
- [ ] Generic instantiation works
- [ ] Monomorphization sites tracked for codegen
- [ ] Reification for FFI supported (opt-in)

### 6.5 Null Safety
- [ ] Option type enforced for nullable values
- [ ] Exhaustiveness checking implemented
- [ ] Explicit unreachable annotations required
- [ ] No implicit nulls in type system

### 6.6 Tests
- [ ] Inference on complex expressions tested
- [ ] Typeclass resolution tested
- [ ] Generics instantiation tested
- [ ] Exhaustiveness checking tested

---

## Section 7: Effects & Borrow Phase Checklist

### 7.1 Effect System
- [ ] Effect kinds defined (IO, Alloc, Parallel, Unsafe)
- [ ] Effect rows implemented (bitset representation)
- [ ] Subeffect partial order implemented
- [ ] Effect polymorphism supported

### 7.2 Borrow Checker (Advisory)
- [ ] Dataflow analysis for borrows implemented
- [ ] Lifetimes tracked
- [ ] Borrow conflicts detected
- [ ] Advisories emitted (not errors) in default mode

### 7.3 ARC Insertion
- [ ] Uncertain escape points detected
- [ ] ARC increments/decrements inserted
- [ ] Advisories show ARC sites

### 7.4 Strict Mode
- [ ] Advisories converted to errors in strict mode
- [ ] Explicit lifetimes required in strict mode
- [ ] ARC insertion disallowed in strict mode

### 7.5 Tests
- [ ] Effect tracking tested
- [ ] Borrow checker advisories tested
- [ ] ARC insertion tested
- [ ] Strict mode enforcement tested

---

## Section 8: MIR Phase Checklist

### 8.1 MIR Representation
- [ ] MIR instructions defined (SSA form)
- [ ] Basic blocks and CFG implemented
- [ ] Dominance tree computation correct
- [ ] Effect edges explicitly added to instructions

### 8.2 MIR Lowering
- [ ] Typed AST � MIR lowering implemented
- [ ] Effect edges inserted
- [ ] Source spans preserved

### 8.3 MIR Optimizations
- [ ] Inlining heuristics implemented
- [ ] SROA (scalar replacement) implemented
- [ ] GVN (global value numbering) implemented
- [ ] LICM (loop-invariant code motion) implemented
- [ ] DCE (dead code elimination) implemented
- [ ] NRVO (named return value optimization) implemented
- [ ] Devirtualization implemented
- [ ] All optimizations preserve correctness

### 8.4 Loop Vectorization
- [ ] Vectorizable loops detected
- [ ] SIMD instructions generated for x86_64
- [ ] Advisories emitted for non-vectorizable loops

### 8.5 MIR Dumps
- [ ] Human-readable MIR printer implemented
- [ ] JSON export for tooling implemented
- [ ] Spans preserved in dumps

### 8.6 Tests
- [ ] Golden tests for MIR output exist
- [ ] Optimization correctness tests pass
- [ ] Roundtrip tests (AST � MIR � semantics) pass

---

## Section 9: AIR & Backend Phase Checklist

### 9.1 AIR Format
- [ ] NASM-like textual format defined
- [ ] x86_64 instruction set supported
- [ ] Directives for debug info and unwind included
- [ ] AIR format documented

### 9.2 AIR Emission
- [ ] MIR � AIR lowering implemented
- [ ] Register allocation (linear scan) implemented
- [ ] AIR text format emitted

### 9.3 Peephole Optimizations
- [ ] Mov collapse implemented
- [ ] LEA patterns implemented
- [ ] Branch shortening implemented
- [ ] Correctness preserved

### 9.4 Instruction Scheduling
- [ ] Basic block scheduling implemented
- [ ] CPU profiles (Skylake, Zen) used for latency/throughput
- [ ] Pipeline stalls avoided

### 9.5 LLVM Backend
- [ ] AIR � LLVM IR translation implemented
- [ ] LLVM optimization passes integrated (optional)
- [ ] Object files emitted (COFF, ELF)
- [ ] Debug info generated (DWARF, PDB)

### 9.6 Linking
- [ ] LLD used for final linking
- [ ] PE/COFF (Windows) and ELF (Linux) supported
- [ ] SEH (Windows) and unwind info included

### 9.7 AIR Tests
- [ ] AIR roundtrips successfully (emit � parse � emit)
- [ ] Golden tests for AIR output exist
- [ ] Debug info correctness validated
- [ ] Binaries execute correctly on target platforms

---

## Section 10: Interop & Concurrency Phase Checklist

### 10.1 C ABI Support
- [ ] Stable name mangling implemented
- [ ] C headers generated from Aurora declarations
- [ ] Safety shims for common C pitfalls implemented
- [ ] C FFI works bidirectionally

### 10.2 Goroutines & Channels
- [ ] Work-stealing scheduler (M:N) implemented
- [ ] Channel send/recv operations implemented
- [ ] Integration with async runtime works
- [ ] Goroutines execute correctly
- [ ] Channels communicate correctly

### 10.3 Async/Await
- [ ] Async functions lowered to state machines
- [ ] Structured cancellation implemented
- [ ] Event loop integration works
- [ ] Async/await executes correctly

### 10.4 Tests
- [ ] C FFI roundtrip tests pass
- [ ] Goroutine and channel tests pass
- [ ] Async/await integration tests pass

---

## Section 11: Tooling & Diagnostics Phase Checklist

### 11.1 CLI (`aurora`)
- [ ] `ax init`, `ax build`, `ax run` implemented
- [ ] `ax test` (test runner) implemented
- [ ] `ax fmt` (formatter) implemented
- [ ] `ax lint` (linter) implemented
- [ ] `ax doc` (doc generator) implemented
- [ ] All CLI commands work end-to-end
- [ ] UX is polished

### 11.2 JSON Diagnostics
- [ ] Diagnostic JSON schema defined
- [ ] Diagnostics emitted with spans, fix-its, confidence
- [ ] Doc URLs included for each diagnostic
- [ ] Schema is stable

### 11.3 LSP Server
- [ ] LSP protocol handler implemented
- [ ] Completions implemented
- [ ] Go-to-definition, find references implemented
- [ ] Hover (type info) implemented
- [ ] Diagnostics and code actions implemented
- [ ] Rename refactoring implemented
- [ ] LSP integrates with VS Code, Neovim, etc.

### 11.4 Testing Framework
- [ ] `#[test]` attribute handling implemented
- [ ] Test runner with parallel execution implemented
- [ ] Property-based tests (proptest) supported
- [ ] Test reports generated
- [ ] All test types run correctly

---

## Section 12: Security & Documentation Phase Checklist

### 12.1 SBOM Generation
- [ ] Dependency graph traversal implemented
- [ ] SBOM generated (SPDX format)
- [ ] License and version info included
- [ ] SPDX format is valid

### 12.2 Reproducible Builds
- [ ] Lockfile format implemented
- [ ] Deterministic codegen validated
- [ ] Build fingerprint generated
- [ ] Builds are byte-for-byte reproducible
- [ ] Lockfile pins all dependencies

### 12.3 Language Reference
- [ ] Lexical structure documented
- [ ] Grammar and syntax documented
- [ ] Type system and semantics documented
- [ ] Standard library documented
- [ ] Documentation is complete and accurate
- [ ] Examples are runnable

### 12.4 API Documentation
- [ ] API docs generated from source
- [ ] Examples included for all public APIs
- [ ] Docs site published

---

## Section 13: Self-Hosting Phase Checklist

### 13.1 Conformance Test Suite
- [ ] Comprehensive test suite built (unit, integration, golden)
- [ ] Differential tests vs C implementations included
- [ ] Cross-arch testing (QEMU for ARM, RISC-V) configured
- [ ] All conformance tests pass
- [ ] Cross-arch CI green

### 13.2 Performance Benchmarks
- [ ] Microbenchmarks implemented (arithmetic, reductions, etc.)
- [ ] Kernel benchmarks implemented (FFT, GEMM, sorting)
- [ ] Performance targets validated (d1.10� C on kernels)
- [ ] Benchmarks run in CI

### 13.3 Self-Hosting
- [ ] Critical compiler paths rewritten in Aurora
- [ ] Compiler bootstraps with previous version
- [ ] Output is identical (deterministic)
- [ ] Compiler compiles itself

---

## Section 14: Cross-Cutting Concerns

### 14.1 Code Quality
- [ ] No compiler warnings (-Wextra -Wall)
- [ ] No clippy warnings (Rust code)
- [ ] Code formatted with consistent style
- [ ] No dead code or unused imports
- [ ] Comments explain "why", not "what"

### 14.2 Testing
- [ ] Unit tests for all public functions
- [ ] Integration tests for end-to-end scenarios
- [ ] Golden tests for compiler outputs (AST, MIR, AIR)
- [ ] Property tests for critical algorithms
- [ ] Differential tests vs reference implementations
- [ ] 100% coverage on critical paths

### 14.3 Documentation
- [ ] All public APIs documented
- [ ] Examples included for complex APIs
- [ ] Architectural decisions recorded (ADRs)
- [ ] README updated if user-facing changes
- [ ] CHANGELOG updated for each release

### 14.4 Performance
- [ ] No performance regressions on benchmark suite
- [ ] Hot paths profiled and optimized
- [ ] Memory allocations minimized in critical loops
- [ ] Cache locality optimized (arena allocation, etc.)

### 14.5 Security
- [ ] No unsafe code without justification
- [ ] Input validation on all external inputs
- [ ] No panics on invalid user input (graceful errors)
- [ ] Dependencies audited with `cargo audit`
- [ ] SBOM includes all transitive dependencies

### 14.6 Determinism & Reproducibility
- [ ] No timestamps in build artifacts
- [ ] No randomness without fixed seed
- [ ] No filesystem or network access in core compiler (except I/O phase)
- [ ] Build artifacts byte-for-byte identical with same inputs
- [ ] Build fingerprint tracked

---

## Section 15: Orchestrator Gate (Merge Checklist)

Before merging any PR, the Orchestrator must verify:

### 15.1 Prerequisites
- [ ] All task acceptance criteria met
- [ ] All tests pass (unit, integration, golden, property, differential)
- [ ] No regressions on benchmark suite
- [ ] CI green on all platforms (Linux, macOS, Windows)
- [ ] Cross-arch CI green (x86_64, ARM64, RISC-V via QEMU)

### 15.2 Agent Boundaries
- [ ] Agent stayed within its domain
- [ ] No cross-agent contamination
- [ ] Outputs are pure and deterministic
- [ ] Diagnostics follow JSON schema

### 15.3 Quality Standards
- [ ] Code quality checklist complete
- [ ] Testing checklist complete
- [ ] Documentation checklist complete
- [ ] Performance checklist complete
- [ ] Security checklist complete

### 15.4 Spec Compliance
- [ ] No spec violations
- [ ] Constitution principles upheld
- [ ] Non-goals respected (no forbidden features)

### 15.5 Documentation
- [ ] Public APIs documented
- [ ] Architectural decisions recorded
- [ ] CHANGELOG updated (if user-facing changes)
- [ ] README updated (if needed)

### 15.6 Final Approval
- [ ] Orchestrator approval given
- [ ] Agent self-review complete
- [ ] No outstanding concerns

---

## Section 16: Phase Completion Checklists

### Phase 0: Bootstrap
- [X] Repository structure matches plan.md
- [X] CI/CD pipeline configured
- [X] Orchestrator interface defined
- [X] All crates initialized

### Phase 1: Lexer & Grammar
- [X] Token catalog complete
- [X] NFA state machine implemented
- [X] Lexer driver implemented
- [X] Ambiguity report empty (14 edge case tests passing)
- [X] Lexer performance e100 MB/s (benchmarks in place)
- [X] Grammar precedence table complete (16 levels)
- [X] CFG rules complete (all constructs defined)
- [⚠️] Conflict report: 6 epsilon conflicts (acceptable for MVP with Pratt)
- [X] Grammar documentation published (docs/grammar.md)

### Phase 2: Parser & AST
- [ ] AST schema defined and frozen
- [ ] Arena allocator implemented
- [ ] Traversal mechanisms implemented
- [ ] LL parser for declarations implemented
- [ ] Pratt parser for expressions implemented
- [ ] Statement parsing implemented
- [ ] Error recovery implemented
- [ ] Parser tests pass

### Phase 3: Name Resolution
- [ ] Symbol table implemented
- [ ] Hygiene system implemented
- [ ] Module graph built (acyclic)
- [ ] Name resolution pass implemented
- [ ] "Why" chains generated
- [ ] Name resolution tests pass

### Phase 4: Type System
- [ ] Type representation complete
- [ ] HM inference implemented
- [ ] Typeclasses implemented
- [ ] Generics & monomorphization implemented
- [ ] Null safety enforced
- [ ] Type system tests pass

### Phase 5: Effects & Borrow
- [ ] Effect system implemented
- [ ] Borrow checker (advisory mode) implemented
- [ ] ARC insertion implemented
- [ ] Strict mode implemented
- [ ] Effects & borrow tests pass

### Phase 6: MIR
- [ ] MIR representation designed
- [ ] MIR lowering implemented
- [ ] MIR optimizations implemented
- [ ] Loop vectorization implemented
- [ ] MIR dumps implemented
- [ ] MIR tests pass

### Phase 7: AIR & Backend
- [ ] AIR format defined
- [ ] AIR emission implemented
- [ ] Peephole optimizations implemented
- [ ] Instruction scheduling implemented
- [ ] LLVM backend implemented
- [ ] Linking implemented
- [ ] AIR roundtrips successfully
- [ ] Binaries execute correctly

### Phase 8: Interop & Concurrency
- [ ] C ABI support implemented
- [ ] Goroutines & channels implemented
- [ ] Async/await implemented
- [ ] Interop & concurrency tests pass

### Phase 9: Tooling & Diagnostics
- [ ] `aurora` CLI implemented
- [ ] JSON diagnostics implemented
- [ ] LSP server implemented
- [ ] Testing framework implemented
- [ ] Tooling tests pass

### Phase 10: Security & Documentation
- [ ] SBOM generation implemented
- [ ] Reproducible builds validated
- [ ] Language reference complete
- [ ] API documentation complete

### Phase 11: Self-Hosting
- [ ] Conformance test suite complete
- [ ] Performance benchmarks complete
- [ ] Compiler compiles itself
- [ ] MVP complete

---

## Section 17: MVP Completion Checklist

### 17.1 Correctness
- [ ] Zero grammar ambiguities
- [ ] Zero shift-reduce conflicts
- [ ] Principal types for all expressions
- [ ] AIR round-trips successfully
- [ ] 100% conformance test pass rate

### 17.2 Performance
- [ ] Lexer: e100 MB/s
- [ ] Dev builds: within 1.2� Go
- [ ] Compute kernels: d1.10� C (best-effort target)
- [ ] I/O workloads: d1.25� C (best-effort target)

### 17.3 Quality
- [ ] 100% coverage on critical compiler paths
- [ ] Cross-arch CI green (x86_64, ARM64, RISC-V via QEMU)
- [ ] Zero regressions on benchmark suite
- [ ] Deterministic builds (byte-for-byte reproducible)

### 17.4 Developer Experience
- [ ] CLI commands work end-to-end
- [ ] LSP provides basic IDE features
- [ ] Diagnostics include fix-its
- [ ] Documentation complete and accurate

### 17.5 Self-Hosting
- [ ] Compiler compiles itself
- [ ] Output is identical (deterministic)
- [ ] Bootstrap process documented

---

## Section 18: Beta Readiness Checklist (Future)

### 18.1 Ecosystem
- [ ] Python HPy bridge GA
- [ ] Node N-API bridge GA
- [ ] WASI support stable
- [ ] Interop conformance suite passes

### 18.2 Performance
- [ ] Performance targets fully met (d1.10� C on kernels, d1.25� on I/O)
- [ ] ThinLTO integrated
- [ ] Profiler integrated
- [ ] PGO (profile-guided optimization) supported

### 18.3 Advanced Features
- [ ] Effects strict mode stable
- [ ] Actors/supervisors stable
- [ ] Regions/arenas stable
- [ ] Derive macros stable

### 18.4 Platforms
- [ ] AArch64 backend GA
- [ ] RISC-V backend GA
- [ ] Cross-compilation working for all targets

---

## Section 19: 1.0 Readiness Checklist (Future)

### 19.1 Stability
- [ ] ABI stability report published
- [ ] Language spec frozen
- [ ] No breaking changes allowed

### 19.2 Ecosystem
- [ ] Production case studies published
- [ ] Performance comparisons vs C/Rust published
- [ ] Community adoption metrics met

### 19.3 Advanced Features
- [ ] Specialization with coherence stable
- [ ] Optional mini-GC for plugins stable
- [ ] Debugger polish complete (DWARF/PDB)

### 19.4 Supply Chain
- [ ] Supply chain policies enforced
- [ ] Vendoring mode stable
- [ ] Signature verification stable

---

## Checklist Maintenance

This checklist is a living document and should be:

1. **Updated** at the start of each phase with phase-specific criteria
2. **Reviewed** during weekly progress meetings
3. **Enforced** by the Orchestrator at merge time
4. **Refined** based on lessons learned during implementation

**Last Updated**: 2025-11-04
**Next Review**: Upon completion of Phase 0 (Bootstrap)

---

**Document Status**: Living checklist, updated at phase boundaries
