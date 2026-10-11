//! Tests for `CommandContext`.

use app_shell::app_engine::{ApplicationId, CommandContext, ExecutionId, SessionId};

#[test]
fn context_with_just_execution_id() {
    let exec_id = ExecutionId::new();
    let ctx = CommandContext::new(exec_id);
    assert_eq!(ctx.execution_id(), exec_id);
    assert!(ctx.application_id().is_none());
    assert!(ctx.session_id().is_none());
}

#[test]
fn context_builder_adds_application_and_session() {
    let app_id = ApplicationId::new();
    let session_id = SessionId::new();
    let exec_id = ExecutionId::new();
    let ctx = CommandContext::new(exec_id)
        .with_application(app_id)
        .with_session(session_id);

    assert_eq!(ctx.application_id(), Some(app_id));
    assert_eq!(ctx.session_id(), Some(session_id));
    assert_eq!(ctx.execution_id(), exec_id);
}

#[test]
fn context_metadata_can_be_added() {
    let exec_id = ExecutionId::new();
    let ctx = CommandContext::new(exec_id)
        .with_metadata("source", "cli")
        .with_metadata("user", "alice");

    assert_eq!(ctx.metadata().get("source"), Some(&"cli".to_string()));
    assert_eq!(ctx.metadata().get("user"), Some(&"alice".to_string()));
}

#[test]
fn context_is_cloneable() {
    let exec_id = ExecutionId::new();
    let ctx = CommandContext::new(exec_id).with_metadata("k", "v");
    let cloned = ctx.clone();
    assert_eq!(ctx.metadata(), cloned.metadata());
}