//! Layout-validation tests for the Docking environment
//! (current_state/plan.md Phase 03).
//!
//! External coverage is limited to the rules the public API can actually
//! reach — creation is valid-by-construction and placement mutation is
//! operation territory, so only `MissingRoot` and `OrphanDock` (plus the
//! accepted cases) are constructible here. The remaining invariants
//! (dangling references, multi-owner placements, root-placement rules,
//! cycles, sizing state, deterministic precedence) are covered by the
//! in-crate `#[cfg(test)]` fixtures in
//! `src/domain_foundation/environments/docking/validation.rs` and the
//! serde-gated imported-state fixtures beside them.

use app_shell::domain_foundation::environments::docking::{
    DockSlot, DockingError, DockLayout, SplitOrientation, SplitSizes,
};

#[test]
fn empty_layout_is_valid() {
    let layout = DockLayout::new();
    assert!(layout.is_valid());
    assert_eq!(layout.validate(), Ok(()));
}

#[test]
fn single_panel_layout_is_valid() {
    let mut layout = DockLayout::new();
    let panel = layout.create_panel(Some("editor".into()));
    assert!(layout.is_valid());
    assert_eq!(layout.validate(), Ok(()));
    assert_eq!(layout.root(), Some(panel));
}

#[test]
fn split_layout_is_valid_once_placed() {
    let mut layout = DockLayout::new();
    let a = layout.create_panel(None); // auto-root
    let b = layout.create_panel(None); // unplaced
    layout.set_root(None).unwrap();
    let row = layout
        .create_split(SplitOrientation::Row, vec![a, b], SplitSizes::uniform(2).unwrap())
        .unwrap();
    // Mid-operation build state: docks exist, root still detached.
    assert_eq!(layout.validate(), Err(DockingError::MissingRoot { docks: 3 }));

    layout.set_root(Some(row)).unwrap();
    assert!(layout.is_valid());
    assert_eq!(layout.validate(), Ok(()));
}

#[test]
fn nested_split_layout_is_valid() {
    let mut layout = DockLayout::new();
    let a = layout.create_panel(None);
    let b = layout.create_panel(None);
    layout.set_root(None).unwrap();
    let row = layout
        .create_split(SplitOrientation::Row, vec![a, b], SplitSizes::uniform(2).unwrap())
        .unwrap();

    let x = layout.create_panel(None);
    let y = layout.create_panel(None);
    let inner = layout
        .create_split(
            SplitOrientation::Column,
            vec![x, y],
            SplitSizes::uniform(2).unwrap(),
        )
        .unwrap();

    // Both splits must be detached before the new root consumes them.
    layout.set_root(None).unwrap();
    let outer = layout
        .create_split(
            SplitOrientation::Row,
            vec![row, inner],
            SplitSizes::uniform(2).unwrap(),
        )
        .unwrap();
    layout.set_root(Some(outer)).unwrap();

    assert!(layout.is_valid());
    assert_eq!(layout.validate(), Ok(()));
    assert_eq!(layout.depth(), 3);
    // Derived invariant: root reaches every dock.
    assert_eq!(layout.descendants(outer).len() + 1, layout.len());
}

#[test]
fn cleared_layout_is_valid_again() {
    let mut layout = DockLayout::new();
    let a = layout.create_panel(None);
    let b = layout.create_panel(None);
    layout.set_root(None).unwrap();
    let row = layout
        .create_split(SplitOrientation::Row, vec![a, b], SplitSizes::uniform(2).unwrap())
        .unwrap();
    layout.set_root(Some(row)).unwrap();
    assert!(layout.is_valid());

    layout.clear();
    assert!(layout.is_empty());
    assert!(layout.is_valid());
    assert_eq!(layout.validate(), Ok(()));
}

#[test]
fn docks_without_root_rejected() {
    let mut layout = DockLayout::new();
    let _ = layout.create_panel(None);
    let _ = layout.create_panel(None);
    // A persistent (non-transient) rootless layout with docks is invalid.
    layout.set_root(None).unwrap();
    assert_eq!(
        layout.validate(),
        Err(DockingError::MissingRoot { docks: 2 })
    );
    assert!(!layout.is_valid());
}

#[test]
fn unplaced_dock_rejected() {
    let mut layout = DockLayout::new();
    let _ = layout.create_panel(None); // auto-root
    let straggler = layout.create_panel(None); // created unplaced (op flow places it later)
    assert_eq!(
        layout.validate(),
        Err(DockingError::OrphanDock { id: straggler })
    );
    assert!(!layout.is_valid());
}

#[test]
fn validate_is_a_pure_check() {
    let mut layout = DockLayout::new();
    let a = layout.create_panel(None);
    let b = layout.create_panel(None);
    layout.set_root(None).unwrap();
    let row = layout
        .create_split(SplitOrientation::Row, vec![a, b], SplitSizes::uniform(2).unwrap())
        .unwrap();
    layout.set_root(Some(row)).unwrap();
    let before = layout.clone();

    let _ = layout.validate();
    let _ = layout.is_valid();
    let _ = layout.validate();
    assert_eq!(layout, before);
    assert!(layout.is_valid());
}

#[test]
fn is_valid_matches_validate_result() {
    // Valid state.
    let mut layout = DockLayout::new();
    let a = layout.create_panel(Some("a".into()));
    assert!(layout.is_valid());

    // Rootless state with docks — invalid.
    layout.set_root(None).unwrap();
    assert!(!layout.is_valid());

    // Back to a rootless-but-valid layout: re-designate the dock as the
    // root (set_root(None) leaves the dock unplaced; re-rooting it).
    layout.set_root(Some(a)).unwrap();
    assert!(layout.is_valid());
    assert_eq!(layout.placement(a), Some(DockSlot::Root));
}
