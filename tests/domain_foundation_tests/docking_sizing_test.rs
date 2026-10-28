//! Sizing and resizing tests for the Docking environment
//! (current_state/plan.md Phase 06, PRD §9.1–§9.4, §9.7).
//!
//! Coverage: the stored-weight representation contract (verbatim,
//! pair-conserving resize), the `ResizeSiblings` gesture contract
//! (absolute, drift-free frames; rejection-based constraint policy with
//! frontend-translated minimum bounds), the `SetDockCollapsed`
//! preference (no allocation or structural change, always restorable),
//! and the interplay with the Phase-5 redistribution policy.
//!
//! All fixtures are reachable through the public operation API; imported
//! garbage sizing states are assembled in-crate
//! (`src/.../docking/operations.rs`) via the test-only
//! `SplitSizes::from_weights_unchecked` route (agents/testing.md).

use app_shell::domain_foundation::environments::docking::{
    DockId, DockLayout, DockSlot, DockingError, DockingOperation, DockingOutcome, FormerCollection,
    InsertSource, SplitDirection, SplitOrientation, SplitSizes,
};

/// A two-panel row root with the given weights. Panels `a` (child 0)
/// and `b` (child 1) get ids 0 and 1, the row id 2 (the counter is
/// never renumbered).
fn row_with_weights(w: [f64; 2]) -> (DockLayout, DockId, DockId, DockId) {
    let mut layout = DockLayout::new();
    let a = layout.create_panel(Some("a".into()));
    let b = layout.create_panel(Some("b".into()));
    layout.set_root(None).unwrap();
    let row = layout
        .create_split(
            SplitOrientation::Row,
            vec![a, b],
            SplitSizes::from_weights(w.to_vec()).unwrap(),
        )
        .unwrap();
    layout.set_root(Some(row)).unwrap();
    assert!(layout.is_valid());
    (layout, row, a, b)
}

fn weights_of(layout: &DockLayout, row: DockId) -> Vec<f64> {
    layout.get(row).unwrap().sizes().unwrap().weights().to_vec()
}

// --- ResizeSiblings: pair policy (PRD §9.2, §9.4) ---

#[test]
fn resize_two_children_updates_weights_and_reports_the_previous_pair() {
    let (mut layout, row, a, b) = row_with_weights([0.7, 0.3]);
    let outcome = DockingOperation::ResizeSiblings {
        container: row,
        first: 0,
        requested: 0.5,
        first_min: None,
        second_min: None,
    }
    .apply(&mut layout)
    .unwrap();
    // The pair is rewritten; children and allocation stay coherent.
    assert_eq!(weights_of(&layout, row), vec![0.5, 0.5]);
    assert_eq!(layout.get(row).unwrap().children(), Some(&[a, b][..]));
    let DockingOutcome::SiblingsResized { container, first, previous, .. } = &outcome else {
        panic!("expected SiblingsResized");
    };
    assert_eq!(*container, row);
    assert_eq!(*first, 0);
    // The outcome reports the replaced pair (the `Moved` precedent).
    assert_eq!(*previous, [0.7, 0.3]);
    // The inverse records the split container's whole former state.
    let [record] = outcome.inverse().containers.as_slice() else {
        panic!("expected exactly one former collection");
    };
    match record {
        FormerCollection::Split { id, children, sizes } => {
            assert_eq!(*id, row);
            assert_eq!(children, &vec![a, b]);
            assert_eq!(sizes.weights(), &[0.7, 0.3]);
        }
        other => panic!("expected a split record, got {other:?}"),
    }
}

#[test]
fn resize_pair_conserves_the_total_and_leaves_other_children_verbatim() {
    let mut layout = DockLayout::new();
    let a = layout.create_panel(Some("a".into()));
    let b = layout.create_panel(Some("b".into()));
    let c = layout.create_panel(Some("c".into()));
    let d = layout.create_panel(Some("d".into()));
    layout.set_root(None).unwrap();
    let row = layout
        .create_split(
            SplitOrientation::Row,
            vec![a, b, c, d],
            SplitSizes::from_weights(vec![1.0, 2.0, 3.0, 4.0]).unwrap(),
        )
        .unwrap();
    layout.set_root(Some(row)).unwrap();

    DockingOperation::ResizeSiblings {
        container: row,
        first: 1,
        requested: 4.0,
        first_min: None,
        second_min: None,
    }
    .apply(&mut layout)
    .unwrap();
    // The pair (b, c) is rewritten: b takes 4.0, c absorbs the pair
    // total 2.0 + 3.0 − 4.0; the untouched regions stay verbatim.
    assert_eq!(weights_of(&layout, row), vec![1.0, 4.0, 1.0, 4.0]);
}

#[test]
fn resize_frames_compose_from_the_gesture_initial_state() {
    // §9.4 SHOULD: each frame is derived from the gesture's initial
    // state and total displacement — an absolute requested weight — so
    // frames compose without accumulating rounded deltas. Repeating a
    // stalled frame is a no-op, and the final state equals a single
    // application of the last request.
    let (mut layout, row, _, _) = row_with_weights([2.0, 6.0]);
    for requested in [3.0, 5.0, 5.0] {
        DockingOperation::ResizeSiblings {
            container: row,
            first: 0,
            requested,
            first_min: None,
            second_min: None,
        }
        .apply(&mut layout)
        .unwrap();
    }
    assert_eq!(weights_of(&layout, row), vec![5.0, 3.0]);

    // A layout receiving only the final frame lands identically: the
    // frame count does not matter.
    let (mut fresh, fresh_row, _, _) = row_with_weights([2.0, 6.0]);
    DockingOperation::ResizeSiblings {
        container: fresh_row,
        first: 0,
        requested: 5.0,
        first_min: None,
        second_min: None,
    }
    .apply(&mut fresh)
    .unwrap();
    assert_eq!(fresh, layout);
}

// --- Constraint policy (PRD §9.3): reject, never silently resolve ---

#[test]
fn resize_rejects_a_bound_violation_and_leaves_the_layout_unchanged() {
    let (mut layout, row, _, _) = row_with_weights([0.7, 0.3]);
    let before = layout.clone();
    let err = DockingOperation::ResizeSiblings {
        container: row,
        first: 0,
        requested: 0.2,
        first_min: Some(1.0),
        second_min: None,
    }
    .apply(&mut layout)
    .unwrap_err();
    assert_eq!(err, DockingError::SizeBoundViolation { split: row, index: 0 });
    // Atomicity: the rejected request keeps the original allocation.
    assert_eq!(layout, before);
}

#[test]
fn resize_honors_minimum_endpoints_inclusively() {
    let (mut layout, row, _, _) = row_with_weights([2.0, 6.0]);
    // A bound endpoint itself is a valid allocation.
    DockingOperation::ResizeSiblings {
        container: row,
        first: 0,
        requested: 2.0,
        first_min: Some(2.0),
        second_min: None,
    }
    .apply(&mut layout)
    .unwrap();
    assert_eq!(weights_of(&layout, row), vec![2.0, 6.0]);

    // Same rule for the second region's carried minimum.
    let (mut layout2, row2, _, _) = row_with_weights([2.0, 6.0]);
    DockingOperation::ResizeSiblings {
        container: row2,
        first: 0,
        requested: 2.0,
        first_min: None,
        second_min: Some(6.0),
    }
    .apply(&mut layout2)
    .unwrap();
    assert_eq!(weights_of(&layout2, row2), vec![2.0, 6.0]);
}

#[test]
fn resize_enforces_the_second_region_minimum_with_complement_math() {
    let (mut layout, row, _, _) = row_with_weights([0.7, 0.3]);
    // requested 0.9 leaves the second region 0.1, below its carried
    // 0.25 minimum: rejected, naming the second region.
    let err = DockingOperation::ResizeSiblings {
        container: row,
        first: 0,
        requested: 0.9,
        first_min: None,
        second_min: Some(0.25),
    }
    .apply(&mut layout)
    .unwrap_err();
    assert_eq!(err, DockingError::SizeBoundViolation { split: row, index: 1 });

    // Exactly at the complement boundary succeeds.
    DockingOperation::ResizeSiblings {
        container: row,
        first: 0,
        requested: 0.75,
        first_min: None,
        second_min: Some(0.25),
    }
    .apply(&mut layout)
    .unwrap();
    assert_eq!(weights_of(&layout, row), vec![0.75, 0.25]);
}

#[test]
fn resize_rejects_invalid_size_values() {
    let (mut layout, row, _, _) = row_with_weights([0.7, 0.3]);
    for requested in [0.0, -0.5, f64::NAN, f64::INFINITY] {
        let before = layout.clone();
        let err = DockingOperation::ResizeSiblings {
            container: row,
            first: 0,
            requested,
            first_min: None,
            second_min: None,
        }
        .apply(&mut layout)
        .unwrap_err();
        // No bounds were carried: the sizing invariant itself is the
        // bound (finite, strictly positive).
        assert_eq!(err, DockingError::SizeBoundViolation { split: row, index: 0 });
        assert_eq!(layout, before);
    }
}

#[test]
fn resize_names_the_first_checked_region_deterministically() {
    // Both regions violate at once: the pair is checked in ascending
    // child order, so the first region is named.
    let (mut layout, row, _, _) = row_with_weights([0.7, 0.3]);
    let err = DockingOperation::ResizeSiblings {
        container: row,
        first: 0,
        requested: 0.2, // first < 1.0 and second (0.8) < 1.0
        first_min: Some(1.0),
        second_min: Some(1.0),
    }
    .apply(&mut layout)
    .unwrap_err();
    assert_eq!(err, DockingError::SizeBoundViolation { split: row, index: 0 });

    // Total available size insufficient for both minimums (PRD §9.4):
    // every request fails — the checked-first violating region is named.
    let (mut layout2, row2, _, _) = row_with_weights([0.5, 0.5]);
    let err = DockingOperation::ResizeSiblings {
        container: row2,
        first: 0,
        requested: 1.0, // first ok, second = 0.0
        first_min: Some(1.0),
        second_min: Some(1.0),
    }
    .apply(&mut layout2)
    .unwrap_err();
    assert_eq!(err, DockingError::SizeBoundViolation { split: row2, index: 1 });
}

// --- Resize preconditions ---

#[test]
fn resize_preconditions_are_reported() {
    let (mut layout, row, a, _) = row_with_weights([0.7, 0.3]);
    // Missing container.
    let err = DockingOperation::ResizeSiblings {
        container: DockId::from_raw(999),
        first: 0,
        requested: 0.5,
        first_min: None,
        second_min: None,
    }
    .apply(&mut layout)
    .unwrap_err();
    assert_eq!(err, DockingError::DockNotFound { id: DockId::from_raw(999) });

    // Not a split container.
    let err = DockingOperation::ResizeSiblings {
        container: a,
        first: 0,
        requested: 0.5,
        first_min: None,
        second_min: None,
    }
    .apply(&mut layout)
    .unwrap_err();
    assert_eq!(err, DockingError::NotASplit { id: a });

    // The pair would run past the child list (the argument is echoed).
    for first in [1, 9] {
        let err = DockingOperation::ResizeSiblings {
            container: row,
            first,
            requested: 0.5,
            first_min: None,
            second_min: None,
        }
        .apply(&mut layout)
        .unwrap_err();
        assert_eq!(err, DockingError::SplitChildIndexOutOfRange { index: first, child_count: 2 });
    }
}

// --- SetDockCollapsed (PRD §9.7) ---

#[test]
fn set_dock_collapsed_toggles_without_touching_allocation() {
    let (mut layout, row, a, _) = row_with_weights([0.7, 0.3]);
    let outcome = DockingOperation::SetDockCollapsed { dock: a, collapsed: true }
        .apply(&mut layout)
        .unwrap();
    let DockingOutcome::DockCollapsedSet { dock, collapsed, previous, .. } = &outcome else {
        panic!("expected DockCollapsedSet");
    };
    assert_eq!(*dock, a);
    assert!(!*previous, "the panel started expanded");
    assert!(*collapsed);
    // Placement-category and allocation data are verbatim: the flag is
    // presentation meaning only.
    assert_eq!(layout.get(a).unwrap().collapsed(), Some(true));
    assert_eq!(layout.get(row).unwrap().collapsed(), None);
    assert_eq!(weights_of(&layout, row), vec![0.7, 0.3]);
    assert!(layout.is_valid());
}

#[test]
fn restore_returns_exactly_the_pre_collapse_layout() {
    let (mut layout, _, a, _) = row_with_weights([0.7, 0.3]);
    let before = layout.clone();
    DockingOperation::SetDockCollapsed { dock: a, collapsed: true }.apply(&mut layout).unwrap();
    DockingOperation::SetDockCollapsed { dock: a, collapsed: false }.apply(&mut layout).unwrap();
    // The weight was the retained restore state: restoring is exact.
    assert_eq!(layout, before);
}

#[test]
fn set_dock_collapsed_matching_value_is_a_no_op_success() {
    let (mut layout, _, b, _) = row_with_weights([0.7, 0.3]);
    let outcome = DockingOperation::SetDockCollapsed { dock: b, collapsed: false }
        .apply(&mut layout)
        .unwrap();
    let DockingOutcome::DockCollapsedSet { dock, collapsed, previous, .. } = &outcome else {
        panic!("expected DockCollapsedSet");
    };
    assert_eq!(*dock, b);
    assert!(!*collapsed && !*previous);
    // No-op success: the inverse is the identity (nothing recorded).
    assert!(outcome.inverse().containers.is_empty());
    assert!(outcome.inverse().removed.is_empty());
}

#[test]
fn set_dock_collapsed_requires_a_panel() {
    let (mut layout, row, _, _) = row_with_weights([0.7, 0.3]);
    // Non-panel dock (v1 eligible category: panels).
    let err = DockingOperation::SetDockCollapsed { dock: row, collapsed: true }
        .apply(&mut layout)
        .unwrap_err();
    assert_eq!(err, DockingError::IneligibleDock { id: row });
    // Missing dock.
    let err = DockingOperation::SetDockCollapsed { dock: DockId::from_raw(77), collapsed: true }
        .apply(&mut layout)
        .unwrap_err();
    assert_eq!(err, DockingError::DockNotFound { id: DockId::from_raw(77) });
}

#[test]
fn collapse_never_makes_a_dock_unreachable() {
    let (mut layout, row, a, b) = row_with_weights([0.7, 0.3]);
    DockingOperation::SetDockCollapsed { dock: a, collapsed: true }.apply(&mut layout).unwrap();

    // Removing the other child collapses the row: the collapsed member
    // inherits the root and keeps its preference — it stays placed,
    // valid, and traversable.
    DockingOperation::Remove { dock: b }.apply(&mut layout).unwrap();
    assert!(layout.is_valid());
    assert_eq!(layout.root(), Some(a));
    assert!(!layout.contains(row));
    assert_eq!(layout.get(a).unwrap().collapsed(), Some(true));
    assert_eq!(layout.placement(a), Some(DockSlot::Root));

    // Whole-layout maintenance stays a no-op on the committed result.
    layout.normalize().unwrap();
    assert_eq!(layout.len(), 1);
}

#[test]
fn resize_during_collapse_retains_the_live_allocation() {
    // Collapse retains the dock's *current* weight as the restore state:
    // a resize applied while the dock is collapsed changes what the
    // restore brings back (the environment never snapshot-allocates).
    let (mut layout, row, _, b) = row_with_weights([0.7, 0.3]);
    DockingOperation::SetDockCollapsed { dock: b, collapsed: true }.apply(&mut layout).unwrap();
    DockingOperation::ResizeSiblings {
        container: row,
        first: 0,
        requested: 0.5,
        first_min: None,
        second_min: None,
    }
    .apply(&mut layout)
    .unwrap();
    assert_eq!(weights_of(&layout, row), vec![0.5, 0.5]);

    DockingOperation::SetDockCollapsed { dock: b, collapsed: false }.apply(&mut layout).unwrap();
    assert_eq!(weights_of(&layout, row), vec![0.5, 0.5]);
    assert_eq!(layout.get(b).unwrap().collapsed(), Some(false));
    assert!(layout.is_valid());
}

#[test]
fn resize_survives_structural_movement_of_the_pair() {
    // §18.4: preservation of sizing preferences after movement — the
    // resized pair keeps its shares through an insert-beside move of a
    // sibling (the insert copies the target's weight; the pair stays
    // verbatim) and through undo restoration elsewhere.
    let (mut layout, row, _, b) = row_with_weights([0.7, 0.3]);
    DockingOperation::ResizeSiblings {
        container: row,
        first: 0,
        requested: 0.5,
        first_min: None,
        second_min: None,
    }
    .apply(&mut layout)
    .unwrap();

    DockingOperation::InsertBeside {
        source: InsertSource::NewPanel { content: Some("n".into()) },
        target: b,
        direction: SplitDirection::Right,
    }
    .apply(&mut layout)
    .unwrap();
    // b's weight (0.5) was copied for the new neighbor; a's share is
    // verbatim (PRD §9.2 baseline + Phase-6 resize policy).
    assert_eq!(weights_of(&layout, row), vec![0.5, 0.5, 0.5]);
}
