# Aurora Programming Language - Documentation

Welcome to the Aurora programming language documentation. Aurora is a modern, high-performance systems programming language with advanced features for safety, concurrency, and interoperability.

## Quick Links

- [Architecture Overview](architecture.md) - Complete compiler architecture and design
- [Getting Started Guide](getting-started.md) - Installation and first program
- [Language Reference](language-reference.md) - Complete language specification
- [Standard Library](standard-library.md) - Built-in types and functions
- [Compiler Internals](compiler-internals.md) - How the compiler works
- [Build System Guide](build-system.md) - Using the Aurora build tool
- [FFI Guide](ffi-guide.md) - Interoperability with C, Python, and Node.js
- [Concurrency Guide](concurrency-guide.md) - Goroutines, channels, and async/await
- [Performance Guide](performance-guide.md) - Optimization and profiling
- [Security Guide](security-guide.md) - Secure development practices

## About Aurora

Aurora is designed with the following goals:

### Safety First
- Null-safety by default with Option types
- Ownership and borrowing system (advisory mode available)
- Effects system for tracking side effects
- Strong static typing with Hindley-Milner type inference

### High Performance
- Zero-cost abstractions
- CPU-specific optimization profiles
- Multiple IR levels (MIR, AIR) for aggressive optimization
- Performance gates and regression detection

### Concurrency
- M:N work-stealing scheduler for goroutines
- CSP-style channels (buffered and unbuffered)
- Actor model with supervision
- Async/await with structured cancellation

### Interoperability
- C ABI with stable name mangling
- Python interop via HPy
- Node.js N-API support
- WebAssembly/WASI compilation

### Developer Experience
- LSP with completions, hover, and code actions
- Structured JSON diagnostics with fix-its
- Comprehensive error messages
- Fast incremental compilation

## File Extension

Aurora source files use the `.ax` extension (Aurora eXtension).

## Example

```aurora
// hello.ax - Your first Aurora program
fn main() {
    println("Hello, Aurora!");
}
```

## Community

- GitHub: https://github.com/aurora-lang/aurora
- Documentation: https://docs.aurora-lang.org
- Package Registry: https://pkg.aurora-lang.org

## License

Aurora is distributed under the MIT License.
