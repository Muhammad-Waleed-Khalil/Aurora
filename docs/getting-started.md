# Getting Started with Aurora

This guide will help you install Aurora, write your first program, and understand the basic workflow.

## Table of Contents

1. [Installation](#installation)
2. [Your First Program](#your-first-program)
3. [Project Structure](#project-structure)
4. [Build System](#build-system)
5. [Common Commands](#common-commands)
6. [Editor Setup](#editor-setup)
7. [Next Steps](#next-steps)

## Installation

### Prerequisites

Aurora requires:
- Rust 1.75 or later (for building from source)
- LLVM 17 or later (optional, for LLVM backend)
- GCC or Clang (for linking)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/aurora-lang/aurora.git
cd aurora

# Build the compiler
cargo build --release

# Add to PATH (optional)
export PATH="$PWD/target/release:$PATH"

# Verify installation
aurorac --version
```

### Install via Package Manager

```bash
# Coming soon
# curl -sSf https://install.aurora-lang.org | sh
```

## Your First Program

### Hello World

Create a file named `hello.ax`:

```aurora
// hello.ax
fn main() {
    println("Hello, Aurora!");
}
```

Compile and run:

```bash
# Compile
aurorac hello.ax -o hello

# Run
./hello
```

Output:
```
Hello, Aurora!
```

### Interactive REPL (Coming Soon)

```bash
aurora repl
>>> println("Hello from REPL!")
Hello from REPL!
```

## Project Structure

### Creating a New Project

```bash
# Create new project
aurora init my_project
cd my_project
```

This creates:
```
my_project/
├── Aurora.toml        # Project manifest
├── src/
│   └── main.ax       # Entry point
├── tests/
│   └── test.ax       # Tests
└── bench/
    └── bench.ax      # Benchmarks
```

### Aurora.toml

The project manifest file:

```toml
[package]
name = "my_project"
version = "0.1.0"
authors = ["Your Name <you@example.com>"]
edition = "2025"
description = "My awesome Aurora project"
license = "MIT"

[dependencies]
# Example dependency
# http = "1.0.0"

[dev-dependencies]
# Test-only dependencies

[profile.release]
opt_level = 3
debug = false
lto = true
```

## Build System

### Build Profiles

Aurora supports multiple build profiles:

#### Debug Profile (default)
```bash
aurora build
```

- No optimizations (`opt_level = 0`)
- Debug symbols included
- Fast compilation
- Use for development

#### Release Profile
```bash
aurora build --release
```

- Maximum optimizations (`opt_level = 3`)
- No debug symbols
- LTO enabled
- Use for production

#### Custom Profile
```toml
[profile.fast-debug]
opt_level = 1
debug = true
lto = false
```

```bash
aurora build --profile fast-debug
```

### Compilation Targets

#### Native Target
```bash
aurora build
```

#### Cross-Compilation
```bash
# Linux to Windows
aurora cross --target x86_64-pc-windows-gnu

# Linux to macOS
aurora cross --target x86_64-apple-darwin

# Native to WebAssembly
aurora cross --target wasm32-wasi
```

### Incremental Compilation

Aurora uses content-addressed caching for fast incremental builds:

```bash
# First build: compiles everything
aurora build
# Time: ~1000ms

# Second build: only recompiles changes
aurora build
# Time: ~10ms
```

## Common Commands

### Build Commands

```bash
# Build project
aurora build

# Build in release mode
aurora build --release

# Build with specific target
aurora build --target x86_64-unknown-linux-gnu

# Build with multiple jobs
aurora build -j 8

# Clean build artifacts
aurora clean
```

### Run Commands

```bash
# Build and run
aurora run

# Run with arguments
aurora run -- arg1 arg2

# Run with release profile
aurora run --release
```

### Test Commands

```bash
# Run all tests
aurora test

# Run specific test
aurora test test_name

# Run with filter
aurora test integration

# Run in parallel (default)
aurora test

# Run with specific thread count
aurora test --test-threads 4
```

### Benchmark Commands

```bash
# Run all benchmarks
aurora bench

# Run specific benchmark
aurora bench bench_name

# Generate flamegraph
aurora bench --profile
```

### Code Quality Commands

```bash
# Format code
aurora fmt

# Check formatting without writing
aurora fmt --check

# Run linter
aurora lint

# Auto-fix lint issues
aurora lint --fix
```

### Documentation Commands

```bash
# Generate documentation
aurora doc

# Generate and open in browser
aurora doc --open

# Document private items
aurora doc --document-private-items
```

### Dependency Management

```bash
# Add dependency
aurora add http

# Add dev dependency
aurora add --dev testing

# Update dependencies
aurora update

# Remove dependency
aurora remove http
```

## Editor Setup

### Visual Studio Code

Install the Aurora extension:
```bash
# Coming soon
code --install-extension aurora-lang.aurora-vscode
```

Features:
- Syntax highlighting
- IntelliSense (autocomplete)
- Go to definition
- Find references
- Code actions (fix-its)
- Inline errors and warnings

### Vim/Neovim

Install via vim-plug:
```vim
Plug 'aurora-lang/aurora.vim'
```

### Emacs

```elisp
(use-package aurora-mode
  :ensure t)
```

### LSP Configuration

Aurora includes a built-in language server:

```bash
# Start LSP server
aurora lsp
```

Configuration example (VS Code):
```json
{
  "aurora.lsp.enable": true,
  "aurora.lsp.diagnostics": true,
  "aurora.lsp.completions": true,
  "aurora.lsp.hover": true
}
```

## Project Examples

### Simple Library

```aurora
// src/lib.ax
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}
```

### With Tests

```aurora
// tests/test.ax
use my_project::{add, multiply};

#[test]
fn test_add() {
    assert_eq(add(2, 3), 5);
}

#[test]
fn test_multiply() {
    assert_eq(multiply(2, 3), 6);
}
```

Run tests:
```bash
aurora test
```

### With Benchmarks

```aurora
// bench/bench.ax
use my_project::multiply;

#[bench]
fn bench_multiply(b: &mut Bencher) {
    b.iter(|| {
        multiply(123, 456)
    });
}
```

Run benchmarks:
```bash
aurora bench
```

### Error Handling

```aurora
fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err("Division by zero")
    } else {
        Ok(a / b)
    }
}

fn main() {
    match divide(10, 2) {
        Ok(result) => println("Result: {}", result),
        Err(e) => println("Error: {}", e),
    }
}
```

### Option Types (Null Safety)

```aurora
fn find_user(id: i32) -> Option<User> {
    if id == 1 {
        Some(User { name: "Alice", id: 1 })
    } else {
        None
    }
}

fn main() {
    match find_user(1) {
        Some(user) => println("Found: {}", user.name),
        None => println("User not found"),
    }
}
```

### Concurrency with Goroutines

```aurora
use aurora::sync::{spawn, channel};

fn main() {
    let (tx, rx) = channel::<i32>();

    spawn(move || {
        tx.send(42);
    });

    let value = rx.recv();
    println("Received: {}", value);
}
```

### Async/Await

```aurora
async fn fetch_data(url: &str) -> Result<String, Error> {
    let response = http::get(url).await?;
    Ok(response.text().await?)
}

async fn main() {
    match fetch_data("https://api.example.com").await {
        Ok(data) => println("Data: {}", data),
        Err(e) => println("Error: {}", e),
    }
}
```

## Performance Tips

### 1. Use Release Mode for Benchmarks
```bash
aurora bench --release
```

### 2. Enable CPU-Specific Optimizations
```toml
[profile.release]
cpu_profile = "skylake"  # or "zen3", "apple-silicon"
```

### 3. Profile Your Code
```bash
# Generate flamegraph
aurora build --profile
./target/release/my_app

# View profile
firefox flamegraph.svg
```

### 4. Use Inline for Hot Functions
```aurora
#[inline]
fn hot_function() {
    // frequently called code
}
```

### 5. Prefer Stack Allocation
```aurora
// Good: stack allocated
let array = [0; 1000];

// Slower: heap allocated
let vec = Vec::with_capacity(1000);
```

## Debugging

### With GDB
```bash
aurora build --debug
gdb ./target/debug/my_app
```

### With LLDB
```bash
aurora build --debug
lldb ./target/debug/my_app
```

### Print Debugging
```aurora
fn main() {
    let x = 42;
    dbg!(x);  // Prints: [main.ax:3] x = 42
}
```

### Backtrace on Panic
```bash
AURORA_BACKTRACE=1 ./my_app
```

## Security Best Practices

### 1. Use Strict Security Policy

```toml
[security]
allow_unsafe = false
allow_dynamic = false
require_signatures = true
allowed_sources = ["https://pkg.aurora-lang.org"]
```

### 2. Verify Dependencies

```bash
# Check SBOM
aurora sbom

# Verify signatures
aurora verify
```

### 3. Audit Dependencies

```bash
# Show dependency tree
aurora tree

# Check for vulnerabilities
aurora audit
```

## Troubleshooting

### Common Issues

#### 1. Compilation Errors

```
error[E0001]: undefined variable `x`
  --> src/main.ax:5:12
   |
 5 |     println(x);
   |            ^ undefined variable
   |
   = help: consider declaring the variable
```

**Solution**: Declare the variable before use.

#### 2. Type Errors

```
error[E0002]: type mismatch
  --> src/main.ax:3:9
   |
 3 |     let x: i32 = "hello";
   |         ^       ^^^^^^^^ expected i32, found str
```

**Solution**: Ensure types match or use explicit conversion.

#### 3. Borrow Checker Errors (Strict Mode)

```
error[E0003]: cannot borrow `x` as mutable
  --> src/main.ax:4:5
   |
 4 |     modify(&mut x);
   |     ^^^^^^^^^^^^^^ cannot borrow as mutable
```

**Solution**: Add `mut` to the variable declaration or switch to advisory mode.

### Getting Help

- Documentation: https://docs.aurora-lang.org
- GitHub Issues: https://github.com/aurora-lang/aurora/issues
- Discord: https://discord.gg/aurora-lang
- Forum: https://forum.aurora-lang.org

## Next Steps

Now that you have Aurora installed and understand the basics:

1. Read the [Language Reference](language-reference.md) for complete syntax
2. Explore the [Standard Library](standard-library.md) documentation
3. Learn about [Concurrency](concurrency-guide.md) with goroutines and channels
4. Understand [FFI](ffi-guide.md) for C/Python/Node.js interop
5. Optimize performance with the [Performance Guide](performance-guide.md)
6. Study [Compiler Internals](compiler-internals.md) if you're curious

Happy coding with Aurora! 🌟
