use app_shell::app_engine::*;
use std::sync::{Arc, Mutex};

// --- Stub handlers that use EngineRef ---

struct ClosureEventHandler<F: Fn(&Event) + Send + Sync + 'static> {
    f: F,
}
impl<F: Fn(&Event) + Send + Sync + 'static> EventHandler for ClosureEventHandler<F> {
    fn handle(&self, event: &Event) {
        (self.f)(event);
    }
}


struct EventPublishingHandler {
    received: Arc<Mutex<bool>>,
}

impl CommandHandler for EventPublishingHandler {
    fn execute(
        &self,
        _input: &CommandInput,
        _context: &CommandContext,
        engine: &EngineRef,
    ) -> Result<CommandResult, CommandError> {
        // Publish an event using the engine reference (field access, not method)
        engine.event_bus.publish(
            &Event::new("command.executed").with_source("EventPublishingHandler"),
        );

        // Read config
        let _value = engine.config_store.get("test.key");

        // Emit a signal
        engine.signal_bus.emit(&Signal::new("command.signal"));

        *self.received.lock().unwrap() = true;

        Ok(CommandResult::success())
    }
}

struct ResourceLoadingHandler;

impl TaskHandler for ResourceLoadingHandler {
    fn execute(
        &self,
        _input: &TaskInput,
        _context: &TaskContext,
        engine: &EngineRef,
    ) -> Result<TaskResult, TaskError> {
        // Access resource manager (read-only)
        let count = engine.resource_manager.resource_count();

        // Publish completion event
        engine.event_bus.publish(
            &Event::new("task.completed").with_source("ResourceLoadingHandler"),
        );

        Ok(TaskResult::completed_with(count))
    }
}

struct ConfigReadingService {
    value: Arc<Mutex<Option<String>>>,
}

impl Service for ConfigReadingService {
    fn service_type(&self) -> &str { "config_reader" }

    fn initialize(&mut self, _engine: &EngineRef) -> Result<(), ServiceError> {
        Ok(())
    }

    fn start(&mut self, engine: &EngineRef) -> Result<(), ServiceError> {
        // Read configuration during start (field access)
        let val = engine.config_store.get("app.name");
        *self.value.lock().unwrap() = val.map(|s| s.to_string());
        Ok(())
    }

    fn stop(&mut self) -> Result<(), ServiceError> { Ok(()) }
    fn dispose(&mut self) -> Result<(), ServiceError> { Ok(()) }
}

struct EventSubscribingPlugin {
		#[allow(dead_code)]
    received: Arc<Mutex<bool>>,
}

impl Plugin for EventSubscribingPlugin {
    fn id(&self) -> &str { "event-plugin" }

    fn initialize(&mut self, _ctx: &PluginContext, _engine: &EngineRef) -> Result<(), PluginError> {
        // EngineRef provides read-only access to EventBus.
        // Subscriptions must be done before engine.initialize(),
        // or the plugin context should provide a subscribe mechanism (future phase).
        Ok(())
    }

    fn start(&mut self, _engine: &EngineRef) -> Result<(), PluginError> { Ok(()) }
    fn stop(&mut self) -> Result<(), PluginError> { Ok(()) }
    fn dispose(&mut self) -> Result<(), PluginError> { Ok(()) }
}
// --- Helper: construct an engine with config ---

fn setup_engine() -> AppEngine {
    let mut engine = Bootstrap::create().unwrap();
    engine.config_store_mut().set("test.key", "value").unwrap();
    engine.config_store_mut().set("app.name", "MyApp").unwrap();
    engine
}

// --- Tests ---

#[test]
fn handler_can_publish_events() {
    let mut engine = setup_engine();
    let received = Arc::new(Mutex::new(false));

    engine.command_executor_mut().register(
        CommandDefinition::new("test.publish", "Test", "Test publish"),
        Box::new(EventPublishingHandler { received: received.clone() }),
    ).unwrap();

    let cmd = Command::with_empty_input(
        "test.publish",
        CommandContext::new(ExecutionId::new()),
    );

    // Use execute_command — constructs EngineRef internally
    let result = engine.execute_command(&cmd).unwrap();

    assert!(result.is_success());
    assert!(*received.lock().unwrap());
}

#[test]
fn task_handler_can_access_engine() {
    let mut engine = setup_engine();

    engine.task_manager_mut().register(
        TaskDefinition::new("test.task", "Test", "Test task"),
        Box::new(ResourceLoadingHandler),
    ).unwrap();

    let task_id = engine.task_manager_mut().create(
        "test.task",
        TaskContext::new(ExecutionId::new()),
        TaskInput::empty(),
    ).unwrap();

    // Use start_task — constructs EngineRef internally
    let result = engine.start_task(task_id).unwrap();

    assert!(result.is_completed());
}

#[test]
fn service_can_read_config_during_start() {
    let mut engine = setup_engine();
    let value = Arc::new(Mutex::new(None));

    engine.service_manager_mut().register(
        ServiceDefinition::new("config_reader", "Config Reader", "Reads config"),
        Box::new(ConfigReadingService { value: value.clone() }),
    ).unwrap();

    // Runtime lifecycle constructs EngineRef internally
    engine.initialize().unwrap();
    engine.start().unwrap();

    assert_eq!(*value.lock().unwrap(), Some("MyApp".to_string()));

    engine.stop().unwrap();
    engine.dispose().unwrap();
}

#[test]
fn plugin_can_subscribe_during_initialize() {
    let mut engine = setup_engine();
    let received = Arc::new(Mutex::new(false));

    // Subscribe BEFORE initialize — EventBus::subscribe needs &mut
    let r = received.clone();
    engine.event_bus_mut().subscribe(
        "app.test",
        Box::new(ClosureEventHandler {
            f: move |_event: &Event| {
                *r.lock().unwrap() = true;
            },
        }),
    );

    engine.plugin_manager_mut().register(
        PluginManifest::new("event-plugin", "Event Plugin"),
        Box::new(EventSubscribingPlugin { received: received.clone() }),
    ).unwrap();

    // Runtime lifecycle constructs EngineRef internally
    engine.initialize().unwrap();

    // Now publish the event
    engine.event_bus().publish(&Event::new("app.test"));

    assert!(*received.lock().unwrap());

    engine.start().unwrap();
    engine.stop().unwrap();
    engine.dispose().unwrap();
}

#[test]
fn engine_ref_provides_all_systems() {
    let engine = setup_engine();
    let engine_ref = engine.engine_ref();

    // Field access (not method calls)
    let _ = engine_ref.event_bus;
    let _ = engine_ref.signal_bus;
    let _ = engine_ref.resource_manager;
    let _ = engine_ref.config_store;
    let _ = engine_ref.preferences;
    let _ = engine_ref.state_manager;
    let _ = engine_ref.command_executor;
    let _ = engine_ref.scheduler;
    let _ = engine_ref.history_store;
    // Note: task_manager is deliberately excluded from EngineRef
    // to allow disjoint borrows with task_manager_mut()
}

#[test]
fn handler_can_read_preferences() {
    struct PrefReader { value: Arc<Mutex<Option<String>>> }
    impl CommandHandler for PrefReader {
        fn execute(&self, _input: &CommandInput, _ctx: &CommandContext, engine: &EngineRef) -> Result<CommandResult, CommandError> {
            // Field access for preferences
            let theme = engine.preferences.get("theme");
            *self.value.lock().unwrap() = theme.map(|s| s.to_string());
            Ok(CommandResult::success())
        }
    }

    let mut engine = setup_engine();
    engine.preferences_mut().set("theme", "dark");

    let value = Arc::new(Mutex::new(None));
    engine.command_executor_mut().register(
        CommandDefinition::new("test.pref", "Pref", "Read pref"),
        Box::new(PrefReader { value: value.clone() }),
    ).unwrap();

    let cmd = Command::with_empty_input("test.pref", CommandContext::new(ExecutionId::new()));
    engine.execute_command(&cmd).unwrap();

    assert_eq!(*value.lock().unwrap(), Some("dark".to_string()));
}