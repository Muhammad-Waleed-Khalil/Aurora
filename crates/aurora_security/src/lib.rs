//! aurora_security - Supply Chain Security
//!
//! SBOM generation, signature verification, vendoring, and security policies.
//!
//! # Example
//!
//! ```
//! use aurora_security::sbom::{Sbom, Component, ComponentType};
//! use aurora_security::policy::SecurityPolicy;
//!
//! // Create SBOM
//! let mut sbom = Sbom::new("aurora".to_string(), "0.1.0".to_string());
//! let comp = Component::new(
//!     "dep1".to_string(),
//!     "1.0.0".to_string(),
//!     ComponentType::Library,
//! );
//! sbom.add_component(comp);
//!
//! // Create security policy
//! let policy = SecurityPolicy::strict();
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

/// Software Bill of Materials (SBOM)
pub mod sbom;

/// Security policy and enforcement
pub mod policy;

// Re-export main types
pub use policy::{SecurityPolicy, Signature, VerificationResult};
pub use sbom::{Component, ComponentType, Sbom, SbomFormat, SbomMetadata};
