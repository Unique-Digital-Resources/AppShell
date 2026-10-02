//! Tests for `StateStore`.

use app_shell::app_engine::{LifecycleState, StateId, StateStore};

#[test]
fn store_starts_empty() {
    let store: StateStore<LifecycleState> = StateStore::new();
    assert!(store.is_empty());
    assert_eq!(store.len(), 0);
}

#[test]
fn write_and_read_state() {
    let store: StateStore<LifecycleState> = StateStore::new();
    let id = StateId::new(1);
    store.write(id, LifecycleState::Created);
    assert_eq!(store.read(id), Some(LifecycleState::Created));
    assert_eq!(store.len(), 1);
    assert!(!store.is_empty());
}

#[test]
fn read_missing_returns_none() {
    let store: StateStore<LifecycleState> = StateStore::new();
    assert_eq!(store.read(StateId::new(99)), None);
}

#[test]
fn write_overwrites_existing() {
    let store: StateStore<LifecycleState> = StateStore::new();
    let id = StateId::new(1);
    store.write(id, LifecycleState::Created);
    store.write(id, LifecycleState::Running);
    assert_eq!(store.read(id), Some(LifecycleState::Running));
    assert_eq!(store.len(), 1);
}

#[test]
fn remove_state_returns_old_value() {
    let store: StateStore<LifecycleState> = StateStore::new();
    let id = StateId::new(1);
    store.write(id, LifecycleState::Created);
    let removed = store.remove(id);
    assert_eq!(removed, Some(LifecycleState::Created));
    assert!(!store.contains(id));
    assert_eq!(store.len(), 0);
}

#[test]
fn remove_missing_returns_none() {
    let store: StateStore<LifecycleState> = StateStore::new();
    assert_eq!(store.remove(StateId::new(99)), None);
}

#[test]
fn contains_check() {
    let store: StateStore<LifecycleState> = StateStore::new();
    let id = StateId::new(1);
    assert!(!store.contains(id));
    store.write(id, LifecycleState::Created);
    assert!(store.contains(id));
}

#[test]
fn clear_removes_all() {
    let store: StateStore<LifecycleState> = StateStore::new();
    store.write(StateId::new(1), LifecycleState::Created);
    store.write(StateId::new(2), LifecycleState::Running);
    store.clear();
    assert!(store.is_empty());
}

#[test]
fn multiple_independent_slots() {
    let store: StateStore<LifecycleState> = StateStore::new();
    store.write(StateId::new(1), LifecycleState::Created);
    store.write(StateId::new(2), LifecycleState::Running);
    store.write(StateId::new(3), LifecycleState::Stopped);
    assert_eq!(store.len(), 3);
    assert_eq!(store.read(StateId::new(2)), Some(LifecycleState::Running));
}