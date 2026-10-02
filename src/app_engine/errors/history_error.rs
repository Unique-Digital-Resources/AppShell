//! Errors raised by the history system.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoryError {
    EntryNotFound { id: u64 },
    NoUndoAvailable,
    NoRedoAvailable,
    TransactionNotActive,
    TransactionAlreadyActive,
    ReplayExhausted,
}

impl fmt::Display for HistoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HistoryError::EntryNotFound { id } => write!(f, "History entry not found: {}", id),
            HistoryError::NoUndoAvailable => write!(f, "No undo available"),
            HistoryError::NoRedoAvailable => write!(f, "No redo available"),
            HistoryError::TransactionNotActive => write!(f, "No active transaction"),
            HistoryError::TransactionAlreadyActive => {
                write!(f, "Transaction already active")
            }
            HistoryError::ReplayExhausted => write!(f, "Replay entries exhausted"),
        }
    }
}

impl std::error::Error for HistoryError {}