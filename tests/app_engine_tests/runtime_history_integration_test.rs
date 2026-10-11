use app_shell::app_engine::{
    Bootstrap, HistoryEntry, HistoryStore, Lifecycle, TransactionManager,
};

#[test]
fn runtime_owns_history_store() {
    let runtime = Bootstrap::create().unwrap();
    let _: &HistoryStore = runtime.history_store();
}

#[test]
fn runtime_owns_transaction_manager() {
    let runtime = Bootstrap::create().unwrap();
    let _: &TransactionManager = runtime.transaction_manager();
}

#[test]
fn runtime_history_starts_empty() {
    let runtime = Bootstrap::create().unwrap();
    assert!(runtime.history_store().is_empty());
}

#[test]
fn runtime_transaction_manager_starts_inactive() {
    let runtime = Bootstrap::create().unwrap();
    assert!(!runtime.transaction_manager().is_active());
}

#[test]
fn runtime_can_record_and_undo() {
    let mut runtime = Bootstrap::create().unwrap();

    runtime
        .history_store_mut()
        .append(HistoryEntry::new("object.create").undoable());
    runtime
        .history_store_mut()
        .append(HistoryEntry::new("object.move").undoable());

    assert!(runtime.history_store().can_undo());

    let result = runtime.history_store_mut().undo();
    assert_eq!(result.count(), 1);

    let result = runtime.history_store_mut().redo();
    assert_eq!(result.count(), 1);
}

#[test]
fn runtime_can_use_transactions() {
    let mut runtime = Bootstrap::create().unwrap();

    let tx = runtime.transaction_manager_mut().begin();
    runtime
        .history_store_mut()
        .append(HistoryEntry::new("op.a").with_transaction(tx));
    runtime
        .history_store_mut()
        .append(HistoryEntry::new("op.b").with_transaction(tx));
    runtime.transaction_manager_mut().commit();

    // Undo the whole transaction
    let result = runtime.history_store_mut().undo();
    assert_eq!(result.count(), 2);
    assert!(result.transaction_undone());
}

#[test]
fn history_independent_of_lifecycle() {
    let mut runtime = Bootstrap::create().unwrap();
    runtime.initialize().unwrap();
    runtime.start().unwrap();
    assert!(runtime.history_store().is_empty());
    runtime.stop().unwrap();
    runtime.dispose().unwrap();
}