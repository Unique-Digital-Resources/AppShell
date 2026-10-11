//! Tests for `StateTransition`.

use app_shell::app_engine::{LifecycleState, StateTransition};

#[test]
fn transition_holds_from_and_to() {
    let t = StateTransition::new(LifecycleState::Created, LifecycleState::Initialized);
    assert_eq!(t.from(), &LifecycleState::Created);
    assert_eq!(t.to(), &LifecycleState::Initialized);
}

#[test]
fn transition_no_op_detected() {
    let t = StateTransition::new(LifecycleState::Running, LifecycleState::Running);
    assert!(t.is_no_op());
}

#[test]
fn transition_not_no_op_when_different() {
    let t = StateTransition::new(LifecycleState::Created, LifecycleState::Initialized);
    assert!(!t.is_no_op());
}

#[test]
fn transition_equality() {
    let t1 = StateTransition::new(LifecycleState::Created, LifecycleState::Initialized);
    let t2 = StateTransition::new(LifecycleState::Created, LifecycleState::Initialized);
    let t3 = StateTransition::new(LifecycleState::Created, LifecycleState::Running);
    assert_eq!(t1, t2);
    assert_ne!(t1, t3);
}

#[test]
fn transition_is_cloneable() {
    let t1 = StateTransition::new(LifecycleState::Created, LifecycleState::Initialized);
    let t2 = t1.clone();
    assert_eq!(t1, t2);
}