use app_shell::app_engine::{PluginId, PluginState};

#[test]
fn plugin_id_is_unique() {
    let a = PluginId::new();
    let b = PluginId::new();
    assert_ne!(a, b);
}

#[test]
fn plugin_state_queries() {
    assert!(PluginState::Loaded.can_initialize());
    assert!(!PluginState::Started.can_initialize());
    assert!(PluginState::Initialized.can_start());
    assert!(PluginState::Stopped.can_start());
    assert!(PluginState::Started.can_stop());
    assert!(PluginState::Stopped.can_dispose());
    assert!(PluginState::Unloaded.is_terminal());
    assert!(PluginState::Started.is_active());
}

#[test]
fn plugin_state_as_str() {
    assert_eq!(PluginState::Loaded.as_str(), "Loaded");
    assert_eq!(PluginState::Started.as_str(), "Started");
    assert_eq!(PluginState::Unloaded.as_str(), "Unloaded");
}