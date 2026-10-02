//! `CommandHandler` trait and `CommandExecutor` — the execution mechanism.
//!
//! The executor is generic: it looks up a command's definition and handler in
//! the registry, validates that the command is known, and delegates execution
//! to the handler. The handler itself implements domain functionality; App
//! Engine never inspects what the handler does.

//! `CommandHandler` trait and `CommandExecutor`.

use crate::app_engine::capabilities::engine_ref::EngineRef;
use crate::app_engine::commands::command::{Command, CommandInput};
use crate::app_engine::commands::command_context::CommandContext;
use crate::app_engine::commands::command_definition::{CommandDefinition, CommandDefinitionId};
use crate::app_engine::commands::command_registry::CommandRegistry;
use crate::app_engine::commands::command_result::CommandResult;
use crate::app_engine::errors::command_error::CommandError;

pub trait CommandHandler: Send + Sync {
    fn execute(
        &self,
        input: &CommandInput,
        context: &CommandContext,
        engine: &EngineRef,
    ) -> Result<CommandResult, CommandError>;
}

pub struct CommandExecutor {
    registry: CommandRegistry,
}

impl CommandExecutor {
    pub fn new() -> Self {
        Self {
            registry: CommandRegistry::new(),
        }
    }

    pub fn with_registry(registry: CommandRegistry) -> Self {
        Self { registry }
    }


    /// Execute a command.
    ///
    /// Pipeline:
    /// ```text
    /// Command
    ///   │
    ///   ▼
    /// find definition ──→ Registry
    ///   │
    ///   ▼
    /// resolve handler
    ///   │
    ///   ▼
    /// handler.execute(input, context)
    ///   │
    ///   ▼
    /// Result
    /// ```


    /// Execute a command. Panics in the handler are caught and converted to errors.
    pub fn execute(
        &self,
        command: &Command,
        engine: &EngineRef,
    ) -> Result<CommandResult, CommandError> {
        let def_id = command.definition_id();
        if !self.registry.contains(def_id) {
            return Err(CommandError::UnknownCommand {
                id: def_id.as_str().to_string(),
            });
        }
        let handler = self
            .registry
            .handler(def_id)
            .expect("handler must exist when definition is registered");

        let input = command.input();
        let context = command.context();

        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            handler.execute(input, context, engine)
        })) {
            Ok(result) => result,
            Err(panic_payload) => {
                let reason = extract_panic_message_command(&panic_payload);
                Err(CommandError::ExecutionFailed(format!("handler panicked: {}", reason)))
            }
        }
    }

    pub fn register(
        &mut self,
        definition: CommandDefinition,
        handler: Box<dyn CommandHandler>,
    ) -> Result<(), CommandError> {
        self.registry.register(definition, handler)
    }

    pub fn unregister(&mut self, id: &CommandDefinitionId) -> Option<CommandDefinition> {
        self.registry.unregister(id)
    }

    pub fn contains(&self, id: &CommandDefinitionId) -> bool {
        self.registry.contains(id)
    }

    pub fn get(&self, id: &CommandDefinitionId) -> Option<&CommandDefinition> {
        self.registry.get(id)
    }

    pub fn list(&self) -> Vec<&CommandDefinition> {
        self.registry.list()
    }

    pub fn registry(&self) -> &CommandRegistry {
        &self.registry
    }

    pub fn registry_mut(&mut self) -> &mut CommandRegistry {
        &mut self.registry
    }
}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for CommandExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandExecutor")
            .field("registry", &self.registry)
            .finish()
    }
}


fn extract_panic_message_command(payload: &Box<dyn std::any::Any + Send>) -> String {
    payload
        .downcast_ref::<String>()
        .map(|s| s.clone())
        .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown panic".to_string())
}