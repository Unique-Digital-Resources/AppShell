//! Normalization tests for the Docking environment
//! (current_state/plan.md Phase 05, PRD §8.12, §9.2).
//!
//! Coverage: the operation pipeline's cleanup (mutate → normalize →
//! validate → commit) — collapse chains, survivor ordering, weight
//! redistribution after removals — plus the whole-layout
//! `DockLayout::normalize` entry point's guarantees on constructible
//! layouts: a no-op on valid layouts and idempotence on committed
//! results. Atomic rejection (failure leaves the original state
//! unchanged) is re-asserted here against the plan criterion.
//!
//! Below-minimum states are unreachable through the public API, so the
//! rule-core fixtures live in-crate
//! (`src/.../docking/normalization.rs`, `pub(crate)` installers) per the
//! sanctioned invalid-state fixture strategy (agents/testing.md).

use app_shell::domain_foundation::environments::docking::{
    DockId, DockLayout, DockSlot, DockingError, DockingOperation, DockingOutcome, InsertSource,
    SplitDirection, SplitOrientation, SplitSizes,
};

fn uid(n: u64) -> DockId {
    DockId::from_raw(n)
}

// --- Pipeline cleanup: collapse chains through operations (PRD §8.12) ---

#[test]
fn remove_cascades_and_survivor_inherits_the_slot_weight() {
    let mut layout = DockLayout::new();
    let a = layout.create_panel(Some("a".into()));
    let b = layout.create_panel(Some("b".into()));
    let c = layout.create_panel(Some("c".into()));
    layout.set_root(None).unwrap();
    let inner = layout
        .create_split(SplitOrientation::Row, vec![a, b], SplitSizes::uniform(2).unwrap())
        .unwrap();
    let col = layout
        .create_split(
            SplitOrientation::Column,
            vec![inner, c],
            SplitSizes::from_weights(vec![0.7, 0.3]).unwrap(),
        )
        .unwrap();
    layout.set_root(Some(col)).unwrap();
    assert!(layout.is_valid());

    let outcome = DockingOperation::Remove { dock: a }.apply(&mut layout).unwrap();
    let DockingOutcome::Removed { .. } = outcome else { panic!("expected Removed") };
    assert!(layout.is_valid());
    // `inner` collapsed; its survivor `b` inherited the slot reference
    // (order preserved) inside `col`.
    assert!(!layout.contains(inner));
    assert_eq!(layout.placement(b), Some(DockSlot::SplitChild { parent: col, index: 0 }));
    assert_eq!(layout.placement(c), Some(DockSlot::SplitChild { parent: col, index: 1 }));
    // Weight policy: the parent keeps the slot's weight — the survivor
    // inherits the collapsed container's area (PRD §9.2).
    assert_eq!(
        layout.get(col).unwrap().sizes().map(|s| s.weights().to_vec()),
        Some(vec![0.7, 0.3])
    );
    assert_eq!(layout.root(), Some(col));
}

#[test]
fn sequential_removals_collapse_the_full_chain() {
    let mut layout = DockLayout::new();
    let p1 = layout.create_panel(Some("p1".into()));
    let p2 = layout.create_panel(Some("p2".into()));
    let c = layout.create_panel(Some("c".into()));
    let x = layout.create_panel(Some("x".into()));
    layout.set_root(None).unwrap();
    let inner = layout
        .create_split(SplitOrientation::Row, vec![p1, p2], SplitSizes::uniform(2).unwrap())
        .unwrap();
    let col = layout
        .create_split(SplitOrientation::Column, vec![inner, c], SplitSizes::uniform(2).unwrap())
        .unwrap();
    let root_row = layout
        .create_split(
            SplitOrientation::Row,
            vec![col, x],
            SplitSizes::from_weights(vec![0.6, 0.4]).unwrap(),
        )
        .unwrap();
    layout.set_root(Some(root_row)).unwrap();

    // First removal: `inner` collapses into `col`.
    DockingOperation::Remove { dock: p2 }.apply(&mut layout).unwrap();
    assert!(layout.is_valid());
    assert!(!layout.contains(inner));
    assert_eq!(layout.placement(p1), Some(DockSlot::SplitChild { parent: col, index: 0 }));

    // Second removal: `col` collapses into the root slot; its survivor
    // inherits the root slot's weight.
    DockingOperation::Remove { dock: p1 }.apply(&mut layout).unwrap();
    assert!(layout.is_valid());
    assert!(!layout.contains(col));
    assert_eq!(layout.root(), Some(root_row));
    assert_eq!(layout.placement(c), Some(DockSlot::SplitChild { parent: root_row, index: 0 }));
    assert_eq!(
        layout.get(root_row).unwrap().sizes().map(|s| s.weights().to_vec()),
        Some(vec![0.6, 0.4])
    );
}

#[test]
fn remove_from_four_child_split_keeps_order_and_prunes_weight() {
    let mut layout = DockLayout::new();
    let a = layout.create_panel(None);
    let b = layout.create_panel(None);
    let c = layout.create_panel(None);
    let d = layout.create_panel(None);
    layout.set_root(None).unwrap();
    let row = layout
        .create_split(
            SplitOrientation::Row,
            vec![a, b, c, d],
            SplitSizes::from_weights(vec![0.1, 0.2, 0.3, 0.4]).unwrap(),
        )
        .unwrap();
    layout.set_root(Some(row)).unwrap();

    DockingOperation::Remove { dock: b }.apply(&mut layout).unwrap();
    assert!(layout.is_valid());
    // Survivors keep their relative order and shares verbatim: the
    // removed index's weight is deleted, never redistributed (PRD §9.2).
    assert_eq!(layout.descendants(row), vec![a, c, d]);
    assert_eq!(
        layout.get(row).unwrap().sizes().map(|s| s.weights().to_vec()),
        Some(vec![0.1, 0.3, 0.4])
    );
    assert_eq!(layout.root(), Some(row));
}

#[test]
fn remove_tab_member_keeps_order_and_active_member() {
    let mut layout = DockLayout::new();
    let a = layout.create_panel(Some("a".into()));
    let b = layout.create_panel(Some("b".into()));
    let c = layout.create_panel(Some("c".into()));
    layout.set_root(None).unwrap();
    let group = layout.create_tab_group(vec![a, b, c], 2).unwrap();
    layout.set_root(Some(group)).unwrap();

    // Remove a member below the active index: survivors keep their
    // order and the active member keeps its identity.
    DockingOperation::Remove { dock: a }.apply(&mut layout).unwrap();
    assert!(layout.is_valid());
    assert_eq!(layout.descendants(group), vec![b, c]);
    assert_eq!(layout.get(group).unwrap().active(), Some(1));
    assert_eq!(layout.placement(c), Some(DockSlot::TabMember { parent: group, index: 1 }));
}

#[test]
fn remove_last_tab_member_yields_the_explicit_empty_layout() {
    let mut layout = DockLayout::new();
    let a = layout.create_panel(Some("a".into()));
    layout.set_root(None).unwrap(); // detach the auto-rooted panel first
    let group = layout.create_tab_group(vec![a], 0).unwrap();
    layout.set_root(Some(group)).unwrap();
    assert!(layout.is_valid());

    // The final member leaves: the zero-member group resolves (nothing
    // inherits) through the Root slot, yielding the explicit empty
    // layout (PRD §7.4).
    let outcome = DockingOperation::Remove { dock: a }.apply(&mut layout).unwrap();
    let DockingOutcome::Removed { docks, .. } = outcome else { panic!("expected Removed") };
    assert_eq!(docks.len(), 1);
    assert!(layout.is_empty());
    assert_eq!(layout.root(), None);
    assert!(layout.is_valid());
}

// --- `normalize` entry point guarantees ---

#[test]
fn normalize_is_a_no_op_on_valid_layouts() {
    let mut layout = DockLayout::new();
    let a = layout.create_panel(None);
    let b = layout.create_panel(None);
    let c = layout.create_panel(None);
    let d = layout.create_panel(None);
    layout.set_root(None).unwrap();
    let group = layout.create_tab_group(vec![b, c], 0).unwrap();
    let col = layout
        .create_split(
            SplitOrientation::Column,
            vec![a, group, d],
            SplitSizes::from_weights(vec![0.6, 0.3, 0.1]).unwrap(),
        )
        .unwrap();
    layout.set_root(Some(col)).unwrap();
    assert!(layout.is_valid());
    let before = layout.clone();

    layout.normalize().unwrap();
    assert_eq!(layout, before, "a valid layout is already at the fixpoint");
}

#[test]
fn normalize_is_idempotent_after_operations() {
    let mut layout = DockLayout::new();
    let a = layout.create_panel(Some("a".into()));
    let b = layout.create_panel(Some("b".into()));
    let c = layout.create_panel(Some("c".into()));
    layout.set_root(None).unwrap();
    let row = layout
        .create_split(SplitOrientation::Row, vec![a, b, c], SplitSizes::uniform(3).unwrap())
        .unwrap();
    layout.set_root(Some(row)).unwrap();
    DockingOperation::InsertBeside {
        source: InsertSource::NewPanel { content: Some("n".into()) },
        target: c,
        direction: SplitDirection::Right,
    }
    .apply(&mut layout)
    .unwrap();
    DockingOperation::Remove { dock: b }.apply(&mut layout).unwrap();
    assert!(layout.is_valid());

    layout.normalize().unwrap();
    let once = layout.clone();
    layout.normalize().unwrap();
    layout.normalize().unwrap();
    assert_eq!(layout, once, "repeated normalization changes nothing");
}

// --- Atomicity (plan criterion: failure leaves the state unchanged) ---

#[test]
fn failed_operation_leaves_original_state_unchanged() {
    let mut layout = DockLayout::new();
    let a = layout.create_panel(Some("editor".into()));
    let b = layout.create_panel(Some("inspector".into()));
    layout.set_root(None).unwrap();
    let row = layout
        .create_split(SplitOrientation::Row, vec![a, b], SplitSizes::uniform(2).unwrap())
        .unwrap();
    layout.set_root(Some(row)).unwrap();
    let before = layout.clone();

    let err = DockingOperation::Remove { dock: uid(42) }.apply(&mut layout).unwrap_err();
    assert_eq!(err, DockingError::DockNotFound { id: uid(42) });
    // OP-04 / normalization-adjacent criterion: no partial mutation.
    assert_eq!(layout, before);
    assert!(layout.is_valid());
}
