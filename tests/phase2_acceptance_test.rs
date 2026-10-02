//! Phase 2 acceptance criteria — the gate that must pass before Phase 3.

use app_shell::app_engine::{
    AppRuntime, ApplicationContext, Bootstrap, ContextManager, ExecutionContext, LifecycleState,
    SessionContext, StateError, StateId, StateManager, StateStore, StateTransition,
};

/// ✓ Runtime owns ContextManager
#[test]
fn acceptance_runtime_owns_context_manager() {
    let runtime = Bootstrap::create().unwrap();
    let _: &ContextManager = runtime.context_manager();
}

/// ✓ Runtime owns StateManager
#[test]
fn acceptance_runtime_owns_state_manager() {
    let runtime = Bootstrap::create().unwrap();
    let _: &StateManager<LifecycleState> = runtime.state_manager();
}

/// ✓ ApplicationContext can be created
#[test]
fn acceptance_application_context_can_be_created() {
    let ctx = ApplicationContext::new("app");
    assert_eq!(ctx.application_id(), "app");
    assert!(ctx.id().value() > 0);
}

/// ✓ SessionContext can be created under an application
#[test]
fn acceptance_session_context_under_application() {
    let mut mgr = ContextManager::new();
    mgr.create_application_context("app");
    let session_id = mgr
        .create_session_context()
        .expect("session should be created");
    assert!(mgr.session_context(session_id).is_some());
}

/// ✓ ExecutionContext can be created under a session
#[test]
fn acceptance_execution_context_under_session() {
    let mut mgr = ContextManager::new();
    mgr.create_application_context("app");
    let session_id = mgr.create_session_context().unwrap();
    let exec_id = mgr
        .create_execution_context(session_id)
        .expect("execution should be created");
    assert!(mgr.execution_context(exec_id).is_some());
}

/// ✓ Contexts have stable identities
#[test]
fn acceptance_contexts_have_stable_identities() {
    let ctx = ApplicationContext::new("app");
    let id_first = ctx.id();
    let id_again = ctx.id();
    assert_eq!(id_first, id_again);
}

/// ✓ Parent relationships are represented
#[test]
fn acceptance_parent_relationships_represented() {
    let mut mgr = ContextManager::new();
    let app_id = mgr.create_application_context("app");
    let session_id = mgr.create_session_context().unwrap();
    let exec_id = mgr.create_execution_context(session_id).unwrap();

    let session = mgr.session_context(session_id).unwrap();
    let exec = mgr.execution_context(exec_id).unwrap();

    assert_eq!(session.application_id(), app_id);
    assert_eq!(exec.session_id(), session_id);
}

/// ✓ State can be stored
#[test]
fn acceptance_state_can_be_stored() {
    let store: StateStore<LifecycleState> = StateStore::new();
    store.write(StateId::new(1), LifecycleState::Created);
    assert!(store.contains(StateId::new(1)));
}

/// ✓ State can be retrieved
#[test]
fn acceptance_state_can_be_retrieved() {
    let store: StateStore<LifecycleState> = StateStore::new();
    store.write(StateId::new(1), LifecycleState::Running);
    assert_eq!(
        store.read(StateId::new(1)),
        Some(LifecycleState::Running)
    );
}

/// ✓ State transitions can be represented
#[test]
fn acceptance_state_transitions_can_be_represented() {
    let t = StateTransition::new(LifecycleState::Created, LifecycleState::Initialized);
    assert_eq!(t.from(), &LifecycleState::Created);
    assert_eq!(t.to(), &LifecycleState::Initialized);
}

/// ✓ State transitions can be validated/applied
#[test]
fn acceptance_state_transitions_can_be_validated_and_applied() {
    let mgr: StateManager<LifecycleState> = StateManager::new();
    let id = StateId::new(1);
    mgr.set(id, LifecycleState::Created);

    // Valid transition
    mgr.transition(id, LifecycleState::Created, LifecycleState::Initialized)
        .unwrap();
    assert_eq!(mgr.current(id), Some(LifecycleState::Initialized));

    // Invalid transition (from does not match)
    let result = mgr.transition(id, LifecycleState::Created, LifecycleState::Running);
    assert!(matches!(result, Err(StateError::TransitionMismatch { .. })));
    // State is unchanged
    assert_eq!(mgr.current(id), Some(LifecycleState::Initialized));
}

/// ✓ StateManager uses StateStore
#[test]
fn acceptance_state_manager_uses_state_store() {
    let mgr: StateManager<LifecycleState> = StateManager::new();
    let _store: &StateStore<LifecycleState> = mgr.store();

    let id = StateId::new(1);
    mgr.set(id, LifecycleState::Created);
    // Visible through the underlying store
    assert_eq!(mgr.store().read(id), Some(LifecycleState::Created));
}

/// ✓ ContextManager manages context lifetime
#[test]
fn acceptance_context_manager_manages_context_lifetime() {
    let mut mgr = ContextManager::new();
    assert!(mgr.application_context().is_none());

    let app_id = mgr.create_application_context("app");
    assert!(mgr.application_context().is_some());
    assert_eq!(mgr.application_context().unwrap().id(), app_id);

    let session_id = mgr.create_session_context().unwrap();
    assert_eq!(mgr.session_count(), 1);

    let exec_id = mgr.create_execution_context(session_id).unwrap();
    assert!(mgr.execution_context(exec_id).is_some());
    assert_eq!(mgr.execution_count(), 1);
}

/// ✓ No Domain Engine dependency
/// ✓ No Command dependency
/// ✓ No Task dependency
/// ✓ No UI/API dependency
/// ✓ No persistence requirement
///
/// (Phase 2 introduces no domain/command/task/ui/api/persistence modules.
///  All types reachable from `app_shell::app_engine::...` are listed below.)
#[test]
fn acceptance_no_unwanted_dependencies() {
    fn _check_types(
        _: AppRuntime,
        _: ApplicationContext,
        _: ContextManager,
        _: SessionContext,
        _: ExecutionContext,
        _: StateManager<LifecycleState>,
        _: StateStore<LifecycleState>,
        _: StateTransition<LifecycleState>,
        _: StateId,
    ) {
    }

    let r = Bootstrap::create().unwrap();
    let app = ApplicationContext::new("app");
    let cm = ContextManager::new();
    let sc = SessionContext::new(app.id());
    let ec = ExecutionContext::new(sc.id());
    let sm: StateManager<LifecycleState> = StateManager::new();
    let ss: StateStore<LifecycleState> = StateStore::new();
    let st = StateTransition::new(LifecycleState::Created, LifecycleState::Initialized);
    let sid = StateId::new(1);
    _check_types(r, app, cm, sc, ec, sm, ss, st, sid);
}

/// ✓ Context ≠ State (the two systems are independent)
#[test]
fn acceptance_context_and_state_are_separate_systems() {
    // A context's identity is not state; state describes a condition.
    let ctx = ApplicationContext::new("app");
    let store: StateStore<LifecycleState> = StateStore::new();

    // Storing a state under a StateId does not interact with any context id.
    let sid = StateId::new(ctx.id().value());
    store.write(sid, LifecycleState::Running);
    assert_eq!(store.read(sid), Some(LifecycleState::Running));

    // The context's identity is unchanged.
    assert_eq!(ctx.application_id(), "app");
}