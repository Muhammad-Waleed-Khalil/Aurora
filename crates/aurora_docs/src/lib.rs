//! aurora_docs - Documentation Generator for Aurora
//!
//! Generates API documentation, language reference, and guides.
//!
//! # Example
//!
//! ```
//! use aurora_docs::generator::{DocItem, DocKind, DocGenerator};
//!
//! // Create a doc item
//! let item = DocItem::new("add", DocKind::Function, "Adds two numbers")
//!     .with_example("add(1, 2) // Returns 3");
//!
//! // Create a generator
//! let mut gen = DocGenerator::new("/tmp/docs");
//! gen.add_item(item);
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

/// Documentation generator module
pub mod generator;

// Re-export main types
pub use generator::{DocError, DocGenerator, DocItem, DocKind, Result};
