//! Integration tests: undo/redo through the Domain Engine + HistoryStore.
//!
//! Verifies that the App Engine's HistoryStore can drive undo/redo of
//! Canvas2D operations, with canvas state correctly restored and
//! re-applied.

use app_shell::domain_engine::DomainEngine;
use app_shell::domain_foundation::environments::canvas_2d::{
    CanvasOperation, CanvasOutcome, Transform, Vec2,
};

fn create_and_move(engine: &mut DomainEngine) -> app_shell::domain_foundation::environments::canvas_2d::ObjectId {
    let id = match engine
        .execute(CanvasOperation::Create {
            transform: Transform::identity(),
        })
        .unwrap()
    {
        CanvasOutcome::Created { id } => id,
        _ => unreachable!(),
    };

    engine
        .execute(CanvasOperation::Move {
            id,
            translation: Vec2::new(5.0, 5.0),
        })
        .unwrap();

    id
}

#[test]
fn undo_move_restores_previous_position() {
    let mut engine = DomainEngine::new();
    let id = create_and_move(&mut engine);

    // After create + move, position is (5, 5).
    assert_eq!(
        engine.canvas().get(id).unwrap().transform.translation,
        Vec2::new(5.0, 5.0)
    );
    assert_eq!(engine.history().position(), 2);

    // Undo the move.
    let count = engine.undo().unwrap();
    assert_eq!(count, 1);
    assert_eq!(engine.history().position(), 1);

    // Position should be back to (0, 0).
    assert_eq!(
        engine.canvas().get(id).unwrap().transform.translation,
        Vec2::new(0.0, 0.0)
    );
}

#[test]
fn redo_re_applies_move() {
    let mut engine = DomainEngine::new();
    let id = create_and_move(&mut engine);

    // Undo the move.
    engine.undo().unwrap();
    assert_eq!(
        engine.canvas().get(id).unwrap().transform.translation,
        Vec2::new(0.0, 0.0)
    );

    // Redo the move.
    let count = engine.redo().unwrap();
    assert_eq!(count, 1);
    assert_eq!(engine.history().position(), 2);

    // Position should be (5, 5) again.
    assert_eq!(
        engine.canvas().get(id).unwrap().transform.translation,
        Vec2::new(5.0, 5.0)
    );
}

#[test]
fn undo_create_removes_the_object() {
    let mut engine = DomainEngine::new();

    let id = match engine
        .execute(CanvasOperation::Create {
            transform: Transform::identity(),
        })
        .unwrap()
    {
        CanvasOutcome::Created { id } => id,
        _ => unreachable!(),
    };

    assert!(engine.canvas().contains(id));

    // Undo the create.
    engine.undo().unwrap();

    // Object should be gone.
    assert!(!engine.canvas().contains(id));
    assert!(engine.canvas().is_empty());
}

#[test]
fn redo_create_re_creates_the_object() {
    let mut engine = DomainEngine::new();

    let id = match engine
        .execute(CanvasOperation::Create {
            transform: Transform::from_translation(Vec2::new(3.0, 4.0)),
        })
        .unwrap()
    {
        CanvasOutcome::Created { id } => id,
        _ => unreachable!(),
    };

    // Undo the create.
    engine.undo().unwrap();
    assert!(!engine.canvas().contains(id));

    // Redo the create.
    engine.redo().unwrap();

    // Object should be back with the same id and transform.
    assert!(engine.canvas().contains(id));
    let obj = engine.canvas().get(id).unwrap();
    assert_eq!(obj.transform.translation, Vec2::new(3.0, 4.0));
}

#[test]
fn undo_remove_restores_the_object() {
    let mut engine = DomainEngine::new();

    let id = match engine
        .execute(CanvasOperation::Create {
            transform: Transform::from_translation(Vec2::new(7.0, 8.0)),
        })
        .unwrap()
    {
        CanvasOutcome::Created { id } => id,
        _ => unreachable!(),
    };

    // Remove the object.
    engine
        .execute(CanvasOperation::Remove { id })
        .unwrap();

    assert!(!engine.canvas().contains(id));

    // Undo the remove — object should be restored with its original id.
    engine.undo().unwrap();

    assert!(engine.canvas().contains(id));
    let obj = engine.canvas().get(id).unwrap();
    assert_eq!(obj.transform.translation, Vec2::new(7.0, 8.0));
}

#[test]
fn multiple_undo_reverses_in_order() {
    let mut engine = DomainEngine::new();

    let id = match engine
        .execute(CanvasOperation::Create {
            transform: Transform::identity(),
        })
        .unwrap()
    {
        CanvasOutcome::Created { id } => id,
        _ => unreachable!(),
    };

    // Move to (1, 1).
    engine
        .execute(CanvasOperation::Move {
            id,
            translation: Vec2::new(1.0, 1.0),
        })
        .unwrap();

    // Move to (2, 2).
    engine
        .execute(CanvasOperation::Move {
            id,
            translation: Vec2::new(2.0, 2.0),
        })
        .unwrap();

    assert_eq!(
        engine.canvas().get(id).unwrap().transform.translation,
        Vec2::new(2.0, 2.0)
    );
    assert_eq!(engine.history().position(), 3);

    // Undo last move → back to (1, 1).
    engine.undo().unwrap();
    assert_eq!(
        engine.canvas().get(id).unwrap().transform.translation,
        Vec2::new(1.0, 1.0)
    );

    // Undo first move → back to (0, 0).
    engine.undo().unwrap();
    assert_eq!(
        engine.canvas().get(id).unwrap().transform.translation,
        Vec2::new(0.0, 0.0)
    );

    // Undo create → object gone.
    engine.undo().unwrap();
    assert!(!engine.canvas().contains(id));

    assert_eq!(engine.history().position(), 0);
    assert!(!engine.can_undo());
}

#[test]
fn execute_after_undo_clears_redo_stack() {
    let mut engine = DomainEngine::new();

    let id = match engine
        .execute(CanvasOperation::Create {
            transform: Transform::identity(),
        })
        .unwrap()
    {
        CanvasOutcome::Created { id } => id,
        _ => unreachable!(),
    };

    engine
        .execute(CanvasOperation::Move {
            id,
            translation: Vec2::new(5.0, 5.0),
        })
        .unwrap();

    // Undo the move.
    engine.undo().unwrap();
    assert!(!engine.can_redo() == false); // can redo

    // Execute a new operation (transform instead of move).
    engine
        .execute(CanvasOperation::Transform {
            id,
            transform: Transform::from_translation(Vec2::new(9.0, 9.0)),
        })
        .unwrap();

    // Redo stack should be cleared.
    assert!(!engine.can_redo());

    // Canvas should have the new transform.
    assert_eq!(
        engine.canvas().get(id).unwrap().transform.translation,
        Vec2::new(9.0, 9.0)
    );
}

#[test]
fn full_undo_redo_cycle() {
    let mut engine = DomainEngine::new();

    let id = match engine
        .execute(CanvasOperation::Create {
            transform: Transform::identity(),
        })
        .unwrap()
    {
        CanvasOutcome::Created { id } => id,
        _ => unreachable!(),
    };

    engine
        .execute(CanvasOperation::Move {
            id,
            translation: Vec2::new(3.0, 4.0),
        })
        .unwrap();

    engine
        .execute(CanvasOperation::Transform {
            id,
            transform: Transform::from_translation(Vec2::new(3.0, 4.0))
                .with_rotation(0.5),
        })
        .unwrap();

    let original = engine.canvas().get(id).unwrap().transform;

    // Undo everything.
    engine.undo().unwrap(); // undo transform
    engine.undo().unwrap(); // undo move
    engine.undo().unwrap(); // undo create
    assert!(!engine.canvas().contains(id));
    assert_eq!(engine.history().position(), 0);

    // Redo everything.
    engine.redo().unwrap(); // redo create
    engine.redo().unwrap(); // redo move
    engine.redo().unwrap(); // redo transform

    assert!(engine.canvas().contains(id));
    assert_eq!(engine.canvas().get(id).unwrap().transform, original);
    assert_eq!(engine.history().position(), 3);
}

#[test]
fn events_accumulate_across_operations() {
    let mut engine = DomainEngine::new();

    engine
        .execute(CanvasOperation::Create {
            transform: Transform::identity(),
        })
        .unwrap();

    engine
        .execute(CanvasOperation::Create {
            transform: Transform::identity(),
        })
        .unwrap();

    engine
        .execute(CanvasOperation::Create {
            transform: Transform::identity(),
        })
        .unwrap();

    // Three operations → three events.
    assert_eq!(engine.events().len(), 3);
    assert_eq!(engine.history().position(), 3);
    assert_eq!(engine.canvas().len(), 3);
}

#[test]
fn dirty_tracking_and_mark_saved() {
    let mut engine = DomainEngine::new();

    // Initially clean.
    assert!(!engine.is_dirty());

    // Execute an operation → dirty.
    let id = match engine
        .execute(CanvasOperation::Create { transform: Transform::identity() })
        .unwrap()
    {
        CanvasOutcome::Created { id } => id,
        _ => unreachable!(),
    };
    assert!(engine.is_dirty());

    // Mark saved → clean.
    engine.mark_saved();
    assert!(!engine.is_dirty());

    // Undo → dirty again.
    engine.undo().unwrap();
    assert!(engine.is_dirty());
}

#[test]
fn compound_operations_undo_as_one_unit() {
    let mut engine = DomainEngine::new();

    let tx = engine.begin_compound();

    engine.execute(CanvasOperation::Create { transform: Transform::identity() }).unwrap();
    engine.execute(CanvasOperation::Create { transform: Transform::identity() }).unwrap();
    engine.execute(CanvasOperation::Create { transform: Transform::identity() }).unwrap();

    engine.commit_compound();

    assert_eq!(engine.canvas().len(), 3);
    assert_eq!(engine.history().position(), 3);

    // Undo all 3 as one unit.
    engine.undo().unwrap();
    assert_eq!(engine.canvas().len(), 0);
    assert_eq!(engine.history().position(), 0);
}

#[test]
fn auto_rollback_on_error_during_transaction() {
    let mut engine = DomainEngine::new();

    engine.begin_transaction();
    engine.execute(CanvasOperation::Create { transform: Transform::identity() }).unwrap();

    // Try an invalid operation → should auto-rollback.
    let fake_id = app_shell::domain_foundation::environments::canvas_2d::ObjectId::from_raw(999);
    let _err = engine
        .execute(CanvasOperation::Remove { id: fake_id })
        .unwrap_err();

    // Transaction was rolled back.
    assert!(!engine.in_transaction());
}
#[test]
fn rollback_transaction_removes_entries() {
    let mut engine = DomainEngine::new();

    engine.begin_transaction();
    engine
        .execute(CanvasOperation::Create {
            transform: Transform::identity(),
        })
        .unwrap();
    engine
        .execute(CanvasOperation::Create {
            transform: Transform::identity(),
        })
        .unwrap();

    assert_eq!(engine.history().position(), 2);
    assert!(engine.in_transaction());

    engine.rollback_transaction();

    assert!(!engine.in_transaction());
    assert_eq!(engine.history().position(), 0);
    assert_eq!(engine.canvas().len(), 0);
}
#[test]
fn undo_result_provides_entry_ids() {
    let mut engine = DomainEngine::new();

    let id1 = match engine
        .execute(CanvasOperation::Create {
            transform: Transform::identity(),
        })
        .unwrap()
    {
        CanvasOutcome::Created { id } => id,
        _ => unreachable!(),
    };
    let _id2 = engine
        .execute(CanvasOperation::Create {
            transform: Transform::identity(),
        })
        .unwrap();

    // Undo one — should get 1 entry id back.
    let count = engine.undo().unwrap();
    assert_eq!(count, 1);
    assert_eq!(engine.history().position(), 1);

    // Redo — should get 1 entry id back.
    let count = engine.redo().unwrap();
    assert_eq!(count, 1);
    assert_eq!(engine.history().position(), 2);
}

#[test]
fn transaction_undo_provides_all_entry_ids() {
    let mut engine = DomainEngine::new();

    engine.begin_transaction();
    engine
        .execute(CanvasOperation::Create {
            transform: Transform::identity(),
        })
        .unwrap();
    engine
        .execute(CanvasOperation::Create {
            transform: Transform::identity(),
        })
        .unwrap();
    engine
        .execute(CanvasOperation::Create {
            transform: Transform::identity(),
        })
        .unwrap();
    engine.commit_transaction();

    assert_eq!(engine.history().position(), 3);

    // Undo should reverse all 3 as one transaction unit.
    let count = engine.undo().unwrap();
    assert_eq!(count, 3);
    assert_eq!(engine.history().position(), 0);

    // Redo should re-apply all 3.
    let count = engine.redo().unwrap();
    assert_eq!(count, 3);
    assert_eq!(engine.history().position(), 3);
}
