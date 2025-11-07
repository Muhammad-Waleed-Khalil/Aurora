//! aurora_grammar - Aurora Grammar Agent
//!
//! This crate defines the complete Aurora grammar, precedence table,
//! associativity rules, and conflict analysis.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod precedence;
pub mod grammar;
pub mod conflicts;

pub use precedence::{Associativity, Precedence, PrecedenceEntry, PrecedenceTable};
pub use grammar::{AuroraGrammar, GrammarRule, Production, Symbol};
pub use conflicts::{ConflictAnalyzer, ConflictReport, ConflictType};
