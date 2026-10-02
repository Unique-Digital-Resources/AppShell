//! Application-wide context. Represents the environment of one application
//! instance.
//!
//! ✓ application_id, instance_id, runtime info
//! ✗ document, selected_shape, scene, card_database
//!
//! Those belong to the Domain Engine.

use crate::app_engine::context::context::ApplicationId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationContext {
    id: ApplicationId,
    application_id: String,
}

impl ApplicationContext {
    pub fn new(application_id: impl Into<String>) -> Self {
        Self {
            id: ApplicationId::new(),
            application_id: application_id.into(),
        }
    }

    pub fn id(&self) -> ApplicationId {
        self.id
    }

    pub fn application_id(&self) -> &str {
        &self.application_id
    }

    /// Backward-compatible with Phase 1. Returns the same unique value as `id().value()`.
    pub fn instance_id(&self) -> u64 {
        self.id.value()
    }
}