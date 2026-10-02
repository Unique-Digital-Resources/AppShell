//! Tests for `ExecutionContext`.

use app_shell::app_engine::{ExecutionContext, SessionId};

#[test]
fn execution_context_has_unique_id() {
    let session_id = SessionId::new();
    let e1 = ExecutionContext::new(session_id);
    let e2 = ExecutionContext::new(session_id);
    assert_ne!(e1.id(), e2.id());
}

#[test]
fn execution_context_stores_parent_session() {
    let session_id = SessionId::new();
    let exec = ExecutionContext::new(session_id);
    assert_eq!(exec.session_id(), session_id);
}

#[test]
fn execution_context_id_is_copyable() {
    let session_id = SessionId::new();
    let exec = ExecutionContext::new(session_id);
    let id = exec.id();
    let copied = id;
    assert_eq!(id, copied);
    assert!(id.value() > 0);
}

#[test]
fn different_executions_under_same_session_are_distinct() {
    let session_id = SessionId::new();
    let e1 = ExecutionContext::new(session_id);
    let e2 = ExecutionContext::new(session_id);
    assert_ne!(e1, e2);
}

#[test]
fn execution_context_equality_is_value_based() {
    let session_id = SessionId::new();
    let e = ExecutionContext::new(session_id);
    let cloned = e.clone();
    assert_eq!(e, cloned);
}