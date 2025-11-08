//! Security policy configuration and enforcement

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Security policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// Allow dynamic features (reflection, eval, etc.)
    pub allow_dynamic: bool,
    /// Allow unsafe code
    pub allow_unsafe: bool,
    /// Allowed FFI targets
    pub allowed_ffi: HashSet<String>,
    /// Require signature verification for dependencies
    pub require_signatures: bool,
    /// Allowed dependency sources
    pub allowed_sources: Vec<String>,
    /// Maximum dependency depth
    pub max_dependency_depth: usize,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            allow_dynamic: false,
            allow_unsafe: false,
            allowed_ffi: HashSet::new(),
            require_signatures: true,
            allowed_sources: vec!["https://pkg.aurora-lang.org".to_string()],
            max_dependency_depth: 10,
        }
    }
}

impl SecurityPolicy {
    /// Create permissive policy (for development)
    pub fn permissive() -> Self {
        Self {
            allow_dynamic: true,
            allow_unsafe: true,
            allowed_ffi: ["C", "Python", "Node"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            require_signatures: false,
            allowed_sources: vec!["*".to_string()],
            max_dependency_depth: 100,
        }
    }

    /// Create strict policy (for production)
    pub fn strict() -> Self {
        Self {
            allow_dynamic: false,
            allow_unsafe: false,
            allowed_ffi: HashSet::new(),
            require_signatures: true,
            allowed_sources: vec!["https://pkg.aurora-lang.org".to_string()],
            max_dependency_depth: 5,
        }
    }

    /// Check if FFI target is allowed
    pub fn is_ffi_allowed(&self, target: &str) -> bool {
        self.allowed_ffi.contains(target) || self.allowed_ffi.contains("*")
    }

    /// Check if source is allowed
    pub fn is_source_allowed(&self, source: &str) -> bool {
        self.allowed_sources.iter().any(|s| s == "*" || s == source)
    }

    /// Validate policy configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.max_dependency_depth == 0 {
            return Err("max_dependency_depth must be > 0".to_string());
        }

        if self.allowed_sources.is_empty() && self.require_signatures {
            return Err(
                "allowed_sources cannot be empty when require_signatures is true".to_string()
            );
        }

        Ok(())
    }
}

/// Signature verification result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationResult {
    /// Valid signature
    Valid,
    /// Invalid signature
    Invalid,
    /// No signature found
    Missing,
    /// Signature verification skipped
    Skipped,
}

/// Dependency signature
#[derive(Debug, Clone)]
pub struct Signature {
    /// Signing algorithm
    pub algorithm: String,
    /// Signature value
    pub value: Vec<u8>,
    /// Signer public key
    pub public_key: Vec<u8>,
}

impl Signature {
    /// Create new signature
    pub fn new(algorithm: String, value: Vec<u8>, public_key: Vec<u8>) -> Self {
        Self {
            algorithm,
            value,
            public_key,
        }
    }

    /// Verify signature (placeholder - real impl would use crypto)
    pub fn verify(&self, _data: &[u8]) -> VerificationResult {
        // In real implementation, this would perform cryptographic verification
        if !self.value.is_empty() && !self.public_key.is_empty() {
            VerificationResult::Valid
        } else {
            VerificationResult::Invalid
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy() {
        let policy = SecurityPolicy::default();
        assert!(!policy.allow_dynamic);
        assert!(!policy.allow_unsafe);
        assert!(policy.require_signatures);
    }

    #[test]
    fn test_permissive_policy() {
        let policy = SecurityPolicy::permissive();
        assert!(policy.allow_dynamic);
        assert!(policy.allow_unsafe);
        assert!(!policy.require_signatures);
    }

    #[test]
    fn test_strict_policy() {
        let policy = SecurityPolicy::strict();
        assert!(!policy.allow_dynamic);
        assert!(!policy.allow_unsafe);
        assert!(policy.require_signatures);
        assert_eq!(policy.max_dependency_depth, 5);
    }

    #[test]
    fn test_is_ffi_allowed() {
        let mut policy = SecurityPolicy::default();
        policy.allowed_ffi.insert("C".to_string());

        assert!(policy.is_ffi_allowed("C"));
        assert!(!policy.is_ffi_allowed("Python"));
    }

    #[test]
    fn test_is_source_allowed() {
        let policy = SecurityPolicy::default();
        assert!(policy.is_source_allowed("https://pkg.aurora-lang.org"));
        assert!(!policy.is_source_allowed("https://malicious.com"));
    }

    #[test]
    fn test_policy_validation_ok() {
        let policy = SecurityPolicy::default();
        assert!(policy.validate().is_ok());
    }

    #[test]
    fn test_policy_validation_zero_depth() {
        let mut policy = SecurityPolicy::default();
        policy.max_dependency_depth = 0;
        assert!(policy.validate().is_err());
    }

    #[test]
    fn test_policy_validation_empty_sources() {
        let mut policy = SecurityPolicy::default();
        policy.allowed_sources.clear();
        assert!(policy.validate().is_err());
    }

    #[test]
    fn test_signature_creation() {
        let sig = Signature::new(
            "ed25519".to_string(),
            vec![1, 2, 3],
            vec![4, 5, 6],
        );
        assert_eq!(sig.algorithm, "ed25519");
        assert_eq!(sig.value.len(), 3);
    }

    #[test]
    fn test_signature_verify_valid() {
        let sig = Signature::new(
            "ed25519".to_string(),
            vec![1, 2, 3],
            vec![4, 5, 6],
        );
        assert_eq!(sig.verify(&[]), VerificationResult::Valid);
    }

    #[test]
    fn test_signature_verify_invalid() {
        let sig = Signature::new(
            "ed25519".to_string(),
            vec![],
            vec![],
        );
        assert_eq!(sig.verify(&[]), VerificationResult::Invalid);
    }
}
