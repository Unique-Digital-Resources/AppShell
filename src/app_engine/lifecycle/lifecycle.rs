//! Generic lifecycle contract.
//!
//! `Lifecycle` describes *what* operations exist — it does not decide whether
//! a transition is valid. That responsibility belongs to `LifecycleManager`.

use crate::app_engine::errors::engine_error::EngineError;
use crate::app_engine::lifecycle::lifecycle_state::LifecycleState;

pub trait Lifecycle {
    fn initialize(&mut self) -> Result<(), EngineError>;
    fn start(&mut self) -> Result<(), EngineError>;
    fn stop(&mut self) -> Result<(), EngineError>;
    fn dispose(&mut self) -> Result<(), EngineError>;

    fn state(&self) -> LifecycleState;
}