//! Tests for the centralized transition table in `LifecycleManager`.

use app_shell::app_engine::{EngineError, LifecycleManager, LifecycleState};

#[test]
fn full_happy_path() {
    let m = LifecycleManager::new();
    let s = LifecycleState::Created;
    let s = m.initialize(s).expect("initialize");
    assert_eq!(s, LifecycleState::Initialized);
    let s = m.start(s).expect("start");
    assert_eq!(s, LifecycleState::Running);
    let s = m.run(s).expect("run");
    assert_eq!(s, LifecycleState::Running);
    let s = m.stop(s).expect("stop");
    assert_eq!(s, LifecycleState::Stopped);
    let s = m.dispose(s).expect("dispose");
    assert_eq!(s, LifecycleState::Disposed);
}

#[test]
fn initialize_from_non_created_rejected() {
    let m = LifecycleManager::new();
    let result = m.initialize(LifecycleState::Initialized);
    assert!(matches!(result, Err(EngineError::InvalidLifecycleTransition { .. })));
}

#[test]
fn start_from_created_rejected() {
    let m = LifecycleManager::new();
    let result = m.start(LifecycleState::Created);
    assert!(matches!(result, Err(EngineError::InvalidLifecycleTransition { .. })));
}

#[test]
fn run_from_initialized_rejected() {
    let m = LifecycleManager::new();
    let result = m.run(LifecycleState::Initialized);
    assert!(matches!(result, Err(EngineError::InvalidLifecycleTransition { .. })));
}

#[test]
fn stop_from_stopped_rejected() {
    let m = LifecycleManager::new();
    let result = m.stop(LifecycleState::Stopped);
    assert!(matches!(result, Err(EngineError::InvalidLifecycleTransition { .. })));
}

#[test]
fn dispose_from_running_rejected() {
    let m = LifecycleManager::new();
    let result = m.dispose(LifecycleState::Running);
    assert!(matches!(result, Err(EngineError::InvalidLifecycleTransition { .. })));
}

#[test]
fn disposed_blocks_all_transitions() {
    let m = LifecycleManager::new();
    let s = LifecycleState::Disposed;
    assert!(m.initialize(s).is_err());
    assert!(m.start(s).is_err());
    assert!(m.run(s).is_err());
    assert!(m.stop(s).is_err());
    assert!(m.dispose(s).is_err());
}

#[test]
fn invalid_transition_carries_state_names() {
    let m = LifecycleManager::new();
    if let Err(EngineError::InvalidLifecycleTransition { from, to }) =
        m.start(LifecycleState::Created)
    {
        assert_eq!(from, "Created");
        assert_eq!(to, "Running");
    } else {
        panic!("expected InvalidLifecycleTransition");
    }
}

#[test]
fn run_from_stopped_rejected() {
    let m = LifecycleManager::new();
    let result = m.run(LifecycleState::Stopped);
    assert!(matches!(result, Err(EngineError::InvalidLifecycleTransition { .. })));
}