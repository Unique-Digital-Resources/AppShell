use app_shell::app_engine::PluginManifest;

#[test]
fn manifest_stores_basic_fields() {
    let m = PluginManifest::new("image-plugin", "Image Plugin")
        .with_version("2.0.0")
        .with_description("Image processing")
        .with_author("Alice");
    assert_eq!(m.id(), "image-plugin");
    assert_eq!(m.name(), "Image Plugin");
    assert_eq!(m.version(), "2.0.0");
    assert_eq!(m.description(), "Image processing");
    assert_eq!(m.author(), "Alice");
}

#[test]
fn manifest_with_dependencies() {
    let m = PluginManifest::new("a", "A")
        .with_dependency("b")
        .with_dependency("c");
    assert_eq!(m.dependencies(), &["b".to_string(), "c".to_string()]);
}

#[test]
fn manifest_with_capabilities() {
    let m = PluginManifest::new("a", "A")
        .with_capability("commands")
        .with_capability("resources");
    assert_eq!(m.capabilities(), &["commands".to_string(), "resources".to_string()]);
}

#[test]
fn manifest_with_metadata() {
    let m = PluginManifest::new("a", "A")
        .with_metadata("license", "MIT");
    assert_eq!(m.metadata().get("license"), Some(&"MIT".to_string()));
}

#[test]
fn manifest_equality() {
    let a = PluginManifest::new("x", "X");
    let b = PluginManifest::new("x", "X");
    assert_eq!(a, b);
}