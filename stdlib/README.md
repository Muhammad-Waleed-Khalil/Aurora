# Aurora Standard Library

The Aurora Standard Library provides the fundamental building blocks for Aurora programs, including core types and I/O functionality.

## Status: Phase 1 Implementation (Re-export Facade)

This is the initial Phase 1 implementation of the Aurora stdlib. Currently, it re-exports Rust's standard library types to provide a stable API surface while the Aurora compiler is being developed. In Phase 2, these will be replaced with native Aurora implementations.

## Core Types

### `Option<T>` - Null Safety
Represents an optional value - either `Some(T)` or `None`.

```rust
use aurora_stdlib::prelude::*;

let x: Option<i32> = Some(42);
assert_eq!(x.unwrap(), 42);

let y: Option<i32> = None;
assert_eq!(y.unwrap_or(0), 0);
```

**Methods**: `is_some`, `is_none`, `unwrap`, `unwrap_or`, `unwrap_or_else`, `unwrap_or_default`, `map`, `and_then`, `or`, `or_else`, `filter`, `take`, `replace`, and more.

### `Result<T, E>` - Error Handling
Represents either success (`Ok(T)`) or failure (`Err(E)`).

```rust
use aurora_stdlib::prelude::*;

fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err(String::from("division by zero"))
    } else {
        Ok(a / b)
    }
}

match divide(10, 2) {
    Ok(result) => println!("Result: {}", result),
    Err(e) => eprintln!("Error: {}", e),
}
```

**Methods**: `is_ok`, `is_err`, `ok`, `err`, `unwrap`, `expect`, `unwrap_or`, `unwrap_or_else`, `map`, `map_err`, `and_then`, `or_else`, and more.

### `String` - UTF-8 Strings
Owned, heap-allocated, growable UTF-8 string type.

```rust
use aurora_stdlib::prelude::*;

let mut s = String::new();
s.push_str("Hello");
s.push(' ');
s.push_str("World");
assert_eq!(s.as_str(), "Hello World");
```

**Methods**: `new`, `with_capacity`, `from`, `push`, `push_str`, `len`, `is_empty`, `capacity`, `clear`, `contains`, `starts_with`, `ends_with`, and more.

## I/O Functions

### Output Functions

```rust
use aurora_stdlib::io::{print, println, eprint, eprintln};

// Standard output
print("Hello ");
println("World!");  // With newline

// Standard error
eprint("Error: ");
eprintln("Something went wrong!");
```

## Using the Prelude

The prelude imports all commonly-used types and functions:

```rust
use aurora_stdlib::prelude::*;

fn main() {
    let x = Some(42);
    println("The answer is {}", x.unwrap());
}
```

## Test Coverage

- **Unit Tests**: 26 tests covering all core functionality
- **Doc Tests**: 9 documentation examples
- **Coverage**: Option, Result, String, I/O functions
- **Status**: ✅ All tests passing

## Architecture

```
stdlib/
├── Cargo.toml          # Package configuration
├── README.md           # This file
└── src/
    ├── lib.rs          # Main library module + prelude
    ├── option.rs       # Option<T> type (re-export)
    ├── result.rs       # Result<T, E> type (re-export)
    ├── string.rs       # String type (re-export)
    └── io.rs           # I/O functions (print, println, etc.)
```

## Dependencies

- `aurora_runtime` - Runtime library for allocation and panic handling

## Future Plans (Phase 2+)

### Phase 2: Native Implementations
- Replace re-exports with native Aurora implementations
- `#![no_std]` compatible versions
- Custom allocator integration
- Zero-overhead abstractions

### Phase 3: Extended Library
- **Collections**: `Vec<T>`, `HashMap<K, V>`, `HashSet<T>`
- **Iterators**: Full iterator trait system
- **File I/O**: `File`, `Read`, `Write` traits
- **Formatting**: `format!` macro and Display trait
- **Traits**: Copy, Clone, Debug, Display, etc.

### Phase 4: Advanced Features
- **Concurrency**: Channels, sync primitives
- **Async**: Async/await support
- **Network**: TCP/UDP sockets
- **Process**: Command execution, process management

## Version History

### v0.1.0 (Current)
- ✅ Option<T> re-export with tests
- ✅ Result<T, E> re-export with tests
- ✅ String re-export with tests
- ✅ I/O functions: print, println, eprint, eprintln
- ✅ Prelude module for easy imports
- ✅ 26 unit tests + 9 doc tests
- ✅ Integrated into Aurora workspace

## Building

```bash
# Build stdlib
cargo build --package aurora_stdlib

# Run tests
cargo test --package aurora_stdlib

# Build with all workspace
cargo build --workspace
```

## Usage in Aurora Programs

```rust
// Import the prelude
use aurora_stdlib::prelude::*;

fn main() {
    // Option example
    let maybe_number = Some(42);
    match maybe_number {
        Some(n) => println!("Got number: {}", n),
        None => println!("No number"),
    }

    // Result example
    match divide(10, 2) {
        Ok(result) => println!("10 / 2 = {}", result),
        Err(e) => eprintln!("Error: {}", e),
    }

    // String example
    let mut greeting = String::from("Hello");
    greeting.push_str(", Aurora!");
    println!("{}", greeting);
}

fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err(String::from("Cannot divide by zero"))
    } else {
        Ok(a / b)
    }
}
```

## Contributing

This stdlib is part of the Aurora compiler project. Contributions should follow the Aurora development guidelines and maintain compatibility with the compiler's type system and effects system.

## License

MIT

---

**Implementation Status**: Phase 1 Complete ✅
**Next Steps**: Compiler driver integration, native implementations
**Test Status**: 26/26 unit tests passing, 9/9 doc tests passing
