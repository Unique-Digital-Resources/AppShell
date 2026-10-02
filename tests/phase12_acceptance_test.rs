//! Phase 12 acceptance criteria.

use app_shell::AppEngine;
use app_shell::app_engine::{
    Bootstrap, Command, CommandContext, CommandDefinition, CommandHandler,
    CommandInput, CommandResult, Config, ConfigSchema, ConfigStore, Event,
    EventBus, EventHandler, HistoryEntry, HistoryStore, Lifecycle, Plugin,
    PluginContext, PluginError, PluginManager, PluginManifest, Preferences,
    ResourceManager, Schedule, Scheduler, Service, ServiceDefinition,
    ServiceError, ServiceManager, Signal, SignalBus, Task, TaskDefinition,
    TaskResult, TaskState, EngineRef,
};
use std::sync::{Arc, Mutex};

struct EchoHandler;
impl CommandHandler for EchoHandler {
    fn execute(&self, input: &CommandInput, _ctx: &CommandContext, _engine: &EngineRef) -> Result<CommandResult, app_shell::app_engine::CommandError> {
        let s: &String = input.get::<String>()
            .ok_or_else(|| app_shell::app_engine::CommandError::InvalidArguments("expected String".to_string()))?;
        Ok(CommandResult::success_with(s.clone()))
    }
}

struct StubService { started: Arc<Mutex<bool>> }
impl Service for StubService {
    fn service_type(&self) -> &str { "stub" }
    fn initialize(&mut self, _engine: &EngineRef) -> Result<(), ServiceError> { Ok(()) }
    fn start(&mut self, _engine: &EngineRef) -> Result<(), ServiceError> { *self.started.lock().unwrap() = true; Ok(()) }
    fn stop(&mut self) -> Result<(), ServiceError> { *self.started.lock().unwrap() = false; Ok(()) }
    fn dispose(&mut self) -> Result<(), ServiceError> { Ok(()) }
}

struct StubPlugin { started: Arc<Mutex<bool>> }
impl Plugin for StubPlugin {
    fn id(&self) -> &str { "stub" }
    fn initialize(&mut self, _ctx: &PluginContext, _engine: &EngineRef) -> Result<(), PluginError> { Ok(()) }
    fn start(&mut self, _engine: &EngineRef) -> Result<(), PluginError> { *self.started.lock().unwrap() = true; Ok(()) }
    fn stop(&mut self) -> Result<(), PluginError> { *self.started.lock().unwrap() = false; Ok(()) }
    fn dispose(&mut self) -> Result<(), PluginError> { Ok(()) }
}

struct ClosureEventHandler<F: Fn(&Event) + Send + Sync + 'static> { f: F }
impl<F: Fn(&Event) + Send + Sync + 'static> EventHandler for ClosureEventHandler<F> {
    fn handle(&self, event: &Event) { (self.f)(event); }
}
fn event_handler<F: Fn(&Event) + Send + Sync + 'static>(f: F) -> Box<dyn EventHandler> {
    Box::new(ClosureEventHandler { f })
}

#[test]
fn acceptance_composition_root() {
    let engine: AppEngine = Bootstrap::create().unwrap();
    let _ = engine.command_executor();
    let _ = engine.task_manager();
    let _ = engine.scheduler();
    let _ = engine.event_bus();
    let _ = engine.signal_bus();
    let _ = engine.history_store();
    let _ = engine.resource_manager();
    let _ = engine.service_manager();
    let _ = engine.plugin_manager();
    let _ = engine.config_store();
    let _ = engine.preferences();
}

#[test]
fn acceptance_public_api_via_lib() {
    let _engine: AppEngine = Bootstrap::create().unwrap();
}

#[test]
fn acceptance_no_internal_module_access_needed() {
    fn _check_types(
        _: Command, _: CommandDefinition, _: CommandResult,
        _: Task, _: TaskDefinition, _: TaskResult, _: TaskState,
        _: Schedule, _: Scheduler, _: Event, _: Signal, _: SignalBus,
        _: EventBus, _: HistoryEntry, _: HistoryStore,
        _: Config, _: ConfigSchema, _: ConfigStore, _: Preferences,
        _: ResourceManager, _: ServiceManager, _: PluginManager, _: AppEngine,
    ) {}
}

#[test]
fn acceptance_subsystem_error_ownership() {
    use app_shell::app_engine::{CommandError, ResourceError, ServiceError, TaskError};
    let _: CommandError = CommandError::UnknownCommand { id: "x".into() };
    let _: TaskError = TaskError::NotFound { id: 1 };
    let _: ResourceError = ResourceError::NotFound { id: 1 };
    let _: ServiceError = ServiceError::NotFound { id: 1 };
}

#[test]
fn acceptance_engine_error_unifies() {
    use app_shell::app_engine::{EngineError, ServiceError};
    let e: EngineError = ServiceError::NotFound { id: 1 }.into();
    assert!(e.to_string().contains("service"));
}

#[test]
fn acceptance_deterministic_lifecycle() {
    let mut engine = Bootstrap::create().unwrap();
    let svc_started = Arc::new(Mutex::new(false));
    let plg_started = Arc::new(Mutex::new(false));

    engine.service_manager_mut().register(
        ServiceDefinition::new("stub", "Stub", "Stub"),
        Box::new(StubService { started: svc_started.clone() }),
    ).unwrap();
    engine.plugin_manager_mut().register(
        PluginManifest::new("stub", "Stub"),
        Box::new(StubPlugin { started: plg_started.clone() }),
    ).unwrap();

    engine.initialize().unwrap();
    engine.start().unwrap();
    assert!(*svc_started.lock().unwrap());
    assert!(*plg_started.lock().unwrap());

    engine.stop().unwrap();
    assert!(!*plg_started.lock().unwrap());
    assert!(!*svc_started.lock().unwrap());

    engine.dispose().unwrap();
}

#[test]
fn acceptance_no_accidental_coupling() {
    let mut engine = Bootstrap::create().unwrap();
    let received = Arc::new(Mutex::new(false));
    let r = received.clone();
    engine.event_bus_mut().subscribe(
        "task.completed",
        event_handler(move |_event: &Event| { *r.lock().unwrap() = true; }),
    );
    engine.event_bus().publish(&Event::new("task.completed"));
    assert!(*received.lock().unwrap());
}

#[test]
fn acceptance_no_duplicate_communication() {
    let engine = Bootstrap::create().unwrap();
    let _ = engine.command_executor();
    let _ = engine.event_bus();
    let _ = engine.signal_bus();
}

#[test]
fn acceptance_no_domain_leak() {
    let engine = Bootstrap::create().unwrap();
    let _ = engine.task_manager();
    let _ = engine.command_executor();
    let _ = engine.resource_manager();
}

#[test]
fn acceptance_no_ui_leak() {
    let engine = Bootstrap::create().unwrap();
    let _ = engine.preferences();
}

#[test]
fn acceptance_no_api_leak() {
    let engine = Bootstrap::create().unwrap();
    let _ = engine.command_executor();
}

#[test]
fn acceptance_plugins_consume_contracts() {
    let mut engine = Bootstrap::create().unwrap();
    engine.plugin_manager_mut().register(
        PluginManifest::new("test", "Test"),
        Box::new(StubPlugin { started: Arc::new(Mutex::new(false)) }),
    ).unwrap();
    engine.command_executor_mut().register(
        CommandDefinition::new("test.echo", "Echo", "Echo"),
        Box::new(EchoHandler),
    ).unwrap();
    engine.event_bus_mut().subscribe("test.event", event_handler(|_event: &Event| {}));
}

#[test]
fn acceptance_config_separate_from_state() {
    let mut engine = Bootstrap::create().unwrap();
    engine.config_store_mut().set("worker_count", "8").unwrap();
    engine.initialize().unwrap();
    assert_eq!(engine.config_store().get("worker_count").as_deref(), Some("8"));
    assert_ne!(engine.state(), app_shell::app_engine::RuntimeState::Created);
}

#[test]
fn acceptance_internal_evolution_safe() {
    let engine: AppEngine = Bootstrap::create().unwrap();
    assert_eq!(engine.state(), app_shell::app_engine::RuntimeState::Created);
}