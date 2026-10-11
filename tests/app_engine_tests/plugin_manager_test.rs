use app_shell::app_engine::{
    EngineRef,
    EventBus, SignalBus, ResourceManager, ConfigStore, ConfigSchema, Preferences,
    StateManager, CommandExecutor, Scheduler, HistoryStore,
    Plugin, PluginContext, PluginError, PluginId, PluginLoader, PluginManager,
    PluginManifest, PluginState, LifecycleState,
};
use std::sync::{Arc, Mutex};

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
            &self.event_bus,
            &self.signal_bus,
            &self.resource_manager,
            &self.config_store,
            &self.preferences,
            &self.state_manager,
            &self.command_executor,
            &self.scheduler,
            &self.history_store,
        )
    }
}

// --- Stub plugin ---

struct StubPlugin {
    id: String,
    initialized: Arc<Mutex<bool>>,
    started: Arc<Mutex<bool>>,
    stopped: Arc<Mutex<bool>>,
    disposed: Arc<Mutex<bool>>,
    received_context_id: Arc<Mutex<Option<String>>>,
}

impl StubPlugin {
    fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            initialized: Arc::new(Mutex::new(false)),
            started: Arc::new(Mutex::new(false)),
            stopped: Arc::new(Mutex::new(false)),
            disposed: Arc::new(Mutex::new(false)),
            received_context_id: Arc::new(Mutex::new(None)),
        }
    }
}

impl Plugin for StubPlugin {
    fn id(&self) -> &str { &self.id }
    fn initialize(&mut self, ctx: &PluginContext, _engine: &EngineRef) -> Result<(), PluginError> {
        *self.initialized.lock().unwrap() = true;
        *self.received_context_id.lock().unwrap() = Some(ctx.plugin_id().to_string());
        Ok(())
    }
    fn start(&mut self, _engine: &EngineRef) -> Result<(), PluginError> {
        *self.started.lock().unwrap() = true;
        Ok(())
    }
    fn stop(&mut self) -> Result<(), PluginError> {
        *self.stopped.lock().unwrap() = true;
        Ok(())
    }
    fn dispose(&mut self) -> Result<(), PluginError> {
        *self.disposed.lock().unwrap() = true;
        Ok(())
    }
}

fn make_manifest(id: &str) -> PluginManifest {
    PluginManifest::new(id, id)
}

#[test]
fn new_manager_is_empty() {
    let mgr = PluginManager::new();
    assert_eq!(mgr.count(), 0);
}

#[test]
fn register_plugin() {
    let mut mgr = PluginManager::new();
    let id = mgr.register(make_manifest("image"), Box::new(StubPlugin::new("image"))).unwrap();
    assert_eq!(mgr.count(), 1);
    assert!(mgr.contains(id));
    assert!(mgr.contains_type("image"));
    assert_eq!(mgr.state(id), Some(PluginState::Loaded));
}

#[test]
fn register_duplicate_fails() {
    let mut mgr = PluginManager::new();
    mgr.register(make_manifest("image"), Box::new(StubPlugin::new("image"))).unwrap();
    let result = mgr.register(make_manifest("image"), Box::new(StubPlugin::new("image")));
    assert!(matches!(result, Err(PluginError::AlreadyRegistered { .. })));
}

#[test]
fn full_lifecycle() {
    let ctx = TestContext::new();
    let mut mgr = PluginManager::new();
    let id = mgr.register(make_manifest("image"), Box::new(StubPlugin::new("image"))).unwrap();
    let engine_ref = ctx.engine_ref();

    mgr.initialize(id, &engine_ref).unwrap();
    assert_eq!(mgr.state(id), Some(PluginState::Initialized));

    mgr.start(id, &engine_ref).unwrap();
    assert_eq!(mgr.state(id), Some(PluginState::Started));

    mgr.stop(id).unwrap();
    assert_eq!(mgr.state(id), Some(PluginState::Stopped));

    mgr.dispose(id).unwrap();
    assert_eq!(mgr.state(id), Some(PluginState::Unloaded));
}

#[test]
fn invalid_transitions_rejected() {
    let ctx = TestContext::new();
    let mut mgr = PluginManager::new();
    let id = mgr.register(make_manifest("image"), Box::new(StubPlugin::new("image"))).unwrap();
    let engine_ref = ctx.engine_ref();

    // Can't start before initialize
    assert!(mgr.start(id, &engine_ref).is_err());

    mgr.initialize(id, &engine_ref).unwrap();
    // Can't initialize twice
    assert!(mgr.initialize(id, &engine_ref).is_err());

    mgr.start(id, &engine_ref).unwrap();
    // Can't dispose while started (must stop first)
    assert!(mgr.dispose(id).is_err());
}

#[test]
fn plugin_receives_context_during_initialize() {
    let ctx = TestContext::new();
    let mut mgr = PluginManager::new();
    let received = Arc::new(Mutex::new(None::<String>));

    struct CtxPlugin {
        id: String,
        received: Arc<Mutex<Option<String>>>,
    }
    impl Plugin for CtxPlugin {
        fn id(&self) -> &str { &self.id }
        fn initialize(&mut self, ctx: &PluginContext, _engine: &EngineRef) -> Result<(), PluginError> {
            *self.received.lock().unwrap() = Some(ctx.plugin_id().to_string());
            Ok(())
        }
        fn start(&mut self, _engine: &EngineRef) -> Result<(), PluginError> { Ok(()) }
        fn stop(&mut self) -> Result<(), PluginError> { Ok(()) }
        fn dispose(&mut self) -> Result<(), PluginError> { Ok(()) }
    }

    let id = mgr.register(
        make_manifest("ctx-test"),
        Box::new(CtxPlugin { id: "ctx-test".to_string(), received: received.clone() }),
    ).unwrap();

    let engine_ref = ctx.engine_ref();
    mgr.initialize(id, &engine_ref).unwrap();
    assert_eq!(*received.lock().unwrap(), Some("ctx-test".to_string()));
}

#[test]
fn bulk_lifecycle() {
    let ctx = TestContext::new();
    let mut mgr = PluginManager::new();
    mgr.register(make_manifest("a"), Box::new(StubPlugin::new("a"))).unwrap();
    mgr.register(make_manifest("b"), Box::new(StubPlugin::new("b"))).unwrap();
    mgr.register(make_manifest("c"), Box::new(StubPlugin::new("c"))).unwrap();
    let engine_ref = ctx.engine_ref();

    mgr.initialize_all(&engine_ref).unwrap();
    for id in mgr.list_ids() {
        assert_eq!(mgr.state(id), Some(PluginState::Initialized));
    }

    mgr.start_all(&engine_ref).unwrap();
    for id in mgr.list_ids() {
        assert_eq!(mgr.state(id), Some(PluginState::Started));
    }

    mgr.stop_all().unwrap();
    for id in mgr.list_ids() {
        assert_eq!(mgr.state(id), Some(PluginState::Stopped));
    }

    mgr.dispose_all().unwrap();
    for id in mgr.list_ids() {
        assert_eq!(mgr.state(id), Some(PluginState::Unloaded));
    }
}

#[test]
fn dependency_validation() {
    let mut mgr = PluginManager::new();
    let id = mgr.register(
        PluginManifest::new("a", "A").with_dependency("b"),
        Box::new(StubPlugin::new("a")),
    ).unwrap();

    // Dependency "b" is not registered
    let result = mgr.validate_dependencies(id);
    assert!(matches!(result, Err(PluginError::DependencyNotMet { .. })));

    // Register "b"
    mgr.register(make_manifest("b"), Box::new(StubPlugin::new("b"))).unwrap();
    mgr.validate_dependencies(id).unwrap();
}

#[test]
fn unregister_removes_plugin() {
    let mut mgr = PluginManager::new();
    let id = mgr.register(make_manifest("image"), Box::new(StubPlugin::new("image"))).unwrap();
    assert!(mgr.contains(id));
    let removed = mgr.unregister(id);
    assert!(removed.is_some());
    assert!(!mgr.contains(id));
    assert_eq!(mgr.count(), 0);
}

#[test]
fn not_found_returns_error() {
    let ctx = TestContext::new();
    let mut mgr = PluginManager::new();
    let bogus = PluginId::new();
    let engine_ref = ctx.engine_ref();
    let result = mgr.start(bogus, &engine_ref);
    assert!(matches!(result, Err(PluginError::NotFound { .. })));
}

// --- Stub loader ---

struct StaticLoader;

impl PluginLoader for StaticLoader {
    fn can_load(&self, source: &str) -> bool {
        source.starts_with("static:")
    }
    fn load(&self, source: &str) -> Result<(PluginManifest, Box<dyn Plugin>), PluginError> {
        let id = source.strip_prefix("static:").unwrap_or(source);
        Ok((PluginManifest::new(id, id), Box::new(StubPlugin::new(id))))
    }
}

#[test]
fn register_and_use_loader() {
    let ctx = TestContext::new();
    let mut mgr = PluginManager::new();
    mgr.register_loader(Box::new(StaticLoader));
    assert_eq!(mgr.loader_count(), 1);

    let id = mgr.load("static:loaded-plugin").unwrap();
    assert!(mgr.contains(id));
    assert_eq!(mgr.state(id), Some(PluginState::Loaded));

    let engine_ref = ctx.engine_ref();
    mgr.initialize(id, &engine_ref).unwrap();
    assert_eq!(mgr.state(id), Some(PluginState::Initialized));
}

#[test]
fn load_without_loader_fails() {
    let mut mgr = PluginManager::new();
    let result = mgr.load("unknown-source");
    assert!(matches!(result, Err(PluginError::LoadFailed { .. })));
}