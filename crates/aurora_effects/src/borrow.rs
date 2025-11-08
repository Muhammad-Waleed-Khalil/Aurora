//! Borrow Checker (Advisory Mode) for Aurora
//!
//! This module implements Aurora's borrow checker in advisory mode:
//! - Dataflow analysis for borrow tracking
//! - Borrow conflict detection
//! - Advisory emission (warnings, not errors)
//! - Lifetime-aware borrow checking
//!
//! # Advisory Mode
//!
//! In advisory mode, borrow violations emit warnings but do not fail the build.
//! This allows gradual adoption of borrow checking.

use crate::lifetimes::{LifetimeContext, Region};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Borrow ID
pub type BorrowId = u32;

/// Borrow kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BorrowKind {
    /// Shared borrow (&T)
    Shared,
    /// Mutable borrow (&mut T)
    Mutable,
    /// Move (ownership transfer)
    Move,
}

/// Borrow information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Borrow {
    /// Borrow ID
    pub id: BorrowId,
    /// Kind of borrow
    pub kind: BorrowKind,
    /// Borrowed path (e.g., "x.field")
    pub path: String,
    /// Region (lifetime + mutability)
    pub region: Region,
    /// Source location (line number)
    pub location: usize,
}

/// Borrow conflict
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BorrowConflict {
    /// First borrow
    pub borrow1: BorrowId,
    /// Conflicting borrow
    pub borrow2: BorrowId,
    /// Description of conflict
    pub description: String,
}

/// Borrow error (used for advisories)
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BorrowError {
    /// Mutable borrow while borrowed
    #[error("Cannot borrow {0} as mutable because it is already borrowed")]
    AlreadyBorrowed(String),

    /// Use after move
    #[error("Use of moved value: {0}")]
    UseAfterMove(String),

    /// Multiple mutable borrows
    #[error("Cannot borrow {0} as mutable more than once")]
    MultipleMutableBorrows(String),

    /// Borrow outlives owner
    #[error("Borrowed value {0} does not live long enough")]
    BorrowOutlivesOwner(String),
}

/// Borrow result type
pub type BorrowResult<T> = Result<T, BorrowError>;

/// Advisory (warning about borrow issue)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Advisory {
    /// Borrow error
    pub error: String,
    /// Location
    pub location: usize,
    /// Severity (0 = info, 1 = warning, 2 = error in strict mode)
    pub severity: u8,
    /// Suggested fix
    pub suggestion: Option<String>,
}

impl Advisory {
    /// Create advisory from borrow error
    pub fn from_error(error: BorrowError, location: usize) -> Self {
        let suggestion = match &error {
            BorrowError::AlreadyBorrowed(_) => {
                Some("Consider using a different lifetime or scope".to_string())
            }
            BorrowError::UseAfterMove(_) => {
                Some("Consider cloning the value or using a reference".to_string())
            }
            BorrowError::MultipleMutableBorrows(_) => {
                Some("Use separate scopes or refactor to avoid simultaneous borrows".to_string())
            }
            BorrowError::BorrowOutlivesOwner(_) => {
                Some("Ensure borrowed value lives long enough".to_string())
            }
        };

        Self {
            error: error.to_string(),
            location,
            severity: 1, // Warning by default
            suggestion,
        }
    }

    /// Create info advisory
    pub fn info(message: String, location: usize) -> Self {
        Self {
            error: message,
            location,
            severity: 0,
            suggestion: None,
        }
    }
}

/// Borrow checker state
#[derive(Debug, Clone)]
pub struct BorrowChecker {
    /// All borrows
    borrows: HashMap<BorrowId, Borrow>,
    /// Active borrows (currently live)
    active: HashSet<BorrowId>,
    /// Moved values
    moved: HashSet<String>,
    /// Lifetime context
    lifetime_ctx: LifetimeContext,
    /// Next borrow ID
    next_id: BorrowId,
    /// Detected conflicts
    conflicts: Vec<BorrowConflict>,
    /// Advisories
    advisories: Vec<Advisory>,
}

impl BorrowChecker {
    /// Create new borrow checker
    pub fn new() -> Self {
        Self {
            borrows: HashMap::new(),
            active: HashSet::new(),
            moved: HashSet::new(),
            lifetime_ctx: LifetimeContext::new(),
            next_id: 0,
            conflicts: Vec::new(),
            advisories: Vec::new(),
        }
    }

    /// Record a borrow
    pub fn record_borrow(
        &mut self,
        kind: BorrowKind,
        path: String,
        region: Region,
        location: usize,
    ) -> BorrowId {
        let id = self.next_id;
        self.next_id += 1;

        let borrow = Borrow {
            id,
            kind,
            path: path.clone(),
            region,
            location,
        };

        // Check for conflicts
        if let Err(e) = self.check_borrow(&borrow) {
            self.advisories
                .push(Advisory::from_error(e, location));
        }

        self.borrows.insert(id, borrow);
        self.active.insert(id);
        id
    }

    /// Check if a borrow is valid (advisory mode)
    fn check_borrow(&self, borrow: &Borrow) -> BorrowResult<()> {
        // Check if value was moved
        if self.moved.contains(&borrow.path) {
            return Err(BorrowError::UseAfterMove(borrow.path.clone()));
        }

        // Check for conflicting borrows
        for &active_id in &self.active {
            if let Some(active_borrow) = self.borrows.get(&active_id) {
                if self.conflicts_with(borrow, active_borrow) {
                    return self.conflict_error(borrow, active_borrow);
                }
            }
        }

        Ok(())
    }

    /// Check if two borrows conflict
    fn conflicts_with(&self, b1: &Borrow, b2: &Borrow) -> bool {
        // Same path or overlapping paths
        if !self.paths_overlap(&b1.path, &b2.path) {
            return false;
        }

        // Mutable borrow conflicts with any other borrow
        if b1.kind == BorrowKind::Mutable || b2.kind == BorrowKind::Mutable {
            return true;
        }

        // Shared borrows don't conflict with each other
        false
    }

    /// Check if two paths overlap
    pub fn paths_overlap(&self, p1: &str, p2: &str) -> bool {
        // Simplified: exact match or prefix
        p1 == p2 || p1.starts_with(&format!("{}.", p2)) || p2.starts_with(&format!("{}.", p1))
    }

    /// Generate conflict error
    fn conflict_error(&self, b1: &Borrow, b2: &Borrow) -> BorrowResult<()> {
        match (b1.kind, b2.kind) {
            (BorrowKind::Mutable, BorrowKind::Mutable) => {
                Err(BorrowError::MultipleMutableBorrows(b1.path.clone()))
            }
            (BorrowKind::Mutable, _) | (_, BorrowKind::Mutable) => {
                Err(BorrowError::AlreadyBorrowed(b1.path.clone()))
            }
            _ => Ok(()),
        }
    }

    /// Record a move
    pub fn record_move(&mut self, path: String, location: usize) {
        self.moved.insert(path.clone());

        // Invalidate borrows of this path
        // Collect IDs to remove first to avoid borrow checker issues
        let to_remove: Vec<BorrowId> = self
            .active
            .iter()
            .filter(|&&id| {
                if let Some(borrow) = self.borrows.get(&id) {
                    self.paths_overlap(&borrow.path, &path)
                } else {
                    false
                }
            })
            .copied()
            .collect();

        for id in to_remove {
            self.active.remove(&id);
        }

        self.advisories.push(Advisory::info(
            format!("Value moved: {}", path),
            location,
        ));
    }

    /// End a borrow (lifetime ends)
    pub fn end_borrow(&mut self, id: BorrowId) {
        self.active.remove(&id);
    }

    /// Enter new scope
    pub fn push_scope(&mut self) {
        self.lifetime_ctx.push_scope();
    }

    /// Exit scope (end borrows in this scope)
    pub fn pop_scope(&mut self) {
        self.lifetime_ctx.pop_scope();

        // End borrows that go out of scope
        // Simplified: end all active borrows
        // In reality, would check lifetime scopes
        self.active.clear();
    }

    /// Get all conflicts
    pub fn conflicts(&self) -> &[BorrowConflict] {
        &self.conflicts
    }

    /// Get all advisories
    pub fn advisories(&self) -> &[Advisory] {
        &self.advisories
    }

    /// Check if there are any advisories
    pub fn has_advisories(&self) -> bool {
        !self.advisories.is_empty()
    }

    /// Get advisory count
    pub fn advisory_count(&self) -> usize {
        self.advisories.len()
    }

    /// Clear advisories (for testing)
    pub fn clear_advisories(&mut self) {
        self.advisories.clear();
    }
}

impl Default for BorrowChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Dataflow analysis for borrows
#[derive(Debug, Clone)]
pub struct BorrowDataflow {
    /// Borrows at each program point
    pub borrows_in: HashMap<usize, HashSet<BorrowId>>,
    pub borrows_out: HashMap<usize, HashSet<BorrowId>>,
    /// Moves at each program point
    pub moves: HashMap<usize, HashSet<String>>,
}

impl BorrowDataflow {
    /// Create new dataflow analysis
    pub fn new() -> Self {
        Self {
            borrows_in: HashMap::new(),
            borrows_out: HashMap::new(),
            moves: HashMap::new(),
        }
    }

    /// Record borrows at program point
    pub fn set_borrows(&mut self, point: usize, borrows: HashSet<BorrowId>) {
        self.borrows_in.insert(point, borrows.clone());
        self.borrows_out.insert(point, borrows);
    }

    /// Record move at program point
    pub fn record_move(&mut self, point: usize, path: String) {
        self.moves.entry(point).or_default().insert(path);
    }

    /// Get live borrows at program point
    pub fn live_borrows(&self, point: usize) -> HashSet<BorrowId> {
        self.borrows_out.get(&point).cloned().unwrap_or_default()
    }
}

impl Default for BorrowDataflow {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_borrow() {
        let mut checker = BorrowChecker::new();
        let region = Region::static_region(false);

        let b1 = checker.record_borrow(BorrowKind::Shared, "x".to_string(), region.clone(), 1);
        let b2 = checker.record_borrow(BorrowKind::Shared, "x".to_string(), region.clone(), 2);

        // Shared borrows don't conflict
        assert_eq!(checker.advisory_count(), 0);
        assert_ne!(b1, b2);
    }

    #[test]
    fn test_mutable_borrow_conflict() {
        let mut checker = BorrowChecker::new();
        let region = Region::static_region(true);

        checker.record_borrow(BorrowKind::Mutable, "x".to_string(), region.clone(), 1);
        checker.record_borrow(BorrowKind::Mutable, "x".to_string(), region.clone(), 2);

        // Should generate advisory
        assert!(checker.has_advisories());
        assert_eq!(checker.advisory_count(), 1);
    }

    #[test]
    fn test_use_after_move() {
        let mut checker = BorrowChecker::new();
        let region = Region::static_region(false);

        checker.record_move("x".to_string(), 1);
        checker.record_borrow(BorrowKind::Shared, "x".to_string(), region, 2);

        // Should generate advisory
        assert!(checker.has_advisories());
    }

    #[test]
    fn test_scopes() {
        let mut checker = BorrowChecker::new();
        let region = Region::static_region(false);

        checker.push_scope();
        checker.record_borrow(BorrowKind::Shared, "x".to_string(), region, 1);
        assert!(!checker.active.is_empty());

        checker.pop_scope();
        assert!(checker.active.is_empty());
    }

    #[test]
    fn test_paths_overlap() {
        let checker = BorrowChecker::new();

        assert!(checker.paths_overlap("x", "x"));
        assert!(checker.paths_overlap("x.field", "x"));
        assert!(checker.paths_overlap("x", "x.field"));
        assert!(!checker.paths_overlap("x", "y"));
    }

    #[test]
    fn test_advisory_from_error() {
        let error = BorrowError::AlreadyBorrowed("x".to_string());
        let advisory = Advisory::from_error(error, 42);

        assert_eq!(advisory.location, 42);
        assert_eq!(advisory.severity, 1);
        assert!(advisory.suggestion.is_some());
    }

    #[test]
    fn test_borrow_dataflow() {
        let mut dataflow = BorrowDataflow::new();

        let mut borrows = HashSet::new();
        borrows.insert(1);
        borrows.insert(2);

        dataflow.set_borrows(10, borrows.clone());
        assert_eq!(dataflow.live_borrows(10), borrows);

        dataflow.record_move(20, "x".to_string());
        assert!(dataflow.moves.get(&20).unwrap().contains("x"));
    }

    #[test]
    fn test_end_borrow() {
        let mut checker = BorrowChecker::new();
        let region = Region::static_region(false);

        let id = checker.record_borrow(BorrowKind::Shared, "x".to_string(), region, 1);
        assert!(checker.active.contains(&id));

        checker.end_borrow(id);
        assert!(!checker.active.contains(&id));
    }
}
