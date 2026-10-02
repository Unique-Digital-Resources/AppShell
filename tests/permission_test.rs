use app_shell::app_engine::*;
use std::sync::Arc;

// --- PermissionCheckedEngineRef construction ---

#[test]
fn permission_checked_ref_constructs() {
    let engine = app_shell::app_engine::TestEngine::new();
    let engine_ref = engine.engine_ref();

    let checked = PermissionCheckedEngineRef::new(
        &engine_ref,
        PluginPermissions::FULL,
        "test_plugin",
    );

    assert_eq!(checked.plugin_id(), "test_plugin");
    assert!(checked.permissions().can_load_resources());
}

#[test]
fn permission_checked_ref_with_no_permissions() {
    let engine = app_shell::app_engine::TestEngine::new();
    let engine_ref = engine.engine_ref();

    let checked = PermissionCheckedEngineRef::new(
        &engine_ref,
        PluginPermissions::new(0),  // No permissions
        "restricted_plugin",
    );

    assert!(!checked.permissions().can_load_resources());
    assert!(!checked.permissions().can_write_config());
    assert!(!checked.permissions().can_publish_events());
}

// --- Read-only access (always allowed) ---

#[test]
fn read_only_access_always_allowed_with_no_permissions() {
    let engine = app_shell::app_engine::TestEngine::new();
    let engine_ref = engine.engine_ref();

    let checked = PermissionCheckedEngineRef::new(
        &engine_ref,
        PluginPermissions::new(0),  // No permissions
        "restricted",
    );

    // Read access always works
    let _ = checked.config_store();
    let _ = checked.preferences();
    let _ = checked.state_manager();
    let _ = checked.history_store();
    let _ = checked.scheduler();
    let _ = checked.event_bus();
    let _ = checked.signal_bus();
    let _ = checked.command_executor();
    let _ = checked.resource_manager();
}

#[test]
fn read_only_access_with_full_permissions() {
    let engine = app_shell::app_engine::TestEngine::new();
    let engine_ref = engine.engine_ref();

    let checked = PermissionCheckedEngineRef::new(
        &engine_ref,
        PluginPermissions::FULL,
        "full_access",
    );

    // All read access works
    assert_eq!(checked.config_store().key_count(), 0);
    assert!(checked.preferences().is_empty());
}

// --- Permission-checked write access ---

#[test]
fn try_load_resource_with_permission_succeeds() {
    let engine = app_shell::app_engine::TestEngine::new();
    let engine_ref = engine.engine_ref();

    let checked = PermissionCheckedEngineRef::new(
        &engine_ref,
        PluginPermissions::new(LOAD_RESOURCES),
        "resource_plugin",
    );

    let result = checked.try_load_resource();
    assert!(result.is_ok());
    let mgr = result.unwrap();
    assert_eq!(mgr.resource_count(), 0);
}

#[test]
fn try_load_resource_without_permission_fails() {
    let engine = app_shell::app_engine::TestEngine::new();
    let engine_ref = engine.engine_ref();

    let checked = PermissionCheckedEngineRef::new(
        &engine_ref,
        PluginPermissions::new(0),  // No permissions
        "restricted",
    );

    let result = checked.try_load_resource();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, PluginSecurityError::PermissionDenied { .. }));
    assert!(err.to_string().contains("LOAD_RESOURCES"));
    assert!(err.to_string().contains("restricted"));
}

#[test]
fn try_write_config_with_permission_succeeds() {
    let engine = app_shell::app_engine::TestEngine::new();
    let engine_ref = engine.engine_ref();

    let checked = PermissionCheckedEngineRef::new(
        &engine_ref,
        PluginPermissions::new(WRITE_CONFIG),
        "config_plugin",
    );

    let result = checked.try_write_config();
    assert!(result.is_ok());
}

#[test]
fn try_write_config_without_permission_fails() {
    let engine = app_shell::app_engine::TestEngine::new();
    let engine_ref = engine.engine_ref();

    let checked = PermissionCheckedEngineRef::new(
        &engine_ref,
        PluginPermissions::new(0),
        "no_config",
    );

    let result = checked.try_write_config();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("WRITE_CONFIG"));
}

#[test]
fn try_publish_event_with_permission_succeeds() {
    let engine = app_shell::app_engine::TestEngine::new();
    let engine_ref = engine.engine_ref();

    let checked = PermissionCheckedEngineRef::new(
        &engine_ref,
        PluginPermissions::new(PUBLISH_EVENTS),
        "publisher",
    );

    let result = checked.try_publish_event();
    assert!(result.is_ok());
    let bus = result.unwrap();
    bus.publish(&Event::new("test.event"));
}

#[test]
fn try_publish_event_without_permission_fails() {
    let engine = app_shell::app_engine::TestEngine::new();
    let engine_ref = engine.engine_ref();

    let checked = PermissionCheckedEngineRef::new(
        &engine_ref,
        PluginPermissions::new(0),
        "silent",
    );

    let result = checked.try_publish_event();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("PUBLISH_EVENTS"));
}

#[test]
fn try_register_commands_with_permission_succeeds() {
    let engine = app_shell::app_engine::TestEngine::new();
    let engine_ref = engine.engine_ref();

    let checked = PermissionCheckedEngineRef::new(
        &engine_ref,
        PluginPermissions::new(REGISTER_COMMANDS),
        "command_plugin",
    );

    let result = checked.try_register_commands();
    assert!(result.is_ok());
}

#[test]
fn try_register_commands_without_permission_fails() {
    let engine = app_shell::app_engine::TestEngine::new();
    let engine_ref = engine.engine_ref();

    let checked = PermissionCheckedEngineRef::new(
        &engine_ref,
        PluginPermissions::new(0),
        "no_commands",
    );

    let result = checked.try_register_commands();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("REGISTER_COMMANDS"));
}

// --- Combined permissions ---

#[test]
fn combined_permissions_work() {
    let engine = app_shell::app_engine::TestEngine::new();
    let engine_ref = engine.engine_ref();

    let checked = PermissionCheckedEngineRef::new(
        &engine_ref,
        PluginPermissions::new(LOAD_RESOURCES | PUBLISH_EVENTS),
        "combined",
    );

    // Has LOAD_RESOURCES
    assert!(checked.try_load_resource().is_ok());
    // Has PUBLISH_EVENTS
    assert!(checked.try_publish_event().is_ok());
    // Does NOT have WRITE_CONFIG
    assert!(checked.try_write_config().is_err());
    // Does NOT have REGISTER_COMMANDS
    assert!(checked.try_register_commands().is_err());
}

// --- Full access (backward compatible) ---

#[test]
fn full_access_allows_everything() {
    let engine = app_shell::app_engine::TestEngine::new();
    let engine_ref = engine.engine_ref();

    let checked = PermissionCheckedEngineRef::new(
        &engine_ref,
        PluginPermissions::FULL,
        "full_access_plugin",
    );

    // All try_* methods succeed
    assert!(checked.try_load_resource().is_ok());
    assert!(checked.try_write_config().is_ok());
    assert!(checked.try_publish_event().is_ok());
    assert!(checked.try_subscribe_events().is_ok());
    assert!(checked.try_register_commands().is_ok());
    assert!(checked.try_register_tasks().is_ok());
    assert!(checked.try_register_services().is_ok());
}

#[test]
fn default_permissions_are_full_access() {
    let engine = app_shell::app_engine::TestEngine::new();
    let engine_ref = engine.engine_ref();

    // Default permissions = FULL_ACCESS
    let checked = PermissionCheckedEngineRef::new(
        &engine_ref,
        PluginPermissions::default(),
        "default_perms",
    );

    assert!(checked.try_load_resource().is_ok());
    assert!(checked.try_write_config().is_ok());
    assert!(checked.try_publish_event().is_ok());
}

// --- Integration with PluginManager ---

#[test]
fn plugin_manager_queries_permissions() {
    let mut mgr = PluginManager::new();

    let id = mgr.register(
        PluginManifest::new("limited", "Limited")
            .with_permissions(PluginPermissions::new(REGISTER_COMMANDS)),
        Box::new(StubPlugin),
    ).unwrap();

    assert!(mgr.can_register_commands(id));
    assert!(!mgr.can_load_resources(id));
    assert!(!mgr.can_publish_events(id));
}

#[test]
fn plugin_manager_full_access_by_default() {
    let mut mgr = PluginManager::new();

    let mut id = mgr.register(
        PluginManifest::new("full", "Full"),
        Box::new(StubPlugin),
    ).unwrap();

    assert!(mgr.can_register_commands(id));
    assert!(mgr.can_load_resources(id));
    assert!(mgr.can_publish_events(id));
    assert!(mgr.can_write_config(id));
}

// --- PermissionCheckedEngineRef with real EngineRef from runtime ---

#[test]
fn permission_checked_ref_with_runtime() {
    let engine = Bootstrap::create().unwrap();
    let engine_ref = engine.engine_ref();

    // Plugin with limited permissions
    let checked = PermissionCheckedEngineRef::new(
        &engine_ref,
        PluginPermissions::new(LOAD_RESOURCES | SUBSCRIBE_EVENTS),
        "limited_plugin",
    );

    // Can load resources
    assert!(checked.try_load_resource().is_ok());
    // Can subscribe to events
    assert!(checked.try_subscribe_events().is_ok());
    // Cannot write config
    assert!(checked.try_write_config().is_err());
    // Cannot publish events
    assert!(checked.try_publish_event().is_err());
    // Cannot register commands
    assert!(checked.try_register_commands().is_err());
}

#[test]
fn permission_checked_ref_debug() {
    let engine = app_shell::app_engine::TestEngine::new();
    let engine_ref = engine.engine_ref();

    let checked = PermissionCheckedEngineRef::new(
        &engine_ref,
        PluginPermissions::FULL,
        "debug_test",
    );

    let debug_str = format!("{:?}", checked);
    assert!(debug_str.contains("debug_test"));
}

// --- Stub plugin ---

struct StubPlugin;
impl Plugin for StubPlugin {
    fn id(&self) -> &str { "stub" }
    fn initialize(&mut self, _ctx: &PluginContext, _engine: &EngineRef) -> Result<(), PluginError> { Ok(()) }
    fn start(&mut self, _engine: &EngineRef) -> Result<(), PluginError> { Ok(()) }
    fn stop(&mut self) -> Result<(), PluginError> { Ok(()) }
    fn dispose(&mut self) -> Result<(), PluginError> { Ok(()) }
}

// --- Plugin with restricted permissions still gets lifecycle ---

#[test]
fn restricted_plugin_still_lifecycle() {
    let ctx = app_shell::app_engine::TestEngine::new();
    let engine_ref = ctx.engine_ref();

    let mut mgr = PluginManager::new();
    let id = mgr.register(
        PluginManifest::new("restricted", "Restricted")
            .with_permissions(PluginPermissions::new(0)),
        Box::new(StubPlugin),
    ).unwrap();

    // Lifecycle still works — permissions don't block lifecycle
    mgr.initialize(id, &engine_ref).unwrap();
    assert_eq!(mgr.state(id), Some(PluginState::Initialized));

    mgr.start(id, &engine_ref).unwrap();
    assert_eq!(mgr.state(id), Some(PluginState::Started));

    mgr.stop(id).unwrap();
    mgr.dispose(id).unwrap();
}

// --- Error message contains plugin ID and permission name ---

#[test]
fn permission_denied_error_contains_details() {
    let engine = app_shell::app_engine::TestEngine::new();
    let engine_ref = engine.engine_ref();

    let checked = PermissionCheckedEngineRef::new(
        &engine_ref,
        PluginPermissions::new(0),
        "my_plugin",
    );

    let err = checked.try_load_resource().unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("my_plugin"));
    assert!(msg.contains("LOAD_RESOURCES"));
}