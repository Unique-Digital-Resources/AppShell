use app_shell::app_engine::{
    EngineRef,
    EventBus, SignalBus, ResourceManager, ConfigStore, ConfigSchema, Preferences,
    StateManager, CommandExecutor, Scheduler, HistoryStore,
    ServiceDefinition, ServiceError, ServiceId, ServiceManager, ServiceState,
    Service, LifecycleState,
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

// --- Stub services ---

struct StubService {
    type_name: String,
    started: Arc<Mutex<bool>>,
    stopped: Arc<Mutex<bool>>,
    initialized: Arc<Mutex<bool>>,
    disposed: Arc<Mutex<bool>>,
}

impl StubService {
    fn new(name: &str) -> Self {
        Self {
            type_name: name.to_string(),
            started: Arc::new(Mutex::new(false)),
            stopped: Arc::new(Mutex::new(false)),
            initialized: Arc::new(Mutex::new(false)),
            disposed: Arc::new(Mutex::new(false)),
        }
    }
}

impl Service for StubService {
    fn service_type(&self) -> &str { &self.type_name }
    fn initialize(&mut self, _engine: &EngineRef) -> Result<(), ServiceError> {
        *self.initialized.lock().unwrap() = true;
        Ok(())
    }
    fn start(&mut self, _engine: &EngineRef) -> Result<(), ServiceError> {
        *self.started.lock().unwrap() = true;
        Ok(())
    }
    fn stop(&mut self) -> Result<(), ServiceError> {
        *self.stopped.lock().unwrap() = true;
        Ok(())
    }
    fn dispose(&mut self) -> Result<(), ServiceError> {
        *self.disposed.lock().unwrap() = true;
        Ok(())
    }
}

fn make_def(id: &str) -> ServiceDefinition {
    ServiceDefinition::new(id, id, id)
}

#[test]
fn service_id_is_unique() {
    let a = ServiceId::new();
    let b = ServiceId::new();
    assert_ne!(a, b);
}

#[test]
fn service_state_queries() {
    assert!(ServiceState::Registered.can_initialize());
    assert!(!ServiceState::Running.can_initialize());
    assert!(ServiceState::Initialized.can_start());
    assert!(ServiceState::Stopped.can_start());
    assert!(ServiceState::Running.can_stop());
    assert!(ServiceState::Stopped.can_dispose());
    assert!(ServiceState::Disposed.is_terminal());
    assert!(ServiceState::Running.is_active());
}

#[test]
fn new_manager_is_empty() {
    let mgr = ServiceManager::new();
    assert_eq!(mgr.count(), 0);
}

#[test]
fn register_service() {
    let mut mgr = ServiceManager::new();
    let id = mgr.register(make_def("autosave"), Box::new(StubService::new("autosave"))).unwrap();
    assert_eq!(mgr.count(), 1);
    assert!(mgr.contains(id));
    assert!(mgr.contains_type("autosave"));
    assert_eq!(mgr.state(id), Some(ServiceState::Registered));
}

#[test]
fn register_duplicate_fails() {
    let mut mgr = ServiceManager::new();
    mgr.register(make_def("autosave"), Box::new(StubService::new("autosave"))).unwrap();
    let result = mgr.register(make_def("autosave"), Box::new(StubService::new("autosave")));
    assert!(matches!(result, Err(ServiceError::AlreadyRegistered { .. })));
}

#[test]
fn full_lifecycle_single_service() {
    let ctx = TestContext::new();
    let mut mgr = ServiceManager::new();
    let id = mgr.register(make_def("autosave"), Box::new(StubService::new("autosave"))).unwrap();
    let engine_ref = ctx.engine_ref();

    mgr.initialize(id, &engine_ref).unwrap();
    assert_eq!(mgr.state(id), Some(ServiceState::Initialized));

    mgr.start(id, &engine_ref).unwrap();
    assert_eq!(mgr.state(id), Some(ServiceState::Running));

    mgr.stop(id).unwrap();
    assert_eq!(mgr.state(id), Some(ServiceState::Stopped));

    mgr.dispose(id).unwrap();
    assert_eq!(mgr.state(id), Some(ServiceState::Disposed));
}

#[test]
fn invalid_transitions_rejected() {
    let ctx = TestContext::new();
    let mut mgr = ServiceManager::new();
    let id = mgr.register(make_def("autosave"), Box::new(StubService::new("autosave"))).unwrap();
    let engine_ref = ctx.engine_ref();

    // Can't start before initialize
    let result = mgr.start(id, &engine_ref);
    assert!(matches!(result, Err(ServiceError::InvalidState { .. })));

    mgr.initialize(id, &engine_ref).unwrap();

    // Can't initialize twice
    let result = mgr.initialize(id, &engine_ref);
    assert!(matches!(result, Err(ServiceError::InvalidState { .. })));

    mgr.start(id, &engine_ref).unwrap();

    // Can't initialize while running
    let result = mgr.initialize(id, &engine_ref);
    assert!(matches!(result, Err(ServiceError::InvalidState { .. })));

    // Can't dispose while running (must stop first)
    let result = mgr.dispose(id);
    assert!(matches!(result, Err(ServiceError::InvalidState { .. })));
}

#[test]
fn bulk_lifecycle() {
    let ctx = TestContext::new();
    let mut mgr = ServiceManager::new();
    let _a = mgr.register(make_def("a"), Box::new(StubService::new("a"))).unwrap();
    let _b = mgr.register(make_def("b"), Box::new(StubService::new("b"))).unwrap();
    let _c = mgr.register(make_def("c"), Box::new(StubService::new("c"))).unwrap();
    let engine_ref = ctx.engine_ref();

    mgr.initialize_all(&engine_ref).unwrap();
    for id in mgr.list_ids() {
        assert_eq!(mgr.state(id), Some(ServiceState::Initialized));
    }

    mgr.start_all(&engine_ref).unwrap();
    for id in mgr.list_ids() {
        assert_eq!(mgr.state(id), Some(ServiceState::Running));
    }

    mgr.stop_all().unwrap();
    for id in mgr.list_ids() {
        assert_eq!(mgr.state(id), Some(ServiceState::Stopped));
    }

    mgr.dispose_all().unwrap();
    for id in mgr.list_ids() {
        assert_eq!(mgr.state(id), Some(ServiceState::Disposed));
    }
}

#[test]
fn unregister_removes_service() {
    let mut mgr = ServiceManager::new();
    let id = mgr.register(make_def("autosave"), Box::new(StubService::new("autosave"))).unwrap();
    assert!(mgr.contains(id));
    let removed = mgr.unregister(id);
    assert!(removed.is_some());
    assert!(!mgr.contains(id));
    assert_eq!(mgr.count(), 0);
}

#[test]
fn get_service_by_id() {
    let mut mgr = ServiceManager::new();
    let id = mgr.register(make_def("autosave"), Box::new(StubService::new("autosave"))).unwrap();
    let svc = mgr.get(id).unwrap();
    assert_eq!(svc.service_type(), "autosave");
}

#[test]
fn get_definition_by_type() {
    let mut mgr = ServiceManager::new();
    let id = mgr.register(make_def("autosave"), Box::new(StubService::new("autosave"))).unwrap();
    let def = mgr.get_definition_by_type("autosave").unwrap();
    assert_eq!(def.name(), "autosave");
    let resolved_id = mgr.get_id_by_type("autosave").unwrap();
    assert_eq!(resolved_id, id);
}

#[test]
fn list_returns_all_definitions() {
    let mut mgr = ServiceManager::new();
    mgr.register(make_def("a"), Box::new(StubService::new("a"))).unwrap();
    mgr.register(make_def("b"), Box::new(StubService::new("b"))).unwrap();
    mgr.register(make_def("c"), Box::new(StubService::new("c"))).unwrap();
    let defs = mgr.list();
    assert_eq!(defs.len(), 3);
}

#[test]
fn not_found_returns_error() {
    let ctx = TestContext::new();
    let mut mgr = ServiceManager::new();
    let bogus = ServiceId::new();
    let engine_ref = ctx.engine_ref();
    let result = mgr.start(bogus, &engine_ref);
    assert!(matches!(result, Err(ServiceError::NotFound { .. })));
}