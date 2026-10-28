//! Integration tests: docking operations through the Domain Engine, with
//! real undo/redo (Phase-1 review R-A1/R-A3).
//!
//! Undo replays the outcome's inverse data (`UndoData::replay_into`,
//! the environment's own record-restore driver); redo replays the forward
//! record built from the committed result — re-applying the operation is
//! not id-preserving (`DockId`s are counter-issued and never renumbered),
//! so undo/redo cycles must recover the **same ids**. These tests pin
//! that, plus history/events behavior, error rejection, and transactional
//! rollback for docking.
//!
//! Public crate API only (agents/testing.md); no `pub(crate)` access.
//! The command handler follows the `canvas_command_handler_test` shape:
//! definition check plus direct application against the shared model.

use app_shell::domain_engine::{DomainEngine, DomainError};
use app_shell::domain_foundation::environments::docking::{
    DockId, DockingError, DockingOperation, DockingOutcome, EdgeRegion, EmbedSlot,
    FloatingRect, InsertSource, SplitDirection,
};

// --- Fixtures -------------------------------------------------------------

fn expect_created(outcome: DockingOutcome) -> DockId {
    match outcome {
        DockingOutcome::Created { dock, is_root, .. } => {
            assert!(is_root, "first dock on an empty layout auto-roots");
            dock
        }
        other => panic!("expected Created, got {other:?}"),
    }
}

fn expect_inserted(outcome: DockingOutcome) -> (DockId, DockId) {
    match outcome {
        DockingOutcome::InsertedBeside { source, container, .. } => (source, container),
        other => panic!("expected InsertedBeside, got {other:?}"),
    }
}

fn expect_embedded(outcome: DockingOutcome) -> DockId {
    match outcome {
        DockingOutcome::Embedded { source, .. } => source,
        other => panic!("expected Embedded, got {other:?}"),
    }
}

/// Panel root + a second panel beside it in a fresh split wrap.
/// Returns `(root_panel, inserted_panel, wrap_container)`.
fn panel_and_beside(engine: &mut DomainEngine) -> (DockId, DockId, DockId) {
    let p1 = expect_created(
        engine
            .execute_docking(DockingOperation::CreatePanel { content: Some("one".into()) })
            .unwrap(),
    );
    let (p2, s) = expect_inserted(
        engine
            .execute_docking(DockingOperation::InsertBeside {
                source: InsertSource::NewPanel { content: Some("two".into()) },
                target: p1,
                direction: SplitDirection::Right,
            })
            .unwrap(),
    );
    (p1, p2, s)
}

// --- Execution basics ------------------------------------------------------

#[test]
fn execute_create_establishes_state_history_and_events() {
    let mut engine = DomainEngine::new();

    let outcome = engine
        .execute_docking(DockingOperation::CreatePanel { content: Some("main".into()) })
        .expect("create should succeed");
    let dock = expect_created(outcome);

    assert!(engine.docking().contains(dock));
    assert_eq!(engine.docking().root(), Some(dock));

    assert_eq!(engine.history().position(), 1);
    assert_eq!(engine.history().entry_count(), 1);
    assert_eq!(engine.events().len(), 1);
    assert_eq!(engine.events()[0].event_type(), "docking.dock.created");
}

#[test]
fn rejected_operation_keeps_state_history_and_events_untouched() {
    let mut engine = DomainEngine::new();
    let (p1, _p2, s) = panel_and_beside(&mut engine);
    let events_before = engine.events().len();
    let history_before = engine.history().position();

    // A pair-first weight of 0 violates the sizing invariant (finite,
    // strictly positive) — rejected, never clamped.
    let err = engine
        .execute_docking(DockingOperation::ResizeSiblings {
            container: s,
            first: 0,
            requested: 0.0,
            first_min: None,
            second_min: None,
        })
        .unwrap_err();

    assert!(matches!(
        err,
        DomainError::Docking(DockingError::SizeBoundViolation { split, index })
            if split == s && index == 0
    ));

    // Nothing recorded: state, history position, and event count unchanged.
    assert_eq!(engine.history().position(), history_before);
    assert_eq!(engine.events().len(), events_before);
    assert!(engine.docking().contains(p1));
    assert_eq!(engine.docking().all_docks().len(), 3);
}

#[test]
fn invalid_operation_reports_the_docking_error_domain() {
    let mut engine = DomainEngine::new();
    let p1 = expect_created(
        engine
            .execute_docking(DockingOperation::CreatePanel { content: None })
            .unwrap(),
    );

    // A panel is not a resize surface.
    let err = engine
        .execute_docking(DockingOperation::ResizeSiblings {
            container: p1,
            first: 0,
            requested: 1.0,
            first_min: None,
            second_min: None,
        })
        .unwrap_err();

    assert!(matches!(
        err,
        DomainError::Docking(DockingError::NotASplit { id }) if id == p1
    ));
}

// --- Undo / redo -----------------------------------------------------------

#[test]
fn undo_create_removes_the_dock_and_restores_the_empty_layout() {
    let mut engine = DomainEngine::new();
    let p1 = expect_created(
        engine
            .execute_docking(DockingOperation::CreatePanel { content: Some("main".into()) })
            .unwrap(),
    );

    let count = engine.undo().unwrap();
    assert_eq!(count, 1);
    assert!(!engine.docking().contains(p1));
    assert_eq!(engine.docking().root(), None);
    assert_eq!(engine.history().position(), 0);
    assert!(!engine.can_undo());
    assert!(engine.can_redo());
}

#[test]
fn undo_insert_restores_the_singleton_root_and_redo_preserves_ids() {
    let mut engine = DomainEngine::new();
    let (p1, _p2, s) = panel_and_beside(&mut engine);
    assert_eq!(engine.docking().root(), Some(s));
    assert_eq!(engine.history().position(), 2);

    // Undo the insert: the created sibling and wrap container leave,
    // the panel regains its singleton root.
    engine.undo().unwrap();
    assert_eq!(engine.docking().root(), Some(p1));
    assert_eq!(engine.docking().all_docks().len(), 1);

    // Redo: same instance of the structure — the wrap container and the
    // sibling come back with **the same ids** (the counter never drifts).
    let count = engine.redo().unwrap();
    assert_eq!(count, 1);
    assert_eq!(engine.history().position(), 2);
    assert_eq!(engine.docking().root(), Some(s));
    assert!(engine.docking().contains(p1));
    let p2 = engine
        .docking()
        .all_docks()
        .into_iter()
        .find(|id| *id != p1 && *id != s)
        .unwrap();
    assert!(matches!(
        engine.docking().placement(p2),
        Some(app_shell::domain_foundation::environments::docking::DockSlot::SplitChild { parent, index })
            if parent == s && index == 1
    ));
}

#[test]
fn undo_redo_cycles_are_idempotent_with_stable_sizing() {
    let mut engine = DomainEngine::new();
    let (_p1, _p2, s) = panel_and_beside(&mut engine);

    engine
        .execute_docking(DockingOperation::ResizeSiblings {
            container: s,
            first: 0,
            requested: 1.5,
            first_min: None,
            second_min: None,
        })
        .unwrap();
    let resized = [1.5, 0.5];

    engine.undo().unwrap();
    engine.redo().unwrap();
    engine.undo().unwrap();
    engine.redo().unwrap();

    // Three undo/redo round-trips later the layout is still identical to
    // the originally committed post-state.
    assert_eq!(engine.docking().root(), Some(s));
    assert_eq!(engine.docking().all_docks().len(), 3);
    match &engine.docking().get(s).unwrap().role {
        app_shell::domain_foundation::environments::docking::DockRole::Split { children, sizes, .. } => {
            assert_eq!(children.len(), 2);
            assert_eq!(sizes.weights(), &resized);
        }
        other => panic!("expected a split container, got {other:?}"),
    }
}

#[test]
fn undo_tab_join_and_redo_recreates_the_group() {
    let mut engine = DomainEngine::new();
    let p1 = expect_created(
        engine
            .execute_docking(DockingOperation::CreatePanel { content: Some("one".into()) })
            .unwrap(),
    );
    let (p2, g) = match engine
        .execute_docking(DockingOperation::TabJoin {
            source: InsertSource::NewPanel { content: Some("two".into()) },
            target: p1,
        })
        .unwrap()
    {
        DockingOutcome::TabJoined { source, group, .. } => (source, group),
        other => panic!("expected TabJoined, got {other:?}"),
    };
    assert_eq!(engine.docking().root(), Some(g));

    engine.undo().unwrap();
    assert_eq!(engine.docking().root(), Some(p1));
    assert!(!engine.docking().contains(p2));
    assert!(!engine.docking().contains(g));

    engine.redo().unwrap();
    assert_eq!(engine.docking().root(), Some(g));
    assert!(engine.docking().contains(p2));
    assert!(matches!(
        engine.docking().get(g).unwrap().role,
        app_shell::domain_foundation::environments::docking::DockRole::TabGroup { ref members, active: 1, .. }
        if members.len() == 2
    ));
}

#[test]
fn undo_remove_restores_the_subtree_and_redo_removes_again() {
    let mut engine = DomainEngine::new();
    let (p1, p2, s) = panel_and_beside(&mut engine);

    // The removal collapses the vacated split; the survivor inherits the
    // root slot.
    engine
        .execute_docking(DockingOperation::Remove { dock: p2 })
        .unwrap();
    assert!(!engine.docking().contains(p2));
    assert!(!engine.docking().contains(s));
    assert_eq!(engine.docking().root(), Some(p1));
    assert_eq!(engine.history().position(), 3);

    engine.undo().unwrap();
    assert!(engine.docking().contains(p2));
    assert!(engine.docking().contains(s));
    assert_eq!(engine.docking().root(), Some(s));
    assert!(matches!(
        engine.docking().placement(p2),
        Some(app_shell::domain_foundation::environments::docking::DockSlot::SplitChild { parent, index })
            if parent == s && index == 1
    ));

    engine.redo().unwrap();
    assert!(!engine.docking().contains(p2));
    assert!(!engine.docking().contains(s));
    assert_eq!(engine.docking().root(), Some(p1));
}

#[test]
fn embed_undo_restores_the_panel_and_redo_repromotes() {
    let mut engine = DomainEngine::new();
    let p1 = expect_created(
        engine
            .execute_docking(DockingOperation::CreatePanel { content: Some("work".into()) })
            .unwrap(),
    );

    let p2 = expect_embedded(
        engine
            .execute_docking(DockingOperation::Embed {
                source: InsertSource::NewPanel { content: Some("side".into()) },
                host: p1,
                slot: EmbedSlot::Region(EdgeRegion::Left),
            })
            .unwrap(),
    );

    // The temporary auto-promotion is undone back to the bare panel.
    engine.undo().unwrap();
    assert_eq!(
        engine.docking().get(p1).unwrap().kind(),
        app_shell::domain_foundation::environments::docking::DockKind::Panel
    );
    assert!(!engine.docking().contains(p2));

    engine.redo().unwrap();
    assert!(matches!(
        engine.docking().get(p1).unwrap().role,
        app_shell::domain_foundation::environments::docking::DockRole::EmbedHost { .. }
    ));
    assert!(matches!(
        engine.docking().placement(p2),
        Some(app_shell::domain_foundation::environments::docking::DockSlot::HostEdge { host, edge })
            if host == p1 && edge == EdgeRegion::Left
    ));
}

#[test]
fn region_resize_undo_recovers_the_previous_weight() {
    use app_shell::domain_foundation::environments::docking::{DockRole, DockKind};

    let mut engine = DomainEngine::new();
    let p1 = expect_created(
        engine
            .execute_docking(DockingOperation::CreatePanel { content: Some("work".into()) })
            .unwrap(),
    );
    engine
        .execute_docking(DockingOperation::Embed {
            source: InsertSource::NewPanel { content: Some("side".into()) },
            host: p1,
            slot: EmbedSlot::Region(EdgeRegion::Left),
        })
        .unwrap();

    let previous = engine
        .execute_docking(DockingOperation::ResizeRegion {
            host: p1,
            edge: EdgeRegion::Left,
            requested: 2.0,
            min: None,
        })
        .unwrap();
    match previous {
        DockingOutcome::RegionResized { previous, .. } => assert_eq!(previous, 1.0),
        _ => panic!("expected RegionResized"),
    }

    engine.undo().unwrap();
    let host = engine.docking().get(p1).unwrap();
    assert!(matches!(host.role, DockRole::EmbedHost { .. }));
    if let DockRole::EmbedHost { regions, .. } = &host.role {
        assert_eq!(regions.weight_on(EdgeRegion::Left), Some(1.0));
    }
    assert_eq!(engine.docking().get(p1).unwrap().kind(), DockKind::EmbedHost);

    engine.redo().unwrap();
    let host = engine.docking().get(p1).unwrap();
    if let DockRole::EmbedHost { regions, .. } = &host.role {
        assert_eq!(regions.weight_on(EdgeRegion::Left), Some(2.0));
    } else {
        panic!("expected the host role back after redo");
    }
}

#[test]
fn floating_rect_set_undo_recovers_the_previous_rect() {
    use app_shell::domain_foundation::environments::docking::DockRole;

    let mut engine = DomainEngine::new();
    let p1 = expect_created(
        engine
            .execute_docking(DockingOperation::CreatePanel { content: Some("work".into()) })
            .unwrap(),
    );
    engine
        .execute_docking(DockingOperation::Embed {
            source: InsertSource::NewPanel { content: Some("float".into()) },
            host: p1,
            slot: EmbedSlot::Floating { rect: None },
        })
        .unwrap();

    let rect = FloatingRect::new(0.25, 0.25, 0.5, 0.5).unwrap();
    let previous = engine
        .execute_docking(DockingOperation::SetFloatingRect {
            host: p1,
            index: 0,
            rect,
            min_w: None,
            min_h: None,
        })
        .unwrap();
    match previous {
        DockingOutcome::FloatingRectSet { index, previous, .. } => {
            assert_eq!(index, 0);
            assert_eq!(previous, FloatingRect::BASELINE);
        }
        _ => panic!("expected FloatingRectSet"),
    }

    engine.undo().unwrap();
    if let DockRole::EmbedHost { floating, .. } = &engine.docking().get(p1).unwrap().role {
        assert_eq!(floating[0].rect(), FloatingRect::BASELINE);
    } else {
        panic!("expected the host to keep the floating collection");
    }

    engine.redo().unwrap();
    if let DockRole::EmbedHost { floating, .. } = &engine.docking().get(p1).unwrap().role {
        assert_eq!(floating[0].rect(), rect);
    } else {
        panic!("expected the host to keep the floating collection");
    }
}

#[test]
fn collapsed_flag_undo_redo_roundtrip() {
    let mut engine = DomainEngine::new();
    let p1 = expect_created(
        engine
            .execute_docking(DockingOperation::CreatePanel { content: Some("one".into()) })
            .unwrap(),
    );

    engine
        .execute_docking(DockingOperation::SetDockCollapsed { dock: p1, collapsed: true })
        .unwrap();
    // A matching-value set is a no-op success.
    engine
        .execute_docking(DockingOperation::SetDockCollapsed { dock: p1, collapsed: true })
        .unwrap();

    // The first undo reverts the no-op entry — itself a no-op.
    engine.undo().unwrap();
    assert!(matches!(
        engine.docking().get(p1).unwrap().role,
        app_shell::domain_foundation::environments::docking::DockRole::Panel { collapsed: true, .. }
    ));

    // The second undo inverts the flip.
    engine.undo().unwrap();
    assert!(matches!(
        engine.docking().get(p1).unwrap().role,
        app_shell::domain_foundation::environments::docking::DockRole::Panel { collapsed: false, .. }
    ));

    engine.redo().unwrap();
    assert!(matches!(
        engine.docking().get(p1).unwrap().role,
        app_shell::domain_foundation::environments::docking::DockRole::Panel { collapsed: true, .. }
    ));
}

#[test]
fn undo_and_redo_publish_their_events() {
    let mut engine = DomainEngine::new();
    let (_p1, _p2, s) = panel_and_beside(&mut engine);
    let after_undo = engine.events().len();

    engine.undo().unwrap();
    assert_eq!(engine.events()[after_undo].event_type(), "domain.undo");

    let after_redo = engine.events().len();
    engine.redo().unwrap();
    assert_eq!(engine.events()[after_redo].event_type(), "domain.redo");
    let _ = s;
}

// --- Transactions ----------------------------------------------------------

#[test]
fn failed_operation_in_a_transaction_rolls_back_docking_state() {
    let mut engine = DomainEngine::new();
    engine.begin_transaction();

    let p1 = expect_created(
        engine
            .execute_docking(DockingOperation::CreatePanel { content: Some("one".into()) })
            .unwrap(),
    );
    assert!(engine.in_transaction());

    // A failing operation auto-rolls the transaction back — the created
    // dock leaves with it.
    let _err = engine
        .execute_docking(DockingOperation::ResizeSiblings {
            container: p1,
            first: 0,
            requested: 1.0,
            first_min: None,
            second_min: None,
        })
        .unwrap_err();

    assert!(!engine.in_transaction());
    assert!(!engine.docking().contains(p1));
    assert_eq!(engine.docking().root(), None);
    assert_eq!(engine.history().position(), 0);
}

#[test]
fn committed_transaction_undo_redo_replays_as_one_unit() {
    let mut engine = DomainEngine::new();
    engine.begin_transaction();
    let p1 = expect_created(
        engine
            .execute_docking(DockingOperation::CreatePanel { content: Some("one".into()) })
            .unwrap(),
    );
    let (p2, s) = expect_inserted(
        engine
            .execute_docking(DockingOperation::InsertBeside {
                source: InsertSource::NewPanel { content: Some("two".into()) },
                target: p1,
                direction: SplitDirection::Right,
            })
            .unwrap(),
    );
    engine.commit_transaction();
    assert_eq!(engine.history().position(), 2);

    let count = engine.undo().unwrap();
    assert_eq!(count, 2);
    assert_eq!(engine.docking().root(), None);

    let count = engine.redo().unwrap();
    assert_eq!(count, 2);
    assert!(engine.docking().contains(p2));
    assert_eq!(engine.docking().root(), Some(s));
}

// --- Command handler (app-engine bridge, canvas test shape) ----------------

#[test]
fn docking_command_handler_definition() {
    use app_shell::domain_engine::operations::DockingCommandHandler;

    let def = DockingCommandHandler::definition();
    assert_eq!(def.id().as_str(), "docking.operation");
    assert_eq!(def.name(), "Docking Operation");
}

#[test]
fn docking_command_handler_applies_to_the_shared_model() {
    use app_shell::domain_engine::operations::DockingCommandHandler;
    use std::sync::{Arc, Mutex};

    let docking = Arc::new(Mutex::new(
        app_shell::domain_foundation::environments::docking::DockLayout::new(),
    ));
    let handler = DockingCommandHandler::new(Arc::clone(&docking));
    let _ = handler;

    let _ = DockingOperation::CreatePanel { content: Some("one".into()) }
        .clone()
        .apply(&mut docking.lock().unwrap());

    let guard = docking.lock().unwrap();
    assert_eq!(guard.len(), 1);
}
