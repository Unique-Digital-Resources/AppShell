//! Integration tests: domain operations through the Domain Engine.
//!
//! Verifies that Canvas2D operations can be executed through the Domain
//! Engine, producing correct canvas state, history entries, and events.

use app_shell::domain_engine::DomainEngine;
use app_shell::domain_foundation::environments::canvas_2d::{
    CanvasOperation, CanvasOutcome, Transform, Vec2,
};

#[test]
fn execute_create_produces_object_event_and_history() {
    let mut engine = DomainEngine::new();

    let outcome = engine
        .execute(CanvasOperation::Create {
            transform: Transform::from_translation(Vec2::new(10.0, 20.0)),
        })
        .expect("create should succeed");

    let id = match outcome {
        CanvasOutcome::Created { id } => id,
        other => panic!("expected Created, got {other:?}"),
    };

    // Canvas has the object.
    assert!(engine.canvas().contains(id));
    let obj = engine.canvas().get(id).unwrap();
    assert_eq!(obj.transform.translation, Vec2::new(10.0, 20.0));

    // History recorded one entry.
    assert_eq!(engine.history().position(), 1);
    assert_eq!(engine.history().entry_count(), 1);

    // Event published.
    assert_eq!(engine.events().len(), 1);
}

#[test]
fn execute_move_updates_canvas_and_publishes_event() {
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

    let outcome = engine
        .execute(CanvasOperation::Move {
            id,
            translation: Vec2::new(5.0, 7.0),
        })
        .expect("move should succeed");

    assert!(matches!(outcome, CanvasOutcome::Moved { .. }));

    // Canvas reflects the new translation.
    let obj = engine.canvas().get(id).unwrap();
    assert_eq!(obj.transform.translation, Vec2::new(5.0, 7.0));
    // Rotation and scale preserved.
    assert_eq!(obj.transform.rotation, 0.0);
    assert_eq!(obj.transform.scale, Vec2::splat(1.0));

    // Two events: one for create, one for move.
    assert_eq!(engine.events().len(), 2);
    assert_eq!(engine.history().position(), 2);
}

#[test]
fn execute_transform_replaces_whole_transform() {
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

    let new = Transform::from_translation(Vec2::new(1.0, 2.0))
        .with_rotation(0.5)
        .with_scale(Vec2::splat(2.0));

    engine
        .execute(CanvasOperation::Transform {
            id,
            transform: new,
        })
        .expect("transform should succeed");

    let obj = engine.canvas().get(id).unwrap();
    assert_eq!(obj.transform, new);
}

#[test]
fn execute_remove_deletes_object() {
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

    let outcome = engine
        .execute(CanvasOperation::Remove { id })
        .expect("remove should succeed");

    assert!(matches!(outcome, CanvasOutcome::Removed { .. }));
    assert!(!engine.canvas().contains(id));
    assert!(engine.canvas().is_empty());
}

#[test]
fn execute_on_missing_object_returns_error() {
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

    // Remove the object.
    engine
        .execute(CanvasOperation::Remove { id })
        .unwrap();

    // Now try to move the removed object.
    let err = engine
        .execute(CanvasOperation::Move {
            id,
            translation: Vec2::new(1.0, 1.0),
        })
        .unwrap_err();

    assert!(matches!(
        err,
        app_shell::domain_foundation::environments::canvas_2d::CanvasError::ObjectNotFound { .. }
    ));
}

#[test]
fn sequence_of_operations_composes() {
    let mut engine = DomainEngine::new();

    // Create.
    let id = match engine
        .execute(CanvasOperation::Create {
            transform: Transform::identity(),
        })
        .unwrap()
    {
        CanvasOutcome::Created { id } => id,
        _ => unreachable!(),
    };

    // Move.
    engine
        .execute(CanvasOperation::Move {
            id,
            translation: Vec2::new(3.0, 4.0),
        })
        .unwrap();

    // Transform.
    let finalised =
        Transform::from_translation(Vec2::new(3.0, 4.0)).with_rotation(1.0);
    engine
        .execute(CanvasOperation::Transform {
            id,
            transform: finalised,
        })
        .unwrap();

    // Verify final state.
    assert_eq!(engine.canvas().len(), 1);
    let obj = engine.canvas().get(id).unwrap();
    assert_eq!(obj.transform.translation, Vec2::new(3.0, 4.0));
    assert_eq!(obj.transform.rotation, 1.0);

    // Three operations → three events, three history entries.
    assert_eq!(engine.events().len(), 3);
    assert_eq!(engine.history().position(), 3);
}

#[test]
fn execute_layers_through_domain_engine() {
    use app_shell::domain_foundation::environments::layers::LayersOperation;

    let mut engine = DomainEngine::new();
    let outcome = engine
        .execute_layers(LayersOperation::CreateLayer {
            name: "Base".to_string(),
        })
        .unwrap();

    use app_shell::domain_foundation::environments::layers::LayersOutcome;
    match outcome {
        LayersOutcome::LayerCreated { id } => {
            assert!(engine.layers().contains(id));
        }
        other => panic!("expected LayerCreated, got {other:?}"),
    }
    assert_eq!(engine.events().len(), 1);
}

#[test]
fn execute_text_editor_through_domain_engine() {
    use app_shell::domain_foundation::environments::text_editor::{
        Position, TextEditorOperation,
    };

    let mut engine = DomainEngine::new();
    engine
        .execute_text_editor(TextEditorOperation::Insert {
            position: Position::new(0),
            text: "hello".to_string(),
        })
        .unwrap();

    assert_eq!(engine.document().text(), "hello");
    assert_eq!(engine.events().len(), 1);
}