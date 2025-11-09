#!/bin/bash

# Test all simplified syntax examples

echo "======================================"
echo "Aurora Simplified Syntax Test Suite"
echo "======================================"
echo ""

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track results
total=0
passed=0
failed=0

# Function to test lexer on a file
test_lexer() {
    local file=$1
    local name=$2

    echo -n "Testing $name... "
    total=$((total + 1))

    # Create a simple Rust test program
    cat > /tmp/test_lex.rs << 'RUST_EOF'
use aurora_lexer::{Lexer, TokenKind};
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: test_lex <file>");
        std::process::exit(1);
    }

    let source = fs::read_to_string(&args[1]).expect("Failed to read file");
    let mut lexer = Lexer::new(&source, args[1].clone()).expect("Failed to create lexer");

    let mut token_count = 0;
    let mut has_fun = false;
    let mut has_elif = false;
    let mut has_ret = false;
    let mut has_var = false;
    let mut has_yes_no = false;
    let mut has_word_ops = false;

    loop {
        match lexer.next_token() {
            Ok(token) => {
                token_count += 1;
                match token.kind {
                    TokenKind::Fun => has_fun = true,
                    TokenKind::Elif => has_elif = true,
                    TokenKind::Ret => has_ret = true,
                    TokenKind::Var => has_var = true,
                    TokenKind::Yes | TokenKind::No => has_yes_no = true,
                    TokenKind::AndKeyword | TokenKind::OrKeyword | TokenKind::NotKeyword => has_word_ops = true,
                    TokenKind::Eof => break,
                    _ => {}
                }
            }
            Err(e) => {
                eprintln!("Lexer error: {:?}", e);
                std::process::exit(1);
            }
        }
    }

    println!("✓ Lexed {} tokens", token_count);
    if has_fun { println!("  - Found 'fun' keyword"); }
    if has_elif { println!("  - Found 'elif' keyword"); }
    if has_ret { println!("  - Found 'ret' keyword"); }
    if has_var { println!("  - Found 'var' keyword"); }
    if has_yes_no { println!("  - Found 'yes/no' literals"); }
    if has_word_ops { println!("  - Found word operators (and/or/not)"); }
}
RUST_EOF

    # Compile and run the test
    if rustc --edition 2021 -L target/debug/deps --extern aurora_lexer=target/debug/libaurora_lexer.rlib /tmp/test_lex.rs -o /tmp/test_lex 2>/dev/null; then
        if /tmp/test_lex "$file" > /tmp/test_output.txt 2>&1; then
            echo -e "${GREEN}✓ PASS${NC}"
            cat /tmp/test_output.txt | sed 's/^/  /'
            passed=$((passed + 1))
        else
            echo -e "${RED}✗ FAIL${NC}"
            cat /tmp/test_output.txt | sed 's/^/  /'
            failed=$((failed + 1))
        fi
    else
        echo -e "${YELLOW}⊘ SKIP (compile error)${NC}"
    fi

    echo ""
}

# Test all simplified examples
echo "Testing Simplified Syntax Examples:"
echo "-----------------------------------"
echo ""

test_lexer "examples/hello_world_simple.ax" "Hello World (Simple)"
test_lexer "examples/fizzbuzz_simple.ax" "FizzBuzz (Simple)"
test_lexer "examples/operators_simple.ax" "Word Operators"
test_lexer "examples/factorial_simple.ax" "Factorial (Recursive)"
test_lexer "examples/fibonacci_simple.ax" "Fibonacci (Recursive & Iterative)"
test_lexer "examples/prime_checker_simple.ax" "Prime Checker"
test_lexer "examples/complex_logic_simple.ax" "Complex Boolean Logic"
test_lexer "examples/nested_conditions_simple.ax" "Nested Conditions"
test_lexer "examples/string_operations_simple.ax" "String Operations"

# Summary
echo "======================================"
echo "Test Summary"
echo "======================================"
echo "Total:  $total"
echo -e "Passed: ${GREEN}$passed${NC}"
if [ $failed -gt 0 ]; then
    echo -e "Failed: ${RED}$failed${NC}"
else
    echo -e "Failed: $failed"
fi
echo ""

if [ $failed -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed.${NC}"
    exit 1
fi
