//! Operation tests for the Layers environment.

use app_shell::domain_foundation::environments::layers::{
    LayerId, Layers, LayersError, LayersOperation, LayersOutcome,
};

#[test]
fn create_layer_operation() {
    let mut layers = Layers::new();
    let outcome = LayersOperation::CreateLayer { name: "Base".to_string() }
        .apply(&mut layers)
        .unwrap();

    let id = match outcome {
        LayersOutcome::LayerCreated { id } => id,
        other => panic!("expected LayerCreated, got {other:?}"),
    };
    assert!(layers.contains(id));
    assert_eq!(layers.layer(id).unwrap().name, "Base");
    assert_eq!(layers.root_count(), 1);
}

#[test]
fn create_child_operation() {
    let mut layers = Layers::new();
    let parent = layers.create_layer("Parent".to_string());

    let outcome = LayersOperation::CreateChild {
        parent,
        name: "Child".to_string(),
    }
    .apply(&mut layers)
    .unwrap();

    let id = match outcome {
        LayersOutcome::LayerCreated { id } => id,
        other => panic!("expected LayerCreated, got {other:?}"),
    };
    assert_eq!(layers.layer(id).unwrap().parent, Some(parent));
    assert_eq!(layers.layer(parent).unwrap().children(), &[id]);
}

#[test]
fn remove_layer_operation_returns_subtree() {
    let mut layers = Layers::new();
    let root = layers.create_layer("Root".to_string());
    let child = layers.create_child(root, "Child".to_string()).unwrap();
    let _grandchild = layers.create_child(child, "Grandchild".to_string()).unwrap();

    let outcome = LayersOperation::RemoveLayer { id: root }
        .apply(&mut layers)
        .unwrap();

    match outcome {
        LayersOutcome::LayerRemoved { layers: removed } => {
            assert_eq!(removed.len(), 3);
        }
        other => panic!("expected LayerRemoved, got {other:?}"),
    }
    assert_eq!(layers.len(), 0);
}

#[test]
fn rename_layer_operation() {
    let mut layers = Layers::new();
    let id = layers.create_layer("Old".to_string());

    let outcome = LayersOperation::RenameLayer {
        id,
        name: "New".to_string(),
    }
    .apply(&mut layers)
    .unwrap();

    match outcome {
        LayersOutcome::LayerRenamed { previous_name, .. } => {
            assert_eq!(previous_name, "Old");
        }
        other => panic!("expected LayerRenamed, got {other:?}"),
    }
    assert_eq!(layers.layer(id).unwrap().name, "New");
}

#[test]
fn set_visibility_operation() {
    let mut layers = Layers::new();
    let id = layers.create_layer("Layer".to_string());

    let outcome = LayersOperation::SetVisibility { id, visible: false }
        .apply(&mut layers)
        .unwrap();

    match outcome {
        LayersOutcome::VisibilityChanged { previous_visible, .. } => {
            assert!(previous_visible);
        }
        other => panic!("expected VisibilityChanged, got {other:?}"),
    }
    assert!(!layers.layer(id).unwrap().visible);
}

#[test]
fn set_locked_operation() {
    let mut layers = Layers::new();
    let id = layers.create_layer("Layer".to_string());

    let outcome = LayersOperation::SetLocked { id, locked: true }
        .apply(&mut layers)
        .unwrap();

    match outcome {
        LayersOutcome::LockedChanged { previous_locked, .. } => {
            assert!(!previous_locked);
        }
        other => panic!("expected LockedChanged, got {other:?}"),
    }
    assert!(layers.layer(id).unwrap().locked);
}

#[test]
fn reorder_sibling_operation() {
    let mut layers = Layers::new();
    let a = layers.create_layer("A".to_string());
    let b = layers.create_layer("B".to_string());
    let c = layers.create_layer("C".to_string());

    let outcome = LayersOperation::ReorderSibling { id: c, new_index: 0 }
        .apply(&mut layers)
        .unwrap();

    match outcome {
        LayersOutcome::SiblingReordered { previous_index, .. } => {
            assert_eq!(previous_index, 2);
        }
        other => panic!("expected SiblingReordered, got {other:?}"),
    }
    assert_eq!(layers.roots(), &[c, a, b]);
}

#[test]
fn reparent_operation() {
    let mut layers = Layers::new();
    let parent_a = layers.create_layer("A".to_string());
    let parent_b = layers.create_layer("B".to_string());
    let child = layers.create_child(parent_a, "Child".to_string()).unwrap();

    let outcome = LayersOperation::Reparent {
        id: child,
        new_parent: Some(parent_b),
    }
    .apply(&mut layers)
    .unwrap();

    match outcome {
        LayersOutcome::Reparented { previous_parent, .. } => {
            assert_eq!(previous_parent, Some(parent_a));
        }
        other => panic!("expected Reparented, got {other:?}"),
    }
    assert_eq!(layers.layer(child).unwrap().parent, Some(parent_b));
    assert!(layers.layer(parent_a).unwrap().children().is_empty());
    assert_eq!(layers.layer(parent_b).unwrap().children(), &[child]);
}

#[test]
fn operations_on_missing_layer_fail() {
    let mut layers = Layers::new();
    let fake = LayerId::from_raw(999);

    let err = LayersOperation::RemoveLayer { id: fake }
        .apply(&mut layers)
        .unwrap_err();
    assert!(matches!(err, LayersError::LayerNotFound { .. }));

    let err = LayersOperation::SetVisibility { id: fake, visible: false }
        .apply(&mut layers)
        .unwrap_err();
    assert!(matches!(err, LayersError::LayerNotFound { .. }));

    let err = LayersOperation::Reparent { id: fake, new_parent: None }
        .apply(&mut layers)
        .unwrap_err();
    assert!(matches!(err, LayersError::LayerNotFound { .. }));
}

#[test]
fn reparent_cycle_detected() {
    let mut layers = Layers::new();
    let parent = layers.create_layer("Parent".to_string());
    let child = layers.create_child(parent, "Child".to_string()).unwrap();

    let err = LayersOperation::Reparent {
        id: parent,
        new_parent: Some(child),
    }
    .apply(&mut layers)
    .unwrap_err();
    assert!(matches!(err, LayersError::CycleDetected { .. }));
}

#[test]
fn sequence_of_operations_composes() {
    let mut layers = Layers::new();

    // Create root layers.
    let outcome = LayersOperation::CreateLayer { name: "A".to_string() }
        .apply(&mut layers)
        .unwrap();
    let root_a = match outcome {
        LayersOutcome::LayerCreated { id } => id,
        _ => unreachable!(),
    };

    let outcome = LayersOperation::CreateLayer { name: "B".to_string() }
        .apply(&mut layers)
        .unwrap();
    let root_b = match outcome {
        LayersOutcome::LayerCreated { id } => id,
        _ => unreachable!(),
    };

    // Add child to A.
    let outcome = LayersOperation::CreateChild {
        parent: root_a,
        name: "A1".to_string(),
    }
    .apply(&mut layers)
    .unwrap();
    let child_a1 = match outcome {
        LayersOutcome::LayerCreated { id } => id,
        _ => unreachable!(),
    };

    // Move A1 to B.
    LayersOperation::Reparent {
        id: child_a1,
        new_parent: Some(root_b),
    }
    .apply(&mut layers)
    .unwrap();

    // Hide A.
    LayersOperation::SetVisibility { id: root_a, visible: false }
        .apply(&mut layers)
        .unwrap();

    // Verify final state.
    assert_eq!(layers.len(), 3);
    assert!(!layers.layer(root_a).unwrap().visible);
    assert!(layers.layer(root_a).unwrap().children().is_empty());
    assert_eq!(layers.layer(root_b).unwrap().children(), &[child_a1]);
    assert_eq!(layers.layer(child_a1).unwrap().parent, Some(root_b));
}

#[test]
fn clear_layers_operation() {
    let mut layers = Layers::new();
    layers.create_layer("A".to_string());
    layers.create_layer("B".to_string());

    let outcome = LayersOperation::Clear
        .apply(&mut layers)
        .unwrap();
    assert!(matches!(outcome, LayersOutcome::Cleared));
    assert_eq!(layers.len(), 0);
}

#[test]
fn duplicate_layer_operation() {
    let mut layers = Layers::new();
    let root = layers.create_layer("Original".to_string());
    layers.set_visibility(root, false).unwrap();

    let outcome = LayersOperation::DuplicateLayer { id: root }
        .apply(&mut layers)
        .unwrap();

    match outcome {
        LayersOutcome::LayerDuplicated { source, new_id } => {
            assert_eq!(source, root);
            assert_ne!(new_id, root);
            // Duplicate should be a root.
            assert!(layers.layer(new_id).unwrap().is_root());
            // Properties copied.
            assert_eq!(layers.layer(new_id).unwrap().name, "Original");
            assert!(!layers.layer(new_id).unwrap().visible);
        }
        other => panic!("expected LayerDuplicated, got {other:?}"),
    }
    assert_eq!(layers.len(), 2);
}