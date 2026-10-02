//! Centralized lifecycle transition rules.
//!
//! Given (current state, requested operation), produces either a new state
//! or an `EngineError`. The manager does not execute side effects; it is a
//! pure transition table that other systems delegate to.

use crate::app_engine::errors::engine_error::EngineError;
use crate::app_engine::lifecycle::lifecycle_state::LifecycleState;

#[derive(Debug, Clone, Default)]
pub struct LifecycleManager;

impl LifecycleManager {
    pub fn new() -> Self {
        Self
    }

    pub fn initialize(
        &self,
        current: LifecycleState,
    ) -> Result<LifecycleState, EngineError> {
        if !current.can_initialize() {
            return Err(EngineError::invalid_transition(
                current.as_str(),
                "Initialized",
            ));
        }
        Ok(LifecycleState::Initialized)
    }

    pub fn start(
        &self,
        current: LifecycleState,
    ) -> Result<LifecycleState, EngineError> {
        if !current.can_start() {
            return Err(EngineError::invalid_transition(
                current.as_str(),
                "Running",
            ));
        }
        Ok(LifecycleState::Running)
    }

    pub fn run(
        &self,
        current: LifecycleState,
    ) -> Result<LifecycleState, EngineError> {
        if !current.can_run() {
            return Err(EngineError::invalid_transition(
                current.as_str(),
                "Running",
            ));
        }
        Ok(LifecycleState::Running)
    }

    pub fn stop(
        &self,
        current: LifecycleState,
    ) -> Result<LifecycleState, EngineError> {
        if !current.can_stop() {
            return Err(EngineError::invalid_transition(
                current.as_str(),
                "Stopped",
            ));
        }
        Ok(LifecycleState::Stopped)
    }

    pub fn dispose(
        &self,
        current: LifecycleState,
    ) -> Result<LifecycleState, EngineError> {
        if !current.can_dispose() {
            return Err(EngineError::invalid_transition(
                current.as_str(),
                "Disposed",
            ));
        }
        Ok(LifecycleState::Disposed)
    }
}