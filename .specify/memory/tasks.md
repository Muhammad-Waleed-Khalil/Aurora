# Aurora Implementation Tasks

**Version**: 1.0.0
**Status**: MVP Foundation Phase
**Last Updated**: 2025-11-04

---

## Task Organization

Tasks are organized in **strict dependency order**. Each task has:
- **ID**: Unique identifier (PHASE-AGENT-NUMBER)
- **Owner**: Responsible agent
- **Status**: ` Not Started` | `= In Progress` | ` Complete` | `=� Blocked`
- **Dependencies**: Prerequisites that must be complete first
- **Estimate**: Time estimate in person-days
- **Acceptance Criteria**: Measurable success conditions

---

## Phase 0: Project Bootstrap

### P0-ORG-001: Initialize Repository Structure
**Owner**: Orchestrator
**Status**:  ✅ Complete
**Dependencies**: None
**Estimate**: 0.5 days

**Tasks**:
- [X] Create main repository with Cargo workspace
- [X] Create crate directories for all agents
- [X] Set up `.specify/memory/` directory
- [X] Initialize Git repository
- [X] Create `.gitignore` for Rust projects

**Acceptance Criteria**:
-  Repository structure matches plan.md specification
-  All crate directories exist
-  Cargo.toml workspace configured

**Files Created**:
- `/Cargo.toml`
- `/crates/*/Cargo.toml` (19 crates)
- `/.gitignore`

---

### P0-ORG-002: Set Up CI/CD Pipeline
**Owner**: Orchestrator
**Status**:  ✅ Complete
**Dependencies**: P0-ORG-001
**Estimate**: 1 day

**Tasks**:
- [X] Create GitHub Actions workflow for CI
- [X] Configure matrix testing (Linux, macOS, Windows)
- [X] Set up cross-arch testing with QEMU (ARM64, RISC-V)
- [X] Configure test coverage reporting
- [X] Set up benchmark tracking

**Acceptance Criteria**:
-  CI runs on all PRs
-  Tests execute on x86_64 Linux, Windows, macOS
-  QEMU cross-arch tests configured
-  Coverage reports generated

**Files Created**:
- `.github/workflows/ci.yml`
- `.github/workflows/benchmarks.yml`

---

### P0-ORG-003: Define Orchestrator Interface
**Owner**: Orchestrator
**Status**:  ✅ Complete
**Dependencies**: P0-ORG-001
**Estimate**: 1 day

**Tasks**:
- [X] Define `CompilerDriver` trait for orchestration
- [X] Define `AgentInput` and `AgentOutput` types
- [X] Define `DiagnosticsBundle` schema
- [X] Implement basic compiler driver skeleton

**Acceptance Criteria**:
-  Orchestrator can invoke agent phases
-  Diagnostics are collected centrally
-  All outputs are serializable

**Files Created**:
- `crates/aurorac/src/lib.rs`
- `crates/aurorac/src/driver.rs`
- `crates/aurorac/src/diagnostics.rs`

---

### P0-ORG-004: Rename Project from AXION to Aurora
**Owner**: Orchestrator
**Status**: ✅ Complete
**Dependencies**: P0-ORG-001, P0-ORG-002, P0-ORG-003
**Estimate**: 1 day
**Completed**: 2025-11-05

**Tasks**:
- [X] Rename all source files (AXION → Aurora, AXIR → AIR)
- [X] Rename all crate directories (ax_ → aurora_, axc → aurorac)
- [X] Update all documentation files
- [X] Update CI/CD configurations
- [X] Update agent configuration files
- [X] Update Cargo.toml workspace and metadata
- [X] Verify build system still works

**Acceptance Criteria**:
- ✅ Zero remaining AXION/axion references
- ✅ Zero remaining AXIR/axir references
- ✅ All crate names follow aurora_ prefix
- ✅ Build system compiles successfully
- ✅ All 19 crates renamed correctly
- ✅ AIR (Aurora IR) naming consistent

**Files Modified**:
- All *.rs files (19 crates)
- All *.md files (documentation, specs, agents)
- All Cargo.toml files (workspace + 19 crates)
- .gitignore
- .claude/agents/* (renamed axir.md → air.md)

---

## Phase 1: Lexer & Grammar (Weeks 1-3)

### P1-LEX-001: Define Token Catalog
**Owner**: LexerAgent
**Status**:  Not Started
**Dependencies**: P0-ORG-003
**Estimate**: 2 days
**Completed**: 2025-11-05

**Tasks**:
- [X] Define `TokenKind` enum with all token types
- [X] Define keywords (fn, let, mut, const, etc.)
- [X] Define operators and delimiters
- [X] Define literal types (int, float, string, char, bool)
- [X] Create machine-readable token catalog (JSON)

**Acceptance Criteria**:
-  All Aurora tokens represented
-  JSON schema exported
-  Documentation generated

**Files Created**:
- `crates/aurora_lexer/src/tokens.rs`
- `crates/aurora_lexer/token_catalog.json`

---

### P1-LEX-002: Implement NFA State Machine
**Owner**: LexerAgent
**Status**:  Not Started
**Dependencies**: P1-LEX-001
**Estimate**: 3 days
**Completed**: 2025-11-05

**Tasks**:
- [X] Design NFA state transition table
- [X] Implement maximal-munch tokenization
- [X] Implement reserved-word priority
- [X] Add UTF-8 validation
- [X] Add XID identifier checking

**Acceptance Criteria**:
-  NFA deterministically tokenizes all inputs
-  Maximal-munch correctly disambiguates (e.g., `..` vs `.`)
-  UTF-8 validation rejects invalid sequences
-  XID identifiers accepted, non-XID rejected

**Files Created**:
- `crates/aurora_lexer/src/nfa.rs`
- `crates/aurora_lexer/src/utf8.rs`

---

### P1-LEX-003: Implement Lexer Driver
**Owner**: LexerAgent
**Status**:  Not Started
**Dependencies**: P1-LEX-002
**Estimate**: 2 days
**Completed**: 2025-11-05

**Tasks**:
- [X] Implement `Lexer::new(source: &str) -> Lexer`
- [X] Implement `Lexer::next_token() -> Result<Token, LexError>`
- [X] Track spans (file, line, column, length)
- [X] Handle line comments (`//`)
- [X] Handle block comments (`/* */`, nestable)
- [X] Handle doc comments (`///`, `//!`)

**Acceptance Criteria**:
-  Lexer produces token stream with spans
-  Comments are skipped but tracked for doc extraction
-  Errors include precise location information

**Files Created**:
- `crates/aurora_lexer/src/lib.rs`
- `crates/aurora_lexer/src/span.rs`

---

### P1-LEX-004: Lexer Ambiguity Validation
**Owner**: LexerAgent
**Status**:  Not Started
**Dependencies**: P1-LEX-003
**Estimate**: 1 day
**Completed**: 2025-11-05

**Tasks**:
- [X] Build test suite for all token types
- [X] Test edge cases (e.g., `...`, `1..2`, `..=`)
- [X] Implement ambiguity checker
- [X] Generate ambiguity report

**Acceptance Criteria**:
-  All tokens have test cases
-  Ambiguity report is empty
-  Edge cases correctly tokenized

**Files Created**:
- `crates/aurora_lexer/tests/tokens.rs`
- `crates/aurora_lexer/tests/ambiguity.rs`

---

### P1-LEX-005: Lexer Performance Benchmarks
**Owner**: LexerAgent
**Status**:  Not Started
**Dependencies**: P1-LEX-004
**Estimate**: 1 day
**Completed**: 2025-11-05

**Tasks**:
- [X] Create benchmark suite with criterion
- [X] Test on small (1KB), medium (100KB), large (10MB) files
- [X] Profile with flamegraph
- [X] Optimize hot paths

**Acceptance Criteria**:
-  Lexer throughput e100 MB/s on x86_64
-  Benchmarks tracked in CI

**Files Created**:
- `benches/lexer_bench.rs`

---

### P1-GRAM-001: Define Operator Precedence Table
**Owner**: GrammarAgent
**Status**:  Not Started
**Dependencies**: P1-LEX-001
**Estimate**: 1 day
**Completed**: 2025-11-05

**Tasks**:
- [X] Define 16 precedence levels
- [X] Assign associativity (left, right, none)
- [X] Implement `PrecedenceTable` struct
- [X] Document precedence decisions

**Acceptance Criteria**:
-  All operators have defined precedence
-  Associativity is unambiguous
-  Table is machine-readable

**Files Created**:
- `crates/aurora_grammar/src/precedence.rs`
- `crates/aurora_grammar/precedence.json`

---

### P1-GRAM-002: Define CFG Rules
**Owner**: GrammarAgent
**Status**:  Not Started
**Dependencies**: P1-GRAM-001
**Estimate**: 3 days
**Completed**: 2025-11-05

**Tasks**:
- [X] Write BNF for program items (fn, type, trait, impl, const, mod, use)
- [X] Write BNF for statements (if, for, while, loop, match, let, return)
- [X] Write BNF for patterns (struct, tuple, enum, wildcard)
- [X] Write BNF for types (primitives, compounds, generics, traits)
- [X] Document grammar invariants

**Acceptance Criteria**:
-  Complete BNF notation
-  All constructs covered
-  Grammar is unambiguous (LL(1) for declarations)

**Files Created**:
- `crates/aurora_grammar/src/grammar.rs`
- `docs/grammar.md`

---

### P1-GRAM-003: Grammar Conflict Analysis
**Owner**: GrammarAgent
**Status**:  Not Started
**Dependencies**: P1-GRAM-002
**Estimate**: 2 days
**Completed**: 2025-11-05

**Tasks**:
- [X] Implement LL(1) conflict detector
- [X] Implement FIRST/FOLLOW set computation
- [X] Generate conflict report
- [X] Validate Pratt precedence correctness

**Acceptance Criteria**:
-  Conflict report is empty (zero shift-reduce, zero reduce-reduce)
-  Grammar deterministically parseable

**Files Created**:
- `crates/aurora_grammar/tests/conflicts.rs`
- `crates/aurora_grammar/conflict_report.json`

---

### P1-GRAM-004: Grammar Documentation
**Owner**: GrammarAgent
**Status**:  Not Started
**Dependencies**: P1-GRAM-003
**Estimate**: 1 day
**Completed**: 2025-11-05

**Tasks**:
- [X] Generate railroad diagrams
- [X] Export grammar as JSON/YAML
- [X] Write grammar rationale document

**Acceptance Criteria**:
-  Grammar is published and versioned
-  Diagrams aid understanding
-  Rationale explains design decisions

**Files Created**:
- `docs/grammar.md` (updated)
- `crates/aurora_grammar/grammar.yaml`
- `docs/grammar_rationale.md`

---

## Phase 2: Parser & AST (Weeks 4-7)

### P2-AST-001: Define AST Node Schema
**Owner**: ASTAgent
**Status**:  Not Started
**Dependencies**: P1-GRAM-003
**Estimate**: 3 days

**Tasks**:
- [ ] Define `AstNode` enum with all node kinds
- [ ] Define `Expr` variants (literals, binary ops, calls, etc.)
- [ ] Define `Stmt` variants (let, return, if, match, etc.)
- [ ] Define `Decl` variants (fn, type, trait, impl, etc.)
- [ ] Define `Type` and `Pattern` variants
- [ ] Add spans and hygiene IDs to all nodes

**Acceptance Criteria**:
-  All grammar constructs have AST representation
-  Nodes carry spans for source mapping
-  Hygiene IDs prepared for macro expansion

**Files Created**:
- `crates/aurora_ast/src/nodes.rs`
- `crates/aurora_ast/src/expr.rs`
- `crates/aurora_ast/src/stmt.rs`
- `crates/aurora_ast/src/decl.rs`

---

### P2-AST-002: Implement Arena Allocator
**Owner**: ASTAgent
**Status**:  Not Started
**Dependencies**: P2-AST-001
**Estimate**: 2 days

**Tasks**:
- [ ] Implement bump allocator for AST nodes
- [ ] Implement parent link precomputation
- [ ] Implement preorder/postorder index arrays
- [ ] Optimize for cache locality

**Acceptance Criteria**:
-  Arena allocates nodes contiguously
-  Parent links accessible in O(1)
-  Subtree slicing in O(1)

**Files Created**:
- `crates/aurora_ast/src/arena.rs`

---

### P2-AST-003: Implement Traversal Mechanisms
**Owner**: ASTAgent
**Status**:  Not Started
**Dependencies**: P2-AST-002
**Estimate**: 2 days

**Tasks**:
- [ ] Define `Visitor` trait (iterative, stackless)
- [ ] Implement preorder traversal
- [ ] Implement postorder traversal
- [ ] Implement pretty-printer for debugging

**Acceptance Criteria**:
-  Visitors are iterative (no stack overflow on deep ASTs)
-  Pretty-printer outputs readable AST

**Files Created**:
- `crates/aurora_ast/src/visit.rs`
- `crates/aurora_ast/src/pretty.rs`

---

### P2-AST-004: Freeze AST Schema
**Owner**: ASTAgent
**Status**:  Not Started
**Dependencies**: P2-AST-003
**Estimate**: 1 day

**Tasks**:
- [ ] Export AST schema as JSON
- [ ] Version schema (1.0.0)
- [ ] Document breaking change policy

**Acceptance Criteria**:
-  Schema is frozen for MVP
-  Tools can parse schema JSON
-  Policy prevents accidental breakage

**Files Created**:
- `crates/aurora_ast/schema.json`
- `docs/ast_stability.md`

---

### P2-PAR-001: Implement LL Parser for Declarations
**Owner**: ParserAgent
**Status**:  Not Started
**Dependencies**: P2-AST-004, P1-LEX-003
**Estimate**: 4 days

**Tasks**:
- [ ] Implement `Parser::new(tokens: Vec<Token>) -> Parser`
- [ ] Parse top-level items (program � item*)
- [ ] Parse functions (`fn name(params) -> Type { body }`)
- [ ] Parse types (`type Name = struct { ... }`, `type Name = enum { ... }`)
- [ ] Parse traits (`trait Name { ... }`)
- [ ] Parse impls (`impl Trait for Type { ... }`)
- [ ] Parse modules (`mod name;`, `mod name { ... }`)
- [ ] Parse use statements (`use path::to::item;`)

**Acceptance Criteria**:
-  All declaration forms parse correctly
-  Spans preserved in AST
-  Errors reported with locations

**Files Created**:
- `crates/aurora_parser/src/lib.rs`
- `crates/aurora_parser/src/decls.rs`

---

### P2-PAR-002: Implement Pratt Parser for Expressions
**Owner**: ParserAgent
**Status**:  Not Started
**Dependencies**: P2-PAR-001, P1-GRAM-001
**Estimate**: 4 days

**Tasks**:
- [ ] Implement Pratt table-driven expression parser
- [ ] Parse literals (int, float, string, char, bool)
- [ ] Parse identifiers and paths
- [ ] Parse prefix operators (-, !, ~, *, &)
- [ ] Parse infix operators (arithmetic, comparison, logical, bitwise)
- [ ] Parse postfix operators (calls, field access, indexing)
- [ ] Parse pipelines (`|>`, `<|`)

**Acceptance Criteria**:
-  Expressions parse with correct precedence
-  Associativity is correct
-  Complex expressions nest properly

**Files Created**:
- `crates/aurora_parser/src/exprs.rs`

---

### P2-PAR-003: Implement Statement Parsing
**Owner**: ParserAgent
**Status**:  Not Started
**Dependencies**: P2-PAR-002
**Estimate**: 2 days

**Tasks**:
- [ ] Parse `let` bindings
- [ ] Parse `if`/`else`
- [ ] Parse `match` expressions
- [ ] Parse loops (`for`, `while`, `loop`)
- [ ] Parse `return`, `break`, `continue`, `yield`

**Acceptance Criteria**:
-  All statement forms parse correctly
-  Control flow nesting works

**Files Created**:
- `crates/aurora_parser/src/stmts.rs`

---

### P2-PAR-004: Implement Error Recovery
**Owner**: ParserAgent
**Status**:  Not Started
**Dependencies**: P2-PAR-003
**Estimate**: 2 days

**Tasks**:
- [ ] Identify synchronization points (semicolons, braces)
- [ ] Implement panic-mode recovery
- [ ] Collect multiple errors per parse
- [ ] Generate partial AST on error

**Acceptance Criteria**:
-  Parser recovers from errors
-  Multiple errors reported
-  Partial AST usable for IDE features

**Files Created**:
- `crates/aurora_parser/src/recovery.rs`

---

### P2-PAR-005: Parser Test Suite
**Owner**: ParserAgent
**Status**:  Not Started
**Dependencies**: P2-PAR-004
**Estimate**: 3 days

**Tasks**:
- [ ] Golden tests for all constructs (AST snapshots)
- [ ] Error recovery tests
- [ ] Property tests (fuzz valid/invalid syntax)
- [ ] Differential tests vs grammar spec

**Acceptance Criteria**:
-  All constructs have golden tests
-  Error recovery works on bad input
-  Fuzzing finds no crashes

**Files Created**:
- `crates/aurora_parser/tests/golden/`
- `crates/aurora_parser/tests/recovery.rs`
- `crates/aurora_parser/tests/fuzz.rs`

---

## Phase 3: Name Resolution (Weeks 8-10)

### P3-NAM-001: Design Symbol Table
**Owner**: NameResAgent
**Status**:  Not Started
**Dependencies**: P2-PAR-005
**Estimate**: 2 days

**Tasks**:
- [ ] Define `Symbol` type (name, kind, visibility, span)
- [ ] Define `Scope` hierarchy (module, function, block)
- [ ] Implement symbol insertion and lookup
- [ ] Handle shadowing and visibility rules

**Acceptance Criteria**:
-  Symbols stored with metadata
-  Scopes nest correctly
-  Shadowing handled properly

**Files Created**:
- `crates/aurora_nameres/src/symbols.rs`
- `crates/aurora_nameres/src/scopes.rs`

---

### P3-NAM-002: Implement Hygiene System
**Owner**: NameResAgent
**Status**:  Not Started
**Dependencies**: P3-NAM-001
**Estimate**: 2 days

**Tasks**:
- [ ] Assign hygiene IDs to identifiers
- [ ] Implement scope rebinding after macro expansion
- [ ] Prevent accidental capture

**Acceptance Criteria**:
-  Macros don't capture outside bindings
-  Hygiene IDs tracked through expansion

**Files Created**:
- `crates/aurora_nameres/src/hygiene.rs`

---

### P3-NAM-003: Build Module Graph
**Owner**: NameResAgent
**Status**:  Not Started
**Dependencies**: P3-NAM-001
**Estimate**: 3 days

**Tasks**:
- [ ] Traverse `use` and `mod` declarations
- [ ] Build module dependency graph
- [ ] Detect cycles and report errors
- [ ] Seal import/export tables

**Acceptance Criteria**:
-  Module graph is acyclic
-  Imports resolve to correct modules
-  Cycles are detected and reported

**Files Created**:
- `crates/aurora_nameres/src/modules.rs`

---

### P3-NAM-004: Implement Name Resolution Pass
**Owner**: NameResAgent
**Status**:  Not Started
**Dependencies**: P3-NAM-003
**Estimate**: 3 days

**Tasks**:
- [ ] Resolve all identifiers to symbols
- [ ] Generate "why" chains (provenance tracking)
- [ ] Emit diagnostics for undefined names
- [ ] Emit diagnostics for ambiguous names

**Acceptance Criteria**:
-  All identifiers resolve or error
-  "Why" explanations available
-  Diagnostics are actionable

**Files Created**:
- `crates/aurora_nameres/src/resolve.rs`

---

### P3-NAM-005: Name Resolution Tests
**Owner**: NameResAgent
**Status**:  Not Started
**Dependencies**: P3-NAM-004
**Estimate**: 2 days

**Tasks**:
- [ ] Test shadowing, visibility, imports
- [ ] Test error cases (undefined, ambiguous)
- [ ] Test hygiene (no accidental capture)

**Acceptance Criteria**:
-  All name resolution scenarios tested
-  Error cases produce correct diagnostics

**Files Created**:
- `crates/aurora_nameres/tests/`

---

## Phase 4: Type System (Weeks 11-15)

### P4-TYP-001: Define Type Representation
**Owner**: TypeSystemAgent
**Status**:  Not Started
**Dependencies**: P3-NAM-005
**Estimate**: 2 days

**Tasks**:
- [ ] Define `Type` enum (primitives, compounds, generics, typeclasses)
- [ ] Implement type equality
- [ ] Implement subtyping rules
- [ ] Implement occurs check

**Acceptance Criteria**:
-  All Aurora types represented
-  Type equality is correct
-  Occurs check prevents infinite types

**Files Created**:
- `crates/aurora_types/src/ty.rs`

---

### P4-TYP-002: Implement Hindley-Milner Inference
**Owner**: TypeSystemAgent
**Status**:  Not Started
**Dependencies**: P4-TYP-001
**Estimate**: 5 days

**Tasks**:
- [ ] Implement constraint generation from AST
- [ ] Implement unification algorithm
- [ ] Implement principal type computation
- [ ] Bidirectional type checking (signatures guide inference)

**Acceptance Criteria**:
-  HM inference produces principal types
-  Unannotated expressions infer correctly
-  Inference is deterministic

**Files Created**:
- `crates/aurora_types/src/infer.rs`
- `crates/aurora_types/src/unify.rs`

---

### P4-TYP-003: Implement Typeclasses
**Owner**: TypeSystemAgent
**Status**:  Not Started
**Dependencies**: P4-TYP-002
**Estimate**: 4 days

**Tasks**:
- [ ] Define typeclass representation
- [ ] Implement typeclass resolution (find impls)
- [ ] Enforce coherence (single impl per (trait, type) pair)
- [ ] Support associated types

**Acceptance Criteria**:
-  Typeclasses resolve correctly
-  Coherence violations detected
-  Associated types work

**Files Created**:
- `crates/aurora_types/src/traits.rs`

---

### P4-TYP-004: Implement Generics & Monomorphization
**Owner**: TypeSystemAgent
**Status**:  Not Started
**Dependencies**: P4-TYP-003
**Estimate**: 3 days

**Tasks**:
- [ ] Implement generic instantiation
- [ ] Track monomorphization sites
- [ ] Support reification for FFI (opt-in)

**Acceptance Criteria**:
-  Generics instantiate correctly
-  Monomorphization tracked for codegen

**Files Created**:
- `crates/aurora_types/src/generics.rs`

---

### P4-TYP-005: Implement Null Safety
**Owner**: TypeSystemAgent
**Status**:  Not Started
**Dependencies**: P4-TYP-002
**Estimate**: 2 days

**Tasks**:
- [ ] Enforce Option type for nullable values
- [ ] Implement exhaustiveness checking for matches
- [ ] Require explicit unreachable annotations

**Acceptance Criteria**:
-  No implicit nulls
-  Match exhaustiveness enforced
-  Unreachable annotated

**Files Created**:
- `crates/aurora_types/src/option.rs`
- `crates/aurora_types/src/exhaustive.rs`

---

### P4-TYP-006: Type System Tests
**Owner**: TypeSystemAgent
**Status**:  Not Started
**Dependencies**: P4-TYP-005
**Estimate**: 3 days

**Tasks**:
- [ ] Test inference on complex expressions
- [ ] Test typeclass resolution
- [ ] Test generics instantiation
- [ ] Test exhaustiveness checking

**Acceptance Criteria**:
-  All type system features tested
-  Edge cases covered

**Files Created**:
- `crates/aurora_types/tests/`

---

## Phase 5: Effects & Borrow (Weeks 16-20)

### P5-EFF-001: Implement Effect System
**Owner**: EffectsBorrowAgent
**Status**:  Not Started
**Dependencies**: P4-TYP-006
**Estimate**: 3 days

**Tasks**:
- [ ] Define effect kinds (IO, Alloc, Parallel, Unsafe)
- [ ] Implement effect rows (bitset representation)
- [ ] Implement subeffect partial order
- [ ] Implement effect polymorphism

**Acceptance Criteria**:
-  Effects tracked on function types
-  Subeffecting works correctly

**Files Created**:
- `crates/aurora_effects/src/effects.rs`

---

### P5-EFF-002: Implement Borrow Checker (Advisory)
**Owner**: EffectsBorrowAgent
**Status**:  Not Started
**Dependencies**: P5-EFF-001
**Estimate**: 5 days

**Tasks**:
- [ ] Implement dataflow analysis for borrows
- [ ] Track lifetimes
- [ ] Detect borrow conflicts
- [ ] Emit advisories (not errors)

**Acceptance Criteria**:
-  Borrow conflicts detected
-  Advisories emitted, builds succeed

**Files Created**:
- `crates/aurora_effects/src/borrow.rs`
- `crates/aurora_effects/src/lifetimes.rs`

---

### P5-EFF-003: Implement ARC Insertion
**Owner**: EffectsBorrowAgent
**Status**:  Not Started
**Dependencies**: P5-EFF-002
**Estimate**: 3 days

**Tasks**:
- [ ] Detect uncertain escape points
- [ ] Insert ARC increments/decrements
- [ ] Emit advisories showing ARC sites

**Acceptance Criteria**:
-  ARC inserted at correct points
-  Advisories show where ARC used

**Files Created**:
- `crates/aurora_effects/src/arc.rs`

---

### P5-EFF-004: Implement Strict Mode
**Owner**: EffectsBorrowAgent
**Status**:  Not Started
**Dependencies**: P5-EFF-003
**Estimate**: 2 days

**Tasks**:
- [ ] Convert advisories � errors in strict mode
- [ ] Require explicit lifetimes in strict mode
- [ ] Disallow ARC insertion in strict mode

**Acceptance Criteria**:
-  Strict mode enforces borrow rules
-  Builds fail on violations

**Files Created**:
- `crates/aurora_effects/src/strict.rs`

---

### P5-EFF-005: Effects & Borrow Tests
**Owner**: EffectsBorrowAgent
**Status**:  Not Started
**Dependencies**: P5-EFF-004
**Estimate**: 3 days

**Tasks**:
- [ ] Test effect tracking
- [ ] Test borrow checker advisories
- [ ] Test ARC insertion
- [ ] Test strict mode enforcement

**Acceptance Criteria**:
-  All scenarios tested
-  Strict mode works correctly

**Files Created**:
- `crates/aurora_effects/tests/`

---

## Phase 6: MIR (Weeks 21-25)

### P6-MIR-001: Design MIR Representation
**Owner**: MIRAgent
**Status**:  Not Started
**Dependencies**: P5-EFF-005
**Estimate**: 3 days

**Tasks**:
- [ ] Define MIR instructions (SSA form)
- [ ] Define basic blocks and CFG
- [ ] Implement dominance tree computation
- [ ] Add effect edges to instructions

**Acceptance Criteria**:
-  MIR is in SSA form
-  CFG correctly represents control flow
-  Dominance computed correctly

**Files Created**:
- `crates/aurora_mir/src/mir.rs`
- `crates/aurora_mir/src/cfg.rs`
- `crates/aurora_mir/src/dominance.rs`

---

### P6-MIR-002: Implement MIR Lowering
**Owner**: MIRAgent
**Status**:  Not Started
**Dependencies**: P6-MIR-001
**Estimate**: 4 days

**Tasks**:
- [ ] Lower typed AST � MIR
- [ ] Insert effect edges
- [ ] Preserve source spans

**Acceptance Criteria**:
-  MIR correctly represents AST semantics
-  Spans preserved for diagnostics

**Files Created**:
- `crates/aurora_mir/src/lower.rs`

---

### P6-MIR-003: Implement MIR Optimizations (Part 1)
**Owner**: MIRAgent, OptimizerAgent
**Status**:  Not Started
**Dependencies**: P6-MIR-002
**Estimate**: 4 days

**Tasks**:
- [ ] Implement inlining heuristics
- [ ] Implement SROA (scalar replacement of aggregates)
- [ ] Implement GVN (global value numbering)
- [ ] Implement LICM (loop-invariant code motion)

**Acceptance Criteria**:
-  Optimizations preserve correctness
-  Measurable performance improvements

**Files Created**:
- `crates/aurora_mir/src/opt/inline.rs`
- `crates/aurora_mir/src/opt/sroa.rs`
- `crates/aurora_mir/src/opt/gvn.rs`
- `crates/aurora_mir/src/opt/licm.rs`

---

### P6-MIR-004: Implement MIR Optimizations (Part 2)
**Owner**: MIRAgent, OptimizerAgent
**Status**:  Not Started
**Dependencies**: P6-MIR-003
**Estimate**: 3 days

**Tasks**:
- [ ] Implement DCE (dead code elimination)
- [ ] Implement NRVO (named return value optimization)
- [ ] Implement devirtualization

**Acceptance Criteria**:
-  Dead code removed correctly
-  NRVO eliminates copies
-  Devirtualization works on known types

**Files Created**:
- `crates/aurora_mir/src/opt/dce.rs`
- `crates/aurora_mir/src/opt/nrvo.rs`
- `crates/aurora_mir/src/opt/devirt.rs`

---

### P6-MIR-005: Implement Loop Vectorization
**Owner**: MIRAgent, OptimizerAgent
**Status**:  Not Started
**Dependencies**: P6-MIR-004
**Estimate**: 4 days

**Tasks**:
- [ ] Detect vectorizable loops
- [ ] Generate SIMD instructions for x86_64
- [ ] Emit advisories for non-vectorizable loops

**Acceptance Criteria**:
-  Simple loops vectorize correctly
-  SIMD instructions generated

**Files Created**:
- `crates/aurora_mir/src/opt/simd.rs`

---

### P6-MIR-006: Implement MIR Dumps
**Owner**: MIRAgent
**Status**:  Not Started
**Dependencies**: P6-MIR-005
**Estimate**: 2 days

**Tasks**:
- [ ] Implement human-readable MIR printer
- [ ] Implement JSON export
- [ ] Preserve spans in dumps

**Acceptance Criteria**:
-  MIR dumps are readable
-  JSON export for tooling

**Files Created**:
- `crates/aurora_mir/src/dump.rs`

---

### P6-MIR-007: MIR Tests
**Owner**: MIRAgent
**Status**:  Not Started
**Dependencies**: P6-MIR-006
**Estimate**: 3 days

**Tasks**:
- [ ] Golden tests for MIR output
- [ ] Optimization correctness tests
- [ ] Roundtrip tests (AST � MIR � semantics)

**Acceptance Criteria**:
-  MIR lowering correct
-  Optimizations preserve semantics

**Files Created**:
- `crates/aurora_mir/tests/`

---

## Phase 7: AIR & Backend (Weeks 26-30)

### P7-AIR-001: Design AIR Format
**Owner**: AIRAgent
**Status**:  Not Started
**Dependencies**: P6-MIR-007
**Estimate**: 3 days

**Tasks**:
- [ ] Define NASM-like textual format
- [ ] Support x86_64 instruction set
- [ ] Include directives for debug info and unwind

**Acceptance Criteria**:
-  AIR is human-readable
-  AIR format documented

**Files Created**:
- `crates/aurora_air/src/air.rs`
- `docs/air_format.md`

---

### P7-AIR-002: Implement AIR Emission
**Owner**: AIRAgent
**Status**:  Not Started
**Dependencies**: P7-AIR-001
**Estimate**: 4 days

**Tasks**:
- [ ] Lower MIR � AIR
- [ ] Implement register allocation (linear scan)
- [ ] Emit AIR text format

**Acceptance Criteria**:
-  AIR correctly represents MIR
-  Register allocation works

**Files Created**:
- `crates/aurora_air/src/emit.rs`
- `crates/aurora_air/src/regalloc.rs`

---

### P7-AIR-003: Implement Peephole Optimizations
**Owner**: AIRAgent, OptimizerAgent
**Status**:  Not Started
**Dependencies**: P7-AIR-002
**Estimate**: 3 days

**Tasks**:
- [ ] Mov collapse (remove redundant moves)
- [ ] LEA patterns (use LEA for address arithmetic)
- [ ] Branch shortening (optimize jump distances)

**Acceptance Criteria**:
-  Peepholes improve performance
-  Correctness preserved

**Files Created**:
- `crates/aurora_air/src/peephole.rs`

---

### P7-AIR-004: Implement Instruction Scheduling
**Owner**: AIRAgent, OptimizerAgent
**Status**:  Not Started
**Dependencies**: P7-AIR-003
**Estimate**: 3 days

**Tasks**:
- [ ] Implement basic block scheduling
- [ ] Use CPU profiles (Skylake, Zen) for latency/throughput
- [ ] Avoid pipeline stalls

**Acceptance Criteria**:
-  Scheduling improves performance
-  CPU profiles used correctly

**Files Created**:
- `crates/aurora_air/src/schedule.rs`
- `crates/aurora_air/cpu_profiles/`

---

### P7-BACK-001: Implement LLVM Backend
**Owner**: BackendAgent
**Status**:  Not Started
**Dependencies**: P7-AIR-004
**Estimate**: 5 days

**Tasks**:
- [ ] Translate AIR � LLVM IR
- [ ] Use LLVM optimization passes (optional)
- [ ] Emit object files (COFF, ELF)
- [ ] Generate debug info (DWARF, PDB)

**Acceptance Criteria**:
-  LLVM backend produces correct binaries
-  Debug info is usable

**Files Created**:
- `crates/aurora_backend/src/llvm.rs`

---

### P7-BACK-002: Implement Linking
**Owner**: BackendAgent
**Status**:  Not Started
**Dependencies**: P7-BACK-001
**Estimate**: 2 days

**Tasks**:
- [ ] Use LLD for final linking
- [ ] Support PE/COFF (Windows) and ELF (Linux)
- [ ] Include SEH (Windows) and unwind info

**Acceptance Criteria**:
-  Binaries link correctly
-  Executables run on target platforms

**Files Created**:
- `crates/aurora_backend/src/link.rs`

---

### P7-AIR-005: AIR Tests & Roundtrip
**Owner**: AIRAgent
**Status**:  Not Started
**Dependencies**: P7-BACK-002
**Estimate**: 2 days

**Tasks**:
- [ ] Test AIR roundtrip (emit � parse � emit)
- [ ] Golden tests for AIR output
- [ ] Validate debug info correctness

**Acceptance Criteria**:
-  AIR roundtrips successfully
-  Binaries execute correctly

**Files Created**:
- `crates/aurora_air/tests/`

---

## Phase 8: Interop & Concurrency (Weeks 31-34)

### P8-INT-001: Implement C ABI Support
**Owner**: InteropAgent
**Status**:  Not Started
**Dependencies**: P7-BACK-002
**Estimate**: 3 days

**Tasks**:
- [ ] Implement stable name mangling
- [ ] Generate C headers from Aurora declarations
- [ ] Implement safety shims for common C pitfalls

**Acceptance Criteria**:
-  C FFI works bidirectionally
-  Headers generated correctly

**Files Created**:
- `crates/aurora_interop/src/c_abi.rs`
- `crates/aurora_interop/src/header_gen.rs`

---

### P8-CONC-001: Implement Goroutines & Channels
**Owner**: ConcurrencyAgent
**Status**:  Not Started
**Dependencies**: P7-BACK-002
**Estimate**: 5 days

**Tasks**:
- [ ] Implement work-stealing scheduler (M:N)
- [ ] Implement channel send/recv operations
- [ ] Integrate with async runtime

**Acceptance Criteria**:
-  Goroutines execute correctly
-  Channels communicate correctly

**Files Created**:
- `crates/aurora_concurrency/src/scheduler.rs`
- `crates/aurora_concurrency/src/channels.rs`

---

### P8-CONC-002: Implement Async/Await
**Owner**: ConcurrencyAgent
**Status**:  Not Started
**Dependencies**: P8-CONC-001
**Estimate**: 4 days

**Tasks**:
- [ ] Lower async functions � state machines
- [ ] Implement structured cancellation
- [ ] Integrate with event loop

**Acceptance Criteria**:
-  Async/await works correctly
-  Cancellation structured

**Files Created**:
- `crates/aurora_concurrency/src/async.rs`

---

### P8-INTEROP-002: Interop Tests
**Owner**: InteropAgent, ConcurrencyAgent
**Status**:  Not Started
**Dependencies**: P8-INT-001, P8-CONC-002
**Estimate**: 2 days

**Tasks**:
- [ ] Test C FFI roundtrips
- [ ] Test goroutines and channels
- [ ] Test async/await integration

**Acceptance Criteria**:
-  All interop scenarios work
-  Concurrency tests pass

**Files Created**:
- `crates/aurora_interop/tests/`
- `crates/aurora_concurrency/tests/`

---

## Phase 9: Tooling & Diagnostics (Weeks 35-38)

### P9-BUILD-001: Implement `aurora` CLI
**Owner**: BuildAgent
**Status**:  Not Started
**Dependencies**: P8-INTEROP-002
**Estimate**: 4 days

**Tasks**:
- [ ] Implement `ax init`, `ax build`, `ax run`
- [ ] Implement `ax test` (test runner)
- [ ] Implement `ax fmt` (formatter)
- [ ] Implement `ax lint` (linter)
- [ ] Implement `ax doc` (doc generator)

**Acceptance Criteria**:
-  All CLI commands work end-to-end
-  UX is polished

**Files Created**:
- `crates/aurora_build/src/cli.rs`
- `crates/aurora_build/src/fmt.rs`
- `crates/aurora_build/src/lint.rs`

---

### P9-DIAG-001: Implement JSON Diagnostics
**Owner**: DiagnosticsAgent
**Status**:  Not Started
**Dependencies**: P9-BUILD-001
**Estimate**: 2 days

**Tasks**:
- [ ] Define diagnostic JSON schema
- [ ] Emit diagnostics with spans, fix-its, confidence
- [ ] Include doc URLs for each diagnostic

**Acceptance Criteria**:
-  Diagnostics are machine-consumable
-  Schema is stable

**Files Created**:
- `crates/aurora_diagnostics/src/json.rs`
- `crates/aurora_diagnostics/diagnostic_schema.json`

---

### P9-DIAG-002: Implement LSP Server
**Owner**: DiagnosticsAgent
**Status**:  Not Started
**Dependencies**: P9-DIAG-001
**Estimate**: 5 days

**Tasks**:
- [ ] Implement LSP protocol handler
- [ ] Implement completions
- [ ] Implement go-to-definition, find references
- [ ] Implement hover (type info)
- [ ] Implement diagnostics and code actions
- [ ] Implement rename refactoring

**Acceptance Criteria**:
-  LSP provides basic IDE features
-  Integrates with VS Code, Neovim, etc.

**Files Created**:
- `tools/axlsp/src/main.rs`
- `tools/axlsp/src/completion.rs`
- `tools/axlsp/src/hover.rs`

---

### P9-TEST-001: Implement Testing Framework
**Owner**: TestingAgent
**Status**:  Not Started
**Dependencies**: P9-BUILD-001
**Estimate**: 3 days

**Tasks**:
- [ ] Implement `#[test]` attribute handling
- [ ] Implement test runner with parallel execution
- [ ] Support property-based tests (proptest)
- [ ] Generate test reports

**Acceptance Criteria**:
-  Test framework runs all test types
-  Parallel execution works

**Files Created**:
- `crates/aurora_testing/src/runner.rs`
- `crates/aurora_testing/src/proptest.rs`

---

## Phase 10: Security & Documentation (Weeks 39-41)

### P10-SEC-001: Implement SBOM Generation
**Owner**: SecurityAgent
**Status**:  Not Started
**Dependencies**: P9-TEST-001
**Estimate**: 2 days

**Tasks**:
- [ ] Implement dependency graph traversal
- [ ] Generate SBOM (SPDX format)
- [ ] Include license and version info

**Acceptance Criteria**:
-  SBOM includes all dependencies
-  SPDX format is valid

**Files Created**:
- `crates/aurora_security/src/sbom.rs`

---

### P10-SEC-002: Implement Reproducible Builds
**Owner**: SecurityAgent
**Status**:  Not Started
**Dependencies**: P10-SEC-001
**Estimate**: 2 days

**Tasks**:
- [ ] Implement lockfile format
- [ ] Validate deterministic codegen
- [ ] Generate build fingerprint

**Acceptance Criteria**:
-  Builds are byte-for-byte reproducible
-  Lockfile pins all dependencies

**Files Created**:
- `crates/aurora_security/src/repro.rs`
- `crates/aurora_security/src/lockfile.rs`

---

### P10-DOC-001: Write Language Reference
**Owner**: DocumentationAgent
**Status**:  Not Started
**Dependencies**: P10-SEC-002
**Estimate**: 4 days

**Tasks**:
- [ ] Document lexical structure
- [ ] Document grammar and syntax
- [ ] Document type system and semantics
- [ ] Document standard library

**Acceptance Criteria**:
-  Documentation is complete
-  Examples are runnable

**Files Created**:
- `docs/reference/lexical.md`
- `docs/reference/grammar.md`
- `docs/reference/types.md`
- `docs/reference/stdlib.md`

---

### P10-DOC-002: Generate API Documentation
**Owner**: DocumentationAgent
**Status**:  Not Started
**Dependencies**: P10-DOC-001
**Estimate**: 2 days

**Tasks**:
- [ ] Generate API docs from source
- [ ] Include examples for all public APIs
- [ ] Publish to docs site

**Acceptance Criteria**:
-  API docs are complete
-  Docs site is published

**Files Created**:
- `docs/stdlib/`
- `docs/site/`

---

## Phase 11: Self-Hosting (Weeks 42-44)

### P11-SELF-001: Implement Conformance Test Suite
**Owner**: TestingAgent
**Status**:  Not Started
**Dependencies**: P10-DOC-002
**Estimate**: 4 days

**Tasks**:
- [ ] Build comprehensive test suite (unit, integration, golden)
- [ ] Include differential tests vs C implementations
- [ ] Cross-arch testing (QEMU for ARM, RISC-V)

**Acceptance Criteria**:
-  All conformance tests pass
-  Cross-arch CI green

**Files Created**:
- `tests/conformance/`

---

### P11-SELF-002: Implement Performance Benchmarks
**Owner**: OptimizerAgent, TestingAgent
**Status**:  Not Started
**Dependencies**: P11-SELF-001
**Estimate**: 3 days

**Tasks**:
- [ ] Implement microbenchmarks (arithmetic, reductions, etc.)
- [ ] Implement kernel benchmarks (FFT, GEMM, sorting)
- [ ] Validate performance targets (d1.10� C on kernels)

**Acceptance Criteria**:
-  Benchmarks run in CI
-  Performance targets met (best-effort for MVP)

**Files Created**:
- `benches/perf/`

---

### P11-SELF-003: Self-Hosting Validation
**Owner**: Orchestrator
**Status**:  Not Started
**Dependencies**: P11-SELF-002
**Estimate**: 4 days

**Tasks**:
- [ ] Rewrite critical compiler paths in Aurora
- [ ] Bootstrap compiler with previous version
- [ ] Validate identical output (dogfooding)

**Acceptance Criteria**:
-  Compiler compiles itself
-  Output is identical (deterministic)

**Files Created**:
- Self-hosted compiler components

---

## Summary

**Total Tasks**: 89
**Total Estimated Days**: 196 person-days (~9-10 months with 2-3 contributors)

**Critical Path**:
P0 � P1 (Lexer/Grammar) � P2 (Parser/AST) � P3 (NameRes) � P4 (Types) � P5 (Effects) � P6 (MIR) � P7 (AIR/Backend) � P8 (Interop/Concurrency) � P9 (Tooling) � P10 (Security/Docs) � P11 (Self-Hosting)

**Key Milestones**:
- Week 3: Lexer & Grammar complete
- Week 7: Parser & AST complete
- Week 10: Name Resolution complete
- Week 15: Type System complete
- Week 20: Effects & Borrow complete
- Week 25: MIR complete
- Week 30: AIR & Backend complete
- Week 34: Interop & Concurrency complete
- Week 38: Tooling & Diagnostics complete
- Week 41: Security & Docs complete
- Week 44: MVP Self-Hosted

---

**Document Status**: Living task list, updated weekly
**Last Updated**: 2025-11-04
**Next Update**: Upon completion of P0 (Bootstrap)
