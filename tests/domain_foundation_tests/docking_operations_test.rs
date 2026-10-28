//! Operation tests for the Docking environment
//! (current_state/plan.md Phase 04).
//!
//! Coverage: creation semantics (empty-layout auto-rooting, atomic
//! rejection on populated layouts), insert-beside (reused compatible
//! containers, wraps around incompatible placements, re-rooting, identity
//! preservation on moves), tab grouping (create-or-join, active
//! selection, no-op re-join), tab reorder/activation, removal with the
//! mandatory collapse cascade, and atomic rejection of invalid requests
//! (OP-01…OP-04: a failed operation leaves the layout untouched).
//!
//! Undo round-trips live in-crate (`src/.../docking/operations.rs`,
//! `replay_into` seam). Occupied host regions and floating collections
//! are likewise exercised in-crate: public operations create those
//! placements from Phases 7–8 on, and the `assign` surface is
//! `pub(crate)`.

use app_shell::domain_foundation::environments::docking::{
    DockKind, DockSlot, DockingError, DockingOperation, DockingOutcome, DockLayout, InsertSource,
    SplitDirection, SplitOrientation, SplitSizes,
};

fn uid(n: u64) -> app_shell::domain_foundation::environments::docking::DockId {
    app_shell::domain_foundation::environments::docking::DockId::from_raw(n)
}

fn new_panel(content: &str) -> InsertSource {
    InsertSource::NewPanel { content: Some(content.into()) }
}

// --- Creation (PRD §8.3) ---

#[test]
fn create_panel_on_empty_layout_auto_roots() {
    let mut layout = DockLayout::new();
    let outcome = DockingOperation::CreatePanel { content: Some("editor".into()) }
        .apply(&mut layout)
        .unwrap();
    let DockingOutcome::Created { dock, is_root, .. } = outcome else {
        panic!("expected Created");
    };
    assert!(is_root);
    assert_eq!(layout.root(), Some(dock));
    assert!(layout.is_valid());
    assert_eq!(layout.placement(dock), Some(DockSlot::Root));
}

#[test]
fn create_on_populated_layout_is_rejected_atomically() {
    let mut layout = DockLayout::new();
    let first = layout.create_panel(None);
    assert!(layout.is_valid());
    let before = layout.clone();

    // The created panel would hold no placement; the Phase-3 gate
    // rejects the candidate with the id it allocated.
    let err = DockingOperation::CreatePanel { content: None }
        .apply(&mut layout)
        .unwrap_err();
    assert_eq!(err, DockingError::OrphanDock { id: uid(first.as_raw() + 1) });
    // OP-01: the failed creation is fully discarded.
    assert_eq!(layout, before);
}

#[test]
fn create_embed_host_on_empty_layout_auto_roots() {
    let mut layout = DockLayout::new();
    let outcome = DockingOperation::CreateEmbedHost { content: Some("workspace".into()) }
        .apply(&mut layout)
        .unwrap();
    let DockingOutcome::Created { dock, is_root, .. } = outcome else {
        panic!("expected Created");
    };
    assert!(is_root);
    assert_eq!(layout.kind_of(dock), Some(DockKind::EmbedHost));
    assert!(layout.is_valid());
}

// --- InsertBeside (PRD §8.6) ---

#[test]
fn insert_beside_single_panel_re_roots_into_split() {
    let mut layout = DockLayout::new();
    let root = layout.create_panel(Some("root".into()));

    let outcome = DockingOperation::InsertBeside {
        source: new_panel("right"),
        target: root,
        direction: SplitDirection::Right,
    }
    .apply(&mut layout)
    .unwrap();
    let DockingOutcome::InsertedBeside { source, container, .. } = outcome else {
        panic!("expected InsertedBeside");
    };
    assert!(layout.is_valid());
    assert_eq!(layout.root(), Some(container));
    assert_eq!(layout.get(container).unwrap().orientation(), Some(SplitOrientation::Row));
    assert_eq!(layout.placement(root), Some(DockSlot::SplitChild { parent: container, index: 0 }));
    assert_eq!(layout.placement(source), Some(DockSlot::SplitChild { parent: container, index: 1 }));

    // Inserting on the left flips the order.
    let outcome = DockingOperation::InsertBeside {
        source: new_panel("left"),
        target: root,
        direction: SplitDirection::Left,
    }
    .apply(&mut layout)
    .unwrap();
    let DockingOutcome::InsertedBeside { source, container, .. } = outcome else {
        panic!("expected InsertedBeside");
    };
    assert_eq!(layout.get(container).unwrap().orientation(), Some(SplitOrientation::Row));
    assert_eq!(layout.placement(source), Some(DockSlot::SplitChild { parent: container, index: 0 }));
    assert_eq!(layout.placement(root), Some(DockSlot::SplitChild { parent: container, index: 1 }));
}

#[test]
fn insert_beside_reuses_container_copies_weight() {
    let mut layout = DockLayout::new();
    let a = layout.create_panel(None);
    let b = layout.create_panel(None);
    layout.set_root(None).unwrap();
    let row = layout
        .create_split(SplitOrientation::Row, vec![a, b], SplitSizes::uniform(2).unwrap())
        .unwrap();
    layout.set_root(Some(row)).unwrap();

    let before = layout.clone();
    let outcome = DockingOperation::InsertBeside {
        source: new_panel("c"),
        target: a,
        direction: SplitDirection::Left,
    }
    .apply(&mut layout)
    .unwrap();
    let DockingOutcome::InsertedBeside { source, container, .. } = outcome else {
        panic!("expected InsertedBeside");
    };
    assert_eq!(container, row);
    assert_ne!(layout, before);
    assert_eq!(layout.descendants(row), vec![source, a, b]);
    // Sizing baseline: the new child copies the target's weight.
    assert_eq!(layout.get(row).unwrap().sizes().map(|s| s.weights().to_vec()), Some(vec![1.0, 1.0, 1.0]));
}

#[test]
fn insert_beside_along_incompatible_direction_wraps_target() {
    let mut layout = DockLayout::new();
    let a = layout.create_panel(None);
    let b = layout.create_panel(None);
    layout.set_root(None).unwrap();
    let row = layout
        .create_split(SplitOrientation::Row, vec![a, b], SplitSizes::uniform(2).unwrap())
        .unwrap();
    layout.set_root(Some(row)).unwrap();

    // Top on a row child: a column wraps the target and inherits its
    // child slot inside the row (orthogonal nesting, PRD §8.6).
    let outcome = DockingOperation::InsertBeside {
        source: new_panel("top"),
        target: a,
        direction: SplitDirection::Top,
    }
    .apply(&mut layout)
    .unwrap();
    let DockingOutcome::InsertedBeside { source, container, .. } = outcome else {
        panic!("expected InsertedBeside");
    };
    assert!(layout.is_valid());
    assert_ne!(container, row);
    assert_eq!(layout.get(container).unwrap().orientation(), Some(SplitOrientation::Column));
    assert_eq!(layout.descendants(container), vec![source, a]);
    // The row keeps its children; a now sits inside the fresh column.
    assert_eq!(layout.descendants(row), vec![container, b, source, a]);
    assert_eq!(layout.root(), Some(row));
}

#[test]
fn insert_beside_moves_existing_dock_preserving_identity() {
    let mut layout = DockLayout::new();
    let a = layout.create_panel(Some("a".into()));
    let b = layout.create_panel(Some("b".into()));
    layout.set_root(None).unwrap();
    let row = layout
        .create_split(SplitOrientation::Row, vec![a, b], SplitSizes::uniform(2).unwrap())
        .unwrap();
    layout.set_root(Some(row)).unwrap();

    let outcome = DockingOperation::InsertBeside {
        source: InsertSource::Existing(a),
        target: b,
        direction: SplitDirection::Right,
    }
    .apply(&mut layout)
    .unwrap();
    let DockingOutcome::InsertedBeside { source, target, .. } = outcome else {
        panic!("expected InsertedBeside");
    };
    assert_eq!(source, a);
    assert_eq!(target, b);
    // Identity preserved (docks are not recreated on moves); content
    // preserved; order flipped to [b, a].
    assert_eq!(layout.descendants(row), vec![b, a]);
    assert_eq!(layout.get(a).unwrap().content().map(|c| c.as_str()), Some("a"));
    assert!(layout.is_descendant(a, row));
    // The row stayed two-child: the detach and insert cancel out.
    assert!(layout.contains(row));
    assert_eq!(layout.root(), Some(row));
}

#[test]
fn insert_beside_moves_whole_container_beside_target() {
    let mut layout = DockLayout::new();
    let a = layout.create_panel(None);
    let b = layout.create_panel(None);
    let x = layout.create_panel(None);
    layout.set_root(None).unwrap();
    let inner = layout
        .create_split(SplitOrientation::Row, vec![a, b], SplitSizes::uniform(2).unwrap())
        .unwrap();
    let outer = layout
        .create_split(SplitOrientation::Row, vec![inner, x], SplitSizes::uniform(2).unwrap())
        .unwrap();
    layout.set_root(Some(outer)).unwrap();

    // Move the whole inner split beside x: a container as source keeps
    // its identity and its subtree intact (PRD §8.5).
    let outcome = DockingOperation::InsertBeside {
        source: InsertSource::Existing(inner),
        target: x,
        direction: SplitDirection::Left,
    }
    .apply(&mut layout)
    .unwrap();
    let DockingOutcome::InsertedBeside { source, .. } = outcome else {
        panic!("expected InsertedBeside");
    };
    assert_eq!(source, inner);
    assert!(layout.is_valid());
    // Detach shrunk outer to one child; the insertion refilled it —
    // order back to [inner, x] with the subtree intact.
    assert!(layout.contains(inner));
    assert_eq!(layout.descendants(outer), vec![inner, x, a, b]);
    assert_eq!(layout.root(), Some(outer));
}

#[test]
fn insert_beside_rejects_missing_targets_and_sources() {
    let mut layout = DockLayout::new();
    let a = layout.create_panel(None);
    let before = layout.clone();

    let err = DockingOperation::InsertBeside {
        source: new_panel("x"),
        target: uid(42),
        direction: SplitDirection::Right,
    }
    .apply(&mut layout)
    .unwrap_err();
    assert_eq!(err, DockingError::DockNotFound { id: uid(42) });
    assert_eq!(layout, before);

    let err = DockingOperation::InsertBeside {
        source: InsertSource::Existing(uid(42)),
        target: a,
        direction: SplitDirection::Right,
    }
    .apply(&mut layout)
    .unwrap_err();
    assert_eq!(err, DockingError::DockNotFound { id: uid(42) });
    assert_eq!(layout, before);
}

#[test]
fn insert_beside_rejects_self_and_descendant_targets() {
    let mut layout = DockLayout::new();
    let a = layout.create_panel(None);
    let b = layout.create_panel(None);
    let x = layout.create_panel(None);
    layout.set_root(None).unwrap();
    let inner = layout
        .create_split(SplitOrientation::Row, vec![a, b], SplitSizes::uniform(2).unwrap())
        .unwrap();
    let outer = layout
        .create_split(SplitOrientation::Row, vec![inner, x], SplitSizes::uniform(2).unwrap())
        .unwrap();
    layout.set_root(Some(outer)).unwrap();
    let before = layout.clone();

    // Self-insertion.
    let err = DockingOperation::InsertBeside {
        source: InsertSource::Existing(a),
        target: a,
        direction: SplitDirection::Right,
    }
    .apply(&mut layout)
    .unwrap_err();
    assert_eq!(err, DockingError::CycleDetected { id: a });
    assert_eq!(layout, before);

    // Target inside the source's subtree: the fresh container would sit
    // inside the moved source itself.
    let err = DockingOperation::InsertBeside {
        source: InsertSource::Existing(outer),
        target: a,
        direction: SplitDirection::Right,
    }
    .apply(&mut layout)
    .unwrap_err();
    assert_eq!(err, DockingError::CycleDetected { id: outer });
    assert_eq!(layout, before);
}

#[test]
fn insert_beside_wraps_root_target_following_direction() {
    let mut layout = DockLayout::new();
    let root = layout.create_panel(Some("root".into()));

    // Top on the root: column split with the source ABOVE the root.
    let outcome = DockingOperation::InsertBeside {
        source: new_panel("top"),
        target: root,
        direction: SplitDirection::Top,
    }
    .apply(&mut layout)
    .unwrap();
    let DockingOutcome::InsertedBeside { source, container, .. } = outcome else {
        panic!("expected InsertedBeside");
    };
    assert!(layout.is_valid());
    assert_eq!(layout.get(container).unwrap().orientation(), Some(SplitOrientation::Column));
    assert_eq!(layout.root(), Some(container));
    assert_eq!(layout.placement(source), Some(DockSlot::SplitChild { parent: container, index: 0 }));
    assert_eq!(layout.placement(root), Some(DockSlot::SplitChild { parent: container, index: 1 }));
}

// --- TabJoin (PRD §8.7) ---

#[test]
fn tab_join_creates_group_inheriting_target_slot() {
    let mut layout = DockLayout::new();
    let root = layout.create_panel(Some("a".into()));

    let outcome = DockingOperation::TabJoin { source: new_panel("b"), target: root }
        .apply(&mut layout)
        .unwrap();
    let DockingOutcome::TabJoined { source, group, .. } = outcome else {
        panic!("expected TabJoined");
    };
    assert!(layout.is_valid());
    assert_eq!(layout.kind_of(group), Some(DockKind::TabGroup));
    assert_eq!(layout.root(), Some(group));
    assert_eq!(layout.placement(root), Some(DockSlot::TabMember { parent: group, index: 0 }));
    assert_eq!(layout.placement(source), Some(DockSlot::TabMember { parent: group, index: 1 }));
    // Baseline: the joined member is active.
    assert_eq!(layout.get(group).unwrap().active(), Some(1));
}

#[test]
fn tab_join_joins_existing_group_and_activates_new_member() {
    let mut layout = DockLayout::new();
    let a = layout.create_panel(None);
    let b = layout.create_panel(None);
    layout.set_root(None).unwrap();
    let group = layout.create_tab_group(vec![a, b], 0).unwrap();
    layout.set_root(Some(group)).unwrap();

    let outcome = DockingOperation::TabJoin { source: new_panel("c"), target: a }
        .apply(&mut layout)
        .unwrap();
    let DockingOutcome::TabJoined { group: g, .. } = outcome else {
        panic!("expected TabJoined");
    };
    assert_eq!(g, group);
    assert!(layout.is_valid());
    // Tab order: target list unchanged plus the new member appended.
    assert_eq!(layout.descendants(group).len(), 3);
    assert_eq!(layout.placement(a), Some(DockSlot::TabMember { parent: group, index: 0 }));
    assert_eq!(layout.placement(b), Some(DockSlot::TabMember { parent: group, index: 1 }));
    // Active = the joined member's index.
    assert_eq!(layout.get(group).unwrap().active(), Some(2));
}

#[test]
fn tab_join_of_an_existing_member_is_a_no_op() {
    let mut layout = DockLayout::new();
    let a = layout.create_panel(None);
    let b = layout.create_panel(None);
    let c = layout.create_panel(None);
    layout.set_root(None).unwrap();
    let group = layout.create_tab_group(vec![a, b, c], 0).unwrap();
    layout.set_root(Some(group)).unwrap();
    let before = layout.clone();

    let outcome = DockingOperation::TabJoin { source: InsertSource::Existing(b), target: a }
        .apply(&mut layout)
        .unwrap();
    let DockingOutcome::TabJoined { group: g, .. } = outcome else {
        panic!("expected TabJoined");
    };
    assert_eq!(g, group);
    // Already a member: no structural change; the active member stays.
    assert_eq!(layout, before);
    assert_eq!(layout.get(group).unwrap().active(), Some(0));
}

#[test]
fn tab_join_detached_source_collapses_former_parent() {
    let mut layout = DockLayout::new();
    let a = layout.create_panel(None);
    let b = layout.create_panel(None);
    layout.set_root(None).unwrap();
    let row = layout
        .create_split(SplitOrientation::Row, vec![a, b], SplitSizes::uniform(2).unwrap())
        .unwrap();
    layout.set_root(Some(row)).unwrap();

    // Move b out of the two-child row into a new group around a: the
    // emptied row collapses, its surviving child (the fresh group) takes
    // the root slot.
    let outcome = DockingOperation::TabJoin { source: InsertSource::Existing(b), target: a }
        .apply(&mut layout)
        .unwrap();
    let DockingOutcome::TabJoined { group, .. } = outcome else {
        panic!("expected TabJoined");
    };
    assert!(layout.is_valid());
    assert_eq!(layout.root(), Some(group));
    assert!(!layout.contains(row));
    assert_eq!(layout.placement(b), Some(DockSlot::TabMember { parent: group, index: 1 }));
    assert_eq!(layout.get(group).unwrap().active(), Some(1));
}

#[test]
fn tab_join_rejects_missing_ineligible_and_cyclic() {
    let mut layout = DockLayout::new();
    let a = layout.create_panel(None);
    let b = layout.create_panel(None);
    let x = layout.create_panel(None);
    layout.set_root(None).unwrap();
    let row = layout
        .create_split(SplitOrientation::Row, vec![a, b], SplitSizes::uniform(2).unwrap())
        .unwrap();
    let outer = layout
        .create_split(SplitOrientation::Row, vec![row, x], SplitSizes::uniform(2).unwrap())
        .unwrap();
    layout.set_root(Some(outer)).unwrap();
    let before = layout.clone();

    let err = DockingOperation::TabJoin { source: new_panel("y"), target: uid(42) }
        .apply(&mut layout)
        .unwrap_err();
    assert_eq!(err, DockingError::DockNotFound { id: uid(42) });
    assert_eq!(layout, before);

    // Containers are not v1 tab members (neither target nor source).
    let err = DockingOperation::TabJoin { source: new_panel("y"), target: row }
        .apply(&mut layout)
        .unwrap_err();
    assert_eq!(err, DockingError::IneligibleDock { id: row });
    assert_eq!(layout, before);

    let err = DockingOperation::TabJoin { source: InsertSource::Existing(row), target: a }
        .apply(&mut layout)
        .unwrap_err();
    assert_eq!(err, DockingError::IneligibleDock { id: row });
    assert_eq!(layout, before);

    // Cycle: a panel cannot be its own join target.
    let err = DockingOperation::TabJoin { source: InsertSource::Existing(a), target: a }
        .apply(&mut layout)
        .unwrap_err();
    assert_eq!(err, DockingError::CycleDetected { id: a });
    assert_eq!(layout, before);
}

// --- ReorderTab / SetActiveTab ---

fn tab_layout(
    active: usize,
) -> (
    DockLayout,
    app_shell::domain_foundation::environments::docking::DockId,
) {
    let mut layout = DockLayout::new();
    let a = layout.create_panel(Some("a".into()));
    let b = layout.create_panel(Some("b".into()));
    let c = layout.create_panel(Some("c".into()));
    layout.set_root(None).unwrap();
    let group = layout.create_tab_group(vec![a, b, c], active).unwrap();
    layout.set_root(Some(group)).unwrap();
    (layout, group)
}

#[test]
fn reorder_tab_moves_member_and_active_selection_follows() {
    // Active member is at index 0; move it to the end: the selection
    // moves with the member.
    let (mut layout, group) = tab_layout(0);
    let outcome = DockingOperation::ReorderTab { group, from: 0, to: 2 }
        .apply(&mut layout)
        .unwrap();
    let DockingOutcome::TabReordered { from, to, .. } = outcome else {
        panic!("expected TabReordered");
    };
    assert_eq!(from, 0);
    assert_eq!(to, 2);
    assert!(layout.is_valid());
    assert_eq!(layout.get(group).unwrap().active(), Some(2));

    // Move a non-active member across the active one: the active member
    // keeps its identity.
    let outcome = DockingOperation::ReorderTab { group, from: 2, to: 0 }
        .apply(&mut layout)
        .unwrap();
    assert!(matches!(outcome, DockingOutcome::TabReordered { .. }));
    assert_eq!(layout.get(group).unwrap().active(), Some(0));
}

#[test]
fn reorder_tab_noop_and_rejections() {
    let (mut layout, group) = tab_layout(0);
    let before = layout.clone();

    // No-op at the same index.
    let outcome = DockingOperation::ReorderTab { group, from: 1, to: 1 }
        .apply(&mut layout)
        .unwrap();
    assert!(matches!(outcome, DockingOutcome::TabReordered { .. }));
    assert_eq!(layout, before);

    // Out-of-range args follow the Layers sibling-index convention:
    // validated against the pre-removal member count.
    let err = DockingOperation::ReorderTab { group, from: 3, to: 0 }
        .apply(&mut layout)
        .unwrap_err();
    assert_eq!(err, DockingError::TabIndexOutOfRange { index: 3, member_count: 3 });
    assert_eq!(layout, before);

    let err = DockingOperation::ReorderTab { group, from: 0, to: 3 }
        .apply(&mut layout)
        .unwrap_err();
    assert_eq!(err, DockingError::TabIndexOutOfRange { index: 3, member_count: 3 });
    assert_eq!(layout, before);

    // Unknown group.
    let err = DockingOperation::ReorderTab { group: uid(123), from: 0, to: 0 }
        .apply(&mut layout)
        .unwrap_err();
    assert_eq!(err, DockingError::DockNotFound { id: uid(123) });

    // Not a tab group.
    let mut other = DockLayout::new();
    let panel = other.create_panel(None);
    let err = DockingOperation::ReorderTab { group: panel, from: 0, to: 0 }
        .apply(&mut other)
        .unwrap_err();
    assert_eq!(err, DockingError::NotATabGroup { id: panel });
}

#[test]
fn set_active_tab_selection_and_rejections() {
    let (mut layout, group) = tab_layout(0);

    let outcome = DockingOperation::SetActiveTab { group, index: 2 }
        .apply(&mut layout)
        .unwrap();
    let DockingOutcome::ActiveTabSelected { active, .. } = outcome else {
        panic!("expected ActiveTabSelected");
    };
    assert_eq!(active, 2);
    assert!(layout.is_valid());
    assert_eq!(layout.get(group).unwrap().active(), Some(2));

    let err = DockingOperation::SetActiveTab { group, index: 3 }
        .apply(&mut layout)
        .unwrap_err();
    assert_eq!(err, DockingError::ActiveIndexOutOfRange { index: 3, member_count: 3 });
    assert!(layout.is_valid());

    let mut other = DockLayout::new();
    let host = other.create_embed_host(None);
    let err = DockingOperation::SetActiveTab { group: host, index: 0 }
        .apply(&mut other)
        .unwrap_err();
    assert_eq!(err, DockingError::NotATabGroup { id: host });
}

// --- Remove (PRD §8.11) ---

#[test]
fn remove_last_sibling_of_split_collapses_the_parent() {
    let mut layout = DockLayout::new();
    let a = layout.create_panel(None);
    let b = layout.create_panel(None);
    layout.set_root(None).unwrap();
    let row = layout
        .create_split(SplitOrientation::Row, vec![a, b], SplitSizes::uniform(2).unwrap())
        .unwrap();
    layout.set_root(Some(row)).unwrap();

    let outcome = DockingOperation::Remove { dock: b }.apply(&mut layout).unwrap();
    let DockingOutcome::Removed { docks, .. } = outcome else {
        panic!("expected Removed");
    };
    assert!(layout.is_valid());
    // BFS order: exactly the removed dock.
    assert_eq!(docks.len(), 1);
    assert_eq!(docks[0].id, b);
    // The row collapsed; its survivor inherited the root slot.
    assert_eq!(layout.root(), Some(a));
    assert!(!layout.contains(row));
    assert_eq!(layout.placement(a), Some(DockSlot::Root));
    assert_eq!(layout.len(), 1);
}

#[test]
fn remove_middle_panel_keeps_surviving_order() {
    let mut layout = DockLayout::new();
    let a = layout.create_panel(None);
    let b = layout.create_panel(None);
    let c = layout.create_panel(None);
    layout.set_root(None).unwrap();
    let row = layout
        .create_split(SplitOrientation::Row, vec![a, b, c], SplitSizes::uniform(3).unwrap())
        .unwrap();
    layout.set_root(Some(row)).unwrap();

    let outcome = DockingOperation::Remove { dock: b }.apply(&mut layout).unwrap();
    assert!(matches!(outcome, DockingOutcome::Removed { .. }));
    // Surviving order [a, c]; relative shares preserved by deleting the
    // removed child's weight (PRD §9.2 baseline).
    assert_eq!(layout.descendants(row), vec![a, c]);
    assert!(layout.is_valid());
    assert_eq!(layout.get(row).unwrap().sizes().map(|s| s.weights().to_vec()), Some(vec![1.0, 1.0]));
}

#[test]
fn remove_tab_member_retains_group_and_adjusts_active() {
    let mut layout = DockLayout::new();
    let a = layout.create_panel(None);
    let b = layout.create_panel(None);
    let c = layout.create_panel(None);
    layout.set_root(None).unwrap();
    let group = layout.create_tab_group(vec![a, b, c], 2).unwrap();
    layout.set_root(Some(group)).unwrap();

    // Remove the active member: the selection clamps to the last member
    // and the group survives (PRD §8.11 (c), §8.7 active validity).
    let outcome = DockingOperation::Remove { dock: c }.apply(&mut layout).unwrap();
    let DockingOutcome::Removed { .. } = outcome else { panic!("expected Removed") };
    assert!(layout.is_valid());
    assert!(layout.contains(group));
    assert_eq!(layout.get(group).unwrap().active(), Some(1));
    assert_eq!(layout.root(), Some(group));

    // Remove a member below the active index: the index shifts down.
    let outcome = DockingOperation::Remove { dock: a }.apply(&mut layout).unwrap();
    assert!(matches!(outcome, DockingOutcome::Removed { .. }));
    assert_eq!(layout.get(group).unwrap().active(), Some(0));
}

#[test]
fn remove_removes_whole_subtree_in_bfs_order() {
    let mut layout = DockLayout::new();
    let a = layout.create_panel(None);
    let b = layout.create_panel(None);
    let x = layout.create_panel(None);
    layout.set_root(None).unwrap();
    let inner = layout
        .create_split(SplitOrientation::Row, vec![a, b], SplitSizes::uniform(2).unwrap())
        .unwrap();
    let outer = layout
        .create_split(SplitOrientation::Row, vec![inner, x], SplitSizes::uniform(2).unwrap())
        .unwrap();
    layout.set_root(Some(outer)).unwrap();

    // Removing the root removes everything that hangs below it,
    // yielding the explicit empty layout (PRD §7.4).
    let outcome = DockingOperation::Remove { dock: outer }.apply(&mut layout).unwrap();
    let DockingOutcome::Removed { docks, .. } = outcome else {
        panic!("expected Removed");
    };
    assert!(layout.is_empty());
    assert!(layout.is_valid());
    assert_eq!(docks.len(), 5);
    // BFS order: the removed dock first, then descendants.
    assert_eq!(docks[0].id, outer);
    assert_eq!(docks[1].id, inner);
}

#[test]
fn remove_unknown_dock_leaves_state_untouched() {
    let mut layout = DockLayout::new();
    // A non-empty "before" state makes the untouched assertion meaningful.
    layout.create_panel(None);
    let before = layout.clone();

    let err = DockingOperation::Remove { dock: uid(77) }.apply(&mut layout).unwrap_err();
    assert_eq!(err, DockingError::DockNotFound { id: uid(77) });
    assert_eq!(layout, before);
}
