//! Integration tests: the Phase-10 history integration — one
//! position-indexed executed-step registry inside the Domain Engine
//! (the Phase-9 R-9-1 follow-up).
//!
//! Before the unification, the engine kept a parallel executed-vector
//! per real-undo environment, each aligned with history positions only
//! while that environment's operations were the only real-undo history.
//! Interleaved canvas + docking sequences misaligned both vectors the
//! way the canvas + layers case already did. The registry —
//! `executed_steps[i]` is `Some(step)` exactly when history entry `i`
//! carries a replay step, `None` for the plain-apply family — walks
//! history positions once and dispatches on the step.
//!
//! The registry tests here have no serde dependency (they need no
//! persisted form); the docking persistence/restoration area lives in
//! `docking_persistence_test.rs` (serde-gated, like the sibling env
//! round-trip suites). Public crate API only (agents/testing.md).

use app_shell::domain_engine::{DomainEngine, DomainError};
use app_shell::domain_foundation::environments::canvas_2d::{
    CanvasOperation, CanvasOutcome, Transform, Vec2,
};
use app_shell::domain_foundation::environments::docking::{
    DockId, DockingError, DockingOperation, DockingOutcome, InsertSource, SplitDirection,
};

// --- Fixtures (the docking_undo_redo_test.rs shapes, kept local) -----------

fn expect_created(outcome: DockingOutcome) -> DockId {
    match outcome {
        DockingOutcome::Created { dock, .. } => dock,
        other => panic!("expected Created, got {other:?}"),
    }
}

fn expect_inserted(outcome: DockingOutcome) -> (DockId, DockId) {
    match outcome {
        DockingOutcome::InsertedBeside { source, container, .. } => (source, container),
        other => panic!("expected InsertedBeside, got {other:?}"),
    }
}

// --- One registry, interleaved environments --------------------------------

/// History interleaving canvas and docking operations replays in
/// recorded order: undo unwinds every position back to empty on both
/// environments, and redo recovers the same ids on both sides. This
/// sequence misaligned both per-environment vectors before the
/// unification.
#[test]
fn interleaved_canvas_and_docking_history_replays_in_position_order() {
    let mut engine = DomainEngine::new();

    let p1 = expect_created(
        engine
            .execute_docking(DockingOperation::CreatePanel { content: Some("main".into()) })
            .unwrap(),
    );
    let c1 = match engine
        .execute_canvas(CanvasOperation::Create {
            transform: Transform::from_translation(Vec2::new(1.0, 2.0)),
        })
        .unwrap()
    {
        CanvasOutcome::Created { id } => id,
        other => panic!("expected canvas Created, got {other:?}"),
    };
    let (p2, s) = expect_inserted(
        engine
            .execute_docking(DockingOperation::InsertBeside {
                source: InsertSource::NewPanel { content: Some("two".into()) },
                target: p1,
                direction: SplitDirection::Right,
            })
            .unwrap(),
    );
    engine
        .execute_canvas(CanvasOperation::Move {
            id: c1,
            translation: Vec2::new(9.0, 9.0),
        })
        .unwrap();
    assert_eq!(engine.history().position(), 4);

    for _ in 0..4 {
        engine.undo().unwrap();
    }
    assert_eq!(engine.history().position(), 0);
    assert!(engine.canvas().is_empty());
    assert!(engine.docking().is_empty());

    for _ in 0..4 {
        engine.redo().unwrap();
    }
    assert_eq!(engine.history().position(), 4);
    assert!(engine.canvas().contains(c1));
    assert_eq!(engine.docking().root(), Some(s));
    assert!(engine.docking().contains(p1));
    assert!(engine.docking().contains(p2));
}

/// A canvas-only session keeps replaying correctly after the registry
/// unification (the engine's undo/redo/rollback loops were rewritten
/// around the one registry; the canvas wrapper itself is untouched).
#[test]
fn canvas_only_undo_redo_stays_position_aligned_after_the_registry_unification() {
    let mut engine = DomainEngine::new();
    let c1 = match engine
        .execute_canvas(CanvasOperation::Create {
            transform: Transform::from_translation(Vec2::new(1.0, 2.0)),
        })
        .unwrap()
    {
        CanvasOutcome::Created { id } => id,
        other => panic!("expected canvas Created, got {other:?}"),
    };
    engine
        .execute_canvas(CanvasOperation::Move {
            id: c1,
            translation: Vec2::new(4.0, 5.0),
        })
        .unwrap();

    engine.undo().unwrap();
    assert!(engine.canvas().contains(c1), "undo reverts the move, keeps the object");

    engine.undo().unwrap();
    assert!(engine.canvas().is_empty(), "undo reverts the create");

    engine.redo().unwrap();
    assert!(engine.canvas().contains(c1), "redo restores the same id");
    assert_eq!(engine.canvas().len(), 1);
}

/// Docking and canvas operations inside one committed transaction undo
/// and redo as one unit across both environments, dispatched by the one
/// position-indexed registry.
#[test]
fn committed_transaction_undo_redo_dispatches_the_registry_across_environments() {
    let mut engine = DomainEngine::new();
    let c1 = match engine
        .execute_canvas(CanvasOperation::Create {
            transform: Transform::from_translation(Vec2::new(1.0, 2.0)),
        })
        .unwrap()
    {
        CanvasOutcome::Created { id } => id,
        other => panic!("expected canvas Created, got {other:?}"),
    };

    engine.begin_transaction();
    let p1 = expect_created(
        engine
            .execute_docking(DockingOperation::CreatePanel { content: Some("tx".into()) })
            .unwrap(),
    );
    engine
        .execute_canvas(CanvasOperation::Move {
            id: c1,
            translation: Vec2::new(3.0, 3.0),
        })
        .unwrap();
    engine.commit_transaction();
    assert_eq!(engine.history().position(), 3);

    // The committed transaction's entries replay per position: the tx
    // span holds a docking then a canvas step.
    let undone = engine.undo().unwrap();
    assert_eq!(undone, 2, "the transaction undoes as one unit");
    assert!(engine.docking().is_empty());
    assert!(engine.canvas().contains(c1), "canvas state back at pre-tx");
    assert_eq!(engine.history().position(), 1);

    engine.redo().unwrap();
    assert!(engine.docking().contains(p1), "redo restores the tx unit");
    assert!(engine.canvas().contains(c1));
    assert_eq!(engine.history().position(), 3);
}

/// A rejected docking operation interleaved into history leaves the
/// registry untouched: failures record nothing, so position alignment
/// survives — the surrounding canvas steps still undo/redo cleanly.
#[test]
fn rejected_interleaved_operation_keeps_the_registry_aligned() {
    let mut engine = DomainEngine::new();
    let c1 = match engine
        .execute_canvas(CanvasOperation::Create {
            transform: Transform::from_translation(Vec2::new(1.0, 2.0)),
        })
        .unwrap()
    {
        CanvasOutcome::Created { id } => id,
        other => panic!("expected canvas Created, got {other:?}"),
    };
    let p1 = expect_created(
        engine
            .execute_docking(DockingOperation::CreatePanel { content: Some("main".into()) })
            .unwrap(),
    );

    // A rejected op (a panel is not a resize surface) records nothing.
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

    engine
        .execute_canvas(CanvasOperation::Move {
            id: c1,
            translation: Vec2::new(5.0, 5.0),
        })
        .unwrap();

    // The alignment observed around the failure holds for both
    // environments.
    engine.undo().unwrap();
    assert!(engine.canvas().contains(c1), "only the move reverted");
    engine.undo().unwrap();
    assert!(engine.docking().is_empty(), "the docking create reverted");
    engine.undo().unwrap();
    assert!(engine.canvas().is_empty());

    engine.redo().unwrap();
    assert!(engine.canvas().contains(c1));
    engine.redo().unwrap();
    assert!(engine.docking().root() == Some(p1));
    engine.redo().unwrap();
    assert!(engine.canvas().len() == 1);
    assert!(engine.docking().root() == Some(p1));
}
