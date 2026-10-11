use app_shell::app_engine::{Config, ConfigPropertyDefinition, ConfigSchema};

#[test]
fn new_schema_is_empty() {
    let s = ConfigSchema::new();
    assert_eq!(s.property_count(), 0);
}

#[test]
fn define_property() {
    let mut s = ConfigSchema::new();
    s.define(
        ConfigPropertyDefinition::new("worker_count", "Number of workers")
            .with_default("4")
            .with_min("1")
            .with_max("32"),
    );
    assert_eq!(s.property_count(), 1);
    assert!(s.contains("worker_count"));
    let def = s.get("worker_count").unwrap();
    assert_eq!(def.default(), Some("4"));
    assert_eq!(def.min_value(), Some("1"));
    assert_eq!(def.max_value(), Some("32"));
    assert!(!def.is_required());
}

#[test]
fn required_property() {
    let s = ConfigSchema::new().with_property(
        ConfigPropertyDefinition::new("app.name", "App name").required(),
    );
    assert!(s.get("app.name").unwrap().is_required());
}

#[test]
fn defaults() {
    let s = ConfigSchema::new()
        .with_property(
            ConfigPropertyDefinition::new("app.name", "App name").with_default("AppShell"),
        )
        .with_property(
            ConfigPropertyDefinition::new("worker_count", "Workers").with_default("4"),
        )
        .with_property(
            ConfigPropertyDefinition::new("required_key", "Required"), // no default
        );

    let defaults = s.defaults();
    assert_eq!(defaults.get("app.name"), Some("AppShell"));
    assert_eq!(defaults.get("worker_count"), Some("4"));
    assert!(!defaults.contains("required_key"));
}

#[test]
fn validate_passes() {
    let s = ConfigSchema::new().with_property(
        ConfigPropertyDefinition::new("count", "Count")
            .with_default("4")
            .with_min("1")
            .with_max("32"),
    );
    let mut c = Config::new();
    c.set("count", "8");
    assert!(s.validate(&c).is_ok());
}

#[test]
fn validate_missing_required_fails() {
    let s = ConfigSchema::new().with_property(
        ConfigPropertyDefinition::new("app.name", "App name").required(),
    );
    let c = Config::new();
    assert!(s.validate(&c).is_err());
}

#[test]
fn validate_below_min_fails() {
    let s = ConfigSchema::new().with_property(
        ConfigPropertyDefinition::new("count", "Count")
            .with_min("1")
            .with_max("32"),
    );
    let mut c = Config::new();
    c.set("count", "0");
    assert!(s.validate(&c).is_err());
}

#[test]
fn validate_above_max_fails() {
    let s = ConfigSchema::new().with_property(
        ConfigPropertyDefinition::new("count", "Count")
            .with_min("1")
            .with_max("32"),
    );
    let mut c = Config::new();
    c.set("count", "100");
    assert!(s.validate(&c).is_err());
}

#[test]
fn validate_optional_missing_passes() {
    let s = ConfigSchema::new().with_property(
        ConfigPropertyDefinition::new("optional", "Optional"), // not required
    );
    let c = Config::new();
    assert!(s.validate(&c).is_ok());
}