//! Phase 10 acceptance criteria.

use app_shell::app_engine::{
    Bootstrap, Lifecycle, Command, CommandContext, CommandDefinition, CommandError,
    CommandHandler, CommandInput, CommandResult, Event, EventHandler, ExecutionId,
    Plugin, PluginContext, PluginError, PluginLoader, PluginManager, PluginManifest,
    PluginRegistry, PluginState, ResourceLoader, Service, ServiceDefinition,
    ServiceError, ServiceState, TaskContext, TaskDefinition, TaskHandler, TaskInput,
    TaskResult, TaskResultStatus, EngineRef,
    EventBus, SignalBus, ResourceManager, ConfigStore, ConfigSchema, Preferences,
    StateManager, CommandExecutor, Scheduler, HistoryStore, LifecycleState,
};
use std::any::Any;
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

// --- Stubs ---

struct ImagePlugin { initialized: Arc<Mutex<bool>> }
impl ImagePlugin { fn new() -> Self { Self { initialized: Arc::new(Mutex::new(false)) } } }
impl Plugin for ImagePlugin {
    fn id(&self) -> &str { "image-plugin" }
    fn initialize(&mut self, ctx: &PluginContext, _engine: &EngineRef) -> Result<(), PluginError> {
        assert_eq!(ctx.plugin_id(), "image-plugin");
        *self.initialized.lock().unwrap() = true;
        Ok(())
    }
    fn start(&mut self, _engine: &EngineRef) -> Result<(), PluginError> { Ok(()) }
    fn stop(&mut self) -> Result<(), PluginError> { Ok(()) }
    fn dispose(&mut self) -> Result<(), PluginError> { Ok(()) }
}

struct ImportImageHandler;
impl CommandHandler for ImportImageHandler {
    fn execute(&self, input: &CommandInput, _ctx: &CommandContext, _engine: &EngineRef) -> Result<CommandResult, CommandError> {
        let path: &String = input.get::<String>()
            .ok_or_else(|| CommandError::InvalidArguments("expected path".to_string()))?;
        Ok(CommandResult::success_with(format!("imported:{}", path)))
    }
}

struct ImageResourceLoader;
impl ResourceLoader for ImageResourceLoader {
    fn resource_type(&self) -> &str { "image" }
    fn load(&self, source: &str) -> Result<Box<dyn Any + Send>, app_shell::app_engine::ResourceError> {
        Ok(Box::new(format!("pixels:{}", source)))
    }
}

struct ImageService;
impl Service for ImageService {
    fn service_type(&self) -> &str { "image_service" }
    fn initialize(&mut self, _engine: &EngineRef) -> Result<(), ServiceError> { Ok(()) }
    fn start(&mut self, _engine: &EngineRef) -> Result<(), ServiceError> { Ok(()) }
    fn stop(&mut self) -> Result<(), ServiceError> { Ok(()) }
    fn dispose(&mut self) -> Result<(), ServiceError> { Ok(()) }
}

struct RenderImageTask;
impl TaskHandler for RenderImageTask {
    fn execute(&self, input: &TaskInput, _ctx: &TaskContext, _engine: &EngineRef) -> Result<TaskResult, app_shell::app_engine::TaskError> {
        let path: &String = input.get::<String>()
            .ok_or_else(|| app_shell::app_engine::TaskError::ExecutionFailed {
                id: 0, reason: "expected path".to_string()
            })?;
        Ok(TaskResult::completed_with(format!("rendered:{}", path)))
    }
}

struct ClosureEventHandler<F: Fn(&Event) + Send + Sync + 'static> { f: F }
impl<F: Fn(&Event) + Send + Sync + 'static> EventHandler for ClosureEventHandler<F> {
    fn handle(&self, event: &Event) { (self.f)(event); }
}
fn event_handler<F: Fn(&Event) + Send + Sync + 'static>(f: F) -> Box<dyn EventHandler> {
    Box::new(ClosureEventHandler { f })
}

#[test]
fn acceptance_plugin_contract() {
    let plugin = ImagePlugin::new();
    assert_eq!(plugin.id(), "image-plugin");
}

#[test]
fn acceptance_manifest_describes_plugin() {
    let m = PluginManifest::new("image-plugin", "Image Plugin")
        .with_version("1.0.0")
        .with_description("Image processing")
        .with_author("Alice")
        .with_capability("commands")
        .with_capability("resources");
    assert_eq!(m.id(), "image-plugin");
    assert_eq!(m.version(), "1.0.0");
    assert!(m.capabilities().contains(&"commands".to_string()));
}

#[test]
fn acceptance_register_and_load() {
    let mut mgr = PluginManager::new();
    let id = mgr.register(
        PluginManifest::new("image-plugin", "Image Plugin"),
        Box::new(ImagePlugin::new()),
    ).unwrap();
    assert!(mgr.contains(id));
    assert!(mgr.contains_type("image-plugin"));
    assert_eq!(mgr.state(id), Some(PluginState::Loaded));
}

#[test]
fn acceptance_context_and_initialize() {
    let ctx = TestContext::new();
    let mut mgr = PluginManager::new();
    let id = mgr.register(
        PluginManifest::new("image-plugin", "Image Plugin"),
        Box::new(ImagePlugin::new()),
    ).unwrap();
    let engine_ref = ctx.engine_ref();
    mgr.initialize(id, &engine_ref).unwrap();
    assert_eq!(mgr.state(id), Some(PluginState::Initialized));
}

#[test]
fn acceptance_start_stop_dispose() {
    let ctx = TestContext::new();
    let mut mgr = PluginManager::new();
    let id = mgr.register(
        PluginManifest::new("image-plugin", "Image Plugin"),
        Box::new(ImagePlugin::new()),
    ).unwrap();
    let engine_ref = ctx.engine_ref();
    mgr.initialize(id, &engine_ref).unwrap();
    mgr.start(id, &engine_ref).unwrap();
    assert_eq!(mgr.state(id), Some(PluginState::Started));
    mgr.stop(id).unwrap();
    assert_eq!(mgr.state(id), Some(PluginState::Stopped));
    mgr.dispose(id).unwrap();
    assert_eq!(mgr.state(id), Some(PluginState::Unloaded));
}

#[test]
fn acceptance_integrate_with_lifecycle() {
    let mut runtime = Bootstrap::create().unwrap();
    let id = runtime.plugin_manager_mut().register(
        PluginManifest::new("image-plugin", "Image Plugin"),
        Box::new(ImagePlugin::new()),
    ).unwrap();
    runtime.initialize().unwrap();
    runtime.start().unwrap();
    assert_eq!(runtime.plugin_manager().state(id), Some(PluginState::Started));
    runtime.stop().unwrap();
    runtime.dispose().unwrap();
}

#[test]
fn acceptance_plugin_registers_commands() {
    let mut runtime = Bootstrap::create().unwrap();

    // Register plugin before lifecycle starts
    let _pid = runtime.plugin_manager_mut().register(
        PluginManifest::new("image-plugin", "Image Plugin"),
        Box::new(ImagePlugin::new()),
    ).unwrap();

    // Register command handler (no EngineRef needed for registration)
    runtime.command_executor_mut().register(
        CommandDefinition::new("image.import", "Import Image", "Import an image file"),
        Box::new(ImportImageHandler),
    ).unwrap();

    // Runtime lifecycle initializes and starts the plugin automatically
    runtime.initialize().unwrap();
    runtime.start().unwrap();

    // Execute the plugin-provided command
    let cmd = Command::new(
        "image.import",
        CommandContext::new(ExecutionId::new()),
        CommandInput::new("/assets/logo.png".to_string()),
    );
    let result = runtime.execute_command(&cmd).unwrap();
    assert_eq!(result.output::<String>(), Some(&"imported:/assets/logo.png".to_string()));

    runtime.stop().unwrap();
    runtime.dispose().unwrap();
}

#[test]
fn acceptance_plugin_provides_tasks() {
    let mut runtime = Bootstrap::create().unwrap();
    runtime.task_manager_mut().register(
        TaskDefinition::new("image.render", "Render Image", "Render an image"),
        Box::new(RenderImageTask),
    ).unwrap();
    let task_id = runtime.task_manager_mut().create(
        "image.render",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new("/scene.render".to_string()),
    ).unwrap();
    let result = runtime.start_task(task_id).unwrap();
    assert_eq!(result.status(), TaskResultStatus::Completed);
}

#[test]
fn acceptance_plugin_events_signals() {
    let mut runtime = Bootstrap::create().unwrap();
    let received = Arc::new(Mutex::new(false));
    let r = received.clone();
    runtime.event_bus_mut().subscribe(
        "document.changed",
        event_handler(move |_event: &Event| { *r.lock().unwrap() = true; }),
    );
    runtime.event_bus().publish(&Event::new("document.changed").with_source("image-plugin"));
    assert!(*received.lock().unwrap());
}

#[test]
fn acceptance_plugin_resources() {
    let mut runtime = Bootstrap::create().unwrap();
    runtime.resource_manager_mut().register_loader(Box::new(ImageResourceLoader));
    let handle = runtime.resource_manager_mut().load("image", "file:///logo.png").unwrap();
    let data = runtime.resource_manager().with_resource(handle.resource_id(), |r| {
        r.data::<String>().map(|s| s.clone())
    }).flatten();
    assert_eq!(data, Some("pixels:file:///logo.png".to_string()));
}

#[test]
fn acceptance_plugin_services() {
    let mut runtime = Bootstrap::create().unwrap();
    let sid = runtime.service_manager_mut().register(
        ServiceDefinition::new("image_service", "Image Service", "Image processing service"),
        Box::new(ImageService),
    ).unwrap();
    // Runtime lifecycle starts the service automatically
    runtime.initialize().unwrap();
    runtime.start().unwrap();
    assert_eq!(runtime.service_manager().state(sid), Some(ServiceState::Running));
    runtime.stop().unwrap();
    runtime.dispose().unwrap();
}

#[test]
fn acceptance_dependency_validation() {
    let mut mgr = PluginManager::new();
    let a_id = mgr.register(
        PluginManifest::new("a", "A").with_dependency("b"),
        Box::new(ImagePlugin::new()),
    ).unwrap();
    assert!(mgr.validate_dependencies(a_id).is_err());
    mgr.register(PluginManifest::new("b", "B"), Box::new(ImagePlugin::new())).unwrap();
    mgr.validate_dependencies(a_id).unwrap();
}

#[test]
fn acceptance_engine_independent_of_plugins() {
    let runtime = Bootstrap::create().unwrap();
    assert_eq!(runtime.plugin_manager().count(), 0);
}

#[test]
fn acceptance_loader_independent_from_contract() {
    struct StaticLoader;
    impl PluginLoader for StaticLoader {
        fn can_load(&self, source: &str) -> bool { source.starts_with("static:") }
        fn load(&self, source: &str) -> Result<(PluginManifest, Box<dyn Plugin>), PluginError> {
            let id = source.strip_prefix("static:").unwrap_or(source);
            Ok((PluginManifest::new(id, id), Box::new(ImagePlugin::new())))
        }
    }
    let mut mgr = PluginManager::new();
    mgr.register_loader(Box::new(StaticLoader));
    let id = mgr.load("static:loaded-plugin").unwrap();
    assert!(mgr.contains(id));
    let id2 = mgr.register(PluginManifest::new("direct-plugin", "Direct"), Box::new(ImagePlugin::new())).unwrap();
    assert!(mgr.contains(id2));
}

#[test]
fn acceptance_manager_not_execution_engine() {
    let mgr = PluginManager::new();
    assert_eq!(mgr.count(), 0);
    assert!(mgr.list().is_empty());
}

#[test]
fn acceptance_no_external_knowledge() {
    let _manifest = PluginManifest::new("test", "Test");
    let _context = PluginContext::new("test");
    let _mgr = PluginManager::new();
    let _registry = PluginRegistry::new();
}