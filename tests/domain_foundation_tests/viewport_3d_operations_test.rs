//! Operation tests for the Viewport3D environment.

use app_shell::domain_foundation::environments::viewport_3d::{
    ObjectId3D, Scene, Transform3D, Vec3, Viewport3DError, Viewport3DOperation,
    Viewport3DOutcome,
};

#[test]
fn create_operation_adds_an_object() {
    let mut scene = Scene::new();
    let outcome = Viewport3DOperation::Create {
        transform: Transform3D::from_translation(Vec3::new(1.0, 2.0, 3.0)),
    }
    .apply(&mut scene)
    .unwrap();

    let id = match outcome {
        Viewport3DOutcome::Created { id } => id,
        other => panic!("expected Created, got {other:?}"),
    };
    assert!(scene.contains(id));
    assert_eq!(scene.get(id).unwrap().transform.translation, Vec3::new(1.0, 2.0, 3.0));
}

#[test]
fn move_operation_updates_translation() {
    let mut scene = Scene::new();
    let id = scene.create_object(Transform3D::identity());

    let outcome = Viewport3DOperation::Move {
        id,
        translation: Vec3::new(10.0, 20.0, 30.0),
    }
    .apply(&mut scene)
    .unwrap();

    assert!(matches!(outcome, Viewport3DOutcome::Moved { .. }));
    assert_eq!(scene.get(id).unwrap().transform.translation, Vec3::new(10.0, 20.0, 30.0));
    // Rotation and scale preserved.
    assert_eq!(scene.get(id).unwrap().transform.rotation, Vec3::zero());
    assert_eq!(scene.get(id).unwrap().transform.scale, Vec3::splat(1.0));
}

#[test]
fn rotate_operation_updates_rotation() {
    let mut scene = Scene::new();
    let id = scene.create_object(
        Transform3D::from_translation(Vec3::new(5.0, 5.0, 5.0)),
    );

    let outcome = Viewport3DOperation::Rotate {
        id,
        rotation: Vec3::new(0.1, 0.2, 0.3),
    }
    .apply(&mut scene)
    .unwrap();

    assert!(matches!(outcome, Viewport3DOutcome::Rotated { .. }));
    assert_eq!(scene.get(id).unwrap().transform.rotation, Vec3::new(0.1, 0.2, 0.3));
    // Translation and scale preserved.
    assert_eq!(scene.get(id).unwrap().transform.translation, Vec3::new(5.0, 5.0, 5.0));
    assert_eq!(scene.get(id).unwrap().transform.scale, Vec3::splat(1.0));
}

#[test]
fn scale_operation_updates_scale() {
    let mut scene = Scene::new();
    let id = scene.create_object(
        Transform3D::from_translation(Vec3::new(5.0, 5.0, 5.0)),
    );

    let outcome = Viewport3DOperation::Scale {
        id,
        scale: Vec3::new(2.0, 3.0, 4.0),
    }
    .apply(&mut scene)
    .unwrap();

    assert!(matches!(outcome, Viewport3DOutcome::Scaled { .. }));
    assert_eq!(scene.get(id).unwrap().transform.scale, Vec3::new(2.0, 3.0, 4.0));
    // Translation and rotation preserved.
    assert_eq!(scene.get(id).unwrap().transform.translation, Vec3::new(5.0, 5.0, 5.0));
    assert_eq!(scene.get(id).unwrap().transform.rotation, Vec3::zero());
}

#[test]
fn transform_operation_replaces_whole_transform() {
    let mut scene = Scene::new();
    let id = scene.create_object(Transform3D::identity());

    let new = Transform3D::from_translation(Vec3::new(1.0, 2.0, 3.0))
        .with_rotation(Vec3::new(0.1, 0.2, 0.3))
        .with_scale(Vec3::new(2.0, 2.0, 2.0));

    let outcome = Viewport3DOperation::Transform {
        id,
        transform: new,
    }
    .apply(&mut scene)
    .unwrap();

    match outcome {
        Viewport3DOutcome::Transformed { previous, .. } => {
            assert_eq!(previous, Transform3D::identity());
        }
        other => panic!("expected Transformed, got {other:?}"),
    }
    assert_eq!(scene.get(id).unwrap().transform, new);
}

#[test]
fn remove_operation_returns_object() {
    let mut scene = Scene::new();
    let transform = Transform3D::from_translation(Vec3::new(3.0, 4.0, 5.0));
    let id = scene.create_object(transform);

    let outcome = Viewport3DOperation::Remove { id }
        .apply(&mut scene)
        .unwrap();

    match outcome {
        Viewport3DOutcome::Removed { object } => {
            assert_eq!(object.id, id);
            assert_eq!(object.transform, transform);
        }
        other => panic!("expected Removed, got {other:?}"),
    }
    assert!(!scene.contains(id));
    assert!(scene.is_empty());
}

#[test]
fn operations_on_missing_ids_fail() {
    let mut scene = Scene::new();
    let id = scene.create_object(Transform3D::identity());
    scene.remove_object(id).unwrap();

    let err = Viewport3DOperation::Move {
        id,
        translation: Vec3::zero(),
    }
    .apply(&mut scene)
    .unwrap_err();
    assert!(matches!(err, Viewport3DError::ObjectNotFound { .. }));

    let err = Viewport3DOperation::Rotate {
        id,
        rotation: Vec3::zero(),
    }
    .apply(&mut scene)
    .unwrap_err();
    assert!(matches!(err, Viewport3DError::ObjectNotFound { .. }));

    let err = Viewport3DOperation::Scale {
        id,
        scale: Vec3::splat(1.0),
    }
    .apply(&mut scene)
    .unwrap_err();
    assert!(matches!(err, Viewport3DError::ObjectNotFound { .. }));

    let err = Viewport3DOperation::Remove { id }
        .apply(&mut scene)
        .unwrap_err();
    assert!(matches!(err, Viewport3DError::ObjectNotFound { .. }));
}

#[test]
fn sequence_of_operations_composes() {
    let mut scene = Scene::new();

    // Create.
    let outcome = Viewport3DOperation::Create {
        transform: Transform3D::identity(),
    }
    .apply(&mut scene)
    .unwrap();
    let id = match outcome {
        Viewport3DOutcome::Created { id } => id,
        _ => unreachable!(),
    };

    // Move.
    Viewport3DOperation::Move {
        id,
        translation: Vec3::new(10.0, 20.0, 30.0),
    }
    .apply(&mut scene)
    .unwrap();

    // Rotate.
    Viewport3DOperation::Rotate {
        id,
        rotation: Vec3::new(0.1, 0.2, 0.3),
    }
    .apply(&mut scene)
    .unwrap();

    // Scale.
    Viewport3DOperation::Scale {
        id,
        scale: Vec3::new(2.0, 3.0, 4.0),
    }
    .apply(&mut scene)
    .unwrap();

    // Verify final state.
    assert_eq!(scene.len(), 1);
    let obj = scene.get(id).unwrap();
    assert_eq!(obj.transform.translation, Vec3::new(10.0, 20.0, 30.0));
    assert_eq!(obj.transform.rotation, Vec3::new(0.1, 0.2, 0.3));
    assert_eq!(obj.transform.scale, Vec3::new(2.0, 3.0, 4.0));
}

#[test]
fn move_outcome_records_previous_transform_for_undo() {
    let mut scene = Scene::new();
    let original = Transform3D::from_translation(Vec3::new(1.0, 1.0, 1.0));
    let id = scene.create_object(original);

    let outcome = Viewport3DOperation::Move {
        id,
        translation: Vec3::new(9.0, 9.0, 9.0),
    }
    .apply(&mut scene)
    .unwrap();

    match outcome {
        Viewport3DOutcome::Moved { previous, .. } => assert_eq!(previous, original),
        other => panic!("expected Moved, got {other:?}"),
    }
}

#[test]
fn set_parent_operation() {
    let mut scene = Scene::new();
    let parent = scene.create_object(Transform3D::identity());
    let child = scene.create_object(Transform3D::identity());

    let outcome = Viewport3DOperation::SetParent {
        id: child,
        parent: Some(parent),
    }
    .apply(&mut scene)
    .unwrap();

    match outcome {
        Viewport3DOutcome::ParentSet { previous_parent, .. } => {
            assert_eq!(previous_parent, None);
        }
        other => panic!("expected ParentSet, got {other:?}"),
    }
    assert_eq!(scene.get(child).unwrap().parent, Some(parent));
}

#[test]
fn clear_scene_operation() {
    let mut scene = Scene::new();
    scene.create_object(Transform3D::identity());
    scene.create_object(Transform3D::identity());

    let outcome = Viewport3DOperation::Clear
        .apply(&mut scene)
        .unwrap();

    match outcome {
        Viewport3DOutcome::Cleared { count } => assert_eq!(count, 2),
        other => panic!("expected Cleared, got {other:?}"),
    }
    assert!(scene.is_empty());
}