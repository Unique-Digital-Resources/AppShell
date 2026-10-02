use app_shell::app_engine::{Resource, ResourceId, ResourceKey, ResourceState};

#[test]
fn resource_id_is_unique() {
    let a = ResourceId::new();
    let b = ResourceId::new();
    assert_ne!(a, b);
}

#[test]
fn resource_starts_unloaded() {
    let r = Resource::new("image", "file:///logo.png");
    assert_eq!(r.resource_type(), "image");
    assert_eq!(r.source(), "file:///logo.png");
    assert_eq!(r.state(), ResourceState::Unloaded);
    assert!(!r.has_data());
}

#[test]
fn resource_with_data_is_loaded() {
    let r = Resource::new("image", "file:///logo.png").with_data("pixel_data".to_string());
    assert_eq!(r.state(), ResourceState::Loaded);
    assert!(r.has_data());
    assert_eq!(r.data::<String>(), Some(&"pixel_data".to_string()));
}

#[test]
fn resource_metadata() {
    let r = Resource::new("image", "file:///logo.png")
        .with_metadata("size", "1024x768")
        .with_metadata("format", "png");
    assert_eq!(r.metadata().get("size"), Some(&"1024x768".to_string()));
    assert_eq!(r.metadata().get("format"), Some(&"png".to_string()));
}

#[test]
fn resource_key_equality() {
    let a = ResourceKey::new("image", "file:///logo.png");
    let b = ResourceKey::new("image", "file:///logo.png");
    let c = ResourceKey::new("font", "file:///logo.png");
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn resource_state_queries() {
    assert!(ResourceState::Loaded.is_loaded());
    assert!(!ResourceState::Unloaded.is_loaded());
    assert!(ResourceState::Loading.is_active());
    assert!(ResourceState::Unloaded.is_terminal());
    assert!(ResourceState::Released.is_terminal());
    assert!(!ResourceState::Loaded.is_terminal());
}