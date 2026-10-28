//! `CommandContext` — the execution environment for a command.
//!
//! Phase 2 established `ApplicationContext`, `SessionContext`, and
//! `ExecutionContext`. `CommandContext` references those by ID rather than
//! duplicating them.
//!
//! ✓ execution ID, session ID, application ID, generic metadata
//! ✗ current document, selected card, active scene (those belong to the domain)

//! `CommandContext` — execution environment for a command.

use std::collections::HashMap;

use crate::app_engine::context::context::{ApplicationId, ExecutionId, SessionId};
use crate::app_engine::execution::deadline::Deadline;

#[derive(Debug, Clone)]
pub struct CommandContext {
    application_id: Option<ApplicationId>,
    session_id: Option<SessionId>,
    execution_id: ExecutionId,
    metadata: HashMap<String, String>,
    deadline: Deadline,
}

impl CommandContext {
    pub fn new(execution_id: ExecutionId) -> Self {
        Self {
            application_id: None,
            session_id: None,
            execution_id,
            metadata: HashMap::new(),
            deadline: Deadline::none(),
        }
    }

    pub fn with_application(mut self, id: ApplicationId) -> Self {
        self.application_id = Some(id);
        self
    }

    pub fn with_session(mut self, id: SessionId) -> Self {
        self.session_id = Some(id);
        self
    }

    pub fn with_metadata(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set a deadline for this command.
    pub fn with_deadline(mut self, deadline: Deadline) -> Self {
        self.deadline = deadline;
        self
    }

    pub fn application_id(&self) -> Option<ApplicationId> {
        self.application_id
    }

    pub fn session_id(&self) -> Option<SessionId> {
        self.session_id
    }

    pub fn execution_id(&self) -> ExecutionId {
        self.execution_id
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    // ----- Phase 14: Deadline -----

    /// Check if the deadline has passed.
    pub fn is_expired(&self) -> bool {
        self.deadline.is_expired()
    }

    /// Get the deadline.
    pub fn deadline(&self) -> &Deadline {
        &self.deadline
    }
}