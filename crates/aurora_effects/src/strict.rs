//! Strict Mode for Aurora Borrow Checker
//!
//! This module implements strict mode enforcement:
//! - Convert advisories to errors
//! - Require explicit lifetimes
//! - Disallow ARC insertion
//! - Enforce all borrow rules strictly

use crate::arc::ArcContext;
use crate::borrow::BorrowChecker;
use crate::lifetimes::{Lifetime, LifetimeContext};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Strict mode error
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum StrictError {
    /// Borrow violation in strict mode
    #[error("Borrow rule violation: {0}")]
    BorrowViolation(String),

    /// Missing lifetime annotation
    #[error("Explicit lifetime required for {0}")]
    MissingLifetime(String),

    /// ARC not allowed in strict mode
    #[error("ARC insertion not allowed in strict mode")]
    ArcNotAllowed,

    /// Implicit conversion not allowed
    #[error("Implicit conversion not allowed: {0}")]
    ImplicitConversion(String),

    /// Unsafe operation in safe context
    #[error("Unsafe operation requires unsafe block: {0}")]
    UnsafeRequired(String),
}

/// Strict mode result
pub type StrictResult<T> = Result<T, StrictError>;

/// Strict mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrictConfig {
    /// Require explicit lifetimes
    pub require_lifetimes: bool,
    /// Disallow ARC insertion
    pub disallow_arc: bool,
    /// Treat advisories as errors
    pub advisories_are_errors: bool,
    /// Disallow implicit conversions
    pub disallow_implicit_conversions: bool,
    /// Require unsafe blocks
    pub require_unsafe_blocks: bool,
}

impl StrictConfig {
    /// Create default strict configuration
    pub fn strict() -> Self {
        Self {
            require_lifetimes: true,
            disallow_arc: true,
            advisories_are_errors: true,
            disallow_implicit_conversions: true,
            require_unsafe_blocks: true,
        }
    }

    /// Create permissive configuration (advisory mode)
    pub fn permissive() -> Self {
        Self {
            require_lifetimes: false,
            disallow_arc: false,
            advisories_are_errors: false,
            disallow_implicit_conversions: false,
            require_unsafe_blocks: false,
        }
    }

    /// Create configuration with specific settings
    pub fn custom() -> Self {
        Self {
            require_lifetimes: false,
            disallow_arc: false,
            advisories_are_errors: false,
            disallow_implicit_conversions: false,
            require_unsafe_blocks: true,
        }
    }
}

impl Default for StrictConfig {
    fn default() -> Self {
        Self::permissive()
    }
}

/// Strict mode checker
#[derive(Debug, Clone)]
pub struct StrictChecker {
    /// Configuration
    config: StrictConfig,
    /// Collected errors
    errors: Vec<StrictError>,
}

impl StrictChecker {
    /// Create new strict checker
    pub fn new(config: StrictConfig) -> Self {
        Self {
            config,
            errors: Vec::new(),
        }
    }

    /// Create strict mode checker
    pub fn strict() -> Self {
        Self::new(StrictConfig::strict())
    }

    /// Create permissive mode checker
    pub fn permissive() -> Self {
        Self::new(StrictConfig::permissive())
    }

    /// Check borrow checker results
    pub fn check_borrow_checker(&mut self, checker: &BorrowChecker) -> StrictResult<()> {
        if !self.config.advisories_are_errors {
            return Ok(());
        }

        // Convert advisories to errors
        for advisory in checker.advisories() {
            if advisory.severity > 0 {
                // Warning or higher
                self.errors
                    .push(StrictError::BorrowViolation(advisory.error.clone()));
            }
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors[0].clone())
        }
    }

    /// Check lifetime annotations
    pub fn check_lifetime(&mut self, lifetime: &Lifetime, context: &str) -> StrictResult<()> {
        if !self.config.require_lifetimes {
            return Ok(());
        }

        // In strict mode, require explicit lifetimes (no anonymous)
        if lifetime.is_anon() {
            let error = StrictError::MissingLifetime(context.to_string());
            self.errors.push(error.clone());
            Err(error)
        } else {
            Ok(())
        }
    }

    /// Check ARC insertion
    pub fn check_arc(&mut self, arc_ctx: &ArcContext) -> StrictResult<()> {
        if !self.config.disallow_arc {
            return Ok(());
        }

        if !arc_ctx.sites().is_empty() {
            let error = StrictError::ArcNotAllowed;
            self.errors.push(error.clone());
            Err(error)
        } else {
            Ok(())
        }
    }

    /// Check implicit conversion
    pub fn check_implicit_conversion(&mut self, description: String) -> StrictResult<()> {
        if !self.config.disallow_implicit_conversions {
            return Ok(());
        }

        let error = StrictError::ImplicitConversion(description);
        self.errors.push(error.clone());
        Err(error)
    }

    /// Check unsafe operation
    pub fn check_unsafe(&mut self, operation: String, in_unsafe_block: bool) -> StrictResult<()> {
        if !self.config.require_unsafe_blocks {
            return Ok(());
        }

        if !in_unsafe_block {
            let error = StrictError::UnsafeRequired(operation);
            self.errors.push(error.clone());
            Err(error)
        } else {
            Ok(())
        }
    }

    /// Get all errors
    pub fn errors(&self) -> &[StrictError] {
        &self.errors
    }

    /// Check if any errors occurred
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get error count
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Clear errors
    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }

    /// Get configuration
    pub fn config(&self) -> &StrictConfig {
        &self.config
    }

    /// Check all rules
    pub fn check_all(
        &mut self,
        checker: &BorrowChecker,
        arc_ctx: &ArcContext,
        lifetime_ctx: &LifetimeContext,
    ) -> StrictResult<()> {
        // Check borrow rules
        self.check_borrow_checker(checker)?;

        // Check ARC usage
        self.check_arc(arc_ctx)?;

        // Check lifetimes (would need AST traversal in real impl)
        // For now, just validate that context exists
        let _ = lifetime_ctx;

        if self.has_errors() {
            Err(self.errors[0].clone())
        } else {
            Ok(())
        }
    }
}

/// Strict mode enforcement for the entire module
#[derive(Debug, Clone)]
pub struct StrictModeEnforcer {
    /// Strict checker
    checker: StrictChecker,
    /// Module being checked
    module_name: String,
}

impl StrictModeEnforcer {
    /// Create new enforcer
    pub fn new(module_name: String, config: StrictConfig) -> Self {
        Self {
            checker: StrictChecker::new(config),
            module_name,
        }
    }

    /// Enforce strict mode on module
    pub fn enforce(
        &mut self,
        borrow_checker: &BorrowChecker,
        arc_ctx: &ArcContext,
        lifetime_ctx: &LifetimeContext,
    ) -> StrictResult<()> {
        self.checker
            .check_all(borrow_checker, arc_ctx, lifetime_ctx)
    }

    /// Get module name
    pub fn module_name(&self) -> &str {
        &self.module_name
    }

    /// Get errors
    pub fn errors(&self) -> &[StrictError] {
        self.checker.errors()
    }

    /// Has errors
    pub fn has_errors(&self) -> bool {
        self.checker.has_errors()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::borrow::BorrowKind;
    use crate::lifetimes::Region;

    #[test]
    fn test_strict_config() {
        let strict = StrictConfig::strict();
        assert!(strict.require_lifetimes);
        assert!(strict.disallow_arc);
        assert!(strict.advisories_are_errors);

        let permissive = StrictConfig::permissive();
        assert!(!permissive.require_lifetimes);
        assert!(!permissive.disallow_arc);
        assert!(!permissive.advisories_are_errors);
    }

    #[test]
    fn test_strict_checker_permissive() {
        let checker = StrictChecker::permissive();
        assert!(!checker.config().require_lifetimes);
    }

    #[test]
    fn test_strict_checker_strict() {
        let checker = StrictChecker::strict();
        assert!(checker.config().require_lifetimes);
        assert!(checker.config().disallow_arc);
    }

    #[test]
    fn test_check_lifetime_strict() {
        let mut checker = StrictChecker::strict();
        let anon_lifetime = Lifetime::Anon(0);

        let result = checker.check_lifetime(&anon_lifetime, "test");
        assert!(result.is_err());
        assert!(checker.has_errors());
    }

    #[test]
    fn test_check_lifetime_permissive() {
        let mut checker = StrictChecker::permissive();
        let anon_lifetime = Lifetime::Anon(0);

        let result = checker.check_lifetime(&anon_lifetime, "test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_arc_strict() {
        let mut checker = StrictChecker::strict();
        let mut arc_ctx = ArcContext::new(false);

        // Insert some ARC
        arc_ctx
            .insert_arc("x".to_string(), 1, "test".to_string())
            .unwrap();

        let result = checker.check_arc(&arc_ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_arc_permissive() {
        let mut checker = StrictChecker::permissive();
        let mut arc_ctx = ArcContext::new(false);

        arc_ctx
            .insert_arc("x".to_string(), 1, "test".to_string())
            .unwrap();

        let result = checker.check_arc(&arc_ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_borrow_checker_strict() {
        let mut strict = StrictChecker::strict();
        let mut borrow_checker = BorrowChecker::new();

        // Create a borrow violation
        let region = Region::static_region(true);
        borrow_checker.record_borrow(BorrowKind::Mutable, "x".to_string(), region.clone(), 1);
        borrow_checker.record_borrow(BorrowKind::Mutable, "x".to_string(), region, 2);

        let result = strict.check_borrow_checker(&borrow_checker);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_unsafe() {
        let mut strict = StrictChecker::strict();

        let result = strict.check_unsafe("deref raw pointer".to_string(), false);
        assert!(result.is_err());

        let result = strict.check_unsafe("deref raw pointer".to_string(), true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_strict_enforcer() {
        let config = StrictConfig::strict();
        let mut enforcer = StrictModeEnforcer::new("test_module".to_string(), config);

        assert_eq!(enforcer.module_name(), "test_module");

        let borrow_checker = BorrowChecker::new();
        let arc_ctx = ArcContext::new(true);
        let lifetime_ctx = LifetimeContext::new();

        let result = enforcer.enforce(&borrow_checker, &arc_ctx, &lifetime_ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_accumulation() {
        let mut checker = StrictChecker::strict();

        checker
            .check_lifetime(&Lifetime::Anon(0), "test1")
            .ok();
        checker
            .check_lifetime(&Lifetime::Anon(1), "test2")
            .ok();

        assert_eq!(checker.error_count(), 2);

        checker.clear_errors();
        assert_eq!(checker.error_count(), 0);
    }
}
