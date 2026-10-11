//! `Replay` — iterator over recorded history entries for re-execution.
//!
//! Replay reuses existing Command/Execution mechanisms rather than becoming
//! a second execution engine.

//! `Replay` — iterator over recorded history entries for re-execution.

use crate::app_engine::history::history_entry::HistoryEntryId;

pub struct Replay {
    entries: Vec<(HistoryEntryId, String, bool)>,
    position: usize,
}

impl Replay {
    pub fn new(entries: Vec<(HistoryEntryId, String, bool)>) -> Self {
        Self {
            entries,
            position: 0,
        }
    }

    pub fn remaining(&self) -> usize {
        self.entries.len() - self.position
    }

    pub fn total(&self) -> usize {
        self.entries.len()
    }

    pub fn reset(&mut self) {
        self.position = 0;
    }
}

impl Iterator for Replay {
    type Item = (HistoryEntryId, String, bool);

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.entries.len() {
            let entry = self.entries[self.position].clone();
            self.position += 1;
            Some(entry)
        } else {
            None
        }
    }
}