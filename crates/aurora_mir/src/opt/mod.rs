//! MIR Optimization Passes

pub mod inline;
pub mod sroa;
pub mod gvn;
pub mod licm;
pub mod dce;
pub mod nrvo;
pub mod devirt;
pub mod simd;

pub use inline::{InlineHeuristics, Inliner};
pub use sroa::SROA;
pub use gvn::GVN;
pub use licm::LICM;
pub use dce::DCE;
pub use nrvo::NRVO;
pub use devirt::Devirtualizer;
pub use simd::LoopVectorizer;
