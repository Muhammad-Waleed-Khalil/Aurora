//! Integration tests for Aurora parser
//!
//! These tests verify the parser can handle real Aurora code examples.

use aurora_parser::Parser;
use std::fs;

#[test]
fn test_parse_hello_world() {
    let source = fs::read_to_string("../../examples/hello_world.ax")
        .expect("Could not read hello_world.ax");
    let parser = Parser::new(&source, "hello_world.ax".to_string()).unwrap();
    let (program, _arena) = parser.parse_program().unwrap();
    assert!(!program.items.is_empty());
}

#[test]
fn test_parse_functions() {
    let source = fs::read_to_string("../../examples/functions.ax")
        .expect("Could not read functions.ax");
    let parser = Parser::new(&source, "functions.ax".to_string()).unwrap();
    let (program, _arena) = parser.parse_program().unwrap();
    assert!(!program.items.is_empty());
}

#[test]
fn test_parse_control_flow() {
    let source = fs::read_to_string("../../examples/control_flow.ax")
        .expect("Could not read control_flow.ax");
    let parser = Parser::new(&source, "control_flow.ax".to_string()).unwrap();
    let (program, _arena) = parser.parse_program().unwrap();
    assert!(!program.items.is_empty());
}

#[test]
fn test_parse_structs() {
    let source = fs::read_to_string("../../examples/structs.ax")
        .expect("Could not read structs.ax");
    let parser = Parser::new(&source, "structs.ax".to_string()).unwrap();
    let (program, _arena) = parser.parse_program().unwrap();
    assert!(!program.items.is_empty());
}
