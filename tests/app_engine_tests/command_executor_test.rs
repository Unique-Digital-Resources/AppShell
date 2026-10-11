use app_shell::app_engine::{
    Command, CommandContext, CommandDefinition, CommandDefinitionId, CommandError,
    CommandExecutor, CommandHandler, CommandInput, CommandResult, EngineRef,
    ExecutionId,
    EventBus, SignalBus, ResourceManager, ConfigStore, ConfigSchema, Preferences,
    StateManager, Scheduler, HistoryStore, LifecycleState,
};

// --- Test context for constructing EngineRef without a full AppEngine ---

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

// --- Stub handlers ---

struct EchoHandler;

impl CommandHandler for EchoHandler {
    fn execute(
        &self,
        input: &CommandInput,
        _ctx: &CommandContext,
        _engine: &EngineRef,
    ) -> Result<CommandResult, CommandError> {
        match input.get::<String>() {
            Some(s) => Ok(CommandResult::success_with(s.clone())),
            None => Err(CommandError::InvalidArguments("expected String".to_string())),
        }
    }
}

struct AddHandler;

#[derive(Debug)]
struct AddInput { a: i32, b: i32 }

impl CommandHandler for AddHandler {
    fn execute(
        &self,
        input: &CommandInput,
        _ctx: &CommandContext,
        _engine: &EngineRef,
    ) -> Result<CommandResult, CommandError> {
        let add: &AddInput = input.get::<AddInput>()
            .ok_or_else(|| CommandError::InvalidArguments("expected AddInput".to_string()))?;
        Ok(CommandResult::success_with(add.a + add.b))
    }
}

struct FailHandler;

impl CommandHandler for FailHandler {
    fn execute(
        &self,
        _input: &CommandInput,
        _ctx: &CommandContext,
        _engine: &EngineRef,
    ) -> Result<CommandResult, CommandError> {
        Err(CommandError::ExecutionFailed("intentional".to_string()))
    }
}

fn build_executor_with_echo() -> (TestContext, CommandExecutor) {
    let ctx = TestContext::new();
    let mut exec = CommandExecutor::new();
    let def = CommandDefinition::new("app.echo", "Echo", "Echoes the input string");
    exec.register(def, Box::new(EchoHandler)).unwrap();
    (ctx, exec)
}

#[test]
fn execute_unknown_command_returns_error() {
    let (ctx, exec) = (TestContext::new(), CommandExecutor::new());
    let cmd = Command::with_empty_input("app.unknown", CommandContext::new(ExecutionId::new()));
    let engine_ref = ctx.engine_ref();
    let result = exec.execute(&cmd, &engine_ref);
    assert!(matches!(result, Err(CommandError::UnknownCommand { .. })));
}

#[test]
fn execute_registered_command_succeeds() {
    let (ctx, exec) = build_executor_with_echo();
    let cmd = Command::new(
        "app.echo",
        CommandContext::new(ExecutionId::new()),
        CommandInput::new("hello".to_string()),
    );
    let engine_ref = ctx.engine_ref();
    let result = exec.execute(&cmd, &engine_ref).unwrap();
    assert!(result.is_success());
    assert_eq!(result.output::<String>(), Some(&"hello".to_string()));
}

#[test]
fn execute_with_invalid_input_returns_command_error() {
    let (ctx, exec) = build_executor_with_echo();
    let cmd = Command::new(
        "app.echo",
        CommandContext::new(ExecutionId::new()),
        CommandInput::new(42_u64),
    );
    let engine_ref = ctx.engine_ref();
    let result = exec.execute(&cmd, &engine_ref);
    assert!(matches!(result, Err(CommandError::InvalidArguments(_))));
}

#[test]
fn execute_custom_input_type() {
    let ctx = TestContext::new();
    let mut exec = CommandExecutor::new();
    exec.register(
        CommandDefinition::new("math.add", "Add", "Add two integers"),
        Box::new(AddHandler),
    ).unwrap();

    let cmd = Command::new(
        "math.add",
        CommandContext::new(ExecutionId::new()),
        CommandInput::new(AddInput { a: 3, b: 4 }),
    );
    let engine_ref = ctx.engine_ref();
    let result = exec.execute(&cmd, &engine_ref).unwrap();
    assert!(result.is_success());
    assert_eq!(result.output::<i32>(), Some(&7));
}

#[test]
fn execute_handler_can_fail() {
    let ctx = TestContext::new();
    let mut exec = CommandExecutor::new();
    exec.register(
        CommandDefinition::new("app.fail", "Fail", "Always fails"),
        Box::new(FailHandler),
    ).unwrap();

    let cmd = Command::with_empty_input("app.fail", CommandContext::new(ExecutionId::new()));
    let engine_ref = ctx.engine_ref();
    let result = exec.execute(&cmd, &engine_ref);
    assert!(matches!(result, Err(CommandError::ExecutionFailed(_))));
}

#[test]
fn executor_resolves_command_definition_by_id() {
    let (_ctx, exec) = build_executor_with_echo();
    let def = exec.get(&CommandDefinitionId::new("app.echo")).expect("definition must exist");
    assert_eq!(def.name(), "Echo");
}

#[test]
fn executor_lists_registered_commands() {
    let _ctx = TestContext::new();
    let mut exec = CommandExecutor::new();
    exec.register(CommandDefinition::new("app.echo", "Echo", "Echo"), Box::new(EchoHandler)).unwrap();
    exec.register(CommandDefinition::new("math.add", "Add", "Add"), Box::new(AddHandler)).unwrap();
    let list = exec.list();
    assert_eq!(list.len(), 2);
}

#[test]
fn executor_unregister_removes_command() {
    let (_ctx, mut exec) = build_executor_with_echo();
    assert!(exec.contains(&CommandDefinitionId::new("app.echo")));
    let removed = exec.unregister(&CommandDefinitionId::new("app.echo"));
    assert!(removed.is_some());
    assert!(!exec.contains(&CommandDefinitionId::new("app.echo")));

    let cmd = Command::with_empty_input("app.echo", CommandContext::new(ExecutionId::new()));
    let engine_ref = _ctx.engine_ref();
    assert!(exec.execute(&cmd, &engine_ref).is_err());
}