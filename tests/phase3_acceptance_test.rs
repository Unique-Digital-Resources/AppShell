//! Phase 3 acceptance criteria.

use app_shell::app_engine::{
    AppEngine, Bootstrap, Command, CommandContext, CommandDefinition, CommandDefinitionId,
    CommandError, CommandExecutor, CommandHandler, CommandInput, CommandRegistry,
    CommandResult, EngineRef, ExecutionId,
};

// --- Stub "domain" handler ---

struct StubDomainHandler;

impl CommandHandler for StubDomainHandler {
    fn execute(
        &self,
        input: &CommandInput,
        _context: &CommandContext,
        _engine: &EngineRef,
    ) -> Result<CommandResult, CommandError> {
        if let Some(s) = input.get::<String>() {
            Ok(CommandResult::success_with(s.clone()))
        } else {
            Err(CommandError::InvalidArguments("expected String".to_string()))
        }
    }
}

/// 1. Commands have stable identities
/// 2. Commands can carry input
/// 3. Commands can carry execution context
#[test]
fn acceptance_commands_basics() {
    let cmd = Command::new(
        "app.echo",
        CommandContext::new(ExecutionId::new()),
        CommandInput::new("payload".to_string()),
    );
    assert_eq!(cmd.definition_id(), &CommandDefinitionId::new("app.echo"));
    assert_eq!(cmd.input().get::<String>(), Some(&"payload".to_string()));
    assert!(cmd.id().value() > 0);
}

/// 4. Command definitions describe available capabilities
#[test]
fn acceptance_definitions_describe_capabilities() {
    let def = CommandDefinition::new("project.export", "Export Project", "Exports the current project");
    assert_eq!(def.id(), &CommandDefinitionId::new("project.export"));
    assert_eq!(def.name(), "Export Project");
    assert_eq!(def.description(), "Exports the current project");
}

/// 5. Commands can be registered
/// 6. Commands can be discovered
#[test]
fn acceptance_register_and_discover() {
    let mut reg = CommandRegistry::new();
    reg.register(
        CommandDefinition::new("app.echo", "Echo", "Echo"),
        Box::new(StubDomainHandler),
    ).unwrap();
    reg.register(
        CommandDefinition::new("app.quit", "Quit", "Quit"),
        Box::new(StubDomainHandler),
    ).unwrap();
    assert_eq!(reg.len(), 2);
    assert!(reg.contains(&CommandDefinitionId::new("app.echo")));
}

/// 7. Commands can be resolved by ID
#[test]
fn acceptance_resolve_by_id() {
    let mut reg = CommandRegistry::new();
    reg.register(
        CommandDefinition::new("app.echo", "Echo", "Echo"),
        Box::new(StubDomainHandler),
    ).unwrap();
    let def = reg.get(&CommandDefinitionId::new("app.echo")).unwrap();
    assert_eq!(def.name(), "Echo");
}

/// 8. Executor can execute a registered command
#[test]
fn acceptance_executor_executes() {
    let mut engine = Bootstrap::create().unwrap();
    engine.command_executor_mut().register(
        CommandDefinition::new("app.echo", "Echo", "Echo"),
        Box::new(StubDomainHandler),
    ).unwrap();

    let cmd = Command::new(
        "app.echo",
        CommandContext::new(ExecutionId::new()),
        CommandInput::new("hello".to_string()),
    );
    let result = engine.execute_command(&cmd).unwrap();
    assert!(result.is_success());
    assert_eq!(result.output::<String>(), Some(&"hello".to_string()));
}

/// 9. Invalid commands produce CommandError
#[test]
fn acceptance_invalid_command_error() {
    let engine = Bootstrap::create().unwrap();
    let cmd = Command::with_empty_input("app.unknown", CommandContext::new(ExecutionId::new()));
    let result = engine.execute_command(&cmd);
    assert!(matches!(result, Err(CommandError::UnknownCommand { .. })));
}

/// 10. Invalid input produces CommandError
#[test]
fn acceptance_invalid_input_error() {
    let mut engine = Bootstrap::create().unwrap();
    engine.command_executor_mut().register(
        CommandDefinition::new("app.echo", "Echo", "Echo"),
        Box::new(StubDomainHandler),
    ).unwrap();

    let cmd = Command::new(
        "app.echo",
        CommandContext::new(ExecutionId::new()),
        CommandInput::new(42_u64),
    );
    let result = engine.execute_command(&cmd);
    assert!(matches!(result, Err(CommandError::InvalidArguments(_))));
}

/// 11. Command execution returns CommandResult
#[test]
fn acceptance_returns_command_result() {
    let mut engine = Bootstrap::create().unwrap();
    engine.command_executor_mut().register(
        CommandDefinition::new("app.echo", "Echo", "Echo"),
        Box::new(StubDomainHandler),
    ).unwrap();

    let cmd = Command::new(
        "app.echo",
        CommandContext::new(ExecutionId::new()),
        CommandInput::new("hello".to_string()),
    );
    let result: CommandResult = engine.execute_command(&cmd).unwrap();
    assert!(result.is_success());
}

/// 12. Domain functionality exposed as a command
#[test]
fn acceptance_domain_exposed_as_command() {
    let mut engine = Bootstrap::create().unwrap();
    engine.command_executor_mut().register(
        CommandDefinition::new("domain.greet", "Greet", "Greet a name"),
        Box::new(StubDomainHandler),
    ).unwrap();

    let cmd = Command::new(
        "domain.greet",
        CommandContext::new(ExecutionId::new()),
        CommandInput::new("Alice".to_string()),
    );
    let result = engine.execute_command(&cmd).unwrap();
    assert_eq!(result.output::<String>(), Some(&"Alice".to_string()));
}

/// 13. App Engine does not know the meaning of domain commands
#[test]
fn acceptance_app_engine_does_not_know_domain_meaning() {
    let mut engine = Bootstrap::create().unwrap();
    engine.command_executor_mut().register(
        CommandDefinition::new("any.domain.cmd", "Any", "Any domain command"),
        Box::new(StubDomainHandler),
    ).unwrap();

    let cmd = Command::new(
        "any.domain.cmd",
        CommandContext::new(ExecutionId::new()),
        CommandInput::new("anything".to_string()),
    );
    // Execution succeeds without App Engine understanding what the command does.
    let result = engine.execute_command(&cmd);
    assert!(result.is_ok());
}

/// 14. Command system operates without UI / API / MCP / Tasks
#[test]
fn acceptance_operates_without_external_systems() {
    fn _check_types(
        _: AppEngine,
        _: Command,
        _: CommandContext,
        _: CommandDefinition,
        _: CommandDefinitionId,
        _: CommandExecutor,
        _: impl CommandHandler,
        _: CommandInput,
        _: CommandRegistry,
        _: CommandResult,
    ) where
        AppEngine: app_shell::app_engine::Lifecycle,
    {}

    let r = Bootstrap::create().unwrap();
    let cmd = Command::with_empty_input("app.noop", CommandContext::new(ExecutionId::new()));
    let ctx = CommandContext::new(ExecutionId::new());
    let def = CommandDefinition::new("app.noop", "Noop", "No operation");
    let id = CommandDefinitionId::new("app.noop");
    let exec = CommandExecutor::new();
    let input = CommandInput::empty();
    let registry = CommandRegistry::new();
    let result = CommandResult::success();
    _check_types(r, cmd, ctx, def, id, exec, StubDomainHandler, input, registry, result);

    let _: &CommandExecutor = Bootstrap::create().unwrap().command_executor();
}