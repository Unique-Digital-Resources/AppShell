use app_shell::app_engine::{
    Bootstrap, ConfigStore, Lifecycle, Preferences,
};

#[test]
fn runtime_owns_config_store() {
    let runtime = Bootstrap::create().unwrap();
    let _: &ConfigStore = runtime.config_store();
}

#[test]
fn runtime_owns_preferences() {
    let runtime = Bootstrap::create().unwrap();
    let _: &Preferences = runtime.preferences();
}

#[test]
fn runtime_config_store_starts_empty() {
    let runtime = Bootstrap::create().unwrap();
    assert_eq!(runtime.config_store().key_count(), 0);
}

#[test]
fn runtime_preferences_starts_empty() {
    let runtime = Bootstrap::create().unwrap();
    assert!(runtime.preferences().is_empty());
}

#[test]
fn runtime_can_set_and_get_config() {
    let mut runtime = Bootstrap::create().unwrap();
    runtime.config_store_mut().set("app.test", "hello").unwrap();
    assert_eq!(runtime.config_store().get("app.test").as_deref(), Some("hello"));
}

#[test]
fn runtime_can_set_preferences() {
    let mut runtime = Bootstrap::create().unwrap();
    runtime.preferences_mut().set("theme", "dark");
    assert_eq!(runtime.preferences().get("theme").as_deref(), Some("dark"));
}

#[test]
fn configuration_independent_of_lifecycle() {
    let mut runtime = Bootstrap::create().unwrap();
    runtime.initialize().unwrap();
    runtime.start().unwrap();
    assert_eq!(runtime.config_store().key_count(), 0);
    assert!(runtime.preferences().is_empty());
    runtime.stop().unwrap();
    runtime.dispose().unwrap();
}