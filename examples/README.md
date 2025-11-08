# Aurora Examples

This directory contains example programs demonstrating various Aurora language features.

## Basic Examples

### hello_world.ax
The traditional first program. Demonstrates basic println functionality.

```bash
aurora run examples/hello_world.ax
```

### variables.ax
Variable declaration, types, and basic operations with integers, floats, booleans, and strings.

```bash
aurora run examples/variables.ax
```

### functions.ax
Function declaration, parameters, return values, and recursion.

```bash
aurora run examples/functions.ax
```

### control_flow.ax
Control structures: if/else, match expressions, loops (while, for, loop), break, and continue.

```bash
aurora run examples/control_flow.ax
```

### structs.ax
Struct definition, instantiation, and methods. Demonstrates Point and Rectangle types.

```bash
aurora run examples/structs.ax
```

### enums.ax
Enum definition, pattern matching, and enum variants with data.

```bash
aurora run examples/enums.ax
```

### option_result.ax
Null-safety with Option<T> and error handling with Result<T, E>.

```bash
aurora run examples/option_result.ax
```

### arrays_slices.ax
Array operations, slicing, and iteration.

```bash
aurora run examples/arrays_slices.ax
```

### strings.ax
String manipulation, concatenation, formatting, and methods.

```bash
aurora run examples/strings.ax
```

### collections.ax
Dynamic collections: Vec, HashMap, and HashSet.

```bash
aurora run examples/collections.ax
```

## Compiling Examples

To compile an example without running:

```bash
aurora build examples/hello_world.ax -o hello
./hello
```

To compile with optimizations:

```bash
aurora build --release examples/hello_world.ax -o hello
./hello
```

## Testing Examples

To run all examples as tests:

```bash
aurora test examples/
```

## Learning Path

Recommended order for learning:

1. `hello_world.ax` - Start here
2. `variables.ax` - Learn about types
3. `functions.ax` - Functions and recursion
4. `control_flow.ax` - Control structures
5. `structs.ax` - Data structures
6. `enums.ax` - Algebraic data types
7. `option_result.ax` - Error handling
8. `arrays_slices.ax` - Working with sequences
9. `strings.ax` - String operations
10. `collections.ax` - Dynamic data structures

## Advanced Examples

More advanced examples will be added covering:
- Traits and generics
- Closures and iterators
- Concurrency (goroutines and channels)
- File I/O
- Networking
- FFI (Foreign Function Interface)
- Unsafe code
- Macros

## Contributing

To contribute a new example:

1. Create a new .ax file in this directory
2. Add comprehensive comments explaining the code
3. Update this README with a description
4. Test that the example compiles and runs correctly

## Questions?

See the main documentation at `/docs` or visit https://docs.aurora-lang.org
