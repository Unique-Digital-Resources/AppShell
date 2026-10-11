//! `UndoResult` — result of an undo operation.

use crate::app_engine::history::history_entry::HistoryEntryId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndoResult {
    undone_entry_ids: Vec<HistoryEntryId>,
    transaction_undone: bool,
}

impl UndoResult {
    pub fn empty() -> Self {
        Self {
            undone_entry_ids: Vec::new(),
            transaction_undone: false,
        }
    }

    pub fn new(undone_entry_ids: Vec<HistoryEntryId>, transaction_undone: bool) -> Self {
        Self {
            undone_entry_ids,
            transaction_undone,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.undone_entry_ids.is_empty()
    }

    pub fn count(&self) -> usize {
        self.undone_entry_ids.len()
    }

    pub fn undone_entry_ids(&self) -> &[HistoryEntryId] {
        &self.undone_entry_ids
    }

    pub fn transaction_undone(&self) -> bool {
        self.transaction_undone
    }
}

impl Default for UndoResult {
    fn default() -> Self {
        Self::empty()
    }
}