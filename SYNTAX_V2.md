# Aurora Syntax v2.0 - Simpler Than Python

## Design Philosophy

Aurora v2 syntax is designed to be **the simplest possible syntax** while maintaining power:
- ✅ Shorter keywords than Python
- ✅ No type annotations required (full inference)
- ✅ Indentation-based (like Python)
- ✅ No semicolons
- ✅ No braces
- ✅ Minimal punctuation
- ✅ Natural language flow

## Comparison

### Python
```python
def add(a, b):
    return a + b

def main():
    print("Hello, World!")
    x = 10
    y = 5
    result = add(x, y)
    print(f"Result: {result}")

if __name__ == "__main__":
    main()
```

### Aurora v2 (SIMPLER!)
```aurora
fun add(a, b)
    return a + b

fun main()
    print "Hello, World!"
    x = 10
    y = 5
    result = add(x, y)
    print "Result:", result

# No if __name__ == "__main__" needed - just runs main()
```

## Keywords Simplified

| Python | Aurora v1 (Rust-like) | Aurora v2 (Simple) |
|--------|----------------------|-------------------|
| `def` | `fn` | `fun` |
| `return` | `return` | `return` or `ret` |
| `if` | `if` | `if` |
| `elif` | `else if` | `elif` |
| `else` | `else` | `else` |
| `while` | `while` | `while` |
| `for` | `for` | `for` |
| `in` | `in` | `in` |
| `class` | `struct` | `type` |
| `True/False` | `true/false` | `yes/no` or `true/false` |
| `None` | `None` | `none` |
| `and/or/not` | `&&/\|\|/!` | `and/or/not` |
| `print()` | `println!()` | `print` (no parens needed!) |

## Complete Syntax

### 1. Functions

```aurora
# Simple function (no return type)
fun greet(name)
    print "Hello", name

# Function with return
fun add(a, b)
    return a + b

# One-liner (implicit return)
fun double(x) = x * 2

# Multiple returns
fun min_max(a, b)
    if a < b
        return a, b
    return b, a
```

### 2. Variables

```aurora
# Type inference (no annotations needed!)
x = 10              # int
name = "Alice"      # string
pi = 3.14          # float
active = true      # bool

# Mutable (optional `var` keyword)
var count = 0
count = count + 1

# Immutable (default)
const MAX = 100
```

### 3. Control Flow

```aurora
# If/elif/else
if x > 10
    print "Big"
elif x > 5
    print "Medium"
else
    print "Small"

# Inline if (ternary)
result = "even" if x % 2 == 0 else "odd"

# While loop
while count < 10
    print count
    count = count + 1

# For loop
for i in 1..10
    print i

for item in items
    print item

# Match (pattern matching)
match value
    1 -> print "One"
    2 -> print "Two"
    _ -> print "Other"
```

### 4. Data Types

```aurora
# Lists (no brackets needed for simple cases)
numbers = [1, 2, 3, 4, 5]
names = ["Alice", "Bob", "Charlie"]

# Dictionaries
person = {name: "Alice", age: 30, active: true}

# Tuples
point = (10, 20)

# Custom types
type Point
    x: int
    y: int

# Create instance (no `new` keyword)
p = Point(10, 20)
print p.x, p.y

# Enums
type Color
    | Red
    | Green
    | Blue

color = Color.Red
```

### 5. String Interpolation

```aurora
# Simple (like Python f-strings but easier)
name = "Alice"
age = 30
print "Name: {name}, Age: {age}"

# Multi-line strings (no triple quotes needed)
message = """
    Hello, World!
    This is Aurora.
    """
```

### 6. Print Statement (Special!)

```aurora
# No parentheses needed!
print "Hello"
print "Name:", name
print "Numbers:", 1, 2, 3

# With format
print "Result: {result}"

# Debug print
debug x, y, z
```

### 7. Comments

```aurora
# Single line comment

##
Multi-line comment
can span multiple lines
##
```

### 8. Modules

```aurora
# Import
use math
use file.read as read_file

# Define module
module utils
    fun helper()
        return 42
```

### 9. Error Handling

```aurora
# Try/catch (simplified)
try
    result = divide(10, 0)
catch err
    print "Error:", err

# Result type (optional)
fun divide(a, b)
    if b == 0
        return Error("Division by zero")
    return Ok(a / b)
```

### 10. Lambdas

```aurora
# Simple lambda
double = (x) => x * 2

# Multi-line lambda
process = (x) =>
    y = x * 2
    return y + 1

# Lambda with multiple args
add = (a, b) => a + b
```

## Hello World Programs

### Minimal
```aurora
print "Hello, World!"
```

### With function
```aurora
fun main()
    print "Hello, World!"
```

### Complete
```aurora
fun greet(name)
    print "Hello, {name}!"

fun main()
    greet("World")
    greet("Aurora")
```

## Examples

### 1. FizzBuzz
```aurora
for i in 1..100
    if i % 15 == 0
        print "FizzBuzz"
    elif i % 3 == 0
        print "Fizz"
    elif i % 5 == 0
        print "Buzz"
    else
        print i
```

### 2. Factorial
```aurora
fun factorial(n)
    if n <= 1
        return 1
    return n * factorial(n - 1)

print factorial(5)  # Output: 120
```

### 3. Web Server (Future)
```aurora
use http

fun handler(request)
    return Response(
        status: 200,
        body: "Hello, World!"
    )

server = http.Server(port: 8000)
server.run(handler)
```

## Key Differences from Python

1. **No colons** - Indentation is enough
2. **`fun` instead of `def`** - Shorter!
3. **`print` is a statement** - No parentheses needed
4. **Type inference** - Never write types
5. **Simpler keywords** - `yes/no` instead of `True/False` (optional)
6. **No `self`** - Methods automatically bound
7. **Better string interpolation** - `{var}` everywhere
8. **Pattern matching** - Built-in `match`
9. **Result types** - Better error handling
10. **No `__init__` or `__main__`** - Just works!

## Implementation Plan

1. Update lexer for new keywords (`fun`, `print` as statement, etc.)
2. Update parser for indentation-based syntax
3. Update examples to new syntax
4. Wire remaining integration
5. Test hello_world.ax compilation

This is **the simplest syntax possible** while being more powerful than Python!
