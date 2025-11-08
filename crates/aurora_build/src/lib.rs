//! aurora_build - Build System and CLI
//!
//! Implements the Aurora build tool, workspace management, and build profiles.
//!
//! # Example
//!
//! ```
//! use aurora_build::workspace::{Manifest, PackageMetadata};
//! use aurora_build::cli::{BuildArgs, Profile};
//!
//! // Create build arguments
//! let args = BuildArgs {
//!     profile: Profile::Release,
//!     ..Default::default()
//! };
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

/// Command-line interface
pub mod cli;

/// Workspace and package management
pub mod workspace;

// Re-export main types
pub use cli::{BuildArgs, Command, Profile, TestArgs};
pub use workspace::{BuildCache, BuildError, Dependency, Manifest, PackageMetadata, Result};
