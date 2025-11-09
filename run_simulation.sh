#!/bin/bash

# Simulate program outputs based on code logic

echo "======================================"
echo "Aurora Example Programs - Simulated Outputs"
echo "======================================"
echo ""

# 1. Hello World
echo "1. HELLO WORLD (SIMPLE)"
echo "$ ./hello_world_simple"
echo "Hello, World!"
echo "Welcome to Aurora - Simpler than Python!"
echo ""

# 2. FizzBuzz (first 20 numbers)
echo "2. FIZZBUZZ (First 20 numbers)"
echo "$ ./fizzbuzz_simple | head -20"
for i in $(seq 1 20); do
    if [ $((i % 15)) -eq 0 ]; then
        echo "FizzBuzz"
    elif [ $((i % 3)) -eq 0 ]; then
        echo "Fizz"
    elif [ $((i % 5)) -eq 0 ]; then
        echo "Buzz"
    else
        echo "$i"
    fi
done
echo "..."
echo ""

# 3. Operators
echo "3. WORD-BASED OPERATORS"
echo "$ ./operators_simple"
echo "Both conditions are true"
echo "At least one is true"
echo "inactive is false"
echo ""

# 4. Factorial
echo "4. FACTORIAL (RECURSIVE)"
echo "$ ./factorial_simple"
echo "Factorial of 5 is:"
echo "120"
echo "Factorial of 10 is:"
echo "3628800"
echo ""

# 5. Fibonacci (first 11 numbers)
echo "5. FIBONACCI (BOTH APPROACHES)"
echo "$ ./fibonacci_simple"
echo "Fibonacci (recursive):"
for i in 0 1 1 2 3 5 8 13 21 34 55; do
    echo "$i"
done
echo "Fibonacci (iterative):"
for i in 0 1 1 2 3 5 8 13 21 34 55; do
    echo "$i"
done
echo ""

# 6. Prime Checker
echo "6. PRIME CHECKER"
echo "$ ./prime_checker_simple"
echo "Prime numbers up to 50:"
echo "2 3 5 7 11 13 17 19 23 29 31 37 41 43 47" | tr ' ' '\n'
echo ""

# 7. Complex Logic
echo "7. COMPLEX BOOLEAN LOGIC"
echo "$ ./complex_logic_simple"
echo "Access granted!"
echo "Adult student"
echo "Teen user with benefits"
echo "Senior user"
echo ""

# 8. Nested Conditions
echo "8. NESTED CONDITIONS (STRESS TEST)"
echo "$ ./nested_conditions_simple"
echo "Very large negative"
echo "Large negative"
echo "Small negative"
echo "Zero"
echo "One"
echo "Two"
echo "Small positive"
echo "Round number"
echo "Round number"
echo "Large positive"
echo "Very large positive"
echo ""

# 9. String Operations
echo "9. STRING OPERATIONS"
echo "$ ./string_operations_simple"
echo "Good morning"
echo "Alice"
echo "Good afternoon"
echo "Alice"
echo "Good evening"
echo "Alice"
echo ""

echo "======================================"
echo "All simulations complete!"
echo "======================================"
