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
//! - `exprs`: Expression parsing (Pratt parser) - TODO
//! - `stmts`: Statement parsing - TODO
//! - `types`: Type parsing - TODO
//! - `patterns`: Pattern parsing - TODO
//! - `error`: Error types and recovery

#![warn(missing_docs)]
#![warn(clippy::all)]

mod decls;
mod error;
mod parser;

pub use error::{ParseError, ParseResult};
pub use parser::Parser;
