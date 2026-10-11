use app_shell::app_engine::{
    EngineRef,
    EventBus, SignalBus, ResourceManager, ConfigStore, ConfigSchema, Preferences,
    StateManager, CommandExecutor, Scheduler, HistoryStore,
    TaskContext, TaskError, TaskHandler, TaskInput, TaskManager, TaskResult,
    TaskState, ExecutionId, TaskDefinition, LifecycleState,
};

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

struct EchoHandler;
impl TaskHandler for EchoHandler {
    fn execute(
        &self,
        input: &TaskInput,
        _ctx: &TaskContext,
        _engine: &EngineRef,
    ) -> Result<TaskResult, TaskError> {
        match input.get::<String>() {
            Some(s) => Ok(TaskResult::completed_with(s.clone())),
            None => Err(TaskError::ExecutionFailed { id: 0, reason: "expected String".to_string() }),
        }
    }
}

struct FailHandler;
impl TaskHandler for FailHandler {
    fn execute(
        &self,
        _input: &TaskInput,
        _ctx: &TaskContext,
        _engine: &EngineRef,
    ) -> Result<TaskResult, TaskError> {
        Err(TaskError::ExecutionFailed { id: 0, reason: "intentional".to_string() })
    }
}

fn setup_manager() -> TaskManager {
    let mgr = TaskManager::new();
    mgr.register(
        TaskDefinition::new("app.echo", "Echo", "Echo input").with_cancel_support(),
        Box::new(EchoHandler),
    ).unwrap();
    mgr.register(
        TaskDefinition::new("app.fail", "Fail", "Always fails"),
        Box::new(FailHandler),
    ).unwrap();
    mgr
}

#[test]
fn create_task_returns_id() {
    let mgr = setup_manager();
    let id = mgr.create(
        "app.echo",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new("hello".to_string()),
    ).unwrap();
    assert!(mgr.contains(id));
    assert_eq!(mgr.count(), 1);
    assert_eq!(mgr.get_state(id), Some(TaskState::Pending));
}

#[test]
fn create_unknown_definition_fails() {
    let mgr = setup_manager();
    let result = mgr.create("unknown", TaskContext::new(ExecutionId::new()), TaskInput::empty());
    assert!(matches!(result, Err(TaskError::UnknownDefinition { .. })));
}

#[test]
fn start_completes_task() {
    let ctx = TestContext::new();
    let mgr = setup_manager();
    let id = mgr.create(
        "app.echo",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new("hello".to_string()),
    ).unwrap();

    let engine_ref = ctx.engine_ref();
    let result = mgr.start(id, &engine_ref).unwrap();
    assert!(result.is_completed());
    assert_eq!(result.output::<String>(), Some(&"hello".to_string()));
    assert_eq!(mgr.get_state(id), Some(TaskState::Completed));
}

#[test]
fn start_failing_task_sets_failed_state() {
    let ctx = TestContext::new();
    let mgr = setup_manager();
    let id = mgr.create(
        "app.fail",
        TaskContext::new(ExecutionId::new()),
        TaskInput::empty(),
    ).unwrap();

    let engine_ref = ctx.engine_ref();
    let result = mgr.start(id, &engine_ref);
    assert!(result.is_err());
    assert_eq!(mgr.get_state(id), Some(TaskState::Failed));
}

#[test]
fn cancel_pending_task() {
    let mgr = setup_manager();
    let id = mgr.create(
        "app.echo",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new("hello".to_string()),
    ).unwrap();
    mgr.cancel(id).unwrap();
    assert_eq!(mgr.get_state(id), Some(TaskState::Cancelled));
}

#[test]
fn cancel_non_cancellable_fails() {
    let mgr = TaskManager::new();
    mgr.register(
        TaskDefinition::new("app.rigid", "Rigid", "No cancel"),
        Box::new(EchoHandler),
    ).unwrap();
    let id = mgr.create(
        "app.rigid",
        TaskContext::new(ExecutionId::new()),
        TaskInput::empty(),
    ).unwrap();
    let result = mgr.cancel(id);
    assert!(matches!(result, Err(TaskError::NotCancellable { .. })));
}

#[test]
fn pause_resume_pausable_task() {
    let mgr = TaskManager::new();
    mgr.register(
        TaskDefinition::new("app.pausable", "Pausable", "Pausable").with_pause_support(),
        Box::new(EchoHandler),
    ).unwrap();
    let id = mgr.create(
        "app.pausable",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new("hi".to_string()),
    ).unwrap();
    let result = mgr.pause(id);
    assert!(matches!(result, Err(TaskError::NotPausable { .. })));
}

#[test]
fn start_already_started_fails() {
    let ctx = TestContext::new();
    let mgr = setup_manager();
    let id = mgr.create(
        "app.echo",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new("hello".to_string()),
    ).unwrap();

    let engine_ref = ctx.engine_ref();
    mgr.start(id, &engine_ref).unwrap();
    let result = mgr.start(id, &engine_ref);
    assert!(matches!(result, Err(TaskError::InvalidState { .. })));
}

#[test]
fn multiple_tasks_managed() {
    let ctx = TestContext::new();
    let mgr = setup_manager();
    let id1 = mgr.create("app.echo", TaskContext::new(ExecutionId::new()), TaskInput::new("a".to_string())).unwrap();
    let id2 = mgr.create("app.echo", TaskContext::new(ExecutionId::new()), TaskInput::new("b".to_string())).unwrap();
    let id3 = mgr.create("app.fail", TaskContext::new(ExecutionId::new()), TaskInput::empty()).unwrap();

    assert_eq!(mgr.count(), 3);
    let engine_ref = ctx.engine_ref();
    mgr.start(id1, &engine_ref).unwrap();
    mgr.start(id2, &engine_ref).unwrap();
    let _ = mgr.start(id3, &engine_ref);
    assert_eq!(mgr.get_state(id1), Some(TaskState::Completed));
    assert_eq!(mgr.get_state(id2), Some(TaskState::Completed));
    assert_eq!(mgr.get_state(id3), Some(TaskState::Failed));
}

#[test]
fn remove_task() {
    let mgr = setup_manager();
    let id = mgr.create("app.echo", TaskContext::new(ExecutionId::new()), TaskInput::empty()).unwrap();
    assert!(mgr.remove(id).is_some());
    assert!(!mgr.contains(id));
    assert_eq!(mgr.count(), 0);
}

#[test]
fn task_context_gets_task_id_after_create() {
    let mgr = setup_manager();
    let id = mgr.create(
        "app.echo",
        TaskContext::new(ExecutionId::new()),
        TaskInput::empty(),
    ).unwrap();
    assert_eq!(mgr.task_context_task_id(id), Some(id));
}