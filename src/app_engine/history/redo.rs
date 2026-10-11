//! `RedoResult` — result of a redo operation.

use crate::app_engine::history::history_entry::HistoryEntryId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedoResult {
    redone_entry_ids: Vec<HistoryEntryId>,
    transaction_redone: bool,
}

impl RedoResult {
    pub fn empty() -> Self {
        Self {
            redone_entry_ids: Vec::new(),
            transaction_redone: false,
        }
    }

    pub fn new(redone_entry_ids: Vec<HistoryEntryId>, transaction_redone: bool) -> Self {
        Self {
            redone_entry_ids,
            transaction_redone,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.redone_entry_ids.is_empty()
    }

    pub fn count(&self) -> usize {
        self.redone_entry_ids.len()
    }

    pub fn redone_entry_ids(&self) -> &[HistoryEntryId] {
        &self.redone_entry_ids
    }

    pub fn transaction_redone(&self) -> bool {
        self.transaction_redone
    }
}

impl Default for RedoResult {
    fn default() -> Self {
        Self::empty()
    }
}