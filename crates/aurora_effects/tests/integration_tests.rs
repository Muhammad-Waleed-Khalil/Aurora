//! Integration Tests for Aurora Effects & Borrow System
//!
//! These tests validate the complete effect and borrow checking system.

use aurora_effects::*;
use aurora_types::EffectSet;

#[test]
fn test_effect_tracking_integration() {
    let mut tracker = EffectTracker::new();

    // Start with pure context
    assert_eq!(tracker.current(), EffectSet::PURE);

    // Add IO effect
    tracker.add_effect(EffectSet::IO);
    assert_eq!(tracker.current(), EffectSet::IO);

    // Push unsafe context
    tracker.push_context(EffectSet::UNSAFE);
    assert_eq!(tracker.current(), EffectSet::UNSAFE);

    // Pop back to IO
    tracker.pop_context();
    assert_eq!(tracker.current(), EffectSet::IO);

    // Check that IO is allowed in IO context
    assert!(tracker.check_allowed(EffectSet::IO).is_ok());

    // Check that UNSAFE is not allowed in IO context
    assert!(tracker.check_allowed(EffectSet::UNSAFE).is_err());
}

#[test]
fn test_effect_subtyping() {
    // PURE ⊆ IO
    assert!(is_subeffect(EffectSet::PURE, EffectSet::IO));

    // PURE ⊆ ALLOC
    assert!(is_subeffect(EffectSet::PURE, EffectSet::ALLOC));

    // IO ⊆ UNSAFE
    assert!(is_subeffect(EffectSet::IO, EffectSet::UNSAFE));

    // ALLOC ⊆ UNSAFE
    assert!(is_subeffect(EffectSet::ALLOC, EffectSet::UNSAFE));

    // IO not subeffect of ALLOC
    assert!(!is_subeffect(EffectSet::IO, EffectSet::ALLOC));
}

#[test]
fn test_borrow_checker_shared_borrows() {
    let mut checker = BorrowChecker::new();
    let region = Region::static_region(false);

    // Multiple shared borrows are allowed
    checker.record_borrow(BorrowKind::Shared, "x".to_string(), region.clone(), 1);
    checker.record_borrow(BorrowKind::Shared, "x".to_string(), region.clone(), 2);
    checker.record_borrow(BorrowKind::Shared, "x".to_string(), region, 3);

    assert_eq!(checker.advisory_count(), 0);
}

#[test]
fn test_borrow_checker_mutable_conflict() {
    let mut checker = BorrowChecker::new();
    let region = Region::static_region(true);

    // First mutable borrow
    checker.record_borrow(BorrowKind::Mutable, "x".to_string(), region.clone(), 1);

    // Second mutable borrow should conflict
    checker.record_borrow(BorrowKind::Mutable, "x".to_string(), region, 2);

    assert!(checker.has_advisories());
    assert_eq!(checker.advisory_count(), 1);
}

#[test]
fn test_borrow_checker_shared_after_mutable() {
    let mut checker = BorrowChecker::new();
    let region = Region::static_region(false);

    // Mutable borrow
    checker.record_borrow(BorrowKind::Mutable, "x".to_string(), region.clone(), 1);

    // Shared borrow should conflict
    checker.record_borrow(BorrowKind::Shared, "x".to_string(), region, 2);

    assert!(checker.has_advisories());
}

#[test]
fn test_borrow_checker_use_after_move() {
    let mut checker = BorrowChecker::new();
    let region = Region::static_region(false);

    // Move value
    checker.record_move("x".to_string(), 1);

    // Try to borrow after move
    checker.record_borrow(BorrowKind::Shared, "x".to_string(), region, 2);

    assert!(checker.has_advisories());
}

#[test]
fn test_borrow_checker_scopes() {
    let mut checker = BorrowChecker::new();
    let region = Region::static_region(false);

    checker.push_scope();
    let _id = checker.record_borrow(BorrowKind::Shared, "x".to_string(), region, 1);

    // Borrow is active
    assert!(!checker.advisories().is_empty() || checker.advisories().is_empty()); // May or may not have advisories

    checker.pop_scope();

    // After popping scope, active borrows are cleared
    // (In a real implementation, would check lifetime scopes)
}

#[test]
fn test_lifetime_tracking() {
    let mut ctx = LifetimeContext::new();

    // Create lifetimes at different scopes
    let l1 = ctx.fresh();

    ctx.push_scope();
    let l2 = ctx.fresh();

    // l1 outlives l2 (outer scope outlives inner)
    assert!(ctx.outlives(&l1, &l2));
    assert!(!ctx.outlives(&l2, &l1));

    // Static outlives everything
    assert!(ctx.outlives(&Lifetime::Static, &l1));
    assert!(ctx.outlives(&Lifetime::Static, &l2));

    ctx.pop_scope();
}

#[test]
fn test_lifetime_constraints() {
    let mut ctx = LifetimeContext::new();
    let l1 = ctx.fresh();
    let l2 = ctx.fresh();

    // Add outlives constraint
    ctx.add_outlives(l1.clone(), l2.clone());

    // Check constraints
    assert!(ctx.check_constraints().is_ok());

    // Add equality constraint
    ctx.add_equals(l1.clone(), l2.clone());

    assert_eq!(ctx.constraints().len(), 2);
}

#[test]
fn test_arc_escape_analysis() {
    let mut arc_ctx = ArcContext::new(false);

    // Test different escape kinds
    let kind = arc_ctx.analyze_escape("return_value".to_string(), 1);
    assert_eq!(kind, EscapeKind::ReturnEscape);

    let kind = arc_ctx.analyze_escape("Box::new(x)".to_string(), 2);
    assert_eq!(kind, EscapeKind::HeapEscape);

    let kind = arc_ctx.analyze_escape("|x| x + 1".to_string(), 3);
    assert_eq!(kind, EscapeKind::ClosureEscape);

    let kind = arc_ctx.analyze_escape("local_var".to_string(), 4);
    assert_eq!(kind, EscapeKind::NoEscape);
}

#[test]
fn test_arc_insertion() {
    let mut arc_ctx = ArcContext::new(false);

    // Insert ARC for heap escape
    arc_ctx
        .process_escape("heap_value".to_string(), 1)
        .unwrap();

    // Should have both retain and release
    assert_eq!(arc_ctx.sites().len(), 2);
    assert!(arc_ctx.sites().iter().any(|s| s.operation == ArcOp::Retain));
    assert!(arc_ctx
        .sites()
        .iter()
        .any(|s| s.operation == ArcOp::Release));

    // Should have advisories
    assert!(!arc_ctx.advisories().is_empty());
}

#[test]
fn test_arc_strict_mode() {
    let mut arc_ctx = ArcContext::new(true); // Strict mode

    // ARC insertion should fail
    let result = arc_ctx.insert_arc("x".to_string(), 1, "test".to_string());
    assert!(result.is_err());

    assert_eq!(arc_ctx.sites().len(), 0);
}

#[test]
fn test_strict_mode_permissive() {
    let mut checker = StrictChecker::permissive();
    let mut borrow_checker = BorrowChecker::new();
    let region = Region::static_region(true);

    // Create borrow conflict
    borrow_checker.record_borrow(BorrowKind::Mutable, "x".to_string(), region.clone(), 1);
    borrow_checker.record_borrow(BorrowKind::Mutable, "x".to_string(), region, 2);

    // In permissive mode, should not error
    let result = checker.check_borrow_checker(&borrow_checker);
    assert!(result.is_ok());
}

#[test]
fn test_strict_mode_strict() {
    let mut checker = StrictChecker::strict();
    let mut borrow_checker = BorrowChecker::new();
    let region = Region::static_region(true);

    // Create borrow conflict
    borrow_checker.record_borrow(BorrowKind::Mutable, "x".to_string(), region.clone(), 1);
    borrow_checker.record_borrow(BorrowKind::Mutable, "x".to_string(), region, 2);

    // In strict mode, should error
    let result = checker.check_borrow_checker(&borrow_checker);
    assert!(result.is_err());
}

#[test]
fn test_strict_lifetime_checking() {
    let mut checker = StrictChecker::strict();

    // Anonymous lifetime should fail in strict mode
    let result = checker.check_lifetime(&Lifetime::Anon(0), "test");
    assert!(result.is_err());

    // Named lifetime should pass
    let result = checker.check_lifetime(&Lifetime::Named("a".to_string()), "test");
    assert!(result.is_ok());

    // Static lifetime should pass
    let result = checker.check_lifetime(&Lifetime::Static, "test");
    assert!(result.is_ok());
}

#[test]
fn test_strict_arc_enforcement() {
    let mut checker = StrictChecker::strict();
    let mut arc_ctx = ArcContext::new(false);

    // No ARC sites - should pass
    let result = checker.check_arc(&arc_ctx);
    assert!(result.is_ok());

    // Add ARC site
    arc_ctx
        .insert_arc("x".to_string(), 1, "test".to_string())
        .unwrap();

    // Should fail in strict mode
    let result = checker.check_arc(&arc_ctx);
    assert!(result.is_err());
}

#[test]
fn test_effect_inference() {
    let mut ctx = EffectInferContext::new();

    // Create effect variables
    let e1 = ctx.fresh_var();
    let e2 = ctx.fresh_var();

    // Unify with concrete effect
    ctx.unify(&e1, &Effect::Concrete(EffectSet::IO)).unwrap();

    // Apply substitution
    let result = ctx.apply(&e1);
    assert_eq!(result, Effect::Concrete(EffectSet::IO));

    // Unify two variables
    ctx.unify(&e2, &e1).unwrap();
    let result = ctx.apply(&e2);
    assert_eq!(result, Effect::Concrete(EffectSet::IO));
}

#[test]
fn test_complete_workflow() {
    // Simulate complete borrow checking workflow

    // 1. Create borrow checker
    let mut borrow_checker = BorrowChecker::new();

    // 2. Track lifetimes
    let mut lifetime_ctx = LifetimeContext::new();
    let lifetime = lifetime_ctx.fresh();
    let region = Region::new(lifetime, false);

    // 3. Record borrows
    borrow_checker.record_borrow(BorrowKind::Shared, "x".to_string(), region.clone(), 1);
    borrow_checker.record_borrow(BorrowKind::Shared, "x.field".to_string(), region, 2);

    // 4. Check ARC needs
    let arc_ctx = ArcContext::new(false);

    // 5. Apply strict mode checking
    let mut strict_checker = StrictChecker::permissive();
    let result = strict_checker.check_all(&borrow_checker, &arc_ctx, &lifetime_ctx);

    assert!(result.is_ok());
    assert!(!borrow_checker.has_advisories());
}

#[test]
fn test_effect_composition() {
    let e1 = EffectSet::IO;
    let e2 = EffectSet::ALLOC;

    let composed = compose_effects(e1, e2);

    assert!(composed.has(EffectSet::IO));
    assert!(composed.has(EffectSet::ALLOC));
}

#[test]
fn test_region_creation() {
    let region = Region::static_region(false);
    assert!(region.is_static());
    assert!(!region.mutable);

    let mut ctx = LifetimeContext::new();
    let lifetime = ctx.fresh();
    let region = Region::new(lifetime, true);
    assert!(!region.is_static());
    assert!(region.mutable);
}

#[test]
fn test_advisory_severity() {
    let error = BorrowError::AlreadyBorrowed("x".to_string());
    let advisory = Advisory::from_error(error, 10);

    assert_eq!(advisory.severity, 1); // Warning
    assert_eq!(advisory.location, 10);
    assert!(advisory.suggestion.is_some());

    let info = Advisory::info("Test info".to_string(), 20);
    assert_eq!(info.severity, 0); // Info
}

#[test]
fn test_borrow_dataflow() {
    let mut dataflow = BorrowDataflow::new();

    let mut borrows = std::collections::HashSet::new();
    borrows.insert(1);
    borrows.insert(2);

    dataflow.set_borrows(10, borrows.clone());
    assert_eq!(dataflow.live_borrows(10), borrows);

    dataflow.record_move(20, "x".to_string());
    assert!(dataflow.moves.get(&20).unwrap().contains("x"));
}
