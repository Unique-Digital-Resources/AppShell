//! `TaskHandler` trait and `TaskExecutor`.

use std::collections::HashMap;
use std::sync::RwLock;

use crate::app_engine::capabilities::engine_ref::EngineRef;
use crate::app_engine::errors::task_error::TaskError;
use crate::app_engine::tasks::task::{Task, TaskInput};
use crate::app_engine::tasks::task_context::TaskContext;
use crate::app_engine::tasks::task_definition::{TaskDefinition, TaskDefinitionId};
use crate::app_engine::tasks::task_result::TaskResult;

#[cfg(feature = "async-runtime")]
use crate::app_engine::concurrency::async_task_handler::{AsyncTaskFuture, AsyncTaskHandler};

/// Whether a handler is sync or async.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandlerKind {
    Sync,
    #[cfg(feature = "async-runtime")]
    Async,
}

pub trait TaskHandler: Send + Sync {
    fn execute(
        &self,
        input: &TaskInput,
        context: &TaskContext,
        engine: &EngineRef,
    ) -> Result<TaskResult, TaskError>;
}

/// Internal enum for storing either sync or async handlers.
enum StoredHandler {
    Sync(Box<dyn TaskHandler>),
    #[cfg(feature = "async-runtime")]
    Async(Box<dyn AsyncTaskHandler>),
}

pub struct TaskExecutor {
    entries: RwLock<HashMap<TaskDefinitionId, (TaskDefinition, StoredHandler)>>,
}

impl TaskExecutor {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }

    /// Register a sync handler.
    pub fn register(
        &self,
        definition: TaskDefinition,
        handler: Box<dyn TaskHandler>,
    ) -> Result<(), TaskError> {
        let mut entries = self.entries.write().unwrap();
        if entries.contains_key(definition.id()) {
            return Err(TaskError::AlreadyRegistered {
                id: definition.id().as_str().to_string(),
            });
        }
        entries.insert(definition.id().clone(), (definition, StoredHandler::Sync(handler)));
        Ok(())
    }

    /// Register an async handler (only with async-runtime feature).
    #[cfg(feature = "async-runtime")]
    pub fn register_async(
        &self,
        definition: TaskDefinition,
        handler: Box<dyn AsyncTaskHandler>,
    ) -> Result<(), TaskError> {
        let mut entries = self.entries.write().unwrap();
        if entries.contains_key(definition.id()) {
            return Err(TaskError::AlreadyRegistered {
                id: definition.id().as_str().to_string(),
            });
        }
        entries.insert(definition.id().clone(), (definition, StoredHandler::Async(handler)));
        Ok(())
    }

    pub fn unregister(&self, id: &TaskDefinitionId) -> Option<TaskDefinition> {
        self.entries.write().unwrap().remove(id).map(|(def, _)| def)
    }

    pub fn get_definition(&self, id: &TaskDefinitionId) -> Option<TaskDefinition> {
        // Clone the definition (it's Clone)
        self.entries.read().unwrap().get(id).map(|(def, _)| def.clone())
    }

    pub fn contains(&self, id: &TaskDefinitionId) -> bool {
        self.entries.read().unwrap().contains_key(id)
    }

    pub fn list(&self) -> Vec<TaskDefinition> {
        self.entries.read().unwrap().values().map(|(def, _)| def.clone()).collect()
    }

    pub fn len(&self) -> usize {
        self.entries.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.read().unwrap().is_empty()
    }

    /// Get the handler kind (sync or async) for a definition.
    pub fn handler_kind(&self, id: &TaskDefinitionId) -> Option<HandlerKind> {
        let entries = self.entries.read().unwrap();
        entries.get(id).map(|(_, h)| match h {
            StoredHandler::Sync(_) => HandlerKind::Sync,
            #[cfg(feature = "async-runtime")]
            StoredHandler::Async(_) => HandlerKind::Async,
        })
    }

    /// Execute a task. Panics in the handler are caught and converted to errors.
    pub fn execute(
        &self,
        task: &Task,
        engine: &EngineRef,
    ) -> Result<TaskResult, TaskError> {
        let def_id = task.definition_id();
        let entries = self.entries.read().unwrap();

        let entry = entries.get(def_id).ok_or_else(|| TaskError::UnknownDefinition {
            id: def_id.as_str().to_string(),
        })?;

        let task_id_val = task.id().value();

        match &entry.1 {
            StoredHandler::Sync(handler) => {
                let input = task.input();
                let context = task.context();

                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    handler.execute(input, context, engine)
                })) {
                    Ok(result) => result,
                    Err(panic_payload) => {
                        let reason = extract_panic_message(&panic_payload);
                        Err(TaskError::ExecutionFailed {
                            id: task_id_val,
                            reason: format!("handler panicked: {}", reason),
                        })
                    }
                }
            }
            #[cfg(feature = "async-runtime")]
            StoredHandler::Async(_) => Err(TaskError::ExecutionFailed {
                id: task_id_val,
                reason: "async handler cannot be executed synchronously; use start_async()".to_string(),
            }),
        }
    }

    #[cfg(feature = "async-runtime")]
    pub fn execute_async(
        &self,
        task: &Task,
        engine: &EngineRef,
    ) -> Result<AsyncTaskFuture, TaskError> {
        let def_id = task.definition_id();
        let entries = self.entries.read().unwrap();

        let entry = entries.get(def_id).ok_or_else(|| TaskError::UnknownDefinition {
            id: def_id.as_str().to_string(),
        })?;

        let task_id_val = task.id().value();

        match &entry.1 {
            StoredHandler::Sync(_) => Err(TaskError::ExecutionFailed {
                id: task_id_val,
                reason: "sync handler cannot be executed asynchronously; use start()".to_string(),
            }),
            StoredHandler::Async(handler) => {
                let input = task.input();
                let context = task.context();

                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    handler.execute(input, context, engine)
                })) {
                    Ok(future) => Ok(future),
                    Err(panic_payload) => {
                        let reason = extract_panic_message(&panic_payload);
                        Err(TaskError::ExecutionFailed {
                            id: task_id_val,
                            reason: format!("async handler panicked: {}", reason),
                        })
                    }
                }
            }
        }
    }
}

impl Default for TaskExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for TaskExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskExecutor")
            .field("count", &self.entries.read().unwrap().len())
            .finish()
    }
}

/// Extract a human-readable message from a panic payload.
fn extract_panic_message(payload: &Box<dyn std::any::Any + Send>) -> String {
    payload
        .downcast_ref::<String>()
        .map(|s| s.clone())
        .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown panic".to_string())
}