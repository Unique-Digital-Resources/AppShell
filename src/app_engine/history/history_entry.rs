//! `HistoryEntry` — one recorded application operation.

use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use crate::app_engine::history::transaction::TransactionId;
use crate::app_engine::tasks::task::TaskId;

static HISTORY_ENTRY_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HistoryEntryId(u64);

impl HistoryEntryId {
    pub fn new() -> Self {
        Self(HISTORY_ENTRY_ID_COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Default for HistoryEntryId {
    fn default() -> Self {
        Self::new()
    }
}

/// A recorded application operation.
///
/// Contains enough information to understand, undo, or replay the operation
/// without duplicating the execution engine.
pub struct HistoryEntry {
    id: HistoryEntryId,
    operation: String,
    command_id: Option<String>,
    task_id: Option<TaskId>,
    source: Option<String>,
    timestamp: Instant,
    data: Option<Box<dyn Any + Send>>,
    data_type_name: &'static str,
    inverse_data: Option<Box<dyn Any + Send>>,
    inverse_type_name: &'static str,
    undoable: bool,
    transaction_id: Option<TransactionId>,
}

impl HistoryEntry {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            id: HistoryEntryId::new(),
            operation: operation.into(),
            command_id: None,
            task_id: None,
            source: None,
            timestamp: Instant::now(),
            data: None,
            data_type_name: "()",
            inverse_data: None,
            inverse_type_name: "()",
            undoable: false,
            transaction_id: None,
        }
    }

    pub fn with_data<T: Any + Send>(mut self, data: T) -> Self {
        self.data_type_name = std::any::type_name::<T>();
        self.data = Some(Box::new(data));
        self
    }

    pub fn with_inverse<T: Any + Send>(mut self, data: T) -> Self {
        self.inverse_type_name = std::any::type_name::<T>();
        self.inverse_data = Some(Box::new(data));
        self
    }

    pub fn with_command_id(mut self, id: impl Into<String>) -> Self {
        self.command_id = Some(id.into());
        self
    }

    pub fn with_task_id(mut self, id: TaskId) -> Self {
        self.task_id = Some(id);
        self
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn undoable(mut self) -> Self {
        self.undoable = true;
        self
    }

    pub fn non_undoable(mut self) -> Self {
        self.undoable = false;
        self
    }

    pub fn with_transaction(mut self, id: TransactionId) -> Self {
        self.transaction_id = Some(id);
        self
    }

    pub fn id(&self) -> HistoryEntryId {
        self.id
    }

    pub fn operation(&self) -> &str {
        &self.operation
    }

    pub fn command_id(&self) -> Option<&str> {
        self.command_id.as_deref()
    }

    pub fn task_id(&self) -> Option<TaskId> {
        self.task_id
    }

    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    pub fn timestamp(&self) -> Instant {
        self.timestamp
    }

    pub fn data<T: Any + Send>(&self) -> Option<&T> {
        self.data.as_ref().and_then(|d| d.downcast_ref::<T>())
    }

    pub fn has_data(&self) -> bool {
        self.data.is_some()
    }

    pub fn data_type_name(&self) -> &'static str {
        self.data_type_name
    }

    pub fn inverse_data<T: Any + Send>(&self) -> Option<&T> {
        self.inverse_data
            .as_ref()
            .and_then(|d| d.downcast_ref::<T>())
    }

    pub fn has_inverse_data(&self) -> bool {
        self.inverse_data.is_some()
    }

    pub fn inverse_type_name(&self) -> &'static str {
        self.inverse_type_name
    }

    pub fn is_undoable(&self) -> bool {
        self.undoable
    }

    pub fn transaction_id(&self) -> Option<TransactionId> {
        self.transaction_id
    }
}

impl std::fmt::Debug for HistoryEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HistoryEntry")
            .field("id", &self.id)
            .field("operation", &self.operation)
            .field("command_id", &self.command_id)
            .field("source", &self.source)
            .field("undoable", &self.undoable)
            .field("transaction_id", &self.transaction_id)
            .field("data_type", &self.data_type_name)
            .field("inverse_type", &self.inverse_type_name)
            .finish()
    }
}