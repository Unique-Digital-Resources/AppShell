use app_shell::app_engine::*;
use std::sync::{Arc, Mutex};

// --- Panicking handlers ---

struct PanickingTaskHandler;
impl TaskHandler for PanickingTaskHandler {
    fn execute(&self, _input: &TaskInput, _ctx: &TaskContext, _engine: &EngineRef) -> Result<TaskResult, TaskError> {
        panic!("task handler crashed!");
    }
}

struct PanickingCommandHandler;
impl CommandHandler for PanickingCommandHandler {
    fn execute(&self, _input: &CommandInput, _ctx: &CommandContext, _engine: &EngineRef) -> Result<CommandResult, CommandError> {
        panic!("command handler crashed!");
    }
}

struct PanickingService;
impl Service for PanickingService {
    fn service_type(&self) -> &str { "panicking_service" }
    fn initialize(&mut self, _engine: &EngineRef) -> Result<(), ServiceError> {
        panic!("service init crashed!");
    }
    fn start(&mut self, _engine: &EngineRef) -> Result<(), ServiceError> {
        panic!("service start crashed!");
    }
    fn stop(&mut self) -> Result<(), ServiceError> { Ok(()) }
    fn dispose(&mut self) -> Result<(), ServiceError> { Ok(()) }
}

struct PanickingPlugin;
impl Plugin for PanickingPlugin {
    fn id(&self) -> &str { "panicking_plugin" }
    fn initialize(&mut self, _ctx: &PluginContext, _engine: &EngineRef) -> Result<(), PluginError> {
        panic!("plugin init crashed!");
    }
    fn start(&mut self, _engine: &EngineRef) -> Result<(), PluginError> {
        panic!("plugin start crashed!");
    }
    fn stop(&mut self) -> Result<(), PluginError> { Ok(()) }
    fn dispose(&mut self) -> Result<(), PluginError> { Ok(()) }
}

// --- Non-panicking handlers (for comparison) ---

struct OkTaskHandler;
impl TaskHandler for OkTaskHandler {
    fn execute(&self, _input: &TaskInput, _ctx: &TaskContext, _engine: &EngineRef) -> Result<TaskResult, TaskError> {
        Ok(TaskResult::completed())
    }
}

struct OkCommandHandler;
impl CommandHandler for OkCommandHandler {
    fn execute(&self, _input: &CommandInput, _ctx: &CommandContext, _engine: &EngineRef) -> Result<CommandResult, CommandError> {
        Ok(CommandResult::success())
    }
}

// --- Task handler panic isolation ---

#[test]
fn panicking_task_handler_returns_error_not_crash() {
    let engine = app_shell::app_engine::TestEngine::new();
    engine.task_manager.register(
        TaskDefinition::new("panic.task", "Panic", "Panics"),
        Box::new(PanickingTaskHandler),
    ).unwrap();

    let task_id = engine.task_manager.create(
        "panic.task",
        TaskContext::new(ExecutionId::new()),
        TaskInput::empty(),
    ).unwrap();

    let engine_ref = engine.engine_ref();
    let result = engine.task_manager.start(task_id, &engine_ref);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, TaskError::ExecutionFailed { .. }));
    assert!(err.to_string().contains("handler panicked"));
    assert!(err.to_string().contains("task handler crashed!"));

    // Task should be in Failed state
    assert_eq!(engine.task_manager.get_state(task_id), Some(TaskState::Failed));
}

#[test]
fn non_panicking_task_handler_still_works() {
    let engine = app_shell::app_engine::TestEngine::new();
    engine.task_manager.register(
        TaskDefinition::new("ok.task", "OK", "OK"),
        Box::new(OkTaskHandler),
    ).unwrap();

    let task_id = engine.task_manager.create(
        "ok.task",
        TaskContext::new(ExecutionId::new()),
        TaskInput::empty(),
    ).unwrap();

    let engine_ref = engine.engine_ref();
    let result = engine.task_manager.start(task_id, &engine_ref).unwrap();

    assert!(result.is_completed());
    assert_eq!(engine.task_manager.get_state(task_id), Some(TaskState::Completed));
}

// --- Command handler panic isolation ---

#[test]
fn panicking_command_handler_returns_error_not_crash() {
    let mut engine = app_shell::app_engine::TestEngine::new();
    engine.command_executor.register(
        CommandDefinition::new("panic.cmd", "Panic", "Panics"),
        Box::new(PanickingCommandHandler),
    ).unwrap();

    let cmd = Command::with_empty_input("panic.cmd", CommandContext::new(ExecutionId::new()));
    let engine_ref = engine.engine_ref();
    let result = engine.command_executor.execute(&cmd, &engine_ref);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, CommandError::ExecutionFailed(_)));
    assert!(err.to_string().contains("handler panicked"));
    assert!(err.to_string().contains("command handler crashed!"));
}

#[test]
fn non_panicking_command_handler_still_works() {
    let mut engine = app_shell::app_engine::TestEngine::new();
    engine.command_executor.register(
        CommandDefinition::new("ok.cmd", "OK", "OK"),
        Box::new(OkCommandHandler),
    ).unwrap();

    let cmd = Command::with_empty_input("ok.cmd", CommandContext::new(ExecutionId::new()));
    let engine_ref = engine.engine_ref();
    let result = engine.command_executor.execute(&cmd, &engine_ref).unwrap();

    assert!(result.is_success());
}

// --- Service panic isolation ---

#[test]
fn panicking_service_initialize_returns_error_not_crash() {
    use app_shell::app_engine::{
        EventBus, SignalBus, ResourceManager, ConfigStore, ConfigSchema, Preferences,
        StateManager, CommandExecutor, Scheduler, HistoryStore, LifecycleState,
    };

    // Build a TestContext for EngineRef
    let ctx = app_shell::app_engine::TestEngine::new();
    let engine_ref = ctx.engine_ref();

    let mut mgr = ServiceManager::new();
    let id = mgr.register(
        ServiceDefinition::new("panic_svc", "Panic Service", "Panics"),
        Box::new(PanickingService),
    ).unwrap();

    let result = mgr.initialize(id, &engine_ref);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("service panicked"));
}

#[test]
fn panicking_service_start_returns_error_not_crash() {
    let ctx = app_shell::app_engine::TestEngine::new();
    let engine_ref = ctx.engine_ref();

    struct InitOnlyService;
    impl Service for InitOnlyService {
        fn service_type(&self) -> &str { "init_only" }
        fn initialize(&mut self, _engine: &EngineRef) -> Result<(), ServiceError> { Ok(()) }
        fn start(&mut self, _engine: &EngineRef) -> Result<(), ServiceError> {
            panic!("service start crashed!");
        }
        fn stop(&mut self) -> Result<(), ServiceError> { Ok(()) }
        fn dispose(&mut self) -> Result<(), ServiceError> { Ok(()) }
    }

    let mut mgr = ServiceManager::new();
    let id = mgr.register(
        ServiceDefinition::new("init_only", "Init Only", "Init only"),
        Box::new(InitOnlyService),
    ).unwrap();

    mgr.initialize(id, &engine_ref).unwrap();
    let result = mgr.start(id, &engine_ref);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("service panicked"));
}

// --- Plugin panic isolation ---

#[test]
fn panicking_plugin_initialize_returns_error_not_crash() {
    let ctx = app_shell::app_engine::TestEngine::new();
    let engine_ref = ctx.engine_ref();

    let mut mgr = PluginManager::new();
    let id = mgr.register(
        PluginManifest::new("panic_plugin", "Panic Plugin"),
        Box::new(PanickingPlugin),
    ).unwrap();

    let result = mgr.initialize(id, &engine_ref);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("plugin panicked"));
}

#[test]
fn panicking_plugin_start_returns_error_not_crash() {
    let ctx = app_shell::app_engine::TestEngine::new();
    let engine_ref = ctx.engine_ref();

    struct InitOnlyPlugin;
    impl Plugin for InitOnlyPlugin {
        fn id(&self) -> &str { "init_only_plugin" }
        fn initialize(&mut self, _ctx: &PluginContext, _engine: &EngineRef) -> Result<(), PluginError> { Ok(()) }
        fn start(&mut self, _engine: &EngineRef) -> Result<(), PluginError> {
            panic!("plugin start crashed!");
        }
        fn stop(&mut self) -> Result<(), PluginError> { Ok(()) }
        fn dispose(&mut self) -> Result<(), PluginError> { Ok(()) }
    }

    let mut mgr = PluginManager::new();
    let id = mgr.register(
        PluginManifest::new("init_only_plugin", "Init Only Plugin"),
        Box::new(InitOnlyPlugin),
    ).unwrap();

    mgr.initialize(id, &engine_ref).unwrap();
    let result = mgr.start(id, &engine_ref);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("plugin panicked"));
}

// --- Mutex not poisoned after panic ---

#[test]
fn task_manager_not_poisoned_after_panic() {
    let engine = app_shell::app_engine::TestEngine::new();
    engine.task_manager.register(
        TaskDefinition::new("panic.task2", "Panic", "Panics"),
        Box::new(PanickingTaskHandler),
    ).unwrap();

    let task_id = engine.task_manager.create(
        "panic.task2",
        TaskContext::new(ExecutionId::new()),
        TaskInput::empty(),
    ).unwrap();

    let engine_ref = engine.engine_ref();
    let _ = engine.task_manager.start(task_id, &engine_ref); // panics, caught

    // TaskManager should still be usable
    assert_eq!(engine.task_manager.count(), 1);
    assert_eq!(engine.task_manager.get_state(task_id), Some(TaskState::Failed));

    // Can create another task
    let _task_id2 = engine.task_manager.create(
        "panic.task2",
        TaskContext::new(ExecutionId::new()),
        TaskInput::empty(),
    ).unwrap();
    assert_eq!(engine.task_manager.count(), 2);
}