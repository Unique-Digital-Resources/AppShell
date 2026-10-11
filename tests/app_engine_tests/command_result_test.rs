//! Tests for `CommandResult`.

use app_shell::app_engine::{CommandResult, ExecutionId};

#[test]
fn success_no_output() {
    let r = CommandResult::success();
    assert!(r.is_success());
    assert!(!r.is_failure());
    assert!(!r.has_output());
}

#[test]
fn success_with_output() {
    let r = CommandResult::success_with("hello".to_string());
    assert!(r.is_success());
    assert!(r.has_output());
    assert_eq!(r.output::<String>(), Some(&"hello".to_string()));
}

#[test]
fn output_wrong_type_returns_none() {
    let r = CommandResult::success_with(42_u64);
    assert!(r.output::<String>().is_none());
    assert!(r.output::<u64>().is_some());
}

#[test]
fn failure_no_output() {
    let r = CommandResult::failure();
    assert!(r.is_failure());
    assert!(!r.is_success());
    assert!(!r.has_output());
}

#[test]
fn failure_with_output() {
    let r = CommandResult::failure_with("reason".to_string());
    assert!(r.is_failure());
    assert_eq!(r.output::<String>(), Some(&"reason".to_string()));
}

#[test]
fn result_metadata_builder() {
    let r = CommandResult::success()
        .with_metadata("duration_ms", "42")
        .with_metadata("trace_id", "abc");
    assert_eq!(r.metadata().get("duration_ms"), Some(&"42".to_string()));
    assert_eq!(r.metadata().get("trace_id"), Some(&"abc".to_string()));
}

#[test]
fn result_execution_id_builder() {
    let exec_id = ExecutionId::new();
    let r = CommandResult::success().with_execution_id(exec_id);
    assert_eq!(r.execution_id(), Some(exec_id));
}