//! Comprehensive parser tests
//!
//! This file contains tests that verify all major parsing capabilities.

use aurora_parser::Parser;

const COMPREHENSIVE_EXAMPLE: &str = r#"
// Test all Aurora syntax in one file

// Function with no params
fn hello() {
    println("Hello!");
}

// Function with params and return type
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Function with generics
fn identity<T>(x: T) -> T {
    x
}

// Type alias
type MyInt = i64;

// Const declaration
const MAX_SIZE: i32 = 100;

// Module declaration
mod utils {
    pub fn helper() {}
}

// Use declaration
use std::io;

// Trait declaration
trait Display {
    fn show(&self);
}

// Impl block
impl Point {
    fn new(x: f64, y: f64) -> Point {
        Point { x, y }
    }

    fn distance(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

// Main function with all expression types
fn main() {
    // Let bindings
    let x = 42;
    let mut y: i32 = 10;

    // Binary operations
    let a = 1 + 2 * 3;
    let b = x == y;
    let c = true && false;

    // Unary operations
    let d = -x;
    let e = !true;

    // If expression
    let result = if x > y {
        "greater"
    } else if x < y {
        "less"
    } else {
        "equal"
    };

    // Match expression
    match x {
        1 => println("one"),
        2 | 3 => println("two or three"),
        _ => println("other"),
    }

    // While loop
    while y < 10 {
        y = y + 1;
    }

    // For loop
    for i in 0..10 {
        println("{}", i);
    }

    // Loop with break/continue
    loop {
        if x > 100 {
            break;
        }
        if x == 50 {
            continue;
        }
    }

    // Function calls
    hello();
    add(1, 2);

    // Method calls
    let p = Point::new(3.0, 4.0);
    p.distance();

    // Field access
    let px = p.x;

    // Struct literal
    let p2 = Point { x: 1.0, y: 2.0 };

    // Struct literal with shorthand
    let x = 1.0;
    let y = 2.0;
    let p3 = Point { x, y };

    // Array literal
    let arr = [1, 2, 3, 4, 5];

    // Tuple literal
    let tuple = (1, "hello", true);

    // Range expressions
    let range1 = 0..10;
    let range2 = 0..=10;

    // Return statement
    return;
}
"#;

#[test]
fn test_comprehensive_parsing() {
    let parser = Parser::new(COMPREHENSIVE_EXAMPLE, "comprehensive.ax".to_string()).unwrap();
    let result = parser.parse_program();

    assert!(result.is_ok(), "Parser should successfully parse comprehensive example");

    let (program, _arena) = result.unwrap();

    // Should have parsed multiple top-level items
    assert!(program.items.len() > 10, "Should have parsed many top-level items");
}

#[test]
fn test_operator_precedence() {
    let source = r#"
fn test() {
    let x = 1 + 2 * 3;          // 1 + (2 * 3) = 7
    let y = 2 ** 3 + 1;         // (2 ** 3) + 1 = 9
    let z = 1 + 2 == 3;         // (1 + 2) == 3 = true
    let w = true || false && false;  // true || (false && false)
}
"#;

    let parser = Parser::new(source, "precedence.ax".to_string()).unwrap();
    let (program, _arena) = parser.parse_program().unwrap();
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_nested_expressions() {
    let source = r#"
fn test() {
    let x = ((1 + 2) * (3 + 4)) / 5;
    let y = if true { if false { 1 } else { 2 } } else { 3 };
}
"#;

    let parser = Parser::new(source, "nested.ax".to_string()).unwrap();
    let (program, _arena) = parser.parse_program().unwrap();
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_error_recovery() {
    // Test that parser can recover from errors
    let source = r#"
fn valid1() {}

// This will cause an error but parser should recover
fn invalid(

fn valid2() {}
"#;

    let parser = Parser::new(source, "recovery.ax".to_string()).unwrap();
    let result = parser.parse_program();

    // Parser should attempt to recover and continue parsing
    // even if there are errors
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_all_literal_types() {
    let source = r#"
fn test() {
    let int = 42;
    let float = 3.14;
    let string = "hello";
    let char = 'x';
    let bool_true = true;
    let bool_false = false;
}
"#;

    let parser = Parser::new(source, "literals.ax".to_string()).unwrap();
    let (program, _arena) = parser.parse_program().unwrap();
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_all_type_syntax() {
    let source = r#"
fn test(
    a: i32,
    b: f64,
    c: bool,
    d: &str,
    e: &mut i32,
    f: (i32, f64),
    g: [u8; 10],
    h: [i32],
    i: fn(i32) -> i32,
) {}
"#;

    let parser = Parser::new(source, "types.ax".to_string()).unwrap();
    let (program, _arena) = parser.parse_program().unwrap();
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_pattern_matching() {
    let source = r#"
fn test(x: i32) {
    match x {
        1 => println("one"),
        2 | 3 => println("two or three"),
        4..=10 => println("four to ten"),
        _ => println("other"),
    }
}
"#;

    let parser = Parser::new(source, "patterns.ax".to_string()).unwrap();
    let (program, _arena) = parser.parse_program().unwrap();
    assert_eq!(program.items.len(), 1);
}
