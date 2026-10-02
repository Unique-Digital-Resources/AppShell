//! Session context — a logical session within an application.
//!
//! A session could later represent a user session, workspace session, CLI
//! invocation, UI session, automation session, or agent session without App
//! Engine needing to know which one.
//!
//! Do NOT equate `SessionContext` with authentication. Identity/auth can be
//! supplied by a later API/security layer.

use crate::app_engine::context::context::{ApplicationId, SessionId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionContext {
    id: SessionId,
    application_id: ApplicationId,
}

impl SessionContext {
    pub fn new(application_id: ApplicationId) -> Self {
        Self {
            id: SessionId::new(),
            application_id,
        }
    }

    pub fn id(&self) -> SessionId {
        self.id
    }

    pub fn application_id(&self) -> ApplicationId {
        self.application_id
    }
}