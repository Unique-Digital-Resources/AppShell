//! Base context abstraction and typed context identifiers.
//!
//! All context types share a common identity concept via typed IDs.
//! The `Context` struct is a minimal base — kept concrete rather than trait-based
//! to avoid premature abstraction.

use std::sync::atomic::{AtomicU64, Ordering};

static CONTEXT_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

fn next_context_id() -> u64 {
    CONTEXT_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ApplicationId(u64);

impl ApplicationId {
    pub fn new() -> Self {
        Self(next_context_id())
    }
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Default for ApplicationId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SessionId(u64);

impl SessionId {
    pub fn new() -> Self {
        Self(next_context_id())
    }
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ExecutionId(u64);

impl ExecutionId {
    pub fn new() -> Self {
        Self(next_context_id())
    }
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Default for ExecutionId {
    fn default() -> Self {
        Self::new()
    }
}

/// Base context abstraction. Holds identity and optional parent relationship.
///
/// This is intentionally minimal. Specialized contexts (`ApplicationContext`,
/// `SessionContext`, `ExecutionContext`) carry their own typed IDs and parent
/// references; `Context` exists to establish the shared concept.
#[derive(Debug, Clone)]
pub struct Context {
    id: u64,
    parent: Option<u64>,
}

impl Context {
    /// Create a root context with an auto-generated id and no parent.
    /// Backward-compatible with Phase 1.
    pub fn new() -> Self {
        Self::new_root(next_context_id())
    }

    pub fn new_root(id: u64) -> Self {
        Self { id, parent: None }
    }

    pub fn new_with_parent(id: u64, parent: u64) -> Self {
        Self {
            id,
            parent: Some(parent),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn parent(&self) -> Option<u64> {
        self.parent
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}