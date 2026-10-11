//! Tests for `StateId`.

use app_shell::app_engine::StateId;

#[test]
fn state_id_holds_value() {
    let id = StateId::new(42);
    assert_eq!(id.value(), 42);
}

#[test]
fn state_id_equality() {
    let a = StateId::new(1);
    let b = StateId::new(1);
    let c = StateId::new(2);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn state_id_is_copyable() {
    let id = StateId::new(7);
    let copied = id;
    assert_eq!(id, copied);
}

#[test]
fn state_id_from_u64() {
    let id: StateId = 5u64.into();
    assert_eq!(id.value(), 5);
}

#[test]
fn state_id_orders_naturally() {
    let small = StateId::new(1);
    let big = StateId::new(100);
    assert!(small < big);
}