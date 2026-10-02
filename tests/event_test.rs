use app_shell::app_engine::{Event, EventId};

#[test]
fn event_id_is_unique() {
    let a = EventId::new();
    let b = EventId::new();
    assert_ne!(a, b);
}

#[test]
fn event_has_type_and_timestamp() {
    let e = Event::new("task.completed");
    assert_eq!(e.event_type(), "task.completed");
    assert!(e.id().value() > 0);
}

#[test]
fn event_with_payload() {
    let e = Event::new("task.completed").with_payload(42_i32);
    assert!(e.has_payload());
    assert_eq!(e.payload::<i32>(), Some(&42));
    assert!(e.payload::<String>().is_none());
}

#[test]
fn event_with_source() {
    let e = Event::new("task.completed").with_source("TaskExecutor");
    assert_eq!(e.source(), Some("TaskExecutor"));
}

#[test]
fn event_no_payload_by_default() {
    let e = Event::new("noop");
    assert!(!e.has_payload());
    assert_eq!(e.payload_type_name(), "()");
}