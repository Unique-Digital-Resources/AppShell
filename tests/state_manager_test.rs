//! Tests for `StateManager`.

use app_shell::app_engine::{
    LifecycleState, StateError, StateId, StateManager, StateStore, StateTransition,
};

#[test]
fn set_and_retrieve_state() {
    let mgr: StateManager<LifecycleState> = StateManager::new();
    let id = StateId::new(1);
    mgr.set(id, LifecycleState::Created);
    assert_eq!(mgr.current(id), Some(LifecycleState::Created));
}

#[test]
fn current_returns_none_for_missing() {
    let mgr: StateManager<LifecycleState> = StateManager::new();
    assert_eq!(mgr.current(StateId::new(99)), None);
}

#[test]
fn set_overwrites_existing() {
    let mgr: StateManager<LifecycleState> = StateManager::new();
    let id = StateId::new(1);
    mgr.set(id, LifecycleState::Created);
    mgr.set(id, LifecycleState::Running);
    assert_eq!(mgr.current(id), Some(LifecycleState::Running));
}

#[test]
fn apply_transition_succeeds_when_from_matches() {
    let mgr: StateManager<LifecycleState> = StateManager::new();
    let id = StateId::new(1);
    mgr.set(id, LifecycleState::Created);
    let transition = StateTransition::new(LifecycleState::Created, LifecycleState::Initialized);
    mgr.apply_transition(id, &transition).unwrap();
    assert_eq!(mgr.current(id), Some(LifecycleState::Initialized));
}

#[test]
fn apply_transition_fails_when_from_mismatches() {
    let mgr: StateManager<LifecycleState> = StateManager::new();
    let id = StateId::new(1);
    mgr.set(id, LifecycleState::Running);
    let transition = StateTransition::new(LifecycleState::Created, LifecycleState::Initialized);
    let result = mgr.apply_transition(id, &transition);
    assert!(matches!(result, Err(StateError::TransitionMismatch { .. })));
    // State should be unchanged
    assert_eq!(mgr.current(id), Some(LifecycleState::Running));
}

#[test]
fn apply_transition_fails_when_state_missing() {
    let mgr: StateManager<LifecycleState> = StateManager::new();
    let id = StateId::new(1);
    let transition = StateTransition::new(LifecycleState::Created, LifecycleState::Initialized);
    let result = mgr.apply_transition(id, &transition);
    assert!(matches!(result, Err(StateError::StateNotFound)));
}

#[test]
fn transition_helper_method_works() {
    let mgr: StateManager<LifecycleState> = StateManager::new();
    let id = StateId::new(1);
    mgr.set(id, LifecycleState::Created);
    mgr.transition(id, LifecycleState::Created, LifecycleState::Running)
        .unwrap();
    assert_eq!(mgr.current(id), Some(LifecycleState::Running));
}

#[test]
fn remove_state_returns_old_value() {
    let mgr: StateManager<LifecycleState> = StateManager::new();
    let id = StateId::new(1);
    mgr.set(id, LifecycleState::Created);
    let removed = mgr.remove(id);
    assert_eq!(removed, Some(LifecycleState::Created));
    assert!(!mgr.contains(id));
}

#[test]
fn state_manager_uses_state_store() {
    let mgr: StateManager<LifecycleState> = StateManager::new();
    let _store: &StateStore<LifecycleState> = mgr.store();

    let id = StateId::new(1);
    mgr.set(id, LifecycleState::Created);
    // Visible through the underlying store
    assert_eq!(mgr.store().read(id), Some(LifecycleState::Created));
}

#[test]
fn multiple_independent_state_slots() {
    let mgr: StateManager<LifecycleState> = StateManager::new();
    let id_a = StateId::new(1);
    let id_b = StateId::new(2);
    mgr.set(id_a, LifecycleState::Created);
    mgr.set(id_b, LifecycleState::Running);
    assert_eq!(mgr.current(id_a), Some(LifecycleState::Created));
    assert_eq!(mgr.current(id_b), Some(LifecycleState::Running));

    // Transition one without affecting the other
    mgr.transition(id_a, LifecycleState::Created, LifecycleState::Initialized)
        .unwrap();
    assert_eq!(mgr.current(id_a), Some(LifecycleState::Initialized));
    assert_eq!(mgr.current(id_b), Some(LifecycleState::Running));
}