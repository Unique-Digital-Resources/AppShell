//! Tests for `LifecycleState` query methods.

use app_shell::app_engine::LifecycleState;

#[test]
fn created_can_only_initialize() {
    let s = LifecycleState::Created;
    assert!(s.can_initialize());
    assert!(!s.can_start());
    assert!(!s.can_run());
    assert!(!s.can_stop());
    assert!(!s.can_dispose());
}

#[test]
fn initialized_can_only_start() {
    let s = LifecycleState::Initialized;
    assert!(!s.can_initialize());
    assert!(s.can_start());
    assert!(!s.can_run());
    assert!(!s.can_stop());
    assert!(!s.can_dispose());
}

#[test]
fn running_can_run_and_stop() {
    let s = LifecycleState::Running;
    assert!(s.can_run());
    assert!(s.can_stop());
    assert!(!s.can_start());
}

#[test]
fn stopped_can_only_dispose() {
    let s = LifecycleState::Stopped;
    assert!(s.can_dispose());
    assert!(!s.can_initialize());
    assert!(!s.can_start());
    assert!(!s.can_stop());
}

#[test]
fn disposed_is_terminal() {
    assert!(LifecycleState::Disposed.is_terminal());
    assert!(!LifecycleState::Running.is_terminal());
}

#[test]
fn as_str_matches_display() {
    assert_eq!(LifecycleState::Created.as_str(), "Created");
    assert_eq!(LifecycleState::Running.to_string(), "Running");
}