//! Execution context — the environment of one particular execution.
//!
//! Becomes important in later phases when Commands, Tasks, Jobs, and
//! Workflows are introduced. Phase 2 keeps it minimal.

use crate::app_engine::context::context::{ExecutionId, SessionId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionContext {
    id: ExecutionId,
    session_id: SessionId,
}

impl ExecutionContext {
    pub fn new(session_id: SessionId) -> Self {
        Self {
            id: ExecutionId::new(),
            session_id,
        }
    }

    pub fn id(&self) -> ExecutionId {
        self.id
    }

    pub fn session_id(&self) -> SessionId {
        self.session_id
    }
}