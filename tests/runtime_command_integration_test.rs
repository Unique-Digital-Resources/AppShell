use app_shell::app_engine::{
    Bootstrap, Command, CommandContext, CommandDefinition, CommandError,
    CommandHandler, CommandInput, CommandResult, EngineRef, ExecutionId, Lifecycle,
    CommandExecutor,
};

struct GreetHandler;

impl CommandHandler for GreetHandler {
    fn execute(
        &self,
        input: &CommandInput,
        _context: &CommandContext,
        _engine: &EngineRef,
    ) -> Result<CommandResult, CommandError> {
        let name: &String = input.get::<String>()
            .ok_or_else(|| CommandError::InvalidArguments("expected name".to_string()))?;
        Ok(CommandResult::success_with(format!("Hello, {}!", name)))
    }
}

#[test]
fn runtime_owns_command_executor() {
    let runtime = Bootstrap::create().unwrap();
    let _: &CommandExecutor = runtime.command_executor();
}

#[test]
fn runtime_command_executor_starts_empty() {
    let runtime = Bootstrap::create().unwrap();
    assert!(runtime.command_executor().list().is_empty());
}

#[test]
fn runtime_can_register_and_execute_command() {
    let mut runtime = Bootstrap::create().unwrap();

    let def = CommandDefinition::new("app.greet", "Greet", "Greet someone by name");
    runtime.command_executor_mut().register(def, Box::new(GreetHandler)).unwrap();

    let cmd = Command::new(
        "app.greet",
        CommandContext::new(ExecutionId::new()),
        CommandInput::new("Alice".to_string()),
    );

    let result = runtime.execute_command(&cmd).unwrap();
    assert!(result.is_success());
    assert_eq!(result.output::<String>(), Some(&"Hello, Alice!".to_string()));
}

#[test]
fn runtime_command_executor_independent_of_lifecycle() {
    let mut runtime = Bootstrap::create().unwrap();
    let _ = runtime.command_executor_mut();
    runtime.initialize().unwrap();
    runtime.start().unwrap();
    assert!(runtime.command_executor().list().is_empty());
    runtime.stop().unwrap();
    runtime.dispose().unwrap();
}

#[test]
fn runtime_executor_unknown_command_returns_error() {
    let runtime = Bootstrap::create().unwrap();
    let cmd = Command::with_empty_input("app.missing", CommandContext::new(ExecutionId::new()));
    let result = runtime.execute_command(&cmd);
    assert!(matches!(result, Err(CommandError::UnknownCommand { .. })));
}