//! `TaskResult` — the outcome of task execution.

use std::any::Any;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskResultStatus {
    Completed,
    Cancelled,
    Failed,
}

pub struct TaskResult {
    status: TaskResultStatus,
    output: Option<Box<dyn Any + Send>>,
    error: Option<String>,
    metadata: HashMap<String, String>,
}

impl TaskResult {
    pub fn completed() -> Self {
        Self {
            status: TaskResultStatus::Completed,
            output: None,
            error: None,
            metadata: HashMap::new(),
        }
    }

    pub fn completed_with<T: Any + Send>(output: T) -> Self {
        Self {
            status: TaskResultStatus::Completed,
            output: Some(Box::new(output)),
            error: None,
            metadata: HashMap::new(),
        }
    }

    pub fn cancelled() -> Self {
        Self {
            status: TaskResultStatus::Cancelled,
            output: None,
            error: None,
            metadata: HashMap::new(),
        }
    }

    pub fn failed(error: impl Into<String>) -> Self {
        Self {
            status: TaskResultStatus::Failed,
            output: None,
            error: Some(error.into()),
            metadata: HashMap::new(),
        }
    }

    pub fn failed_with<T: Any + Send>(error: impl Into<String>, output: T) -> Self {
        Self {
            status: TaskResultStatus::Failed,
            output: Some(Box::new(output)),
            error: Some(error.into()),
            metadata: HashMap::new(),
        }
    }

    pub fn status(&self) -> TaskResultStatus {
        self.status
    }

    pub fn is_completed(&self) -> bool {
        self.status == TaskResultStatus::Completed
    }

    pub fn is_cancelled(&self) -> bool {
        self.status == TaskResultStatus::Cancelled
    }

    pub fn is_failed(&self) -> bool {
        self.status == TaskResultStatus::Failed
    }

    pub fn is_success(&self) -> bool {
        self.is_completed()
    }

    pub fn has_output(&self) -> bool {
        self.output.is_some()
    }

    pub fn output<T: Any + Send>(&self) -> Option<&T> {
        self.output.as_ref().and_then(|o| o.downcast_ref::<T>())
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn with_metadata(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

impl std::fmt::Debug for TaskResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskResult")
            .field("status", &self.status)
            .field("has_output", &self.has_output())
            .field("error", &self.error)
            .field("metadata", &self.metadata)
            .finish()
    }
}