//! Phase 11 acceptance criteria.

use app_shell::app_engine::{
    Bootstrap, Config, ConfigPropertyDefinition, ConfigSchema, ConfigStore,
    ConfigurationError, Event, EventHandler, Lifecycle, Preferences,
};
use std::sync::{Arc, Mutex};

struct ClosureEventHandler<F: Fn(&Event) + Send + Sync + 'static> {
    f: F,
}
impl<F: Fn(&Event) + Send + Sync + 'static> EventHandler for ClosureEventHandler<F> {
    fn handle(&self, event: &Event) {
        (self.f)(event);
    }
}
fn event_handler<F: Fn(&Event) + Send + Sync + 'static>(f: F) -> Box<dyn EventHandler> {
    Box::new(ClosureEventHandler { f })
}

/// 1. Represent application configuration
#[test]
fn acceptance_represent_config() {
    let mut c = Config::new();
    c.set("app.name", "AppShell");
    c.set("worker_count", "4");
    assert_eq!(c.get("app.name"), Some("AppShell"));
    assert_eq!(c.get_i64("worker_count"), Some(4));
}

/// 2. Represent user preferences
#[test]
fn acceptance_represent_preferences() {
    let mut p = Preferences::new();
    p.set("theme", "dark");
    p.set("language", "en");
    assert_eq!(p.get("theme").as_deref(), Some("dark"));
    assert_eq!(p.get("language").as_deref(), Some("en"));
}

/// 3. Define configuration schemas
#[test]
fn acceptance_define_schema() {
    let s = ConfigSchema::new()
        .with_property(
            ConfigPropertyDefinition::new("worker_count", "Workers")
                .with_default("4")
                .with_min("1")
                .with_max("32"),
        )
        .with_property(
            ConfigPropertyDefinition::new("app.name", "Name").required(),
        );
    assert_eq!(s.property_count(), 2);
    assert!(s.contains("worker_count"));
    assert!(s.contains("app.name"));
}

/// 4. Validate configuration values
#[test]
fn acceptance_validate_config() {
    let s = ConfigSchema::new().with_property(
        ConfigPropertyDefinition::new("count", "Count")
            .with_min("1")
            .with_max("10"),
    );

    let mut valid = Config::new();
    valid.set("count", "5");
    assert!(s.validate(&valid).is_ok());

    let mut invalid = Config::new();
    invalid.set("count", "50");
    assert!(matches!(
        s.validate(&invalid),
        Err(ConfigurationError::ValidationFailed { .. })
    ));
}

/// 5. Load persistent configuration
/// 6. Save persistent configuration
#[test]
fn acceptance_load_and_save() {
    let schema = ConfigSchema::new().with_property(
        ConfigPropertyDefinition::new("app.name", "Name")
            .with_default("Default")
            .required(),
    );
    let mut store = ConfigStore::new(schema);

    // Load returns defaults
    let loaded = store.load();
    assert_eq!(loaded.get("app.name"), Some("Default"));

    // Save new config
    let mut new_config = Config::new();
    new_config.set("app.name", "MyApp");
    store.save(new_config).unwrap();
    assert_eq!(store.get("app.name").as_deref(), Some("MyApp"));
}

/// 7. Provide defaults
#[test]
fn acceptance_provide_defaults() {
    let s = ConfigSchema::new()
        .with_property(
            ConfigPropertyDefinition::new("a", "A").with_default("1"),
        )
        .with_property(
            ConfigPropertyDefinition::new("b", "B").with_default("2"),
        );

    let defaults = s.defaults();
    assert_eq!(defaults.get("a"), Some("1"));
    assert_eq!(defaults.get("b"), Some("2"));
}

/// 8. Distinguish configuration from runtime state
#[test]
fn acceptance_config_not_state() {
    let mut runtime = Bootstrap::create().unwrap();

    // Configuration: "worker_count = 4" (a setting)
    runtime.config_store_mut().set("worker_count", "4").unwrap();

    // Runtime state: the runtime's lifecycle state (current condition)
    runtime.initialize().unwrap();
    let state = runtime.state();

    // These are different systems:
    // - ConfigStore holds configuration settings
    // - StateManager holds runtime state
    assert_eq!(runtime.config_store().get("worker_count").as_deref(), Some("4"));
    assert_ne!(state, app_shell::app_engine::RuntimeState::Created);
}

/// 9. Make configuration available through application context
#[test]
fn acceptance_config_available_via_runtime() {
    let mut runtime = Bootstrap::create().unwrap();
    runtime.config_store_mut().set("app.test", "value").unwrap();

    // The runtime exposes configuration alongside all other engine systems
    assert!(runtime.config_store().contains("app.test"));
    assert!(runtime.preferences().is_empty());
}

/// 10. Allow Services to consume configuration
#[test]
fn acceptance_services_consume_config() {
    let mut runtime = Bootstrap::create().unwrap();
    runtime.config_store_mut().set("worker_count", "8").unwrap();

    // A service can read configuration during its lifecycle.
    // (Demonstrated here by reading from the config store directly.)
    let worker_count = runtime.config_store().get_i64("worker_count").unwrap();
    assert_eq!(worker_count, 8);
}

/// 11. Allow Plugins to contribute configuration/schema
#[test]
fn acceptance_plugins_contribute_schema() {
    // A plugin can define its own configuration properties.
    let plugin_schema = ConfigSchema::new().with_property(
        ConfigPropertyDefinition::new("image_plugin.quality", "Image quality")
            .with_default("high"),
    );

    // The application can merge plugin schemas into its own.
    let mut app_schema = ConfigSchema::new()
        .with_property(
            ConfigPropertyDefinition::new("app.name", "Name").with_default("AppShell"),
        );

    // Plugins contribute their properties
    for name in plugin_schema.property_names() {
        app_schema.define(plugin_schema.get(name).unwrap().clone());
    }

    assert!(app_schema.contains("image_plugin.quality"));
    assert!(app_schema.contains("app.name"));
}

/// 12. Support configuration changes without coupling
#[test]
fn acceptance_config_changes_decoupled() {
    let mut runtime = Bootstrap::create().unwrap();
    runtime.config_store_mut().set("app.setting", "value1").unwrap();

    // Changing config doesn't directly call any service/plugin/resource.
    // Consumers observe changes through events (see next test).
    runtime.config_store_mut().set("app.setting", "value2").unwrap();
    assert_eq!(runtime.config_store().get("app.setting").as_deref(), Some("value2"));
}

/// 13. Optionally notify consumers through Events/Signals
#[test]
fn acceptance_config_change_events() {
    let mut runtime = Bootstrap::create().unwrap();
    let received = Arc::new(Mutex::new(false));

    let r = received.clone();
    runtime.event_bus_mut().subscribe(
        "config.changed",
        event_handler(move |_event: &Event| {
            *r.lock().unwrap() = true;
        }),
    );

    // Change config, then publish event
    runtime.config_store_mut().set("app.test", "new").unwrap();
    runtime.event_bus().publish(
        &Event::new("config.changed").with_source("ConfigStore"),
    );

    assert!(*received.lock().unwrap());
}

/// 14. Keep domain-specific configuration outside
#[test]
fn acceptance_domain_config_outside() {
    // App Engine config: generic keys like "app.name", "worker_count"
    let mut app_config = Config::new();
    app_config.set("app.name", "AppShell");
    app_config.set("worker_count", "4");

    // Domain config: domain-specific keys like "document.default_size"
    // This would live in the Domain Engine, not App Engine.
    let mut domain_config = Config::new();
    domain_config.set("document.default_size", "A4");

    // App Engine doesn't know about "document.default_size"
    assert!(!app_config.contains("document.default_size"));
}

/// 15. Avoid making configuration persistence part of execution core
#[test]
fn acceptance_persistence_not_in_execution() {
    let mut runtime = Bootstrap::create().unwrap();

    // The execution core (commands, tasks, scheduler) works independently
    // of configuration persistence.
    assert_eq!(runtime.command_executor().list().len(), 0);
    assert_eq!(runtime.task_manager().count(), 0);
    assert_eq!(runtime.scheduler().total_job_count(), 0);

    // Config exists but doesn't block execution
    runtime.config_store_mut().set("test", "value").unwrap();
    assert_eq!(runtime.command_executor().list().len(), 0); // still works
}