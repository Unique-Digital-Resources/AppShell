//! Model and query tests for the Layers environment.

use app_shell::domain_foundation::environments::layers::{
    Layer, LayerId, Layers, LayersError,
};

#[test]
fn new_hierarchy_is_empty() {
    let layers = Layers::new();
    assert!(layers.is_empty());
    assert_eq!(layers.len(), 0);
    assert_eq!(layers.root_count(), 0);
    assert_eq!(layers.depth(), 0);
}

#[test]
fn create_layer_adds_root() {
    let mut layers = Layers::new();
    let id = layers.create_layer("Background".to_string());
    assert!(layers.contains(id));
    assert_eq!(layers.root_count(), 1);
    assert_eq!(layers.roots(), &[id]);
    let layer = layers.layer(id).unwrap();
    assert_eq!(layer.name, "Background");
    assert!(layer.visible);
    assert!(!layer.locked);
    assert!(layer.is_root());
    assert!(!layer.has_children());
}

#[test]
fn create_child_adds_to_parent() {
    let mut layers = Layers::new();
    let parent = layers.create_layer("Parent".to_string());
    let child = layers.create_child(parent, "Child".to_string()).unwrap();

    assert!(layers.contains(child));
    let c = layers.layer(child).unwrap();
    assert_eq!(c.parent, Some(parent));
    assert!(!c.is_root());

    let p = layers.layer(parent).unwrap();
    assert_eq!(p.children(), &[child]);
    assert!(p.has_children());
    assert_eq!(p.child_count(), 1);
}

#[test]
fn create_child_on_missing_parent_fails() {
    let mut layers = Layers::new();
    let fake = LayerId::from_raw(999);
    let err = layers.create_child(fake, "Orphan".to_string()).unwrap_err();
    assert!(matches!(err, LayersError::LayerNotFound { id } if id == fake));
}

#[test]
fn multi_level_hierarchy() {
    let mut layers = Layers::new();
    let root = layers.create_layer("Root".to_string());
    let child = layers.create_child(root, "Child".to_string()).unwrap();
    let grandchild = layers.create_child(child, "Grandchild".to_string()).unwrap();

    assert_eq!(layers.len(), 3);
    assert_eq!(layers.depth(), 3);
    assert_eq!(layers.descendants(root), vec![child, grandchild]);
    assert!(layers.is_descendant(grandchild, root));
    assert!(layers.is_descendant(grandchild, child));
    assert!(!layers.is_descendant(root, grandchild));
}

#[test]
fn remove_layer_cascades_to_descendants() {
    let mut layers = Layers::new();
    let root = layers.create_layer("Root".to_string());
    let child = layers.create_child(root, "Child".to_string()).unwrap();
    let _grandchild = layers.create_child(child, "Grandchild".to_string()).unwrap();

    assert_eq!(layers.len(), 3);

    let removed = layers.remove_layer(root).unwrap();
    assert_eq!(removed.len(), 3); // root + child + grandchild
    assert!(!layers.contains(root));
    assert!(!layers.contains(child));
    assert_eq!(layers.len(), 0);
}

#[test]
fn remove_from_middle_of_sibling_list() {
    let mut layers = Layers::new();
    let parent = layers.create_layer("Parent".to_string());
    let a = layers.create_child(parent, "A".to_string()).unwrap();
    let b = layers.create_child(parent, "B".to_string()).unwrap();
    let c = layers.create_child(parent, "C".to_string()).unwrap();

    layers.remove_layer(b).unwrap();

    let p = layers.layer(parent).unwrap();
    assert_eq!(p.children(), &[a, c]);
    assert!(!layers.contains(b));
}

#[test]
fn set_visibility_and_locked() {
    let mut layers = Layers::new();
    let id = layers.create_layer("Layer".to_string());

    let prev = layers.set_visibility(id, false).unwrap();
    assert!(prev);
    assert!(!layers.layer(id).unwrap().visible);

    let prev = layers.set_locked(id, true).unwrap();
    assert!(!prev);
    assert!(layers.layer(id).unwrap().locked);
}

#[test]
fn rename_layer() {
    let mut layers = Layers::new();
    let id = layers.create_layer("Old".to_string());

    let prev = layers.rename(id, "New".to_string()).unwrap();
    assert_eq!(prev, "Old");
    assert_eq!(layers.layer(id).unwrap().name, "New");
}

#[test]
fn reorder_root_siblings() {
    let mut layers = Layers::new();
    let a = layers.create_layer("A".to_string());
    let b = layers.create_layer("B".to_string());
    let c = layers.create_layer("C".to_string());

    assert_eq!(layers.roots(), &[a, b, c]);

    let prev = layers.reorder_sibling(b, 0).unwrap();
    assert_eq!(prev, 1);
    assert_eq!(layers.roots(), &[b, a, c]);
}

#[test]
fn reorder_child_siblings() {
    let mut layers = Layers::new();
    let parent = layers.create_layer("Parent".to_string());
    let a = layers.create_child(parent, "A".to_string()).unwrap();
    let b = layers.create_child(parent, "B".to_string()).unwrap();
    let c = layers.create_child(parent, "C".to_string()).unwrap();

    let prev = layers.reorder_sibling(c, 0).unwrap();
    assert_eq!(prev, 2);
    assert_eq!(layers.layer(parent).unwrap().children(), &[c, a, b]);
}

#[test]
fn reorder_out_of_range_fails() {
    let mut layers = Layers::new();
    let a = layers.create_layer("A".to_string());
    let _b = layers.create_layer("B".to_string());

    let err = layers.reorder_sibling(a, 5).unwrap_err();
    assert!(matches!(err, LayersError::InvalidSiblingIndex { .. }));
}

#[test]
fn reparent_to_different_parent() {
    let mut layers = Layers::new();
    let parent_a = layers.create_layer("A".to_string());
    let parent_b = layers.create_layer("B".to_string());
    let child = layers.create_child(parent_a, "Child".to_string()).unwrap();

    let prev_parent = layers.reparent(child, Some(parent_b)).unwrap();
    assert_eq!(prev_parent, Some(parent_a));
    assert_eq!(layers.layer(child).unwrap().parent, Some(parent_b));
    assert_eq!(layers.layer(parent_a).unwrap().children(), &[]);
    assert_eq!(layers.layer(parent_b).unwrap().children(), &[child]);
}

#[test]
fn reparent_to_root() {
    let mut layers = Layers::new();
    let parent = layers.create_layer("Parent".to_string());
    let child = layers.create_child(parent, "Child".to_string()).unwrap();

    let prev = layers.reparent(child, None).unwrap();
    assert_eq!(prev, Some(parent));
    assert!(layers.layer(child).unwrap().is_root());
    assert!(layers.layer(parent).unwrap().children().is_empty());
    assert!(layers.roots().contains(&child));
}

#[test]
fn reparent_cycle_rejected() {
    let mut layers = Layers::new();
    let parent = layers.create_layer("Parent".to_string());
    let child = layers.create_child(parent, "Child".to_string()).unwrap();
    let grandchild = layers.create_child(child, "Grandchild".to_string()).unwrap();

    // Moving parent under grandchild would create a cycle.
    let err = layers.reparent(parent, Some(grandchild)).unwrap_err();
    assert!(matches!(err, LayersError::CycleDetected { .. }));

    // Moving parent under child would also create a cycle.
    let err = layers.reparent(parent, Some(child)).unwrap_err();
    assert!(matches!(err, LayersError::CycleDetected { .. }));
}

#[test]
fn reparent_to_self_rejected() {
    let mut layers = Layers::new();
    let id = layers.create_layer("Self".to_string());

    let err = layers.reparent(id, Some(id)).unwrap_err();
    assert!(matches!(err, LayersError::CycleDetected { .. }));
}

#[test]
fn insert_layer_restores_subtree() {
    let mut layers = Layers::new();
    let root = layers.create_layer("Root".to_string());
    let child = layers.create_child(root, "Child".to_string()).unwrap();

    let removed = layers.remove_layer(root).unwrap();
    assert_eq!(layers.len(), 0);

    // Re-insert all removed layers in order.
    for layer in removed {
        layers.insert_layer(layer);
    }

    assert!(layers.contains(root));
    assert!(layers.contains(child));
    assert_eq!(layers.layer(root).unwrap().children(), &[child]);
    assert_eq!(layers.layer(child).unwrap().parent, Some(root));
}

#[test]
fn effective_visibility_propagates_from_parent() {
    let mut layers = Layers::new();
    let root = layers.create_layer("Root".to_string());
    let child = layers.create_child(root, "Child".to_string()).unwrap();
    let grandchild = layers.create_child(child, "Grandchild".to_string()).unwrap();

    assert!(layers.effective_visibility(grandchild));

    layers.set_visibility(root, false).unwrap();
    assert!(!layers.effective_visibility(grandchild));
    assert!(!layers.effective_visibility(child));
    assert!(!layers.effective_visibility(root));

    layers.set_visibility(root, true).unwrap();
    layers.set_visibility(child, false).unwrap();
    assert!(!layers.effective_visibility(grandchild));
    assert!(layers.effective_visibility(root));
}

#[test]
fn layers_clear_empties() {
    let mut layers = Layers::new();
    layers.create_layer("A".to_string());
    layers.create_layer("B".to_string());
    assert_eq!(layers.len(), 2);
    
    layers.clear();
    assert_eq!(layers.len(), 0);
    assert!(layers.roots().is_empty());
}

#[test]
fn layers_partial_eq_works() {
    let a = Layers::new();
    let b = Layers::new();
    assert_eq!(a, b);
}

#[test]
fn effective_locked_propagates_from_parent() {
    let mut layers = Layers::new();
    let root = layers.create_layer("Root".to_string());
    let child = layers.create_child(root, "Child".to_string()).unwrap();
    let grandchild = layers.create_child(child, "Grandchild".to_string()).unwrap();

    assert!(!layers.effective_locked(grandchild));

    layers.set_locked(root, true).unwrap();
    assert!(layers.effective_locked(grandchild));
    assert!(layers.effective_locked(child));
    assert!(layers.effective_locked(root));

    layers.set_locked(root, false).unwrap();
    layers.set_locked(child, true).unwrap();
    assert!(layers.effective_locked(grandchild));
    assert!(!layers.effective_locked(root));
}

#[test]
fn layer_path_from_root_to_leaf() {
    let mut layers = Layers::new();
    let root = layers.create_layer("Root".to_string());
    let child = layers.create_child(root, "Child".to_string()).unwrap();
    let grandchild = layers.create_child(child, "Grandchild".to_string()).unwrap();

    let path = layers.layer_path(grandchild);
    assert_eq!(path, vec![root, child, grandchild]);

    let path = layers.layer_path(child);
    assert_eq!(path, vec![root, child]);

    let path = layers.layer_path(root);
    assert_eq!(path, vec![root]);
}

#[test]
fn layer_path_on_missing_returns_empty() {
    let layers = Layers::new();
    let fake = app_shell::domain_foundation::environments::layers::LayerId::from_raw(999);
    assert!(layers.layer_path(fake).is_empty());
}

#[test]
fn move_up_and_down() {
    let mut layers = Layers::new();
    let a = layers.create_layer("A".to_string());
    let b = layers.create_layer("B".to_string());
    let c = layers.create_layer("C".to_string());

    // [a, b, c]
    assert_eq!(layers.roots(), &[a, b, c]);

    // Move c up one → [a, c, b]
    assert!(layers.move_up(c).unwrap());
    assert_eq!(layers.roots(), &[a, c, b]);

    // Move c up again → [c, a, b]
    assert!(layers.move_up(c).unwrap());
    assert_eq!(layers.roots(), &[c, a, b]);

    // Already at front.
    assert!(!layers.move_up(c).unwrap());
    assert_eq!(layers.roots(), &[c, a, b]);

    // Move c down → [a, c, b]
    assert!(layers.move_down(c).unwrap());
    assert_eq!(layers.roots(), &[a, c, b]);

    // Move a down → [c, a, b]
    assert!(layers.move_down(a).unwrap());
    assert_eq!(layers.roots(), &[c, a, b]);

    // Move b down → already at back.
    assert!(!layers.move_down(b).unwrap());
    assert_eq!(layers.roots(), &[c, a, b]);
}
#[test]
fn move_up_down_on_child_siblings() {
    let mut layers = Layers::new();
    let parent = layers.create_layer("Parent".to_string());
    let a = layers.create_child(parent, "A".to_string()).unwrap();
    let b = layers.create_child(parent, "B".to_string()).unwrap();
    let c = layers.create_child(parent, "C".to_string()).unwrap();

    assert_eq!(layers.layer(parent).unwrap().children(), &[a, b, c]);

    // Move c to front.
    layers.move_up(c).unwrap();
    layers.move_up(c).unwrap();
    assert_eq!(layers.layer(parent).unwrap().children(), &[c, a, b]);

    // Move c to back.
    layers.move_down(c).unwrap();
    layers.move_down(c).unwrap();
    assert_eq!(layers.layer(parent).unwrap().children(), &[a, b, c]);
}