//! Tests for `AppRuntime` lifecycle and accessors.

use app_shell::app_engine::{
    AppRuntime, EngineError, Lifecycle, LifecycleState, RuntimeState,
};

#[test]
fn new_runtime_starts_in_created_state() {
    let runtime = AppRuntime::new().unwrap();
    assert_eq!(runtime.state(), RuntimeState::Created);
}

#[test]
fn full_lifecycle_progression() {
    let mut r = AppRuntime::new().unwrap();
    r.initialize().unwrap();
    assert_eq!(r.state(), RuntimeState::Initialized);
    r.start().unwrap();
    assert_eq!(r.state(), RuntimeState::Running);
    r.run().unwrap();
    assert_eq!(r.state(), RuntimeState::Running);
    r.stop().unwrap();
    assert_eq!(r.state(), RuntimeState::Stopped);
    r.dispose().unwrap();
    assert_eq!(r.state(), RuntimeState::Disposed);
}

#[test]
fn calling_initialize_twice_fails() {
    let mut r = AppRuntime::new().unwrap();
    r.initialize().unwrap();
    let result = r.initialize();
    assert!(matches!(result, Err(EngineError::InvalidLifecycleTransition { .. })));
}

#[test]
fn calling_start_before_initialize_fails() {
    let mut r = AppRuntime::new().unwrap();
    let result = r.start();
    assert!(matches!(result, Err(EngineError::InvalidLifecycleTransition { .. })));
}

#[test]
fn calling_run_before_start_fails() {
    let mut r = AppRuntime::new().unwrap();
    r.initialize().unwrap();
    let result = r.run();
    assert!(matches!(result, Err(EngineError::InvalidLifecycleTransition { .. })));
}

#[test]
fn calling_stop_before_start_fails() {
    let mut r = AppRuntime::new().unwrap();
    r.initialize().unwrap();
    let result = r.stop();
    assert!(matches!(result, Err(EngineError::InvalidLifecycleTransition { .. })));
}

#[test]
fn calling_dispose_before_stop_fails() {
    let mut r = AppRuntime::new().unwrap();
    r.initialize().unwrap();
    r.start().unwrap();
    let result = r.dispose();
    assert!(matches!(result, Err(EngineError::InvalidLifecycleTransition { .. })));
}

#[test]
fn disposed_runtime_rejects_all_operations() {
    let mut r = AppRuntime::new().unwrap();
    r.initialize().unwrap();
    r.start().unwrap();
    r.stop().unwrap();
    r.dispose().unwrap();

    assert!(r.initialize().is_err());
    assert!(r.start().is_err());
    assert!(r.run().is_err());
    assert!(r.stop().is_err());
    assert!(r.dispose().is_err());
}

#[test]
fn stop_sets_stop_requested_flag() {
    let mut r = AppRuntime::new().unwrap();
    r.initialize().unwrap();
    r.start().unwrap();
    assert!(!r.stop_requested());
    r.stop().unwrap();
    assert!(r.stop_requested());
}

#[test]
fn context_is_accessible_after_construction() {
    let r = AppRuntime::new().unwrap();
    assert_eq!(r.context().application_id(), "app_shell");
    assert!(r.context().instance_id() > 0);
}

#[test]
fn runtime_state_matches_lifecycle_state() {
    let runtime_state: RuntimeState = RuntimeState::Created;
    let lifecycle_state: LifecycleState = LifecycleState::Created;
    assert_eq!(runtime_state, lifecycle_state);
}