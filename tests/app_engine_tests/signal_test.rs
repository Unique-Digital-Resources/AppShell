use app_shell::app_engine::{Signal, SignalId};

#[test]
fn signal_id_is_unique() {
    let a = SignalId::new();
    let b = SignalId::new();
    assert_ne!(a, b);
}

#[test]
fn signal_has_type_and_timestamp() {
    let s = Signal::new("task.progress_changed");
    assert_eq!(s.signal_type(), "task.progress_changed");
    assert!(s.id().value() > 0);
}

#[test]
fn signal_with_payload() {
    let s = Signal::new("state.changed").with_payload("running".to_string());
    assert!(s.has_payload());
    assert_eq!(s.payload::<String>(), Some(&"running".to_string()));
}

#[test]
fn signal_with_source() {
    let s = Signal::new("state.changed").with_source("StateManager");
    assert_eq!(s.source(), Some("StateManager"));
}

#[test]
fn signal_no_payload_by_default() {
    let s = Signal::new("noop");
    assert!(!s.has_payload());
}