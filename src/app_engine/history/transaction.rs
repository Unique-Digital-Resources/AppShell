//! `TransactionId` and `TransactionManager` — grouping of multiple operations
//! into one logical undo/redo unit.

use std::sync::atomic::{AtomicU64, Ordering};

static TRANSACTION_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TransactionId(u64);

impl TransactionId {
    pub fn new() -> Self {
        Self(TRANSACTION_ID_COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Default for TransactionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TransactionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Transaction({})", self.0)
    }
}

/// Manages the current transaction state.
///
/// ```text
/// Begin → Operations → Commit (keep entries)
/// Begin → Operations → Rollback (discard entries)
/// ```
pub struct TransactionManager {
    current: Option<TransactionId>,
}

impl TransactionManager {
    pub fn new() -> Self {
        Self { current: None }
    }

    /// Begin a new transaction. Returns the transaction ID.
    pub fn begin(&mut self) -> TransactionId {
        let id = TransactionId::new();
        self.current = Some(id);
        id
    }

    /// Commit the current transaction. Returns the committed transaction ID.
    pub fn commit(&mut self) -> Option<TransactionId> {
        self.current.take()
    }

    /// Rollback the current transaction. Returns the rolled-back transaction ID.
    /// The caller is responsible for removing rolled-back entries from the store.
    pub fn rollback(&mut self) -> Option<TransactionId> {
        self.current.take()
    }

    pub fn current(&self) -> Option<TransactionId> {
        self.current
    }

    pub fn is_active(&self) -> bool {
        self.current.is_some()
    }
}

impl Default for TransactionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for TransactionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransactionManager")
            .field("active", &self.current.is_some())
            .finish()
    }
}