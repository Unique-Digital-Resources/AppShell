//! `CommandResult` — the generic output of command execution.
//!
//! The App Engine does not need to understand the internal structure of any
//! domain-specific result payload; it just holds a type-erased output box.

use std::any::Any;
use std::collections::HashMap;

use crate::app_engine::context::context::ExecutionId;

pub struct CommandResult {
    success: bool,
    output: Option<Box<dyn Any + Send>>,
    metadata: HashMap<String, String>,
    execution_id: Option<ExecutionId>,
}

impl CommandResult {
    pub fn success() -> Self {
        Self {
            success: true,
            output: None,
            metadata: HashMap::new(),
            execution_id: None,
        }
    }

    pub fn success_with<T: Any + Send>(output: T) -> Self {
        Self {
            success: true,
            output: Some(Box::new(output)),
            metadata: HashMap::new(),
            execution_id: None,
        }
    }

    pub fn failure() -> Self {
        Self {
            success: false,
            output: None,
            metadata: HashMap::new(),
            execution_id: None,
        }
    }

    pub fn failure_with<T: Any + Send>(output: T) -> Self {
        Self {
            success: false,
            output: Some(Box::new(output)),
            metadata: HashMap::new(),
            execution_id: None,
        }
    }

    pub fn is_success(&self) -> bool {
        self.success
    }

    pub fn is_failure(&self) -> bool {
        !self.success
    }

    pub fn has_output(&self) -> bool {
        self.output.is_some()
    }

    pub fn output<T: Any + Send>(&self) -> Option<&T> {
        self.output.as_ref().and_then(|o| o.downcast_ref::<T>())
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn execution_id(&self) -> Option<ExecutionId> {
        self.execution_id
    }

    pub fn with_metadata(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn with_execution_id(mut self, id: ExecutionId) -> Self {
        self.execution_id = Some(id);
        self
    }
}

impl std::fmt::Debug for CommandResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandResult")
            .field("success", &self.success)
            .field("has_output", &self.has_output())
            .field("metadata", &self.metadata)
            .field("execution_id", &self.execution_id)
            .finish()
    }
}