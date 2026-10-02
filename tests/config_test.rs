use app_shell::app_engine::Config;

#[test]
fn new_config_is_empty() {
    let c = Config::new();
    assert!(c.is_empty());
    assert_eq!(c.len(), 0);
}

#[test]
fn set_and_get() {
    let mut c = Config::new();
    c.set("app.name", "AppShell");
    assert_eq!(c.get("app.name"), Some("AppShell"));
    assert!(c.contains("app.name"));
}

#[test]
fn get_missing_returns_none() {
    let c = Config::new();
    assert!(c.get("missing").is_none());
}

#[test]
fn get_typed_values() {
    let mut c = Config::new();
    c.set("count", "42");
    c.set("ratio", "3.14");
    c.set("enabled", "true");

    assert_eq!(c.get_i64("count"), Some(42));
    assert_eq!(c.get_f64("ratio"), Some(3.14));
    assert_eq!(c.get_bool("enabled"), Some(true));
}

#[test]
fn remove() {
    let mut c = Config::new();
    c.set("key", "value");
    let removed = c.remove("key");
    assert_eq!(removed, Some("value".to_string()));
    assert!(!c.contains("key"));
}

#[test]
fn merge() {
    let mut a = Config::new();
    a.set("a", "1");
    a.set("b", "2");

    let mut b = Config::new();
    b.set("b", "3");
    b.set("c", "4");

    a.merge(&b);
    assert_eq!(a.get("a"), Some("1"));
    assert_eq!(a.get("b"), Some("3")); // overwritten
    assert_eq!(a.get("c"), Some("4"));
}

#[test]
fn set_if_absent() {
    let mut c = Config::new();
    c.set_if_absent("key", "first");
    c.set_if_absent("key", "second");
    assert_eq!(c.get("key"), Some("first"));
}

#[test]
fn keys_iterator() {
    let mut c = Config::new();
    c.set("a", "1");
    c.set("b", "2");
    let mut keys: Vec<&str> = c.keys().collect();
    keys.sort();
    assert_eq!(keys, vec!["a", "b"]);
}

#[test]
fn clone() {
    let mut a = Config::new();
    a.set("key", "value");
    let b = a.clone();
    assert_eq!(a.get("key"), b.get("key"));
}