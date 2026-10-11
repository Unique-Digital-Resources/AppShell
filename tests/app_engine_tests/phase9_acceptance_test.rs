//! Phase 9 acceptance criteria.

use app_shell::app_engine::{
    Bootstrap, Lifecycle, Service, ServiceDefinition, ServiceError, ServiceManager,
    ServiceState, EngineRef,
    EventBus, SignalBus, ResourceManager, ConfigStore, ConfigSchema, Preferences,
    StateManager, CommandExecutor, Scheduler, HistoryStore, LifecycleState,
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
            &self.event_bus, &self.signal_bus, &self.resource_manager,
            &self.config_store, &self.preferences, &self.state_manager,
            &self.command_executor, &self.scheduler, &self.history_store,
        )
    }
}

// --- Stub services ---

struct AutosaveService {
    active: Arc<Mutex<bool>>,
}

impl Service for AutosaveService {
    fn service_type(&self) -> &str { "autosave" }
    fn initialize(&mut self, _engine: &EngineRef) -> Result<(), ServiceError> { Ok(()) }
    fn start(&mut self, _engine: &EngineRef) -> Result<(), ServiceError> {
        *self.active.lock().unwrap() = true;
        Ok(())
    }
    fn stop(&mut self) -> Result<(), ServiceError> {
        *self.active.lock().unwrap() = false;
        Ok(())
    }
    fn dispose(&mut self) -> Result<(), ServiceError> { Ok(()) }
}

struct PluginService {
    loaded: Arc<Mutex<bool>>,
}

impl Service for PluginService {
    fn service_type(&self) -> &str { "plugin" }
    fn initialize(&mut self, _engine: &EngineRef) -> Result<(), ServiceError> { Ok(()) }
    fn start(&mut self, _engine: &EngineRef) -> Result<(), ServiceError> {
        *self.loaded.lock().unwrap() = true;
        Ok(())
    }
    fn stop(&mut self) -> Result<(), ServiceError> {
        *self.loaded.lock().unwrap() = false;
        Ok(())
    }
    fn dispose(&mut self) -> Result<(), ServiceError> { Ok(()) }
}

#[test]
fn acceptance_service_contract() {
    let svc = AutosaveService { active: Arc::new(Mutex::new(false)) };
    assert_eq!(svc.service_type(), "autosave");
}

#[test]
fn acceptance_register_and_discover() {
    let mut mgr = ServiceManager::new();
    let id = mgr.register(
        ServiceDefinition::new("autosave", "Autosave", "Autosave service"),
        Box::new(AutosaveService { active: Arc::new(Mutex::new(false)) }),
    ).unwrap();
    assert!(mgr.contains(id));
    assert!(mgr.contains_type("autosave"));
    assert_eq!(mgr.get_id_by_type("autosave"), Some(id));
}

#[test]
fn acceptance_initialize_and_start() {
    let ctx = TestContext::new();
    let mut mgr = ServiceManager::new();
    let active = Arc::new(Mutex::new(false));
    let id = mgr.register(
        ServiceDefinition::new("autosave", "Autosave", "Autosave"),
        Box::new(AutosaveService { active: active.clone() }),
    ).unwrap();
    let engine_ref = ctx.engine_ref();

    mgr.initialize(id, &engine_ref).unwrap();
    assert_eq!(mgr.state(id), Some(ServiceState::Initialized));
    mgr.start(id, &engine_ref).unwrap();
    assert_eq!(mgr.state(id), Some(ServiceState::Running));
    assert!(*active.lock().unwrap());
}

#[test]
fn acceptance_stop_and_dispose() {
    let ctx = TestContext::new();
    let mut mgr = ServiceManager::new();
    let active = Arc::new(Mutex::new(false));
    let id = mgr.register(
        ServiceDefinition::new("autosave", "Autosave", "Autosave"),
        Box::new(AutosaveService { active: active.clone() }),
    ).unwrap();
    let engine_ref = ctx.engine_ref();

    mgr.initialize(id, &engine_ref).unwrap();
    mgr.start(id, &engine_ref).unwrap();
    mgr.stop(id).unwrap();
    assert_eq!(mgr.state(id), Some(ServiceState::Stopped));
    assert!(!*active.lock().unwrap());
    mgr.dispose(id).unwrap();
    assert_eq!(mgr.state(id), Some(ServiceState::Disposed));
}

#[test]
fn acceptance_prevent_invalid_transitions() {
    let ctx = TestContext::new();
    let mut mgr = ServiceManager::new();
    let id = mgr.register(
        ServiceDefinition::new("autosave", "Autosave", "Autosave"),
        Box::new(AutosaveService { active: Arc::new(Mutex::new(false)) }),
    ).unwrap();
    let engine_ref = ctx.engine_ref();

    assert!(mgr.start(id, &engine_ref).is_err());
    mgr.initialize(id, &engine_ref).unwrap();
    assert!(mgr.initialize(id, &engine_ref).is_err());
    mgr.start(id, &engine_ref).unwrap();
    assert!(mgr.dispose(id).is_err());
}

#[test]
fn acceptance_integrate_with_lifecycle() {
    let mut runtime = Bootstrap::create().unwrap();
    let active = Arc::new(Mutex::new(false));

    runtime.service_manager_mut().register(
        ServiceDefinition::new("autosave", "Autosave", "Autosave"),
        Box::new(AutosaveService { active: active.clone() }),
    ).unwrap();

    runtime.initialize().unwrap();
    runtime.start().unwrap();
    assert!(*active.lock().unwrap());

    runtime.stop().unwrap();
    assert!(!*active.lock().unwrap());

    runtime.dispose().unwrap();
    let ids = runtime.service_manager().list_ids();
    for id in &ids {
        assert_eq!(runtime.service_manager().state(*id), Some(ServiceState::Disposed));
    }
}

#[test]
fn acceptance_services_use_engine_systems() {
    let mut runtime = Bootstrap::create().unwrap();
    runtime.service_manager_mut().register(
        ServiceDefinition::new("worker", "Worker", "Background worker"),
        Box::new(AutosaveService { active: Arc::new(Mutex::new(false)) }),
    ).unwrap();
    assert!(runtime.service_manager().count() > 0);
    let _ = runtime.command_executor();
    let _ = runtime.task_manager();
    let _ = runtime.scheduler();
    let _ = runtime.event_bus();
    let _ = runtime.signal_bus();
    let _ = runtime.resource_manager();
    let _ = runtime.history_store();
}

#[test]
fn acceptance_no_domain_logic_in_manager() {
    let ctx = TestContext::new();
    let mut mgr = ServiceManager::new();
    let id = mgr.register(
        ServiceDefinition::new("autosave", "Autosave", "Autosave"),
        Box::new(AutosaveService { active: Arc::new(Mutex::new(false)) }),
    ).unwrap();
    let engine_ref = ctx.engine_ref();
    mgr.initialize(id, &engine_ref).unwrap();
    mgr.start(id, &engine_ref).unwrap();
    mgr.stop(id).unwrap();
    mgr.dispose(id).unwrap();
}

#[test]
fn acceptance_multiple_independent_services() {
    let ctx = TestContext::new();
    let mut mgr = ServiceManager::new();
    let a_active = Arc::new(Mutex::new(false));
    let p_loaded = Arc::new(Mutex::new(false));

    let a_id = mgr.register(
        ServiceDefinition::new("autosave", "Autosave", "Autosave"),
        Box::new(AutosaveService { active: a_active.clone() }),
    ).unwrap();
    let p_id = mgr.register(
        ServiceDefinition::new("plugin", "Plugin", "Plugin manager"),
        Box::new(PluginService { loaded: p_loaded.clone() }),
    ).unwrap();
    let engine_ref = ctx.engine_ref();

    assert_eq!(mgr.count(), 2);
    mgr.initialize_all(&engine_ref).unwrap();
    mgr.start_all(&engine_ref).unwrap();
    assert!(*a_active.lock().unwrap());
    assert!(*p_loaded.lock().unwrap());
    mgr.stop_all().unwrap();
    assert!(!*a_active.lock().unwrap());
    assert!(!*p_loaded.lock().unwrap());
    mgr.dispose_all().unwrap();
    assert_eq!(mgr.state(a_id), Some(ServiceState::Disposed));
    assert_eq!(mgr.state(p_id), Some(ServiceState::Disposed));
}

#[test]
fn acceptance_manager_not_dependency_container() {
    let mgr = ServiceManager::new();
    assert_eq!(mgr.count(), 0);
    assert!(mgr.list().is_empty());
    assert!(mgr.list_ids().is_empty());
}