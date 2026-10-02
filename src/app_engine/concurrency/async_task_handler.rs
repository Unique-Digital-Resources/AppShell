//! `AsyncTaskHandler` — async counterpart to `TaskHandler`.
//!
//! Uses `Box<dyn Future>` instead of `#[async_trait]` to avoid external
//! dependencies. Async runtimes (tokio, async-std) can implement this
//! trait by returning `Box::pin(async { ... })`.

use std::pin::Pin;

use crate::app_engine::capabilities::engine_ref::EngineRef;
use crate::app_engine::errors::task_error::TaskError;
use crate::app_engine::tasks::task::TaskInput;
use crate::app_engine::tasks::task_context::TaskContext;
use crate::app_engine::tasks::task_result::TaskResult;

/// A boxed future returned by async task handlers.
pub type AsyncTaskFuture = Pin<Box<dyn std::future::Future<Output = Result<TaskResult, TaskError>> + Send>>;

/// Async counterpart to `TaskHandler`.
///
/// Implement this trait for handlers that perform I/O or need concurrency.
/// The async runtime (tokio, async-std, or a custom executor) polls the
/// returned future to completion.
pub trait AsyncTaskHandler: Send + Sync {
    fn execute(
        &self,
        input: &TaskInput,
        context: &TaskContext,
        engine: &EngineRef,
    ) -> AsyncTaskFuture;
}