//! Aurora MIR (Mid-Level Intermediate Representation)
//!
//! This crate provides the MIR for Aurora compiler:
//! - SSA form representation
//! - Control Flow Graph (CFG)
//! - Dominance tree computation
//! - MIR lowering from typed AST
//! - Optimization passes
//! - MIR dumps and serialization

pub mod cfg;
pub mod dump;
pub mod lower;
pub mod mir;
pub mod opt;

pub use cfg::{DominatorTree, Loop, CFG};
pub use dump::MirDumper;
pub use lower::MirBuilder;
pub use mir::*;
pub use opt::*;
