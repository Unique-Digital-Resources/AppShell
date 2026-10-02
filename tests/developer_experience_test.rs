use app_shell::app_engine::*;

// --- Version tests ---

#[test]
fn version_construction() {
    let v = Version::new(2, 1, 3);
    assert_eq!(v.major(), 2);
    assert_eq!(v.minor(), 1);
    assert_eq!(v.patch(), 3);
}

#[test]
fn version_display() {
    let v = Version::new(1, 2, 3);
    assert_eq!(v.to_string(), "1.2.3");
}

#[test]
fn version_compatibility() {
    let v1 = Version::new(1, 0, 0);
    let v2 = Version::new(1, 5, 3);
    let v3 = Version::new(2, 0, 0);

    assert!(v1.is_compatible_with(&v2)); // Same major
    assert!(!v1.is_compatible_with(&v3)); // Different major
}

#[test]
fn version_ordering() {
    let v1 = Version::new(1, 0, 0);
    let v2 = Version::new(1, 1, 0);
    let v3 = Version::new(2, 0, 0);

    assert!(v1 < v2);
    assert!(v2 < v3);
    assert!(v1 < v3);
}

#[test]
fn version_at_least() {
    let v1 = Version::new(1, 2, 3);
    let v2 = Version::new(1, 2, 0);
    let v3 = Version::new(1, 3, 0);

    assert!(v1.is_at_least(&v2));
    assert!(!v1.is_at_least(&v3));
}

// --- PayloadSchema tests ---

#[test]
fn payload_schema_of_type() {
    let schema = PayloadSchema::of::<String>();
    assert!(schema.matches::<String>());
    assert!(!schema.matches::<i32>());
}

#[test]
fn payload_schema_with_description() {
    let schema = PayloadSchema::of_with_description::<String>("The document title");
    assert_eq!(schema.description(), "The document title");
    assert!(schema.matches::<String>());
}

#[test]
fn payload_schema_display() {
    let schema = PayloadSchema::of::<i32>();
    assert!(schema.to_string().contains("i32"));
}

#[test]
fn payload_schema_equality() {
    let s1 = PayloadSchema::of::<String>();
    let s2 = PayloadSchema::of::<String>();
    let s3 = PayloadSchema::of::<i32>();
    assert_eq!(s1, s2);
    assert_ne!(s1, s3);
}

// --- CommandDefinition with version + schema ---

#[test]
fn command_definition_with_version() {
    let def = CommandDefinition::new("app.test", "Test", "Test")
        .with_version(Version::new(2, 0, 0));
    assert_eq!(def.version(), &Version::new(2, 0, 0));
}

#[test]
fn command_definition_with_schemas() {
    let def = CommandDefinition::new("app.test", "Test", "Test")
        .with_input_schema(PayloadSchema::of::<String>())
        .with_output_schema(PayloadSchema::of::<i32>());

    assert!(def.input_schema().is_some());
    assert!(def.output_schema().is_some());
    assert!(def.input_schema().unwrap().matches::<String>());
    assert!(def.output_schema().unwrap().matches::<i32>());
}

#[test]
fn command_definition_compatibility() {
    let def = CommandDefinition::new("app.test", "Test", "Test")
        .with_version(Version::new(1, 5, 0));

    assert!(def.is_compatible_with(&Version::new(1, 0, 0)));
    assert!(!def.is_compatible_with(&Version::new(2, 0, 0)));
}

// --- TaskDefinition with version + schema ---

#[test]
fn task_definition_with_version() {
    let def = TaskDefinition::new("app.render", "Render", "Render")
        .with_version(Version::new(3, 1, 0));
    assert_eq!(def.version(), &Version::new(3, 1, 0));
}

#[test]
fn task_definition_with_schemas() {
    let def = TaskDefinition::new("app.render", "Render", "Render")
        .with_input_schema(PayloadSchema::of::<String>())
        .with_output_schema(PayloadSchema::of::<u64>());

    assert!(def.input_schema().is_some());
    assert!(def.output_schema().is_some());
}

// --- PluginManifest with API version ---

#[test]
fn manifest_with_engine_api_version() {
    let m = PluginManifest::new("test", "Test")
        .with_engine_api_version(Version::new(2, 0, 0));
    assert_eq!(m.engine_api_version(), &Version::new(2, 0, 0));
}

#[test]
fn manifest_compatibility_check() {
    let m = PluginManifest::new("test", "Test")
        .with_engine_api_version(Version::new(1, 5, 0));

    assert!(m.is_compatible_with(&Version::new(1, 0, 0)));
    assert!(!m.is_compatible_with(&Version::new(2, 0, 0)));
}

// --- MockCommandExecutor tests ---

struct EchoHandler;
impl CommandHandler for EchoHandler {
    fn execute(&self, _input: &CommandInput, _ctx: &CommandContext, _engine: &EngineRef) -> Result<CommandResult, CommandError> {
        Ok(CommandResult::success())
    }
}

#[test]
fn mock_executor_records_executions() {
    let ctx = TestEngine::new();
    let mut exec = MockCommandExecutor::new();
    exec.register(
        CommandDefinition::new("app.echo", "Echo", "Echo"),
        Box::new(EchoHandler),
    ).unwrap();

    let cmd = Command::with_empty_input("app.echo", CommandContext::new(ExecutionId::new()));
    let engine_ref = ctx.engine_ref();
    exec.execute(&cmd, &engine_ref).unwrap();

    exec.assert_executed("app.echo");
    assert_eq!(exec.executed_count(), 1);
}

#[test]
fn mock_executor_assert_not_executed() {
    let _ctx = TestEngine::new();
    let exec = MockCommandExecutor::new();
    // No commands registered, nothing executed
    exec.assert_not_executed("app.missing");
}

// --- MockEventBus tests ---

#[test]
fn mock_event_bus_records_publications() {
    let bus = MockEventBus::new();

    bus.publish(&Event::new("task.completed"));
    bus.publish(&Event::new("document.changed"));

    bus.assert_published("task.completed");
    bus.assert_published("document.changed");
    assert!(!bus.published_events().contains(&"task.failed".to_string()));
}

#[test]
fn mock_event_bus_assert_not_published() {
    let bus = MockEventBus::new();
    bus.assert_not_published("never.published");
}

#[test]
fn mock_event_bus_count() {
    let bus = MockEventBus::new();
    bus.publish(&Event::new("a"));
    bus.publish(&Event::new("b"));
    bus.publish(&Event::new("c"));
    assert_eq!(bus.published_count(), 3);
}

// --- MockSignalBus tests ---

#[test]
fn mock_signal_bus_records_emissions() {
    let bus = MockSignalBus::new();
    bus.emit(&Signal::new("task.progress_changed"));
    bus.emit(&Signal::new("state.changed"));

    bus.assert_emitted("task.progress_changed");
    bus.assert_emitted("state.changed");
    assert_eq!(bus.emitted_count(), 2);
}

// --- TestEngine tests ---

#[test]
fn test_engine_provides_all_subsystems() {
    let engine = TestEngine::new();
    let engine_ref = engine.engine_ref();

    // All systems accessible
    let _ = engine_ref.event_bus;
    let _ = engine_ref.signal_bus;
    let _ = engine_ref.resource_manager;
    let _ = engine_ref.config_store;
    let _ = engine_ref.preferences;
    let _ = engine_ref.state_manager;
    let _ = engine_ref.command_executor;
    let _ = engine_ref.scheduler;
    let _ = engine_ref.history_store;
}

#[test]
fn test_engine_can_register_and_execute() {
    let mut engine = TestEngine::new();
    engine.command_executor.register(
        CommandDefinition::new("app.test", "Test", "Test"),
        Box::new(EchoHandler),
    ).unwrap();

    let cmd = Command::with_empty_input("app.test", CommandContext::new(ExecutionId::new()));
    let engine_ref = engine.engine_ref();
    let result = engine.command_executor.execute(&cmd, &engine_ref).unwrap();
    assert!(result.is_success());
}

// --- Assertion helpers tests ---

#[test]
fn assertion_helpers_work() {
    let ctx = TestEngine::new();
    let mut exec = MockCommandExecutor::new();
    exec.register(
        CommandDefinition::new("app.test", "Test", "Test"),
        Box::new(EchoHandler),
    ).unwrap();

    let cmd = Command::with_empty_input("app.test", CommandContext::new(ExecutionId::new()));
    let engine_ref = ctx.engine_ref();
    exec.execute(&cmd, &engine_ref).unwrap();

    assert_command_executed(&exec, "app.test");
    assert_command_not_executed(&exec, "app.other");
}

#[test]
fn event_assertion_helpers_work() {
    let bus = MockEventBus::new();
    bus.publish(&Event::new("test.event"));

    assert_event_published(&bus, "test.event");
    assert_event_not_published(&bus, "other.event");
}

#[test]
fn signal_assertion_helpers_work() {
    let bus = MockSignalBus::new();
    bus.emit(&Signal::new("test.signal"));

    assert_signal_emitted(&bus, "test.signal");
}