use app_shell::app_engine::{HistoryEntry, HistoryStore, TransactionId};

#[test]
fn new_store_is_empty() {
    let s = HistoryStore::new();
    assert!(s.is_empty());
    assert_eq!(s.entry_count(), 0);
    assert_eq!(s.active_count(), 0);
    assert!(!s.can_undo());
    assert!(!s.can_redo());
}

#[test]
fn append_increases_count() {
    let s = HistoryStore::new();
    s.append(HistoryEntry::new("op.a"));
    s.append(HistoryEntry::new("op.b"));
    assert_eq!(s.entry_count(), 2);
    assert_eq!(s.active_count(), 2);
    assert!(s.can_undo());
    assert!(!s.can_redo());
}

#[test]
fn undo_moves_position_back() {
    let s = HistoryStore::new();
    s.append(HistoryEntry::new("op.a"));
    s.append(HistoryEntry::new("op.b"));

    let result = s.undo();
    assert_eq!(result.count(), 1);
    assert!(!result.is_empty());
    assert_eq!(s.active_count(), 1);
    assert!(s.can_undo());
    assert!(s.can_redo());
}

#[test]
fn redo_moves_position_forward() {
    let s = HistoryStore::new();
    s.append(HistoryEntry::new("op.a"));
    s.append(HistoryEntry::new("op.b"));

    s.undo();
    let result = s.redo();
    assert_eq!(result.count(), 1);
    assert_eq!(s.active_count(), 2);
    assert!(s.can_undo());
    assert!(!s.can_redo());
}

#[test]
fn append_after_undo_invalidates_redo_branch() {
    let s = HistoryStore::new();
    s.append(HistoryEntry::new("op.a"));
    s.append(HistoryEntry::new("op.b"));
    s.append(HistoryEntry::new("op.c"));

    s.undo();
    s.undo();
    assert!(s.can_redo());

    s.append(HistoryEntry::new("op.d"));
    assert!(!s.can_redo());
    assert_eq!(s.entry_count(), 2);
}

#[test]
fn undo_at_beginning_returns_empty() {
    let s = HistoryStore::new();
    let result = s.undo();
    assert!(result.is_empty());
}

#[test]
fn redo_at_end_returns_empty() {
    let s = HistoryStore::new();
    s.append(HistoryEntry::new("op.a"));
    let result = s.redo();
    assert!(result.is_empty());
}

#[test]
fn current_returns_last_active() {
    let s = HistoryStore::new();
    s.append(HistoryEntry::new("op.a"));
    s.append(HistoryEntry::new("op.b"));
    assert_eq!(s.current().unwrap().1, "op.b");

    s.undo();
    assert_eq!(s.current().unwrap().1, "op.a");
}

#[test]
fn get_by_id() {
    let s = HistoryStore::new();
    let id = s.append(HistoryEntry::new("op.a"));
    let result = s.get(id);
    assert!(result.is_some());
    assert_eq!(result.unwrap().1, "op.a");
}

#[test]
fn clear_removes_all() {
    let s = HistoryStore::new();
    s.append(HistoryEntry::new("op.a"));
    s.append(HistoryEntry::new("op.b"));
    s.clear();
    assert!(s.is_empty());
    assert_eq!(s.position(), 0);
}

#[test]
fn undo_transaction_undoes_all_in_group() {
    let s = HistoryStore::new();
    let tx = TransactionId::new();

    s.append(HistoryEntry::new("op.a"));
    s.append(HistoryEntry::new("op.b1").with_transaction(tx));
    s.append(HistoryEntry::new("op.b2").with_transaction(tx));
    s.append(HistoryEntry::new("op.b3").with_transaction(tx));
    s.append(HistoryEntry::new("op.c"));

    assert_eq!(s.active_count(), 5);

    let r = s.undo();
    assert_eq!(r.count(), 1);
    assert!(!r.transaction_undone());
    assert_eq!(s.active_count(), 4);

    let r = s.undo();
    assert_eq!(r.count(), 3);
    assert!(r.transaction_undone());
    assert_eq!(s.active_count(), 1);
}

#[test]
fn redo_transaction_redoes_all_in_group() {
    let s = HistoryStore::new();
    let tx = TransactionId::new();

    s.append(HistoryEntry::new("op.b1").with_transaction(tx));
    s.append(HistoryEntry::new("op.b2").with_transaction(tx));

    let r = s.undo();
    assert_eq!(r.count(), 2);
    assert_eq!(s.active_count(), 0);

    let r = s.redo();
    assert_eq!(r.count(), 2);
    assert!(r.transaction_redone());
    assert_eq!(s.active_count(), 2);
}

#[test]
fn remove_transaction_removes_entries() {
    let s = HistoryStore::new();
    let tx = TransactionId::new();

    s.append(HistoryEntry::new("op.a"));
    s.append(HistoryEntry::new("op.b1").with_transaction(tx));
    s.append(HistoryEntry::new("op.b2").with_transaction(tx));
    s.append(HistoryEntry::new("op.c"));

    s.remove_transaction(tx);
    assert_eq!(s.entry_count(), 2);
}

#[test]
fn replay_returns_active_entries() {
    let s = HistoryStore::new();
    s.append(HistoryEntry::new("op.a"));
    s.append(HistoryEntry::new("op.b"));
    s.append(HistoryEntry::new("op.c"));

    let replay = s.replay();
    assert_eq!(replay.total(), 3);
    assert_eq!(replay.remaining(), 3);
}

#[test]
fn replay_excludes_undone_entries() {
    let s = HistoryStore::new();
    s.append(HistoryEntry::new("op.a"));
    s.append(HistoryEntry::new("op.b"));
    s.append(HistoryEntry::new("op.c"));

    s.undo();
    let replay = s.replay();
    assert_eq!(replay.total(), 2);
}

#[test]
fn replay_is_iterable() {
    let s = HistoryStore::new();
    s.append(HistoryEntry::new("op.a"));
    s.append(HistoryEntry::new("op.b"));
    s.append(HistoryEntry::new("op.c"));

    let ops: Vec<String> = s.replay().map(|(_, op, _)| op).collect();
    assert_eq!(ops, vec!["op.a", "op.b", "op.c"]);
}