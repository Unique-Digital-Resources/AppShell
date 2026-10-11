//! Creates, tracks and resolves contexts.
//!
//! Manages **context identity/lifetime only** — it is not a general-purpose
//! state manager.

use std::collections::HashMap;

use crate::app_engine::context::application_context::ApplicationContext;
use crate::app_engine::context::context::{ApplicationId, ExecutionId, SessionId};
use crate::app_engine::context::execution_context::ExecutionContext;
use crate::app_engine::context::session_context::SessionContext;
use crate::app_engine::errors::context_error::ContextError;

#[derive(Debug)]
pub struct ContextManager {
    application: Option<ApplicationContext>,
    sessions: HashMap<SessionId, SessionContext>,
    executions: HashMap<ExecutionId, ExecutionContext>,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            application: None,
            sessions: HashMap::new(),
            executions: HashMap::new(),
        }
    }

    /// Create a new application context (replaces any existing one).
    pub fn create_application_context(
        &mut self,
        application_id: impl Into<String>,
    ) -> ApplicationId {
        let ctx = ApplicationContext::new(application_id);
        let id = ctx.id();
        self.application = Some(ctx);
        id
    }

    /// Inject an externally-constructed application context.
    pub fn set_application_context(&mut self, context: ApplicationContext) -> ApplicationId {
        let id = context.id();
        self.application = Some(context);
        id
    }

    /// Create a new session under the current application.
    pub fn create_session_context(&mut self) -> Result<SessionId, ContextError> {
        let app_id = self
            .application
            .as_ref()
            .ok_or(ContextError::NoApplicationContext)?
            .id();
        let session = SessionContext::new(app_id);
        let id = session.id();
        self.sessions.insert(id, session);
        Ok(id)
    }

    /// Create a new execution under a session.
    pub fn create_execution_context(
        &mut self,
        session_id: SessionId,
    ) -> Result<ExecutionId, ContextError> {
        if !self.sessions.contains_key(&session_id) {
            return Err(ContextError::SessionNotFound);
        }
        let exec = ExecutionContext::new(session_id);
        let id = exec.id();
        self.executions.insert(id, exec);
        Ok(id)
    }

    pub fn application_context(&self) -> Option<&ApplicationContext> {
        self.application.as_ref()
    }

    pub fn session_context(&self, id: SessionId) -> Option<&SessionContext> {
        self.sessions.get(&id)
    }

    pub fn execution_context(&self, id: ExecutionId) -> Option<&ExecutionContext> {
        self.executions.get(&id)
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn execution_count(&self) -> usize {
        self.executions.len()
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new()
    }
}