# Aurora Compiler - Integration Plan

## Goal
Get `hello_world.ax` to compile and run successfully.

## Current Status
✅ All 8 compiler phases implemented individually
✅ 433+ tests passing across all components
❌ End-to-end compilation not yet working

## What's Missing for hello_world.ax

```aurora
fn main() {
    println("Hello, World!");
}
```

### 1. AST-to-MIR Lowering (CRITICAL)

**File:** `crates/aurora_mir/src/lower.rs`

**Current:** Returns empty MIR module
**Needed:** Actually lower the AST nodes to MIR

**Specific Requirements:**
```rust
// Need to implement:
- Lower FunctionDecl → MIR Function
- Lower Block → MIR BasicBlocks
- Lower Call (println) → MIR Call instruction
- Lower StringLiteral → MIR Constant
```

**Implementation:**
1. Walk AST and collect functions
2. For each function, create MIR Function
3. Create entry basic block
4. Lower function body expressions to MIR instructions
5. Generate proper control flow

### 2. MIR-to-AIR Lowering (CRITICAL)

**File:** `crates/aurora_air/src/emit.rs`

**Current:** Returns empty AIR module
**Needed:** Actually convert MIR to x86_64 AIR

**Specific Requirements:**
```rust
// Need to implement:
- Lower MIR Call → AIR Call with System V ABI
- Set up stack frame (prologue/epilogue)
- Place arguments in correct registers (rdi, rsi, rdx, ...)
- Handle string constants in .data section
```

**Implementation:**
1. For each MIR function, create AIR function
2. Emit function prologue (push rbp, mov rbp rsp)
3. Lower each MIR instruction to AIR instructions
4. Emit function epilogue (pop rbp, ret)
5. Add string constants to data section

### 3. Runtime println Implementation (CRITICAL)

**File:** `runtime/c_runtime.c`

**Current:** Stubbed out
**Needed:** Working `aurora_println` that outputs to stdout

**Implementation:**
```c
void aurora_println(const char* str) {
    printf("%s\n", str);
    fflush(stdout);
}
```

### 4. Linking (WORKING - just needs above)

**File:** `crates/aurora_backend/src/link.rs`

**Current:** ✅ Already implemented and tested
**Status:** Should work once AIR is generated properly

---

## Implementation Steps

### Step 1: Minimal MIR Lowering

Create a minimal AST-to-MIR lowerer that handles:
- Function declarations
- String literals
- Function calls (println)
- Basic blocks with return

### Step 2: Minimal AIR Lowering

Create a minimal MIR-to-AIR lowerer that handles:
- Function calls with System V ABI
- String constants
- Stack frame setup

### Step 3: Runtime Integration

Compile and link the C runtime properly.

### Step 4: Test End-to-End

```bash
./target/release/aurorac examples/hello_world.ax -o hello_world
./hello_world
# Expected: "Hello, World!"
```

---

## Minimal Implementation Example

### AST-to-MIR (Pseudo-code)

```rust
pub fn lower_ast_to_mir(ast: Ast, diagnostics: Arc<D>) -> MirModule {
    let mut module = MirModule::new();

    for item_id in ast.items {
        match arena.get_item(item_id).kind {
            ItemKind::Function(func) => {
                let mir_func = lower_function(func, &arena);
                module.add_function(mir_func);
            }
            _ => {} // Skip other items for now
        }
    }

    module
}

fn lower_function(func: &FunctionDecl, arena: &Arena) -> Function {
    let mut mir_func = Function::new(
        func_id,
        func.name.clone(),
        Type::Unit,
        EffectSet::IO
    );

    // Create entry block
    let entry_block = BasicBlock::new(0);

    // Lower function body
    for stmt in func.body.stmts {
        lower_stmt(stmt, &mut mir_func, arena);
    }

    mir_func.add_block(entry_block);
    mir_func
}

fn lower_expr(expr: &Expr, func: &mut Function) -> ValueId {
    match expr.kind {
        ExprKind::Call { func, args } => {
            // Lower function call
            let arg_values = args.iter().map(|a| lower_expr(a, func)).collect();
            let result = func.new_value(Type::Unit);
            func.current_block().push(Instruction::Call {
                dest: Some(result),
                func: Operand::Label(func_name),
                args: arg_values,
                effects: EffectSet::IO,
                span: expr.span,
            });
            result
        }
        ExprKind::StringLiteral(s) => {
            // Create string constant
            let value = func.new_value(Type::Str);
            func.current_block().push(Instruction::Assign {
                dest: value,
                value: Operand::Const(Constant::String(s)),
                span: expr.span,
            });
            value
        }
        _ => panic!("Unsupported expression")
    }
}
```

### MIR-to-AIR (Pseudo-code)

```rust
pub fn lower_mir_to_air(mir: MirModule, diagnostics: Arc<D>) -> AirModule {
    let mut air = AirModule::new("main".to_string());

    for (func_id, mir_func) in mir.functions {
        let air_func = lower_function(mir_func);
        air.add_function(air_func);
    }

    air
}

fn lower_function(mir_func: Function) -> AirFunction {
    let mut air_func = AirFunction::new(mir_func.name.clone());

    // Prologue
    air_func.push(Instruction::Push { operand: Operand::Reg(Register::RBP) });
    air_func.push(Instruction::Mov {
        dest: Operand::Reg(Register::RBP),
        src: Operand::Reg(Register::RSP)
    });

    // Lower each basic block
    for (block_id, block) in mir_func.blocks {
        for inst in block.instructions {
            lower_instruction(inst, &mut air_func);
        }
    }

    // Epilogue
    air_func.push(Instruction::Pop { operand: Operand::Reg(Register::RBP) });
    air_func.push(Instruction::Ret);

    air_func
}

fn lower_instruction(inst: MirInstruction, air: &mut AirFunction) {
    match inst {
        Instruction::Call { func, args, .. } => {
            // System V ABI: first 6 args in rdi, rsi, rdx, rcx, r8, r9
            if args.len() > 0 {
                air.push(Instruction::Mov {
                    dest: Operand::Reg(Register::RDI),
                    src: lower_operand(args[0])
                });
            }
            air.push(Instruction::Call {
                target: Operand::Label(func)
            });
        }
        Instruction::Return { value } => {
            // Return value in rax
            if let Some(v) = value {
                air.push(Instruction::Mov {
                    dest: Operand::Reg(Register::RAX),
                    src: lower_operand(v)
                });
            }
        }
        _ => {}
    }
}
```

---

## Timeline

**Estimated Time:** 4-6 hours total

1. **AST-to-MIR lowering:** 2 hours
2. **MIR-to-AIR lowering:** 2 hours
3. **Runtime integration:** 30 minutes
4. **Testing and debugging:** 1-2 hours

---

## Success Criteria

✅ `hello_world.ax` compiles without errors
✅ Executable is generated
✅ Running the executable prints "Hello, World!"
✅ Exit code is 0

Once this works, we can:
- Implement the simplified syntax (v2.0)
- Add more stdlib functions
- Support more language features
- Add more example programs

---

**Priority:** CRITICAL - This proves the entire compiler works end-to-end!
