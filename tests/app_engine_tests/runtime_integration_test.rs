//! Tests that verify Runtime owns and exposes ContextManager + StateManager.

use app_shell::app_engine::{
    Bootstrap, ContextManager, Lifecycle, LifecycleState, StateManager,
};

#[test]
fn runtime_owns_context_manager() {
    let runtime = Bootstrap::create().unwrap();
    let _mgr: &ContextManager = runtime.context_manager();
}

#[test]
fn runtime_owns_state_manager() {
    let runtime = Bootstrap::create().unwrap();
    let _mgr: &StateManager<LifecycleState> = runtime.state_manager();
}

#[test]
fn runtime_state_id_is_stable() {
    let runtime = Bootstrap::create().unwrap();
    let id1 = runtime.runtime_state_id();
    let id2 = runtime.runtime_state_id();
    assert_eq!(id1, id2);
}

#[test]
fn runtime_initial_state_in_state_manager() {
    let runtime = Bootstrap::create().unwrap();
    let id = runtime.runtime_state_id();
    assert_eq!(
        runtime.state_manager().current(id),
        Some(LifecycleState::Created)
    );
}

#[test]
fn runtime_lifecycle_updates_state_manager() {
    let mut runtime = Bootstrap::create().unwrap();
    let id = runtime.runtime_state_id();

    runtime.initialize().unwrap();
    assert_eq!(
        runtime.state_manager().current(id),
        Some(LifecycleState::Initialized)
    );

    runtime.start().unwrap();
    assert_eq!(
        runtime.state_manager().current(id),
        Some(LifecycleState::Running)
    );

    runtime.stop().unwrap();
    assert_eq!(
        runtime.state_manager().current(id),
        Some(LifecycleState::Stopped)
    );

    runtime.dispose().unwrap();
    assert_eq!(
        runtime.state_manager().current(id),
        Some(LifecycleState::Disposed)
    );
}

#[test]
fn runtime_context_manager_has_application_context() {
    let runtime = Bootstrap::create().unwrap();
    let app = runtime
        .context_manager()
        .application_context()
        .expect("application context must exist");
    assert_eq!(app.application_id(), "app_shell");
}

#[test]
fn runtime_can_create_session_via_context_manager() {
    let mut runtime = Bootstrap::create().unwrap();
    let session_id = runtime
        .context_manager_mut()
        .create_session_context()
        .unwrap();
    assert!(runtime.context_manager().session_context(session_id).is_some());
    assert_eq!(runtime.context_manager().session_count(), 1);
}

#[test]
fn runtime_can_create_execution_under_session() {
    let mut runtime = Bootstrap::create().unwrap();
    let session_id = runtime
        .context_manager_mut()
        .create_session_context()
        .unwrap();
    let exec_id = runtime
        .context_manager_mut()
        .create_execution_context(session_id)
        .unwrap();
    let exec = runtime
        .context_manager()
        .execution_context(exec_id)
        .expect("execution must exist");
    assert_eq!(exec.session_id(), session_id);
}

#[test]
fn runtime_state_visible_through_both_accessors() {
    use app_shell::app_engine::RuntimeState;

    let mut runtime = Bootstrap::create().unwrap();
    let id = runtime.runtime_state_id();
    runtime.initialize().unwrap();
    // Visible via runtime.state() (Phase 1 field)
    assert_eq!(runtime.state(), RuntimeState::Initialized);
    // Visible via state_manager (Phase 2 infrastructure)
    assert_eq!(
        runtime.state_manager().current(id),
        Some(LifecycleState::Initialized)
    );
}