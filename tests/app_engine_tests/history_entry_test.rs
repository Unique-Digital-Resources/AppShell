use app_shell::app_engine::{HistoryEntry, TaskId, TransactionId};

#[test]
fn entry_has_unique_id() {
    let a = HistoryEntry::new("object.move");
    let b = HistoryEntry::new("object.move");
    assert_ne!(a.id(), b.id());
}

#[test]
fn entry_stores_operation() {
    let e = HistoryEntry::new("object.create");
    assert_eq!(e.operation(), "object.create");
}

#[test]
fn entry_with_data() {
    let e = HistoryEntry::new("object.move").with_data((100.0, 200.0));
    assert!(e.has_data());
    assert_eq!(e.data::<(f64, f64)>(), Some(&(100.0, 200.0)));
}

#[test]
fn entry_with_inverse_data() {
    let e = HistoryEntry::new("object.move")
        .with_data((100.0, 200.0))
        .with_inverse((50.0, 50.0));
    assert!(e.has_inverse_data());
    assert_eq!(e.inverse_data::<(f64, f64)>(), Some(&(50.0, 50.0)));
}

#[test]
fn entry_undoable_by_default_is_false() {
    let e = HistoryEntry::new("object.move");
    assert!(!e.is_undoable());
}

#[test]
fn entry_can_be_marked_undoable() {
    let e = HistoryEntry::new("object.move").undoable();
    assert!(e.is_undoable());
}

#[test]
fn entry_with_command_and_task() {
    let task_id = TaskId::new();
    let e = HistoryEntry::new("object.move")
        .with_command_id("object.move_command")
        .with_task_id(task_id);
    assert_eq!(e.command_id(), Some("object.move_command"));
    assert_eq!(e.task_id(), Some(task_id));
}

#[test]
fn entry_with_transaction() {
    let tx_id = TransactionId::new();
    let e = HistoryEntry::new("object.move").with_transaction(tx_id);
    assert_eq!(e.transaction_id(), Some(tx_id));
}

#[test]
fn entry_with_source() {
    let e = HistoryEntry::new("object.move").with_source("MoveObjectCommand");
    assert_eq!(e.source(), Some("MoveObjectCommand"));
}