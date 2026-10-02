use app_shell::app_engine::{
    Config, ConfigPropertyDefinition, ConfigSchema, ConfigStore, ConfigurationError,
};

fn make_store() -> ConfigStore {
    let schema = ConfigSchema::new()
        .with_property(
            ConfigPropertyDefinition::new("app.name", "Application name")
                .with_default("AppShell")
                .required(),
        )
        .with_property(
            ConfigPropertyDefinition::new("worker_count", "Worker count")
                .with_default("4")
                .with_min("1")
                .with_max("32"),
        );
    ConfigStore::new(schema)
}

#[test]
fn new_store_has_defaults() {
    let store = make_store();
    let config = store.load();
    assert_eq!(config.get("app.name"), Some("AppShell"));
    assert_eq!(config.get("worker_count"), Some("4"));
}

#[test]
fn get_value() {
    let store = make_store();
    assert_eq!(store.get("app.name").as_deref(), Some("AppShell"));
    assert!(store.get("missing").is_none());
}

#[test]
fn set_value() {
    let store = make_store();
    store.set("worker_count", "8").unwrap();
    assert_eq!(store.get("worker_count").as_deref(), Some("8"));
}

#[test]
fn set_invalid_value_fails() {
    let store = make_store();
    let result = store.set("worker_count", "100");
    assert!(matches!(result, Err(ConfigurationError::ValidationFailed { .. })));
    // Original value is unchanged
    assert_eq!(store.get("worker_count").as_deref(), Some("4"));
}

#[test]
fn save_valid_config() {
    let store = make_store();
    let mut config = Config::new();
    config.set("app.name", "MyApp");
    config.set("worker_count", "16");
    store.save(config).unwrap();
    assert_eq!(store.get("app.name").as_deref(), Some("MyApp"));
    assert_eq!(store.get("worker_count").as_deref(), Some("16"));
}

#[test]
fn save_invalid_config_fails() {
    let store = make_store();
    let mut config = Config::new();
    // Missing required "app.name"
    config.set("worker_count", "8");
    let result = store.save(config);
    assert!(matches!(result, Err(ConfigurationError::ValidationFailed { .. })));
}

#[test]
fn reset_to_defaults() {
    let store = make_store();
    store.set("app.name", "Changed").unwrap();
    store.reset();
    assert_eq!(store.get("app.name").as_deref(), Some("AppShell"));
}

#[test]
fn validate_external_config() {
    let store = make_store();
    let mut config = Config::new();
    config.set("app.name", "TestApp");
    assert!(store.validate(&config).is_ok());

    let empty = Config::new();
    assert!(store.validate(&empty).is_err()); // missing required
}

#[test]
fn contains_and_key_count() {
    let store = make_store();
    assert!(store.contains("app.name"));
    assert!(!store.contains("missing"));
    assert_eq!(store.key_count(), 2);
}

#[test]
fn get_typed_values() {
    let store = make_store();
    assert_eq!(store.get_i64("worker_count"), Some(4));
}