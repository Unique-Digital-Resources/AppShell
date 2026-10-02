//! `Task` — one instance of managed work.

use std::any::Any;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::app_engine::tasks::task_context::TaskContext;
use crate::app_engine::tasks::task_definition::TaskDefinitionId;
use crate::app_engine::tasks::task_state::TaskState;

static TASK_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TaskId(u64);

impl TaskId {
    pub fn new() -> Self {
        Self(TASK_ID_COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

/// Type-erased task input with optional serialized form.
pub struct TaskInput {
    data: Option<Box<dyn Any + Send>>,
    serialized: Option<Vec<u8>>,
    type_name: &'static str,
}

impl TaskInput {
    pub fn empty() -> Self {
        Self {
            data: None,
            serialized: None,
            type_name: "()",
        }
    }

    pub fn new<T: Any + Send>(data: T) -> Self {
        Self {
            data: Some(Box::new(data)),
            serialized: None,
            type_name: std::any::type_name::<T>(),
        }
    }

    /// Create from serialized bytes (for IPC / deserialization).
    pub fn from_bytes(type_name: &'static str, bytes: Vec<u8>) -> Self {
        Self {
            data: None,
            serialized: Some(bytes),
            type_name,
        }
    }

    /// Create with both typed and serialized forms.
    pub fn with_serialized<T: Any + Send>(data: T, bytes: Vec<u8>) -> Self {
        Self {
            data: Some(Box::new(data)),
            serialized: Some(bytes),
            type_name: std::any::type_name::<T>(),
        }
    }

    pub fn get<T: Any + Send>(&self) -> Option<&T> {
        self.data.as_ref().and_then(|d| d.downcast_ref::<T>())
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_none() && self.serialized.is_none()
    }

    pub fn has_data(&self) -> bool {
        self.data.is_some()
    }

    pub fn type_name(&self) -> &'static str {
        self.type_name
    }

    /// Get serialized bytes.
    pub fn serialized(&self) -> Option<&[u8]> {
        self.serialized.as_deref()
    }

    pub fn has_serialized(&self) -> bool {
        self.serialized.is_some()
    }

    /// Serialize typed data using a registry.
    pub fn ensure_serialized(
        &mut self,
        registry: &crate::app_engine::serialization::serialization_registry::SerializationRegistry,
    ) -> Result<(), crate::app_engine::errors::serialization_error::SerializationError> {
        if self.serialized.is_none() {
            if let Some(data) = &self.data {
                let bytes = registry.serialize(data.as_ref(), self.type_name)?;
                self.serialized = Some(bytes);
            }
        }
        Ok(())
    }

    /// Deserialize from bytes using a registry.
    pub fn ensure_deserialized(
        &mut self,
        registry: &crate::app_engine::serialization::serialization_registry::SerializationRegistry,
    ) -> Result<(), crate::app_engine::errors::serialization_error::SerializationError> {
        if self.data.is_none() {
            if let Some(bytes) = &self.serialized {
                let data = registry.deserialize(bytes, self.type_name)?;
                self.data = Some(data);
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for TaskInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskInput")
            .field("type", &self.type_name)
            .field("empty", &self.is_empty())
            .finish()
    }
}

impl Default for TaskInput {
    fn default() -> Self {
        Self::empty()
    }
}

/// A task instance.
pub struct Task {
    id: TaskId,
    definition_id: TaskDefinitionId,
    state: TaskState,
    context: TaskContext,
    input: TaskInput,
    supports_pause: bool,
    supports_cancel: bool,
    progress: f32,
    metadata: HashMap<String, String>,
}

impl Task {
    pub fn new(
        definition_id: impl Into<TaskDefinitionId>,
        context: TaskContext,
        input: TaskInput,
        supports_pause: bool,
        supports_cancel: bool,
    ) -> Self {
        Self {
            id: TaskId::new(),
            definition_id: definition_id.into(),
            state: TaskState::Pending,
            context,
            input,
            supports_pause,
            supports_cancel,
            progress: 0.0,
            metadata: HashMap::new(),
        }
    }

    pub fn id(&self) -> TaskId {
        self.id
    }

    pub fn definition_id(&self) -> &TaskDefinitionId {
        &self.definition_id
    }

    pub fn state(&self) -> TaskState {
        self.state
    }

    pub fn context(&self) -> &TaskContext {
        &self.context
    }

    pub fn input(&self) -> &TaskInput {
        &self.input
    }

    pub fn supports_pause(&self) -> bool {
        self.supports_pause
    }

    pub fn supports_cancel(&self) -> bool {
        self.supports_cancel
    }

    pub fn progress(&self) -> f32 {
        self.progress
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn can_start(&self) -> bool {
        self.state.can_start()
    }

    pub fn can_pause(&self) -> bool {
        self.state.can_pause() && self.supports_pause
    }

    pub fn can_resume(&self) -> bool {
        self.state.can_resume()
    }

    pub fn can_cancel(&self) -> bool {
        self.state.can_cancel() && self.supports_cancel
    }

    pub fn is_terminal(&self) -> bool {
        self.state.is_terminal()
    }

    pub fn is_active(&self) -> bool {
        self.state.is_active()
    }

    pub(crate) fn set_state(&mut self, state: TaskState) {
        self.state = state;
    }

    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
    }

    pub(crate) fn set_output_channel(
        &mut self,
        channel: crate::app_engine::execution::output_channel::TaskOutputChannel,
    ) {
        self.context.set_output_channel(channel);
    }
}

impl std::fmt::Debug for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Task")
            .field("id", &self.id)
            .field("definition_id", &self.definition_id)
            .field("state", &self.state)
            .field("supports_pause", &self.supports_pause)
            .field("supports_cancel", &self.supports_cancel)
            .field("progress", &self.progress)
            .finish()
    }
}