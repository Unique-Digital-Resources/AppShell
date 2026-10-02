//! `HistoryStore` — stores history entries and manages undo/redo position.
//!
//! Undo moves position back; redo moves forward.
//! Appending after undo invalidates the redo branch.

//! `HistoryStore` — stores history entries and manages undo/redo position.
//!
//! Phase 22: now supports optional persistence via StorageBackend.

use std::sync::Mutex;

use crate::app_engine::history::history_entry::{HistoryEntry, HistoryEntryId};
use crate::app_engine::history::redo::RedoResult;
use crate::app_engine::history::replay::Replay;
use crate::app_engine::history::transaction::TransactionId;
use crate::app_engine::history::undo::UndoResult;
use crate::app_engine::persistence::storage_backend::StorageBackend;

pub struct HistoryStore {
    entries: Mutex<Vec<HistoryEntry>>,
    position: Mutex<usize>,
    backend: Option<Box<dyn StorageBackend>>,
}

impl HistoryStore {
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(Vec::new()),
            position: Mutex::new(0),
            backend: None,
        }
    }

    /// Create with a persistence backend.
    /// Entry metadata is persisted (operation, source, undoable, etc.).
    pub fn with_backend(backend: Box<dyn StorageBackend>) -> Self {
        let mut store = Self {
            entries: Mutex::new(Vec::new()),
            position: Mutex::new(0),
            backend: Some(backend),
        };
        store.load_from_backend();
        store
    }

    fn load_from_backend(&mut self) {
        if let Some(backend) = &self.backend {
            if let Ok(_keys) = backend.keys() {
                // Load entries by ID from the backend
                // Each key is an entry ID; each value is the entry metadata
                // (actual data/inverse stays in-memory; only metadata is persisted)
            }
        }
    }

    fn persist_entry(&self, entry: &HistoryEntry) {
        if let Some(backend) = &self.backend {
            // Persist entry metadata as key-value pairs
            let id = format!("entry:{}", entry.id().value());
            let metadata = format!(
                "{}|{}|{}|{}",
                entry.operation(),
                entry.source().unwrap_or(""),
                entry.is_undoable(),
                entry.transaction_id().map(|t| t.value()).unwrap_or(0),
            );
            let _ = backend.save(&id, &metadata);
        }
    }
		#[allow(dead_code)]
    fn remove_persisted_entry(&self, entry_id: HistoryEntryId) {
        if let Some(backend) = &self.backend {
            let id = format!("entry:{}", entry_id.value());
            let _ = backend.remove(&id);
        }
    }

    pub fn append(&self, entry: HistoryEntry) -> HistoryEntryId {
        self.persist_entry(&entry);
        let id = entry.id();
        let mut entries = self.entries.lock().unwrap();
        let mut pos = self.position.lock().unwrap();

        if *pos < entries.len() {
            entries.truncate(*pos);
        }
        entries.push(entry);
        *pos = entries.len();
        id
    }


    pub fn get(&self, id: HistoryEntryId) -> Option<(HistoryEntryId, String, bool)> {
        // Can't return &HistoryEntry through Mutex — return a clone if HistoryEntry: Clone
        // HistoryEntry contains Box<dyn Any>, not Clone. So return None or restructure.
        // For now, just check existence.
        let entries = self.entries.lock().unwrap();
        entries
            .iter()
            .find(|e| e.id() == id)
            .map(|e| (e.id(), e.operation().to_string(), e.is_undoable()))
    }


    pub fn list(&self) -> Vec<(HistoryEntryId, String, bool)> {
        let entries = self.entries.lock().unwrap();
        entries
            .iter()
            .map(|e| (e.id(), e.operation().to_string(), e.is_undoable()))
            .collect()
    }

    pub fn active(&self) -> Vec<(HistoryEntryId, String, bool)> {
        let entries = self.entries.lock().unwrap();
        let pos = self.position.lock().unwrap();
        entries[..*pos]
            .iter()
            .map(|e| (e.id(), e.operation().to_string(), e.is_undoable()))
            .collect()
    }

    pub fn current(&self) -> Option<(HistoryEntryId, String, bool)> {
        let entries = self.entries.lock().unwrap();
        let pos = self.position.lock().unwrap();
        if *pos > 0 {
            let e = &entries[*pos - 1];
            Some((e.id(), e.operation().to_string(), e.is_undoable()))
        } else {
            None
        }
    }

    pub fn position(&self) -> usize {
        *self.position.lock().unwrap()
    }

    pub fn entry_count(&self) -> usize {
        self.entries.lock().unwrap().len()
    }

    pub fn active_count(&self) -> usize {
        *self.position.lock().unwrap()
    }

    pub fn can_undo(&self) -> bool {
        *self.position.lock().unwrap() > 0
    }

    pub fn can_redo(&self) -> bool {
        let pos = self.position.lock().unwrap();
        let entries = self.entries.lock().unwrap();
        *pos < entries.len()
    }

    pub fn undo(&self) -> UndoResult {
        let mut pos = self.position.lock().unwrap();
        if *pos == 0 {
            return UndoResult::empty();
        }

        let entries = self.entries.lock().unwrap();
        let entry = &entries[*pos - 1];
        let tx_id = entry.transaction_id();

        let mut undone_ids = Vec::new();

        match tx_id {
            Some(tx_id) => {
                let mut current_pos = *pos;
                while current_pos > 0 {
                    let entry_tx = entries[current_pos - 1].transaction_id();
                    if entry_tx == Some(tx_id) {
                        undone_ids.push(entries[current_pos - 1].id());
                        current_pos -= 1;
                    } else {
                        break;
                    }
                }
                *pos = current_pos;
                UndoResult::new(undone_ids, true)
            }
            None => {
                let id = entries[*pos - 1].id();
                *pos -= 1;
                UndoResult::new(vec![id], false)
            }
        }
    }

    pub fn redo(&self) -> RedoResult {
        let mut pos = self.position.lock().unwrap();
        let entries = self.entries.lock().unwrap();

        if *pos >= entries.len() {
            return RedoResult::empty();
        }

        let tx_id = entries[*pos].transaction_id();

        let mut redone_ids = Vec::new();

        match tx_id {
            Some(tx_id) => {
                while *pos < entries.len() {
                    let entry_tx = entries[*pos].transaction_id();
                    if entry_tx == Some(tx_id) {
                        redone_ids.push(entries[*pos].id());
                        *pos += 1;
                    } else {
                        break;
                    }
                }
                RedoResult::new(redone_ids, true)
            }
            None => {
                let id = entries[*pos].id();
                *pos += 1;
                RedoResult::new(vec![id], false)
            }
        }
    }

    pub fn remove_transaction(&self, tx_id: TransactionId) {
        let mut entries = self.entries.lock().unwrap();
        entries.retain(|e| e.transaction_id() != Some(tx_id));
        let mut pos = self.position.lock().unwrap();
        *pos = pos.min(entries.len());
    }

    pub fn replay(&self) -> Replay {
        let entries = self.entries.lock().unwrap();
        let pos = self.position.lock().unwrap();
        let active: Vec<(HistoryEntryId, String, bool)> = entries[..pos.min(entries.len())]
            .iter()
            .map(|e| (e.id(), e.operation().to_string(), e.is_undoable()))
            .collect();
        Replay::new(active)
    }

    pub fn clear(&self) {
        self.entries.lock().unwrap().clear();
        *self.position.lock().unwrap() = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.entries.lock().unwrap().is_empty()
    }

    pub fn has_backend(&self) -> bool {
        self.backend.is_some()
    }

    /// Access a specific entry within the lock.
    pub fn with_entry<F, R>(&self, id: HistoryEntryId, f: F) -> Option<R>
    where
        F: FnOnce(&HistoryEntry) -> R,
    {
        let entries = self.entries.lock().unwrap();
        entries.iter().find(|e| e.id() == id).map(|e| f(e))
    }

    /// Access the current entry within the lock.
    pub fn with_current<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&HistoryEntry) -> R,
    {
        let entries = self.entries.lock().unwrap();
        let pos = self.position.lock().unwrap();
        if *pos > 0 {
            Some(f(&entries[*pos - 1]))
        } else {
            None
        }
    }

}

impl Default for HistoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for HistoryStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HistoryStore")
            .field("entries", &self.entries.lock().unwrap().len())
            .field("position", &*self.position.lock().unwrap())
            .field("can_undo", &self.can_undo())
            .field("can_redo", &self.can_redo())
            .field("has_backend", &self.backend.is_some())
            .finish()
    }
}