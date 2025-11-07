//! ARC (Automatic Reference Counting) Insertion for Aurora
//!
//! This module implements automatic ARC insertion at uncertain escape points:
//! - Detect where manual lifetime tracking is complex
//! - Insert ARC increments/decrements automatically
//! - Emit advisories showing where ARC is used
//! - Provide escape analysis

use crate::borrow::{Advisory, BorrowChecker};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

/// ARC operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArcOp {
    /// Increment reference count
    Retain,
    /// Decrement reference count
    Release,
}

/// ARC insertion site
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArcSite {
    /// Path to value
    pub path: String,
    /// Operation
    pub operation: ArcOp,
    /// Location in code
    pub location: usize,
    /// Reason for insertion
    pub reason: String,
}

/// Escape analysis result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EscapeKind {
    /// Value does not escape
    NoEscape,
    /// Value escapes to heap
    HeapEscape,
    /// Value escapes through return
    ReturnEscape,
    /// Value escapes through closure
    ClosureEscape,
    /// Uncertain (complex control flow)
    Uncertain,
}

/// Escape information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EscapeInfo {
    /// Path to value
    pub path: String,
    /// Kind of escape
    pub kind: EscapeKind,
    /// Location
    pub location: usize,
}

/// ARC error
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ArcError {
    /// Cannot insert ARC in strict mode
    #[error("ARC insertion not allowed in strict mode")]
    StrictModeViolation,

    /// ARC insertion failed
    #[error("Failed to insert ARC for {0}")]
    InsertionFailed(String),
}

/// ARC result type
pub type ArcResult<T> = Result<T, ArcError>;

/// ARC insertion context
#[derive(Debug, Clone)]
pub struct ArcContext {
    /// Insertion sites
    sites: Vec<ArcSite>,
    /// Escape information
    escapes: Vec<EscapeInfo>,
    /// Values that need ARC
    needs_arc: HashSet<String>,
    /// Advisories
    advisories: Vec<Advisory>,
    /// Strict mode enabled
    strict_mode: bool,
}

impl ArcContext {
    /// Create new ARC context
    pub fn new(strict_mode: bool) -> Self {
        Self {
            sites: Vec::new(),
            escapes: Vec::new(),
            needs_arc: HashSet::new(),
            advisories: Vec::new(),
            strict_mode,
        }
    }

    /// Analyze escape of a value
    pub fn analyze_escape(&mut self, path: String, location: usize) -> EscapeKind {
        // Simplified escape analysis
        // In reality, would analyze:
        // - Return statements
        // - Closure captures
        // - Heap allocations
        // - Complex control flow

        let kind = if path.contains("return") {
            EscapeKind::ReturnEscape
        } else if path.contains("heap") || path.contains("Box") {
            EscapeKind::HeapEscape
        } else if path.contains("closure") || path.contains("lambda") || path.contains("|") {
            EscapeKind::ClosureEscape
        } else if path.contains("complex") {
            EscapeKind::Uncertain
        } else {
            EscapeKind::NoEscape
        };

        self.escapes.push(EscapeInfo {
            path: path.clone(),
            kind: kind.clone(),
            location,
        });

        kind
    }

    /// Insert ARC at uncertain escape point
    pub fn insert_arc(&mut self, path: String, location: usize, reason: String) -> ArcResult<()> {
        if self.strict_mode {
            return Err(ArcError::StrictModeViolation);
        }

        // Insert retain at allocation/assignment
        self.sites.push(ArcSite {
            path: path.clone(),
            operation: ArcOp::Retain,
            location,
            reason: reason.clone(),
        });

        // Mark as needing ARC
        self.needs_arc.insert(path.clone());

        // Create advisory
        self.advisories.push(Advisory::info(
            format!("ARC inserted for '{}': {}", path, reason),
            location,
        ));

        Ok(())
    }

    /// Insert ARC release
    pub fn insert_release(
        &mut self,
        path: String,
        location: usize,
        reason: String,
    ) -> ArcResult<()> {
        if self.strict_mode {
            return Err(ArcError::StrictModeViolation);
        }

        self.sites.push(ArcSite {
            path: path.clone(),
            operation: ArcOp::Release,
            location,
            reason: reason.clone(),
        });

        Ok(())
    }

    /// Check if path needs ARC
    pub fn needs_arc(&self, path: &str) -> bool {
        self.needs_arc.contains(path)
    }

    /// Get all insertion sites
    pub fn sites(&self) -> &[ArcSite] {
        &self.sites
    }

    /// Get all escapes
    pub fn escapes(&self) -> &[EscapeInfo] {
        &self.escapes
    }

    /// Get advisories
    pub fn advisories(&self) -> &[Advisory] {
        &self.advisories
    }

    /// Process escape and insert ARC if needed
    pub fn process_escape(&mut self, path: String, location: usize) -> ArcResult<()> {
        let kind = self.analyze_escape(path.clone(), location);

        match kind {
            EscapeKind::Uncertain | EscapeKind::HeapEscape | EscapeKind::ClosureEscape => {
                self.insert_arc(
                    path.clone(),
                    location,
                    format!("Value escapes ({:?})", kind),
                )?;

                // Insert corresponding release (simplified)
                self.insert_release(path, location, "Automatic cleanup".to_string())?;
            }
            _ => {}
        }

        Ok(())
    }

    /// Set strict mode
    pub fn set_strict_mode(&mut self, strict: bool) {
        self.strict_mode = strict;
    }

    /// Is strict mode enabled
    pub fn is_strict_mode(&self) -> bool {
        self.strict_mode
    }
}

impl Default for ArcContext {
    fn default() -> Self {
        Self::new(false)
    }
}

/// ARC optimizer (removes unnecessary ARC operations)
#[derive(Debug, Clone)]
pub struct ArcOptimizer {
    /// Removed sites
    removed: Vec<usize>,
}

impl ArcOptimizer {
    /// Create new optimizer
    pub fn new() -> Self {
        Self {
            removed: Vec::new(),
        }
    }

    /// Optimize ARC sites
    pub fn optimize(&mut self, sites: &mut Vec<ArcSite>) {
        // Remove redundant retain/release pairs
        let mut to_remove = Vec::new();

        for i in 0..sites.len() {
            for j in (i + 1)..sites.len() {
                if sites[i].path == sites[j].path
                    && sites[i].operation == ArcOp::Retain
                    && sites[j].operation == ArcOp::Release
                    && sites[j].location == sites[i].location
                {
                    // Same path, same location - can remove
                    to_remove.push(i);
                    to_remove.push(j);
                    break;
                }
            }
        }

        // Remove marked sites
        to_remove.sort_unstable();
        to_remove.dedup();
        to_remove.reverse(); // Remove from end to preserve indices

        for idx in to_remove {
            if idx < sites.len() {
                sites.remove(idx);
                self.removed.push(idx);
            }
        }
    }

    /// Get count of removed sites
    pub fn removed_count(&self) -> usize {
        self.removed.len()
    }
}

impl Default for ArcOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Integrate with borrow checker
pub fn arc_from_borrow_analysis(checker: &BorrowChecker, strict_mode: bool) -> ArcContext {
    let mut arc_ctx = ArcContext::new(strict_mode);

    // Analyze advisories for potential ARC sites
    for advisory in checker.advisories() {
        if advisory.error.contains("does not live long enough") {
            // Extract path from error message (simplified)
            if let Some(path) = extract_path_from_error(&advisory.error) {
                let _ = arc_ctx.insert_arc(
                    path,
                    advisory.location,
                    "Lifetime too short".to_string(),
                );
            }
        }
    }

    arc_ctx
}

/// Extract path from error message (simplified)
fn extract_path_from_error(error: &str) -> Option<String> {
    // Very simplified extraction
    error
        .split_whitespace()
        .find(|s| {
            !s.is_empty()
                && s.chars()
                    .next()
                    .map(|c| c.is_alphabetic())
                    .unwrap_or(false)
        })
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_analysis() {
        let mut ctx = ArcContext::new(false);

        let kind = ctx.analyze_escape("return_value".to_string(), 1);
        assert_eq!(kind, EscapeKind::ReturnEscape);

        let kind = ctx.analyze_escape("heap_alloc".to_string(), 2);
        assert_eq!(kind, EscapeKind::HeapEscape);

        let kind = ctx.analyze_escape("closure_capture".to_string(), 3);
        assert_eq!(kind, EscapeKind::ClosureEscape);

        let kind = ctx.analyze_escape("local_var".to_string(), 4);
        assert_eq!(kind, EscapeKind::NoEscape);
    }

    #[test]
    fn test_insert_arc() {
        let mut ctx = ArcContext::new(false);

        let result = ctx.insert_arc("x".to_string(), 1, "Test".to_string());
        assert!(result.is_ok());
        assert_eq!(ctx.sites().len(), 1);
        assert!(ctx.needs_arc("x"));
    }

    #[test]
    fn test_strict_mode_blocks_arc() {
        let mut ctx = ArcContext::new(true);

        let result = ctx.insert_arc("x".to_string(), 1, "Test".to_string());
        assert!(result.is_err());
        assert_eq!(ctx.sites().len(), 0);
    }

    #[test]
    fn test_process_escape() {
        let mut ctx = ArcContext::new(false);

        ctx.process_escape("heap_value".to_string(), 1).unwrap();

        // Should insert both retain and release
        assert_eq!(ctx.sites().len(), 2);
        assert!(ctx.sites().iter().any(|s| s.operation == ArcOp::Retain));
        assert!(ctx
            .sites()
            .iter()
            .any(|s| s.operation == ArcOp::Release));
    }

    #[test]
    fn test_arc_optimizer() {
        let mut optimizer = ArcOptimizer::new();
        let mut sites = vec![
            ArcSite {
                path: "x".to_string(),
                operation: ArcOp::Retain,
                location: 1,
                reason: "Test".to_string(),
            },
            ArcSite {
                path: "x".to_string(),
                operation: ArcOp::Release,
                location: 1,
                reason: "Test".to_string(),
            },
        ];

        optimizer.optimize(&mut sites);
        assert_eq!(sites.len(), 0);
        assert_eq!(optimizer.removed_count(), 2);
    }

    #[test]
    fn test_arc_site_creation() {
        let site = ArcSite {
            path: "test".to_string(),
            operation: ArcOp::Retain,
            location: 42,
            reason: "Testing".to_string(),
        };

        assert_eq!(site.path, "test");
        assert_eq!(site.location, 42);
        assert_eq!(site.operation, ArcOp::Retain);
    }

    #[test]
    fn test_escape_info() {
        let info = EscapeInfo {
            path: "x".to_string(),
            kind: EscapeKind::HeapEscape,
            location: 10,
        };

        assert_eq!(info.kind, EscapeKind::HeapEscape);
        assert_eq!(info.location, 10);
    }

    #[test]
    fn test_no_escape() {
        let mut ctx = ArcContext::new(false);
        ctx.process_escape("local".to_string(), 1).unwrap();

        // Should not insert ARC for no-escape
        assert_eq!(ctx.sites().len(), 0);
    }
}
