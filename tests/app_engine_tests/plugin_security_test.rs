use app_shell::app_engine::{
    PluginManifest, PluginPermissions, PluginManager, Plugin, PluginContext,
    PluginError, PluginId, PluginLoader, EngineRef, DynamicPluginLoader,
    EventBus, SignalBus, ResourceManager, ConfigStore, ConfigSchema, Preferences,
    StateManager, CommandExecutor, Scheduler, HistoryStore, LifecycleState,
    REGISTER_COMMANDS, SUBSCRIBE_EVENTS, FULL_ACCESS,
};

// --- Test context ---

struct TestContext {
    event_bus: EventBus,
    signal_bus: SignalBus,
    resource_manager: ResourceManager,
    config_store: ConfigStore,
    preferences: Preferences,
    state_manager: StateManager<LifecycleState>,
    command_executor: CommandExecutor,
    scheduler: Scheduler,
    history_store: HistoryStore,
}

impl TestContext {
    fn new() -> Self {
        Self {
            event_bus: EventBus::new(),
            signal_bus: SignalBus::new(),
            resource_manager: ResourceManager::new(),
            config_store: ConfigStore::new(ConfigSchema::new()),
            preferences: Preferences::new(),
            state_manager: StateManager::new(),
            command_executor: CommandExecutor::new(),
            scheduler: Scheduler::new(),
            history_store: HistoryStore::new(),
        }
    }

    fn engine_ref(&self) -> EngineRef<'_> {
        EngineRef::new(
            &self.event_bus, &self.signal_bus, &self.resource_manager,
            &self.config_store, &self.preferences, &self.state_manager,
            &self.command_executor, &self.scheduler, &self.history_store,
        )
    }
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

// --- Permission flag tests ---

#[test]
fn permissions_none_has_no_access() {
    let perms = PluginPermissions::new(0);
    assert!(!perms.can_register_commands());
    assert!(!perms.can_load_resources());
    assert!(!perms.can_publish_events());
}

#[test]
fn permissions_full_has_all_access() {
    let perms = PluginPermissions::new(FULL_ACCESS);
    assert!(perms.can_register_commands());
    assert!(perms.can_register_tasks());
    assert!(perms.can_load_resources());
    assert!(perms.can_register_services());
    assert!(perms.can_subscribe_events());
    assert!(perms.can_publish_events());
    assert!(perms.can_read_config());
    assert!(perms.can_write_config());
}

#[test]
fn permissions_bitor_combines() {
    let perms = PluginPermissions::new(REGISTER_COMMANDS)
        | PluginPermissions::new(SUBSCRIBE_EVENTS);
    assert!(perms.can_register_commands());
    assert!(perms.can_subscribe_events());
    assert!(!perms.can_load_resources());
    assert!(!perms.can_publish_events());
}

#[test]
fn permissions_default_is_full() {
    let perms = PluginPermissions::default();
    assert!(perms.can_register_commands());
    assert!(perms.can_load_resources());
}

#[test]
fn permissions_display() {
    let perms = PluginPermissions::new(REGISTER_COMMANDS);
    assert!(perms.to_string().contains("00000001"));
}

// --- Manifest with permissions ---

#[test]
fn manifest_default_has_full_permissions() {
    let m = PluginManifest::new("test", "Test");
    let perms = m.permissions();
    assert!(perms.can_register_commands());
    assert!(perms.can_load_resources());
}

#[test]
fn manifest_with_limited_permissions() {
    let m = PluginManifest::new("limited", "Limited")
        .with_permissions(PluginPermissions::new(
            REGISTER_COMMANDS | SUBSCRIBE_EVENTS,
        ));
    let perms = m.permissions();
    assert!(perms.can_register_commands());
    assert!(perms.can_subscribe_events());
    assert!(!perms.can_load_resources());
    assert!(!perms.can_write_config());
}

#[test]
fn manifest_with_no_permissions() {
    let m = PluginManifest::new("restricted", "Restricted")
        .with_permissions(PluginPermissions::new(0));
    let perms = m.permissions();
    assert!(!perms.can_register_commands());
    assert!(!perms.can_load_resources());
    assert!(!perms.can_publish_events());
}

// --- PluginManager permission queries ---

#[test]
fn manager_queries_plugin_permissions() {
    let mut mgr = PluginManager::new();

    let id = mgr.register(
        PluginManifest::new("limited", "Limited")
            .with_permissions(PluginPermissions::new(REGISTER_COMMANDS)),
        Box::new(StubPlugin),
    ).unwrap();

    assert!(mgr.can_register_commands(id));
    assert!(!mgr.can_load_resources(id));
    assert!(!mgr.can_register_tasks(id));
}

#[test]
fn manager_default_plugin_has_full_access() {
    let mut mgr = PluginManager::new();

    let id = mgr.register(
        PluginManifest::new("full", "Full"),
        Box::new(StubPlugin),
    ).unwrap();

    assert!(mgr.can_register_commands(id));
    assert!(mgr.can_load_resources(id));
    assert!(mgr.can_publish_events(id));
}

#[test]
fn manager_permission_check_for_unregistered_plugin() {
    let mgr = PluginManager::new();
    let bogus = PluginId::new();
    assert!(!mgr.has_permission(bogus, REGISTER_COMMANDS));
    assert!(!mgr.can_register_commands(bogus));
}

// --- Dynamic loader tests ---

#[test]
fn dynamic_loader_detects_shared_libraries() {
    assert!(DynamicPluginLoader::is_shared_library("plugin.so"));
    assert!(DynamicPluginLoader::is_shared_library("plugin.dll"));
    assert!(DynamicPluginLoader::is_shared_library("plugin.dylib"));
    assert!(!DynamicPluginLoader::is_shared_library("plugin.rs"));
    assert!(!DynamicPluginLoader::is_shared_library("plugin.json"));
}

#[test]
fn dynamic_loader_cannot_load_missing_file() {
    let loader = DynamicPluginLoader::new();
    assert!(!loader.can_load("nonexistent.so"));
}

#[test]
fn dynamic_loader_with_search_paths() {
    let mut loader = DynamicPluginLoader::new();
    loader.add_search_path("/tmp/plugins");

    assert_eq!(loader.search_paths_len(), 1);
}

#[test]
fn dynamic_loader_load_returns_descriptive_error() {
    let loader = DynamicPluginLoader::new();
    let result = loader.load("nonexistent.so");
    assert!(matches!(result, Err(PluginError::LoadFailed { .. })));
}

// --- PluginSecurityError tests ---

#[test]
fn permission_denied_display() {
    let e = app_shell::app_engine::PluginSecurityError::PermissionDenied {
        plugin_id: "limited".to_string(),
        permission: "LOAD_RESOURCES".to_string(),
    };
    assert!(e.to_string().contains("limited"));
    assert!(e.to_string().contains("LOAD_RESOURCES"));
}

#[test]
fn plugin_security_error_implements_std_error() {
    fn assert_std_error<E: std::error::Error>(_: E) {}
    assert_std_error(app_shell::app_engine::PluginSecurityError::PermissionDenied {
        plugin_id: "x".to_string(),
        permission: "y".to_string(),
    });
}

// --- Integration: permissions don't block lifecycle ---

#[test]
fn limited_permission_plugin_still_lifecycle() {
    let ctx = TestContext::new();
    let mut mgr = PluginManager::new();

    let id = mgr.register(
        PluginManifest::new("limited", "Limited")
            .with_permissions(PluginPermissions::new(REGISTER_COMMANDS)),
        Box::new(StubPlugin),
    ).unwrap();

    let engine_ref = ctx.engine_ref();
    mgr.initialize(id, &engine_ref).unwrap();
    assert_eq!(mgr.state(id), Some(app_shell::app_engine::PluginState::Initialized));

    mgr.start(id, &engine_ref).unwrap();
    assert_eq!(mgr.state(id), Some(app_shell::app_engine::PluginState::Started));

    mgr.stop(id).unwrap();
    mgr.dispose(id).unwrap();
}