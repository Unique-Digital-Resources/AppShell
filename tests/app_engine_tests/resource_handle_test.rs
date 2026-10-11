use app_shell::app_engine::{ResourceId, ResourceHandle};

#[test]
fn handle_stores_resource_id() {
    let id = ResourceId::new();
    let h = ResourceHandle::new(id);
    assert_eq!(h.resource_id(), id);
}

#[test]
fn handle_with_same_id_are_equal() {
    let id = ResourceId::new();
    let h1 = ResourceHandle::new(id);
    let h2 = ResourceHandle::new(id);
    assert_eq!(h1, h2);
    assert_eq!(h1.resource_id(), h2.resource_id());
}

#[test]
fn handle_equality() {
    let id = ResourceId::new();
    let a = ResourceHandle::new(id);
    let b = ResourceHandle::new(id);
    assert_eq!(a, b);
}

#[test]
fn handle_different_ids_not_equal() {
    let a = ResourceHandle::new(ResourceId::new());
    let b = ResourceHandle::new(ResourceId::new());
    assert_ne!(a, b);
}