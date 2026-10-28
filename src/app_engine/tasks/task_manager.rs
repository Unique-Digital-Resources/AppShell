//! `TaskManager` — owns the task lifecycle and task collection.
//!
//! Phase 18: thread-safe using `Mutex`/`RwLock`. All mutating methods
//! take `&self` instead of `&mut self`.
//!
//! `tasks` uses `Mutex` (not `RwLock`) because `Task` contains
//! `Box<dyn Any + Send>` which is `Send` but not `Sync`.

use std::collections::HashMap;
use std::sync::{Mutex, RwLock};

use crate::app_engine::capabilities::engine_ref::EngineRef;
use crate::app_engine::errors::task_error::TaskError;
use crate::app_engine::execution::cancellation_token::CancellationToken;
use crate::app_engine::execution::output_channel::TaskOutputChannel;
use crate::app_engine::tasks::task::{Task, TaskId, TaskInput};
use crate::app_engine::tasks::task_context::TaskContext;
use crate::app_engine::tasks::task_definition::{TaskDefinition, TaskDefinitionId};
use crate::app_engine::tasks::task_executor::{TaskExecutor, TaskHandler};
use crate::app_engine::tasks::task_result::{TaskResult, TaskResultStatus};
use crate::app_engine::tasks::task_state::TaskState;

pub struct TaskManager {
    // Mutex (not RwLock) because Task contains Box<dyn Any + Send> which is Send but not Sync
    tasks: Mutex<HashMap<TaskId, Task>>,
    // RwLock works here because TaskExecutor contains Box<dyn TaskHandler> which is Send + Sync
    executor: RwLock<TaskExecutor>,
    cancellation_tokens: RwLock<HashMap<TaskId, CancellationToken>>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(HashMap::new()),
            executor: RwLock::new(TaskExecutor::new()),
            cancellation_tokens: RwLock::new(HashMap::new()),
        }
    }

    pub fn register(
        &self,
        definition: TaskDefinition,
        handler: Box<dyn TaskHandler>,
    ) -> Result<(), TaskError> {
        self.executor.write().unwrap().register(definition, handler)
    }

    pub fn create(
        &self,
        definition_id: impl Into<TaskDefinitionId>,
        mut context: TaskContext,
        input: TaskInput,
    ) -> Result<TaskId, TaskError> {
        let def_id = definition_id.into();
        let executor_guard = self.executor.read().unwrap();
        let def = executor_guard.get_definition(&def_id)
            .ok_or_else(|| TaskError::UnknownDefinition {
                id: def_id.as_str().to_string(),
            })?;

        let task_id = TaskId::new();
        context.set_task_id(task_id);

        let token = CancellationToken::new();
        context.set_cancellation_token(token.clone());
        self.cancellation_tokens.write().unwrap().insert(task_id, token);

        let channel = TaskOutputChannel::new(task_id.value());
        context.set_output_channel(channel);

        let task = Task::new(
            def_id,
            context,
            input,
            def.supports_pause(),
            def.supports_cancel(),
        );
        self.tasks.lock().unwrap().insert(task_id, task);
        Ok(task_id)
    }

    pub fn start(
        &self,
        id: TaskId,
        engine: &EngineRef,
    ) -> Result<TaskResult, TaskError> {
        // Step 1: validate and transition to Running
        {
            let mut tasks = self.tasks.lock().unwrap();
            let task = tasks.get_mut(&id).ok_or(TaskError::NotFound { id: id.value() })?;
            if !task.can_start() {
                return Err(TaskError::InvalidState {
                    id: id.value(),
                    current: task.state().as_str().to_string(),
                    requested: "Running".to_string(),
                });
            }
            task.set_state(TaskState::Running);
        }

        // Step 2: execute
        let result = {
            let tasks = self.tasks.lock().unwrap();
            let task = tasks.get(&id).expect("task must exist");
            let executor = self.executor.read().unwrap();
            executor.execute(&task, engine)
        };

        // Step 3: update state
        {
            let mut tasks = self.tasks.lock().unwrap();
            let task = tasks.get_mut(&id).expect("task must exist");
            match &result {
                Ok(r) => {
                    let new_state = match r.status() {
                        TaskResultStatus::Completed => TaskState::Completed,
                        TaskResultStatus::Cancelled => TaskState::Cancelled,
                        TaskResultStatus::Failed => TaskState::Failed,
                    };
                    task.set_state(new_state);
                }
                Err(_) => {
                    task.set_state(TaskState::Failed);
                }
            }
        }

        self.cancellation_tokens.write().unwrap().remove(&id);
        result
    }

    pub fn connect_output(
        &self,
        id: TaskId,
        signal_bus: std::sync::Arc<crate::app_engine::signals::signal_bus::SignalBus>,
    ) {
        let channel = TaskOutputChannel::with_signal_bus(signal_bus, id.value());
        if let Some(task) = self.tasks.lock().unwrap().get_mut(&id) {
            task.set_output_channel(channel);
        }
    }

    pub fn cancel(&self, id: TaskId) -> Result<(), TaskError> {
        if let Some(token) = self.cancellation_tokens.read().unwrap().get(&id) {
            token.cancel();
        }

        let mut tasks = self.tasks.lock().unwrap();
        let task = tasks.get_mut(&id).ok_or(TaskError::NotFound { id: id.value() })?;
        if !task.can_cancel() {
            return Err(TaskError::NotCancellable {
                id: id.value(),
                current: task.state().as_str().to_string(),
            });
        }
        task.set_state(TaskState::Cancelled);
        Ok(())
    }

    pub fn pause(&self, id: TaskId) -> Result<(), TaskError> {
        let mut tasks = self.tasks.lock().unwrap();
        let task = tasks.get_mut(&id).ok_or(TaskError::NotFound { id: id.value() })?;
        if !task.can_pause() {
            return Err(TaskError::NotPausable {
                id: id.value(),
                current: task.state().as_str().to_string(),
            });
        }
        task.set_state(TaskState::Paused);
        Ok(())
    }

    pub fn resume(&self, id: TaskId) -> Result<(), TaskError> {
        let mut tasks = self.tasks.lock().unwrap();
        let task = tasks.get_mut(&id).ok_or(TaskError::NotFound { id: id.value() })?;
        if !task.can_resume() {
            return Err(TaskError::InvalidState {
                id: id.value(),
                current: task.state().as_str().to_string(),
                requested: "Running".to_string(),
            });
        }
        task.set_state(TaskState::Running);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get(&self, _id: TaskId) -> Option<std::sync::MutexGuard<'_, Task>> {
        unimplemented!("Use get_state() instead")
    }

    pub fn get_state(&self, id: TaskId) -> Option<TaskState> {
        self.tasks.lock().unwrap().get(&id).map(|t| t.state())
    }

    pub fn list(&self) -> Vec<TaskState> {
        self.tasks.lock().unwrap().values().map(|t| t.state()).collect()
    }

    pub fn count(&self) -> usize {
        self.tasks.lock().unwrap().len()
    }

    pub fn contains(&self, id: TaskId) -> bool {
        self.tasks.lock().unwrap().contains_key(&id)
    }

    pub fn remove(&self, id: TaskId) -> Option<Task> {
        self.cancellation_tokens.write().unwrap().remove(&id);
        self.tasks.lock().unwrap().remove(&id)
    }

    /// Get the execution ID from a task's context.
    pub fn task_execution_id(&self, id: TaskId) -> Option<crate::app_engine::context::context::ExecutionId> {
        let tasks = self.tasks.lock().unwrap();
        tasks.get(&id).map(|t| t.context().execution_id())
    }

    /// Get the task_id from a task's context.
    pub fn task_context_task_id(&self, id: TaskId) -> Option<TaskId> {
        let tasks = self.tasks.lock().unwrap();
        tasks.get(&id).and_then(|t| t.context().task_id())
    }

    /// Get the context metadata from a task.
    pub fn task_context_metadata(&self, id: TaskId) -> Option<std::collections::HashMap<String, String>> {
        let tasks = self.tasks.lock().unwrap();
        tasks.get(&id).map(|t| t.context().metadata().clone())
    }

    /// Register an async handler (only with async-runtime feature).
    #[cfg(feature = "async-runtime")]
    pub fn register_async(
        &self,
        definition: TaskDefinition,
        handler: Box<dyn crate::app_engine::concurrency::async_task_handler::AsyncTaskHandler>,
    ) -> Result<(), TaskError> {
        self.executor.write().unwrap().register_async(definition, handler)
    }

    /// Start a task asynchronously using an AsyncRuntime.
    /// For sync handlers, delegates to start(). For async handlers, spawns on tokio.
    #[cfg(feature = "async-runtime")]
    pub fn start_async(
        &self,
        id: TaskId,
        engine: &EngineRef,
        async_runtime: &crate::app_engine::concurrency::async_runtime::AsyncRuntime,
    ) -> Result<TaskResult, TaskError> {
        use crate::app_engine::tasks::task_executor::HandlerKind;

        // Check handler kind
        let def_id = {
            let tasks = self.tasks.lock().unwrap();
            let task = tasks.get(&id).ok_or(TaskError::NotFound { id: id.value() })?;
            task.definition_id().clone()
        };

        let executor_guard = self.executor.read().unwrap();
        let kind = executor_guard.handler_kind(&def_id);

        match kind {
            Some(HandlerKind::Sync) | None => {
                drop(executor_guard);
                self.start(id, engine)
            }
            Some(HandlerKind::Async) => {
                // Step 1: validate and transition to Running
                {
                    let mut tasks = self.tasks.lock().unwrap();
                    let task = tasks.get_mut(&id).ok_or(TaskError::NotFound { id: id.value() })?;
                    if !task.can_start() {
                        return Err(TaskError::InvalidState {
                            id: id.value(),
                            current: task.state().as_str().to_string(),
                            requested: "Running".to_string(),
                        });
                    }
                    task.set_state(TaskState::Running);
                }

                // Step 2: get the async future and block on it
                let result = {
                    let tasks = self.tasks.lock().unwrap();
                    let task = tasks.get(&id).expect("task must exist");
                    let future = executor_guard.execute_async(&task, engine)?;
                    drop(tasks);
                    drop(executor_guard);

                    async_runtime.block_on(future)
                };

                // Step 3: update state based on result
                {
                    let mut tasks = self.tasks.lock().unwrap();
                    let task = tasks.get_mut(&id).expect("task must exist");
                    match &result {
                        Ok(r) => {
                            let new_state = match r.status() {
                                TaskResultStatus::Completed => TaskState::Completed,
                                TaskResultStatus::Cancelled => TaskState::Cancelled,
                                TaskResultStatus::Failed => TaskState::Failed,
                            };
                            task.set_state(new_state);
                        }
                        Err(_) => {
                            task.set_state(TaskState::Failed);
                        }
                    }
                }

                self.cancellation_tokens.write().unwrap().remove(&id);
                result
            }
        }
    }

    /// Cancel all running tasks. Propagates cancellation tokens.
    /// Returns the IDs of cancelled tasks.
    pub fn cancel_all(&self) -> Vec<TaskId> {
        let mut cancelled = Vec::new();

        // First pass: cancel tokens for running tasks
        {
            let tokens = self.cancellation_tokens.read().unwrap();
            let tasks = self.tasks.lock().unwrap();
            for (id, task) in tasks.iter() {
                if task.state() == TaskState::Running {
                    if let Some(token) = tokens.get(id) {
                        token.cancel();
                    }
                }
            }
        }

        // Second pass: set state to Cancelled
        {
            let mut tasks = self.tasks.lock().unwrap();
            for (id, task) in tasks.iter_mut() {
                if task.state() == TaskState::Running {
                    task.set_state(TaskState::Cancelled);
                    cancelled.push(*id);
                }
            }
        }

        cancelled
    }

}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}