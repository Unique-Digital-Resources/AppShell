use app_shell::app_engine::{
    EngineRef,
    EventBus, SignalBus, ResourceManager, ConfigStore, ConfigSchema, Preferences,
    StateManager, CommandExecutor, Scheduler, HistoryStore,
    TaskContext, TaskError, TaskHandler, TaskInput, TaskManager, TaskResult,
    TaskState, ExecutionId, TaskDefinition, LifecycleState,
    ResourceError, ResourceHandle, ResourceId, ResourceLoader, ResourceState,
};
use std::any::Any;

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

struct StringLoader;
impl ResourceLoader for StringLoader {
    fn resource_type(&self) -> &str {
        "text"
    }
    fn load(&self, source: &str) -> Result<Box<dyn Any + Send>, ResourceError> {
        Ok(Box::new(format!("content_of:{}", source)))
    }
}

struct FailingLoader;
impl ResourceLoader for FailingLoader {
    fn resource_type(&self) -> &str {
        "broken"
    }
    fn load(&self, source: &str) -> Result<Box<dyn Any + Send>, ResourceError> {
        Err(ResourceError::LoadFailed {
            source: source.to_string(),
            reason: "intentional failure".to_string(),
        })
    }
}

fn setup_mgr() -> ResourceManager {
    let mgr = ResourceManager::new();
    mgr.register_loader(Box::new(StringLoader));
    mgr.register_loader(Box::new(FailingLoader));
    mgr
}

#[test]
fn new_manager_is_empty() {
    let mgr = ResourceManager::new();
    assert_eq!(mgr.resource_count(), 0);
    assert_eq!(mgr.loader_count(), 0);
}

#[test]
fn register_loader() {
    let mgr = ResourceManager::new();
    mgr.register_loader(Box::new(StringLoader));
    assert_eq!(mgr.loader_count(), 1);
    assert!(mgr.has_loader("text"));
    assert!(!mgr.has_loader("image"));
}

#[test]
fn load_returns_handle() {
    let mgr = setup_mgr();
    let handle = mgr.load("text", "file:///hello.txt").unwrap();
    let id = handle.resource_id();
    assert_eq!(mgr.resource_count(), 1);
    assert!(mgr.exists(id));
}

#[test]
fn loaded_resource_has_data() {
    let mgr = setup_mgr();
    let handle = mgr.load("text", "file:///hello.txt").unwrap();
    let id = handle.resource_id();

    let (state, has_data, data) = mgr.with_resource(id, |r| {
        (r.state(), r.has_data(), r.data::<String>().map(|s| s.clone()))
    }).unwrap();

    assert_eq!(state, ResourceState::Loaded);
    assert!(has_data);
    assert_eq!(data, Some("content_of:file:///hello.txt".to_string()));
}

#[test]
fn load_missing_loader_returns_error() {
    let mgr = setup_mgr();
    let result = mgr.load("unknown", "file:///x");
    assert!(matches!(result, Err(ResourceError::LoaderNotFound { .. })));
}

#[test]
fn load_failing_loader_returns_error() {
    let mgr = setup_mgr();
    let result = mgr.load("broken", "file:///broken.txt");
    assert!(matches!(result, Err(ResourceError::LoadFailed { .. })));
}

#[test]
fn load_same_source_returns_cached() {
    let mgr = ResourceManager::new();
    mgr.register_loader(Box::new(StringLoader));

    let h1 = mgr.load("text", "file:///same.txt").unwrap();
    let h2 = mgr.load("text", "file:///same.txt").unwrap();

    assert_eq!(h1.resource_id(), h2.resource_id());
    assert_eq!(mgr.resource_count(), 1);
    assert_eq!(mgr.active_count(h1.resource_id()), 2);
}

#[test]
fn load_different_sources_creates_different_resources() {
    let mgr = setup_mgr();
    let h1 = mgr.load("text", "file:///a.txt").unwrap();
    let h2 = mgr.load("text", "file:///b.txt").unwrap();
    assert_ne!(h1.resource_id(), h2.resource_id());
    assert_eq!(mgr.resource_count(), 2);
}

#[test]
fn release_decrements_ref_count() {
    let mgr = setup_mgr();
    let handle = mgr.load("text", "file:///x.txt").unwrap();
    let id = handle.resource_id();
    assert_eq!(mgr.active_count(id), 1);

    mgr.release(handle).unwrap();
    assert_eq!(mgr.active_count(id), 0);
    assert!(mgr.exists(id));
}

#[test]
fn acquire_increments_ref_count() {
    let mgr = setup_mgr();
    let handle = mgr.load("text", "file:///x.txt").unwrap();
    let id = handle.resource_id();
    assert_eq!(mgr.active_count(id), 1);

    mgr.acquire(&handle).unwrap();
    assert_eq!(mgr.active_count(id), 2);

    mgr.release(handle).unwrap();
    assert_eq!(mgr.active_count(id), 1);
}

#[test]
fn release_invalid_handle_returns_error() {
    let mgr = setup_mgr();
    let bogus = ResourceHandle::new(ResourceId::new());
    let result = mgr.release(bogus);
    assert!(matches!(result, Err(ResourceError::InvalidHandle { .. })));
}

#[test]
fn unload_removes_from_cache() {
    let mgr = setup_mgr();
    let handle = mgr.load("text", "file:///x.txt").unwrap();
    let id = handle.resource_id();
    mgr.release(handle).unwrap();

    mgr.unload(id).unwrap();
    assert!(!mgr.exists(id));
    assert_eq!(mgr.resource_count(), 0);
}

#[test]
fn unload_in_use_fails() {
    let mgr = setup_mgr();
    let handle = mgr.load("text", "file:///x.txt").unwrap();
    let id = handle.resource_id();
    let result = mgr.unload(id);
    assert!(matches!(result, Err(ResourceError::InUse { .. })));
    assert!(mgr.exists(id));
}

#[test]
fn reload_replaces_data() {
    let mgr = setup_mgr();
    let handle = mgr.load("text", "file:///x.txt").unwrap();
    let id = handle.resource_id();

    let has_data = mgr.with_resource(id, |r| r.has_data()).unwrap();
    assert!(has_data);

    mgr.reload(id).unwrap();

    let (state, has_data) = mgr.with_resource(id, |r| {
        (r.state(), r.has_data())
    }).unwrap();
    assert_eq!(state, ResourceState::Loaded);
    assert!(has_data);
}

#[test]
fn reload_missing_resource_fails() {
    let mgr = setup_mgr();
    let bogus = ResourceId::new();
    let result = mgr.reload(bogus);
    assert!(matches!(result, Err(ResourceError::NotFound { .. })));
}

#[test]
fn multiple_resources_managed() {
    let mgr = setup_mgr();
    let h1 = mgr.load("text", "file:///a.txt").unwrap();
    let h1_id = h1.resource_id();
    let h2 = mgr.load("text", "file:///b.txt").unwrap();
    let h3 = mgr.load("text", "file:///c.txt").unwrap();

    assert_eq!(mgr.resource_count(), 3);

    mgr.release(h1).unwrap();
    mgr.unload(h1_id).unwrap();
    assert_eq!(mgr.resource_count(), 2);

    assert!(mgr.is_in_use(h2.resource_id()));
    assert!(mgr.is_in_use(h3.resource_id()));
}

#[test]
fn reload_after_unload_works() {
    let mgr = setup_mgr();
    let handle = mgr.load("text", "file:///x.txt").unwrap();
    let id = handle.resource_id();

    mgr.release(handle).unwrap();
    mgr.unload(id).unwrap();
    assert!(!mgr.exists(id));

    let handle2 = mgr.load("text", "file:///x.txt").unwrap();
    assert!(mgr.exists(handle2.resource_id()));
    assert_eq!(mgr.active_count(handle2.resource_id()), 1);
}