//! `Command` — an executable intent/instance.

use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::app_engine::commands::command_context::CommandContext;
use crate::app_engine::commands::command_definition::CommandDefinitionId;

static COMMAND_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CommandId(u64);

impl CommandId {
    pub fn new() -> Self {
        Self(COMMAND_ID_COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Default for CommandId {
    fn default() -> Self {
        Self::new()
    }
}

/// Type-erased command input with optional serialized form.
pub struct CommandInput {
    data: Option<Box<dyn Any + Send>>,
    serialized: Option<Vec<u8>>,
    type_name: &'static str,
}

impl CommandInput {
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

    /// Get serialized bytes (for IPC / persistence).
    pub fn serialized(&self) -> Option<&[u8]> {
        self.serialized.as_deref()
    }

    pub fn has_serialized(&self) -> bool {
        self.serialized.is_some()
    }

    /// Serialize the typed data using a registry (if not already serialized).
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

    /// Deserialize from bytes using a registry (if not already typed).
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

impl std::fmt::Debug for CommandInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandInput")
            .field("type", &self.type_name)
            .field("empty", &self.is_empty())
            .finish()
    }
}

impl Default for CommandInput {
    fn default() -> Self {
        Self::empty()
    }
}

/// A specific command invocation.
#[derive(Debug)]
pub struct Command {
    id: CommandId,
    definition_id: CommandDefinitionId,
    context: CommandContext,
    input: CommandInput,
}

impl Command {
    pub fn new(
        definition_id: impl Into<CommandDefinitionId>,
        context: CommandContext,
        input: CommandInput,
    ) -> Self {
        Self {
            id: CommandId::new(),
            definition_id: definition_id.into(),
            context,
            input,
        }
    }

    /// Convenience: build a command with no input payload.
    pub fn with_empty_input(
        definition_id: impl Into<CommandDefinitionId>,
        context: CommandContext,
    ) -> Self {
        Self::new(definition_id, context, CommandInput::empty())
    }

    pub fn id(&self) -> CommandId {
        self.id
    }

    pub fn definition_id(&self) -> &CommandDefinitionId {
        &self.definition_id
    }

    pub fn context(&self) -> &CommandContext {
        &self.context
    }

    pub fn input(&self) -> &CommandInput {
        &self.input
    }
}