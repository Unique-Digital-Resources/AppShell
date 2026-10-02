//! Tests for `Command`, `CommandId`, and `CommandInput`.

use app_shell::app_engine::{
    Command, CommandContext, CommandId, CommandInput, CommandDefinitionId, ExecutionId,
};

#[test]
fn command_id_is_unique() {
    let a = CommandId::new();
    let b = CommandId::new();
    assert_ne!(a, b);
}

#[test]
fn command_id_is_copyable() {
    let id = CommandId::new();
    let copied = id;
    assert_eq!(id, copied);
}

#[test]
fn command_input_empty_is_empty() {
    let input = CommandInput::empty();
    assert!(input.is_empty());
    assert!(!input.has_data());
}

#[test]
fn command_input_holds_typed_data() {
    let input = CommandInput::new("hello".to_string());
    assert!(!input.is_empty());
    let s = input.get::<String>().expect("string must be retrievable");
    assert_eq!(s, "hello");
}

#[test]
fn command_input_get_wrong_type_returns_none() {
    let input = CommandInput::new(42_u64);
    assert!(input.get::<String>().is_none());
    assert!(input.get::<u64>().is_some());
}

#[test]
fn command_carries_definition_id_and_input_and_context() {
    let exec_id = ExecutionId::new();
    let context = CommandContext::new(exec_id);
    let cmd = Command::new("app.echo", context, CommandInput::new(42_u32));

    assert_eq!(cmd.definition_id(), &CommandDefinitionId::new("app.echo"));
    assert_eq!(cmd.input().get::<u32>(), Some(&42));
    assert_eq!(cmd.context().execution_id(), exec_id);
    assert!(cmd.id().value() > 0);
}

#[test]
fn command_id_is_stable_across_calls() {
    let exec_id = ExecutionId::new();
    let context = CommandContext::new(exec_id);
    let cmd = Command::with_empty_input("app.quit", context);
    let id1 = cmd.id();
    let id2 = cmd.id();
    assert_eq!(id1, id2);
}

#[test]
fn command_input_default_is_empty() {
    let input: CommandInput = CommandInput::default();
    assert!(input.is_empty());
}