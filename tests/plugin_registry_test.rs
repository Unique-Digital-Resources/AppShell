use app_shell::app_engine::{PluginError, PluginManifest, PluginRegistry};

#[test]
fn new_registry_is_empty() {
    let r = PluginRegistry::new();
    assert!(r.is_empty());
}

#[test]
fn register_and_get() {
    let mut r = PluginRegistry::new();
    let m = PluginManifest::new("a", "A");
    r.register(m).unwrap();
    assert_eq!(r.len(), 1);
    assert!(r.contains("a"));
    assert_eq!(r.get("a").unwrap().name(), "A");
}

#[test]
fn register_duplicate_fails() {
    let mut r = PluginRegistry::new();
    r.register(PluginManifest::new("a", "A")).unwrap();
    let result = r.register(PluginManifest::new("a", "A"));
    assert!(matches!(result, Err(PluginError::AlreadyRegistered { .. })));
}

#[test]
fn unregister_removes() {
    let mut r = PluginRegistry::new();
    r.register(PluginManifest::new("a", "A")).unwrap();
    let removed = r.unregister("a");
    assert!(removed.is_some());
    assert!(!r.contains("a"));
}

#[test]
fn list_returns_all() {
    let mut r = PluginRegistry::new();
    r.register(PluginManifest::new("a", "A")).unwrap();
    r.register(PluginManifest::new("b", "B")).unwrap();
    assert_eq!(r.list().len(), 2);
}