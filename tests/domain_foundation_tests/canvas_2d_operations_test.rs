//! Operation tests for the Canvas2D environment.

use app_shell::domain_foundation::environments::canvas_2d::{
    Canvas, CanvasError, CanvasOperation, CanvasOutcome, Transform, Vec2,
};
use app_shell::domain_engine::operations::canvas_operations::ExecutedCanvasOperation;

#[test]
fn create_operation_adds_an_object() {
    let mut canvas = Canvas::new();
    let outcome = CanvasOperation::Create { transform: Transform::identity() }
        .apply(&mut canvas)
        .expect("create should succeed on empty canvas");

    let id = match outcome {
        CanvasOutcome::Created { id } => id,
        other => panic!("expected Created, got {other:?}"),
    };
    assert!(canvas.contains(id));
    assert_eq!(canvas.len(), 1);
}

#[test]
fn move_operation_updates_translation_and_preserves_rotation_and_scale() {
    let mut canvas = Canvas::new();
    let id = canvas.create_object(
        Transform::identity().with_rotation(0.5).with_scale(Vec2::splat(2.0)),
    );

    let outcome = CanvasOperation::Move { id, translation: Vec2::new(5.0, 7.0) }
        .apply(&mut canvas)
        .expect("move should succeed for existing id");

    assert!(matches!(outcome, CanvasOutcome::Moved { .. }));

    let obj = canvas.get(id).unwrap();
    assert_eq!(obj.transform.translation, Vec2::new(5.0, 7.0));
    assert_eq!(obj.transform.rotation, 0.5);
    assert_eq!(obj.transform.scale, Vec2::splat(2.0));
}

#[test]
fn move_outcome_records_previous_transform_for_undo() {
    let mut canvas = Canvas::new();
    let original = Transform::from_translation(Vec2::new(1.0, 1.0));
    let id = canvas.create_object(original);

    let outcome = CanvasOperation::Move { id, translation: Vec2::new(9.0, 9.0) }
        .apply(&mut canvas)
        .unwrap();

    match outcome {
        CanvasOutcome::Moved { previous, .. } => assert_eq!(previous, original),
        other => panic!("expected Moved, got {other:?}"),
    }
}

#[test]
fn transform_operation_replaces_whole_transform() {
    let mut canvas = Canvas::new();
    let id = canvas.create_object(Transform::identity());
    let new = Transform::from_translation(Vec2::new(1.0, 2.0))
        .with_rotation(0.5)
        .with_scale(Vec2::splat(2.0));

    let outcome = CanvasOperation::Transform { id, transform: new }
        .apply(&mut canvas)
        .expect("transform should succeed for existing id");

    match outcome {
        CanvasOutcome::Transformed { previous, .. } => {
            assert_eq!(previous, Transform::identity());
        }
        other => panic!("expected Transformed, got {other:?}"),
    }
    assert_eq!(canvas.get(id).unwrap().transform, new);
}

#[test]
fn remove_operation_deletes_the_object_and_returns_it() {
    let mut canvas = Canvas::new();
    let transform = Transform::from_translation(Vec2::new(3.0, 4.0));
    let id = canvas.create_object(transform);

    let outcome = CanvasOperation::Remove { id }
        .apply(&mut canvas)
        .expect("remove should succeed for existing id");

    match outcome {
        CanvasOutcome::Removed { object } => {
            assert_eq!(object.id, id);
            assert_eq!(object.transform, transform);
        }
        other => panic!("expected Removed, got {other:?}"),
    }
    assert!(!canvas.contains(id));
    assert!(canvas.is_empty());
}

#[test]
fn operations_on_missing_ids_fail_with_object_not_found() {
    let mut canvas = Canvas::new();
    let id = canvas.create_object(Transform::identity());
    canvas.remove_object(id).unwrap(); // id no longer valid

    let err = CanvasOperation::Move { id, translation: Vec2::zero() }
        .apply(&mut canvas)
        .unwrap_err();
    assert!(matches!(err, CanvasError::ObjectNotFound { .. }));

    let err = CanvasOperation::Transform { id, transform: Transform::identity() }
        .apply(&mut canvas)
        .unwrap_err();
    assert!(matches!(err, CanvasError::ObjectNotFound { .. }));

    let err = CanvasOperation::Remove { id }
        .apply(&mut canvas)
        .unwrap_err();
    assert!(matches!(err, CanvasError::ObjectNotFound { .. }));
}

#[test]
fn sequence_of_operations_composes_correctly() {
    let mut canvas = Canvas::new();

    let outcome = CanvasOperation::Create { transform: Transform::identity() }
        .apply(&mut canvas)
        .unwrap();
    let id = match outcome {
        CanvasOutcome::Created { id } => id,
        other => panic!("expected Created, got {other:?}"),
    };

    CanvasOperation::Move { id, translation: Vec2::new(3.0, 4.0) }
        .apply(&mut canvas)
        .unwrap();

    let finalised = Transform::from_translation(Vec2::new(3.0, 4.0)).with_rotation(1.0);
    CanvasOperation::Transform { id, transform: finalised }
        .apply(&mut canvas)
        .unwrap();

    assert_eq!(canvas.len(), 1);
    let obj = canvas.get(id).unwrap();
    assert_eq!(obj.transform.translation, Vec2::new(3.0, 4.0));
    assert_eq!(obj.transform.rotation, 1.0);
    assert_eq!(obj.transform.scale, Vec2::splat(1.0));
}

#[test]
fn move_forward_operation() {
    let mut canvas = Canvas::new();
    let a = canvas.create_object(Transform::identity());
    let b = canvas.create_object(Transform::identity());
    let c = canvas.create_object(Transform::identity());

    assert_eq!(canvas.draw_order(), &[a, b, c]);

    // Move b forward by one → [a, c, b]
    let outcome = CanvasOperation::MoveForward { id: b }
        .apply(&mut canvas).unwrap();
    match outcome {
        CanvasOutcome::MovedForward { previous_index, .. } => {
            assert_eq!(previous_index, 1);
        }
        other => panic!("expected MovedForward, got {other:?}"),
    }
    assert_eq!(canvas.draw_order(), &[a, c, b]);
}

#[test]
fn move_backward_operation() {
    let mut canvas = Canvas::new();
    let a = canvas.create_object(Transform::identity());
    let b = canvas.create_object(Transform::identity());
    let _c = canvas.create_object(Transform::identity());

    let outcome = CanvasOperation::MoveBackward { id: b }
        .apply(&mut canvas).unwrap();
    match outcome {
        CanvasOutcome::MovedBackward { previous_index, .. } => {
            assert_eq!(previous_index, 1);
        }
        other => panic!("expected MovedBackward, got {other:?}"),
    }
    assert_eq!(canvas.draw_order(), &[b, a, _c]);
}

#[test]
fn reorder_operation() {
    let mut canvas = Canvas::new();
    let a = canvas.create_object(Transform::identity());
    let b = canvas.create_object(Transform::identity());
    let c = canvas.create_object(Transform::identity());

    let outcome = CanvasOperation::Reorder { id: c, new_index: 0 }
        .apply(&mut canvas).unwrap();
    match outcome {
        CanvasOutcome::Reordered { previous_index, .. } => {
            assert_eq!(previous_index, 2);
        }
        other => panic!("expected Reordered, got {other:?}"),
    }
    assert_eq!(canvas.draw_order(), &[c, a, b]);
}

#[test]
fn duplicate_operation() {
    let mut canvas = Canvas::new();
    let id = canvas.create_object(
        Transform::from_translation(Vec2::new(10.0, 20.0)).with_rotation(0.5),
    );

    let outcome = CanvasOperation::Duplicate { id, offset: Vec2::new(5.0, 0.0) }
        .apply(&mut canvas).unwrap();

    match outcome {
        CanvasOutcome::Duplicated { source, new_id } => {
            assert_eq!(source, id);
            assert_ne!(new_id, id);
        }
        other => panic!("expected Duplicated, got {other:?}"),
    }

    assert_eq!(canvas.len(), 2);
    // Original unchanged.
    let orig = canvas.get(id).unwrap();
    assert_eq!(orig.transform.translation, Vec2::new(10.0, 20.0));
    // Duplicate at offset.
    // Find the other object (not the original).
    let dup = canvas.iter().find(|o| o.id != id).unwrap();
    assert_eq!(dup.transform.translation, Vec2::new(15.0, 20.0));
    assert_eq!(dup.transform.rotation, 0.5);
}

#[test]
fn clear_operation() {
    let mut canvas = Canvas::new();
    canvas.create_object(Transform::identity());
    canvas.create_object(Transform::identity());

    let outcome = CanvasOperation::Clear
        .apply(&mut canvas).unwrap();

    match outcome {
        CanvasOutcome::Cleared { count } => assert_eq!(count, 2),
        other => panic!("expected Cleared, got {other:?}"),
    }
    assert!(canvas.is_empty());
}

#[test]
fn move_to_front_undo_restores_position() {
    use app_shell::domain_engine::operations::canvas_operations::ExecutedCanvasOperation;

    let mut canvas = Canvas::new();
    let a = canvas.create_object(Transform::identity());
    let b = canvas.create_object(Transform::identity());
    let c = canvas.create_object(Transform::identity());

    assert_eq!(canvas.draw_order(), &[a, b, c]);

    // Execute MoveToFront through ExecutedCanvasOperation.
    let executed = ExecutedCanvasOperation::execute(
        &mut canvas,
        CanvasOperation::MoveToFront { id: a },
    ).unwrap();

    assert_eq!(canvas.draw_order(), &[b, c, a]);

    // Undo → restore to [a, b, c]
    executed.undo(&mut canvas).unwrap();
    assert_eq!(canvas.draw_order(), &[a, b, c]);
}

#[test]
fn move_to_back_undo_restores_position() {
    use app_shell::domain_engine::operations::canvas_operations::ExecutedCanvasOperation;

    let mut canvas = Canvas::new();
    let a = canvas.create_object(Transform::identity());
    let b = canvas.create_object(Transform::identity());
    let c = canvas.create_object(Transform::identity());

    assert_eq!(canvas.draw_order(), &[a, b, c]);

    // Execute MoveToBack through ExecutedCanvasOperation.
    let executed = ExecutedCanvasOperation::execute(
        &mut canvas,
        CanvasOperation::MoveToBack { id: c },
    ).unwrap();

    assert_eq!(canvas.draw_order(), &[c, a, b]);

    // Undo → restore to [a, b, c]
    executed.undo(&mut canvas).unwrap();
    assert_eq!(canvas.draw_order(), &[a, b, c]);
}