//! `TaskContext` — execution environment for a task.
//!
//! Phase 14: now includes cancellation token, deadline, and output channel.

use std::collections::HashMap;

use crate::app_engine::context::context::{ApplicationId, ExecutionId, SessionId};
use crate::app_engine::execution::cancellation_token::CancellationToken;
use crate::app_engine::execution::deadline::Deadline;
use crate::app_engine::execution::output_channel::TaskOutputChannel;
use crate::app_engine::tasks::task::TaskId;

#[derive(Debug, Clone)]
pub struct TaskContext {
    application_id: Option<ApplicationId>,
    session_id: Option<SessionId>,
    execution_id: ExecutionId,
    task_id: Option<TaskId>,
    metadata: HashMap<String, String>,
    cancellation_token: CancellationToken,
    deadline: Deadline,
    output_channel: TaskOutputChannel,
}

impl TaskContext {
    pub fn new(execution_id: ExecutionId) -> Self {
        let task_id_val = 0; // Will be updated by TaskManager
        Self {
            application_id: None,
            session_id: None,
            execution_id,
            task_id: None,
            metadata: HashMap::new(),
            cancellation_token: CancellationToken::new(),
            deadline: Deadline::none(),
            output_channel: TaskOutputChannel::new(task_id_val),
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

    /// Set a deadline for this task.
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

    pub fn task_id(&self) -> Option<TaskId> {
        self.task_id
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    // ----- Phase 14: Execution Control -----

    /// Check if cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.cancellation_token.is_cancelled()
    }

    /// Get the cancellation token (for the TaskManager to call `cancel()`).
    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancellation_token.clone()
    }

    /// Check if the deadline has passed.
    pub fn is_expired(&self) -> bool {
        self.deadline.is_expired()
    }

    /// Get the deadline.
    pub fn deadline(&self) -> &Deadline {
        &self.deadline
    }

    /// Get the output channel for progress reporting and streaming.
    pub fn output(&self) -> &TaskOutputChannel {
        &self.output_channel
    }

    // ----- Internal setters (used by TaskManager) -----

    pub(crate) fn set_task_id(&mut self, id: TaskId) {
        self.task_id = Some(id);
    }

    pub(crate) fn set_cancellation_token(&mut self, token: CancellationToken) {
        self.cancellation_token = token;
    }

    pub(crate) fn set_output_channel(&mut self, channel: TaskOutputChannel) {
        self.output_channel = channel;
    }
}