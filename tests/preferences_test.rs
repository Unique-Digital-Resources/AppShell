use app_shell::app_engine::Preferences;

#[test]
fn new_preferences_empty() {
    let p = Preferences::new();
    assert!(p.is_empty());
    assert_eq!(p.len(), 0);
}

#[test]
fn set_and_get() {
    let p = Preferences::new();
    p.set("theme", "dark");
    p.set("language", "en");
    p.set("show_toolbar", "true");

    assert_eq!(p.get("theme").as_deref(), Some("dark"));
    assert_eq!(p.get_string("language").as_deref(), Some("en"));
    assert_eq!(p.get_bool("show_toolbar"), Some(true));
}

#[test]
fn get_missing_returns_none() {
    let p = Preferences::new();
    assert_eq!(p.get("missing"), None);
}

#[test]
fn remove() {
    let p = Preferences::new();
    p.set("key", "value");
    let removed = p.remove("key");
    assert_eq!(removed.as_deref(), Some("value"));
    assert!(!p.contains("key"));
}

#[test]
fn contains() {
    let p = Preferences::new();
    p.set("key", "value");
    assert!(p.contains("key"));
    assert!(!p.contains("missing"));
}

#[test]
fn keys_iterator() {
    let p = Preferences::new();
    p.set("a", "1");
    p.set("b", "2");
    let mut keys = p.keys();
    keys.sort();
    assert_eq!(keys, vec!["a".to_string(), "b".to_string()]);
}

#[test]
fn clear() {
    let p = Preferences::new();
    p.set("a", "1");
    p.set("b", "2");
    p.clear();
    assert!(p.is_empty());
}

#[test]
fn clone_not_needed() {
    // Preferences no longer derives Clone (has Mutex).
    // Create two instances with same data instead.
    let a = Preferences::new();
    a.set("key", "value");

    let b = Preferences::new();
    b.set("key", "value");

    assert_eq!(a.get("key").as_deref(), b.get("key").as_deref());
}