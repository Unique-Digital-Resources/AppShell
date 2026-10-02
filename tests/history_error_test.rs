use app_shell::app_engine::HistoryError;

#[test]
fn entry_not_found_display() {
    let err = HistoryError::EntryNotFound { id: 42 };
    assert!(err.to_string().contains("42"));
}

#[test]
fn no_undo_available_display() {
    let err = HistoryError::NoUndoAvailable;
    assert_eq!(err.to_string(), "No undo available");
}

#[test]
fn no_redo_available_display() {
    let err = HistoryError::NoRedoAvailable;
    assert_eq!(err.to_string(), "No redo available");
}

#[test]
fn transaction_not_active_display() {
    let err = HistoryError::TransactionNotActive;
    assert!(err.to_string().contains("No active transaction"));
}

#[test]
fn history_error_implements_std_error() {
    fn assert_std_error<E: std::error::Error>(_: E) {}
    assert_std_error(HistoryError::NoUndoAvailable);
}