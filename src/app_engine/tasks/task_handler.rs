//! `TaskHandler` — contract for executing tasks.
//!
//! Phase 13: handlers now receive `&EngineRef` for controlled engine access.

use crate::app_engine::capabilities::engine_ref::EngineRef;
use crate::app_engine::errors::task_error::TaskError;
use crate::app_engine::tasks::task::{TaskInput};
use crate::app_engine::tasks::task_context::TaskContext;
use crate::app_engine::tasks::task_result::TaskResult;

pub trait TaskHandler: Send + Sync {
    fn execute(
        &self,
        input: &TaskInput,
        context: &TaskContext,
        engine: &EngineRef,
    ) -> Result<TaskResult, TaskError>;
}