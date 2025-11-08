# Compiler Internals

This document provides a deep dive into the Aurora compiler implementation, suitable for contributors and advanced users.

## Table of Contents

1. [Crate Organization](#crate-organization)
2. [Data Structures](#data-structures)
3. [Compilation Flow](#compilation-flow)
4. [Type Inference Algorithm](#type-inference-algorithm)
5. [Borrow Checker](#borrow-checker)
6. [Code Generation](#code-generation)
7. [Optimization Passes](#optimization-passes)
8. [Testing Infrastructure](#testing-infrastructure)
9. [Contributing](#contributing)

## Crate Organization

The Aurora compiler is organized as a Cargo workspace with 18 crates:

```
Aurora/
├── crates/
│   ├── aurora_lexer/       # Tokenization
│   ├── aurora_grammar/     # Grammar definitions
│   ├── aurora_parser/      # Parsing
│   ├── aurora_ast/         # AST definitions
│   ├── aurora_nameres/     # Name resolution
│   ├── aurora_types/       # Type system
│   ├── aurora_effects/     # Effects & borrow checking
│   ├── aurora_mir/         # Mid-level IR
│   ├── aurora_air/         # Low-level IR
│   ├── aurora_backend/     # Code generation
│   ├── aurora_interop/     # FFI support
│   ├── aurora_concurrency/ # Concurrency runtime
│   ├── aurora_optimizer/   # Optimization
│   ├── aurora_build/       # Build system
│   ├── aurora_diagnostics/ # Diagnostics & LSP
│   ├── aurora_testing/     # Test framework
│   ├── aurora_security/    # Security & SBOM
│   ├── aurora_docs/        # Documentation gen
│   └── aurorac/            # Compiler driver
├── docs/                   # Documentation
└── examples/               # Example programs
```

### Dependencies Between Crates

```
aurorac
  ├─> aurora_lexer
  ├─> aurora_parser ──> aurora_ast
  ├─> aurora_nameres ──> aurora_ast
  ├─> aurora_types ──> aurora_ast
  ├─> aurora_effects ──> aurora_types
  ├─> aurora_mir ──> aurora_types
  ├─> aurora_air ──> aurora_mir
  ├─> aurora_backend ──> aurora_air
  ├─> aurora_optimizer ──> aurora_mir, aurora_air
  ├─> aurora_diagnostics
  └─> aurora_build
```

## Data Structures

### Arena Allocation

All AST nodes are allocated in an arena for fast allocation and deallocation:

```rust
// crates/aurora_ast/src/arena.rs

pub struct Arena {
    chunk_size: usize,
    chunks: Vec<Chunk>,
    current: usize,
}

impl Arena {
    pub fn alloc<T>(&mut self, value: T) -> &mut T {
        // Bump allocator implementation
    }
}
```

**Benefits**:
- O(1) allocation
- No fragmentation
- Entire AST freed at once
- Better cache locality

### AST Representation

```rust
// crates/aurora_ast/src/ast.rs

pub struct Expr {
    pub id: ExprId,
    pub kind: ExprKind,
    pub span: Span,
    pub ty: Option<TypeId>,
}

pub enum ExprKind {
    Lit(Literal),
    Var(Symbol),
    Call(Box<Expr>, Vec<Expr>),
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
    // ... more variants
}
```

**Key Features**:
- Every node has unique ID
- Span tracking for error reporting
- Optional type annotation (filled by type checker)
- Boxed recursive fields to control size

### Symbol Tables

```rust
// crates/aurora_nameres/src/scopes.rs

pub struct SymbolTable {
    scopes: Vec<Scope>,
    symbols: HashMap<Symbol, Definition>,
}

pub struct Definition {
    pub name: Symbol,
    pub kind: DefKind,
    pub scope: ScopeId,
    pub span: Span,
}

pub enum DefKind {
    Variable,
    Function,
    Type,
    Module,
}
```

**Invariants**:
- Each symbol has exactly one definition
- Scopes form a tree
- Inner scopes can shadow outer scopes

### Type Representation

```rust
// crates/aurora_types/src/ty.rs

pub enum Type {
    Primitive(PrimitiveType),
    Function(Vec<Type>, Box<Type>),
    Tuple(Vec<Type>),
    Generic(GenericId),
    TypeVar(TypeVarId),
    // ... more variants
}
```

**Type Variables**:
- Used during inference
- Unified to concrete types
- Tracked in substitution map

### MIR (Mid-level IR)

```rust
// crates/aurora_mir/src/mir.rs

pub struct Function {
    pub name: String,
    pub params: Vec<Register>,
    pub blocks: Vec<BasicBlock>,
    pub cfg: ControlFlowGraph,
}

pub struct BasicBlock {
    pub id: BlockId,
    pub instructions: Vec<Instruction>,
    pub terminator: Terminator,
}

pub enum Instruction {
    Assign(Register, Operand),
    Call(Register, String, Vec<Operand>),
    Load(Register, Register, i64),
    Store(Register, i64, Operand),
    // ... more instructions
}
```

**SSA Properties**:
- Every register assigned exactly once
- Phi nodes at join points
- Dominance maintained

## Compilation Flow

### Phase 1: Lexing

```rust
// crates/aurora_lexer/src/lexer.rs

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    tokens: Vec<Token>,
}

impl Lexer {
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        while !self.at_end() {
            self.skip_whitespace();
            let token = self.next_token()?;
            self.tokens.push(token);
        }
        Ok(self.tokens.clone())
    }
}
```

**Tokenization Rules**:
- Maximal munch (longest match wins)
- Keywords checked after identifier
- Unicode identifiers via XID_Start/XID_Continue

### Phase 2: Parsing

```rust
// crates/aurora_parser/src/parser.rs

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    arena: Arena,
}

impl Parser {
    pub fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_pratt_expr(0)  // Pratt parser
    }

    fn parse_pratt_expr(&mut self, min_bp: u8) -> Result<Expr> {
        let mut lhs = self.parse_primary()?;

        while let Some(op) = self.peek_binary_op() {
            let (l_bp, r_bp) = op.binding_power();
            if l_bp < min_bp {
                break;
            }
            self.advance();  // consume operator
            let rhs = self.parse_pratt_expr(r_bp)?;
            lhs = Expr::binary(op, lhs, rhs);
        }

        Ok(lhs)
    }
}
```

**Precedence Levels** (lowest to highest):
1. Assignment (`=`, `+=`, etc.)
2. Logical OR (`||`)
3. Logical AND (`&&`)
4. Comparison (`==`, `!=`, `<`, `>`, etc.)
5. Bitwise OR (`|`)
6. Bitwise XOR (`^`)
7. Bitwise AND (`&`)
8. Shift (`<<`, `>>`)
9. Addition (`+`, `-`)
10. Multiplication (`*`, `/`, `%`)
11. Exponentiation (`**`)
12. Unary (`!`, `-`, `&`, `*`)
13. Pipeline (`|>`, `<|`)
14. Call/Index (`()`, `[]`, `.`)

### Phase 3: Name Resolution

```rust
// crates/aurora_nameres/src/resolver.rs

pub struct Resolver {
    scopes: ScopeTree,
    symbols: SymbolTable,
    current_scope: ScopeId,
}

impl Resolver {
    pub fn resolve(&mut self, ast: &Ast) -> Result<()> {
        // First pass: collect definitions
        self.collect_definitions(ast)?;

        // Second pass: resolve references
        self.resolve_references(ast)?;

        Ok(())
    }

    fn resolve_name(&self, name: &str) -> Option<Definition> {
        let mut scope = self.current_scope;
        loop {
            if let Some(def) = self.symbols.get(scope, name) {
                return Some(def);
            }
            scope = self.scopes.parent(scope)?;
        }
    }
}
```

**Resolution Rules**:
- Inner scopes shadow outer scopes
- Imports add symbols to scope
- Hygiene prevents accidental capture

### Phase 4: Type Checking

Type inference uses **Algorithm W** (Hindley-Milner):

```rust
// crates/aurora_types/src/infer.rs

pub struct TypeChecker {
    env: TypeEnv,
    subst: Substitution,
    fresh_var: u32,
}

impl TypeChecker {
    pub fn infer_expr(&mut self, expr: &Expr) -> Result<Type> {
        match &expr.kind {
            ExprKind::Lit(lit) => Ok(self.infer_literal(lit)),

            ExprKind::Var(name) => {
                self.env.get(name)
                    .ok_or_else(|| Error::undefined_variable(name))
            }

            ExprKind::Call(func, args) => {
                let func_ty = self.infer_expr(func)?;
                let arg_tys = args.iter()
                    .map(|arg| self.infer_expr(arg))
                    .collect::<Result<Vec<_>>>()?;

                let ret_ty = self.fresh_type_var();
                let expected = Type::Function(arg_tys.clone(), Box::new(ret_ty.clone()));

                self.unify(func_ty, expected)?;
                Ok(ret_ty)
            }

            // ... more cases
        }
    }

    fn unify(&mut self, t1: Type, t2: Type) -> Result<()> {
        match (t1, t2) {
            (Type::TypeVar(v), t) | (t, Type::TypeVar(v)) => {
                if self.occurs_check(v, &t) {
                    return Err(Error::infinite_type());
                }
                self.subst.insert(v, t);
                Ok(())
            }

            (Type::Function(p1, r1), Type::Function(p2, r2)) => {
                if p1.len() != p2.len() {
                    return Err(Error::arity_mismatch());
                }
                for (t1, t2) in p1.iter().zip(p2.iter()) {
                    self.unify(t1.clone(), t2.clone())?;
                }
                self.unify(*r1, *r2)
            }

            (t1, t2) if t1 == t2 => Ok(()),

            _ => Err(Error::type_mismatch(t1, t2)),
        }
    }
}
```

**Key Concepts**:
- **Type Variables**: Placeholders during inference
- **Unification**: Make two types equal by finding substitution
- **Occurs Check**: Prevent infinite types
- **Substitution**: Map from type vars to concrete types

### Phase 5: Effects & Borrow Checking

```rust
// crates/aurora_effects/src/borrow.rs

pub struct BorrowChecker {
    live_borrows: HashMap<Place, BorrowKind>,
    dataflow: DataflowAnalysis,
}

impl BorrowChecker {
    pub fn check_function(&mut self, func: &Function) -> Result<()> {
        for block in &func.blocks {
            for instr in &block.instructions {
                self.check_instruction(instr)?;
            }
        }
        Ok(())
    }

    fn check_instruction(&mut self, instr: &Instruction) -> Result<()> {
        match instr {
            Instruction::Store(place, value) => {
                // Check if place is borrowed immutably
                if let Some(BorrowKind::Immutable) = self.live_borrows.get(place) {
                    return Err(Error::cannot_mutate_borrowed(place));
                }
                self.check_move_or_copy(value)?;
            }

            Instruction::Borrow(dest, place, kind) => {
                // Check if compatible with existing borrows
                self.check_borrow_compatible(place, kind)?;
                self.live_borrows.insert(place.clone(), kind);
            }

            // ... more cases
        }
        Ok(())
    }
}
```

**Borrow Rules**:
- One mutable borrow XOR many immutable borrows
- Borrows must not outlive the value
- No use after move (unless Copy)

### Phase 6: MIR Generation

```rust
// crates/aurora_mir/src/mir_gen.rs

pub struct MirGenerator {
    current_function: Function,
    current_block: BlockId,
    register_counter: u32,
}

impl MirGenerator {
    pub fn generate_expr(&mut self, expr: &Expr) -> Register {
        match &expr.kind {
            ExprKind::Lit(lit) => {
                let reg = self.fresh_register();
                self.emit(Instruction::Const(reg, lit.clone()));
                reg
            }

            ExprKind::Binary(op, lhs, rhs) => {
                let lhs_reg = self.generate_expr(lhs);
                let rhs_reg = self.generate_expr(rhs);
                let result = self.fresh_register();
                self.emit(Instruction::Binary(result, *op, lhs_reg, rhs_reg));
                result
            }

            ExprKind::If(cond, then_br, else_br) => {
                let cond_reg = self.generate_expr(cond);

                let then_block = self.new_block();
                let else_block = self.new_block();
                let join_block = self.new_block();

                self.emit_terminator(Terminator::Branch(cond_reg, then_block, else_block));

                // Generate then branch
                self.set_current_block(then_block);
                let then_reg = self.generate_expr(then_br);
                self.emit_terminator(Terminator::Jump(join_block));

                // Generate else branch
                self.set_current_block(else_block);
                let else_reg = self.generate_expr(else_br);
                self.emit_terminator(Terminator::Jump(join_block));

                // Join point with phi
                self.set_current_block(join_block);
                let result = self.fresh_register();
                self.emit(Instruction::Phi(result, vec![
                    (then_block, then_reg),
                    (else_block, else_reg),
                ]));

                result
            }

            // ... more cases
        }
    }
}
```

## Type Inference Algorithm

### Algorithm W (Damas-Milner)

**Input**: Expression `e`, Type environment `Γ`
**Output**: Type `τ`, Substitution `S`

```
W(Γ, x) = (∅, Γ(x))  // Variable

W(Γ, λx.e) =
    let β = fresh type variable
    let (S, τ) = W(Γ[x ↦ β], e)
    return (S, S(β) → τ)  // Abstraction

W(Γ, e1 e2) =
    let (S1, τ1) = W(Γ, e1)
    let (S2, τ2) = W(S1(Γ), e2)
    let β = fresh type variable
    let S3 = unify(S2(τ1), τ2 → β)
    return (S3 ∘ S2 ∘ S1, S3(β))  // Application

W(Γ, let x = e1 in e2) =
    let (S1, τ1) = W(Γ, e1)
    let σ = generalize(S1(Γ), τ1)
    let (S2, τ2) = W(S1(Γ)[x ↦ σ], e2)
    return (S2 ∘ S1, τ2)  // Let
```

**Generalization**: Convert monotype to polytype by quantifying free type variables.

**Instantiation**: Replace quantified variables with fresh type variables.

## Borrow Checker

### Dataflow Analysis

The borrow checker uses dataflow analysis to track borrows:

```rust
pub struct DataflowAnalysis {
    gen: HashMap<BlockId, BitSet>,     // Borrows created
    kill: HashMap<BlockId, BitSet>,    // Borrows ended
    live_in: HashMap<BlockId, BitSet>, // Live at block entry
    live_out: HashMap<BlockId, BitSet>,// Live at block exit
}

impl DataflowAnalysis {
    pub fn compute_liveness(&mut self, cfg: &ControlFlowGraph) {
        let mut changed = true;
        while changed {
            changed = false;
            for block in cfg.reverse_postorder() {
                let old_in = self.live_in[&block].clone();

                // live_out[B] = Union of live_in[S] for all successors S
                let mut live_out = BitSet::new();
                for succ in cfg.successors(block) {
                    live_out.union(&self.live_in[succ]);
                }
                self.live_out.insert(block, live_out.clone());

                // live_in[B] = gen[B] ∪ (live_out[B] - kill[B])
                let mut live_in = live_out;
                live_in.difference(&self.kill[&block]);
                live_in.union(&self.gen[&block]);

                if live_in != old_in {
                    self.live_in.insert(block, live_in);
                    changed = true;
                }
            }
        }
    }
}
```

## Code Generation

### AIR to Machine Code

```rust
// crates/aurora_backend/src/llvm.rs

pub struct LLVMBackend {
    context: Context,
    module: Module,
    builder: Builder,
}

impl LLVMBackend {
    pub fn compile_function(&mut self, func: &AirFunction) -> LLVMValue {
        let llvm_func = self.declare_function(func);

        for (block_id, block) in &func.blocks {
            let llvm_block = self.append_block(llvm_func);
            self.builder.position_at_end(llvm_block);

            for instr in &block.instructions {
                self.compile_instruction(instr);
            }

            self.compile_terminator(&block.terminator);
        }

        llvm_func
    }

    fn compile_instruction(&mut self, instr: &Instruction) {
        match instr {
            Instruction::Mov(dest, src) => {
                let src_val = self.load_operand(src);
                self.store_register(dest, src_val);
            }

            Instruction::Add(dest, lhs, rhs) => {
                let lhs_val = self.load_operand(lhs);
                let rhs_val = self.load_operand(rhs);
                let result = self.builder.build_add(lhs_val, rhs_val);
                self.store_register(dest, result);
            }

            // ... more instructions
        }
    }
}
```

## Optimization Passes

### Inline Functions

```rust
pub struct Inliner {
    inline_threshold: usize,  // Max function size to inline
    call_depth: usize,        // Current recursion depth
}

impl Inliner {
    pub fn should_inline(&self, func: &Function) -> bool {
        // Don't inline recursive calls
        if self.call_depth > 5 {
            return false;
        }

        // Don't inline large functions
        if func.instruction_count() > self.inline_threshold {
            return false;
        }

        // Always inline small functions
        if func.instruction_count() < 10 {
            return true;
        }

        // Check cost model
        self.compute_inline_cost(func) < 100
    }

    fn compute_inline_cost(&self, func: &Function) -> i32 {
        let mut cost = 0;
        for block in &func.blocks {
            for instr in &block.instructions {
                cost += match instr {
                    Instruction::Call(..) => 5,    // Expensive
                    Instruction::Load(..) => 2,
                    Instruction::Store(..) => 2,
                    _ => 1,
                };
            }
        }
        cost
    }
}
```

### Dead Code Elimination

```rust
pub struct DeadCodeEliminator {
    live_values: HashSet<Register>,
}

impl DeadCodeEliminator {
    pub fn eliminate(&mut self, func: &mut Function) {
        // Mark phase: find live values
        self.mark_live_values(func);

        // Sweep phase: remove dead instructions
        for block in &mut func.blocks {
            block.instructions.retain(|instr| {
                match instr {
                    Instruction::Assign(dest, _) => {
                        self.live_values.contains(dest)
                    }
                    _ => true,  // Keep side-effecting instructions
                }
            });
        }
    }

    fn mark_live_values(&mut self, func: &Function) {
        // Start from function return and work backwards
        let mut worklist = vec![];

        // Add all uses in terminators
        for block in &func.blocks {
            worklist.extend(block.terminator.uses());
        }

        // Iterate until fixed point
        while let Some(reg) = worklist.pop() {
            if self.live_values.insert(reg) {
                // Find definition and add its uses
                if let Some(def) = func.find_definition(reg) {
                    worklist.extend(def.uses());
                }
            }
        }
    }
}
```

## Testing Infrastructure

### Unit Tests

Every crate has comprehensive unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_integers() {
        let mut lexer = Lexer::new("123 456 0x10 0b101");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].kind, TokenKind::IntLiteral);
        assert_eq!(tokens[0].text, "123");
    }

    #[test]
    fn test_parser_binary_expr() {
        let mut parser = Parser::new("1 + 2 * 3");
        let expr = parser.parse_expr().unwrap();

        // Should parse as: 1 + (2 * 3)
        assert!(matches!(expr.kind, ExprKind::Binary(BinaryOp::Add, _, _)));
    }
}
```

### Integration Tests

End-to-end tests compile complete programs:

```rust
#[test]
fn test_compile_hello_world() {
    let source = r#"
        fn main() {
            println("Hello, World!");
        }
    "#;

    let result = compile_to_executable(source);
    assert!(result.is_ok());

    let output = run_executable(result.unwrap());
    assert_eq!(output, "Hello, World!\n");
}
```

### Differential Testing

Compare Aurora output against C compiler:

```rust
#[test]
fn test_differential_arithmetic() {
    let aurora_code = r#"
        fn compute(x: i32, y: i32) -> i32 {
            (x + y) * (x - y)
        }
    "#;

    let c_code = r#"
        int compute(int x, int y) {
            return (x + y) * (x - y);
        }
    "#;

    for (x, y) in [(1, 2), (5, 3), (-1, 4)] {
        let aurora_result = run_aurora(aurora_code, x, y);
        let c_result = run_c(c_code, x, y);
        assert_eq!(aurora_result, c_result);
    }
}
```

## Contributing

### Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Run clippy and fix warnings (`cargo clippy`)
- Add tests for new features
- Update documentation

### Pull Request Process

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Make changes and add tests
4. Ensure all tests pass (`cargo test --workspace`)
5. Format code (`cargo fmt`)
6. Run linter (`cargo clippy`)
7. Commit changes (`git commit -m 'Add amazing feature'`)
8. Push to branch (`git push origin feature/amazing-feature`)
9. Open Pull Request

### Performance Testing

Before submitting optimizations:

```bash
# Run benchmarks
cargo bench

# Check for regressions
aurora bench --baseline main --compare feature-branch

# Profile with flamegraph
cargo flamegraph --bench my_bench
```

### Security Considerations

- All dependencies must have signatures
- SBOM generated for every release
- No unsafe code without justification
- Security policy enforced in CI

## Additional Resources

- [Architecture Overview](architecture.md)
- [Language Reference](language-reference.md)
- [API Documentation](https://docs.rs/aurora)
- [GitHub Repository](https://github.com/aurora-lang/aurora)

---

Last updated: 2025-11-08
