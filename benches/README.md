# Aurora Performance Benchmarks

This directory contains performance benchmarks for the Aurora compiler and runtime.

## Benchmark Categories

### Compiler Benchmarks
- **Lexer**: Tokenization throughput (MB/s)
- **Parser**: Parse time for various code sizes
- **Type Checker**: Inference and checking performance
- **Codegen**: AIR emission and optimization time
- **End-to-End**: Full compilation pipeline

### Runtime Benchmarks
- **Compute Kernels**: FFT, GEMM, sorting, compression
- **I/O Workloads**: File I/O, network operations
- **Concurrency**: Goroutines, channels, async/await performance

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench --workspace

# Run specific benchmark suite
cargo bench --package aurora_lexer

# Generate flamegraphs (requires cargo-flamegraph)
cargo flamegraph --bench lexer_bench
```

## Performance Targets

- **Lexer**: ≥100 MB/s on x86_64
- **Dev Builds**: Within 1.2× of Go compile speed
- **Compute Kernels**: ≤1.10× of optimized C
- **I/O Workloads**: ≤1.25× of optimized C
