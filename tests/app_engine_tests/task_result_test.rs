use app_shell::app_engine::{TaskResult, TaskResultStatus};

#[test]
fn completed_no_output() {
    let r = TaskResult::completed();
    assert!(r.is_completed());
    assert!(!r.has_output());
    assert_eq!(r.status(), TaskResultStatus::Completed);
}

#[test]
fn completed_with_output() {
    let r = TaskResult::completed_with(42_i32);
    assert!(r.is_completed());
    assert_eq!(r.output::<i32>(), Some(&42));
}

#[test]
fn cancelled_result() {
    let r = TaskResult::cancelled();
    assert!(r.is_cancelled());
    assert!(!r.is_completed());
}

#[test]
fn failed_with_error_message() {
    let r = TaskResult::failed("disk full");
    assert!(r.is_failed());
    assert_eq!(r.error(), Some("disk full"));
}

#[test]
fn failed_with_output() {
    let r = TaskResult::failed_with("timeout", 500_i64);
    assert!(r.is_failed());
    assert_eq!(r.error(), Some("timeout"));
    assert_eq!(r.output::<i64>(), Some(&500));
}

#[test]
fn metadata_builder() {
    let r = TaskResult::completed()
        .with_metadata("duration_ms", "42");
    assert_eq!(r.metadata().get("duration_ms"), Some(&"42".to_string()));
}