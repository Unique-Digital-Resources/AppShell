//! Tests for the foundational `EngineError` type.

use app_shell::app_engine::EngineError;

#[test]
fn invalid_transition_constructor_builds_correct_variant() {
    let err = EngineError::invalid_transition("Created", "Running");
    match err {
        EngineError::InvalidLifecycleTransition { from, to } => {
            assert_eq!(from, "Created");
            assert_eq!(to, "Running");
        }
        _ => panic!("expected InvalidLifecycleTransition"),
    }
}

#[test]
fn display_invalid_transition_is_human_readable() {
    let err = EngineError::invalid_transition("Created", "Running");
    assert_eq!(
        err.to_string(),
        "Invalid lifecycle transition: Created -> Running"
    );
}

#[test]
fn display_already_running() {
    let err = EngineError::AlreadyRunning;
    assert_eq!(err.to_string(), "Engine is already running");
}

#[test]
fn display_already_initialized() {
    let err = EngineError::AlreadyInitialized;
    assert_eq!(err.to_string(), "Engine is already initialized");
}

#[test]
fn display_not_initialized() {
    let err = EngineError::NotInitialized;
    assert_eq!(err.to_string(), "Engine is not initialized");
}

#[test]
fn display_already_stopped() {
    let err = EngineError::AlreadyStopped;
    assert_eq!(err.to_string(), "Engine is already stopped");
}

#[test]
fn display_already_disposed() {
    let err = EngineError::AlreadyDisposed;
    assert_eq!(err.to_string(), "Engine is already disposed");
}

#[test]
fn display_initialization_failed_includes_reason() {
    let err = EngineError::InitializationFailed("missing config".to_string());
    assert_eq!(err.to_string(), "Initialization failed: missing config");
}

#[test]
fn display_startup_failed_includes_reason() {
    let err = EngineError::StartupFailed("subsystem X".to_string());
    assert_eq!(err.to_string(), "Startup failed: subsystem X");
}

#[test]
fn display_shutdown_failed_includes_reason() {
    let err = EngineError::ShutdownFailed("timeout".to_string());
    assert_eq!(err.to_string(), "Shutdown failed: timeout");
}

#[test]
fn engine_error_implements_std_error() {
    fn assert_std_error<E: std::error::Error>(_: E) {}
    assert_std_error(EngineError::NotInitialized);
}

#[test]
fn engine_error_is_clone_and_eq() {
    let a = EngineError::AlreadyRunning;
    let b = a.clone();
    assert_eq!(a, b);
}