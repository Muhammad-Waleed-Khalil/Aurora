//! aurora_parser - Aurora Parser
//!
//! This crate implements the Aurora parser, which transforms token streams
//! from the lexer into Abstract Syntax Trees (AST).
//!
//! # Architecture
//!
//! The parser uses a hybrid approach:
//! - **LL parsing** for top-level declarations and statements
//! - **Pratt parsing** for expressions with precedence climbing
//!
//! # Parser Structure
//!
//! - `parser`: Core parser implementation
//! - `decls`: Declaration parsing (functions, types, traits, impls)
//! - `exprs`: Expression parsing (Pratt parser)
//! - `stmts`: Statement parsing
//! - `types`: Type parsing
//! - `patterns`: Pattern parsing
//! - `error`: Error types and recovery

#![warn(missing_docs)]
#![warn(clippy::all)]

mod decls;
mod error;
mod exprs;
mod parser;
mod patterns;
mod stmts;
mod types;

pub use error::{ParseError, ParseResult};
pub use parser::Parser;
