//! `CommandRegistry` — discovery and registration of command definitions.
//!
//! The registry stores `(CommandDefinition, CommandHandler)` pairs keyed by
//! `CommandDefinitionId`. Definitions describe *what* commands are available;
//! the registry itself does not execute anything.

use std::collections::HashMap;

use crate::app_engine::commands::command_definition::{CommandDefinition, CommandDefinitionId};
use crate::app_engine::commands::command_executor::CommandHandler;
use crate::app_engine::errors::command_error::CommandError;

pub struct CommandRegistry {
    entries: HashMap<CommandDefinitionId, (CommandDefinition, Box<dyn CommandHandler>)>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn register(
        &mut self,
        definition: CommandDefinition,
        handler: Box<dyn CommandHandler>,
    ) -> Result<(), CommandError> {
        if self.entries.contains_key(definition.id()) {
            return Err(CommandError::AlreadyRegistered {
                id: definition.id().as_str().to_string(),
            });
        }
        self.entries.insert(definition.id().clone(), (definition, handler));
        Ok(())
    }

    pub fn unregister(&mut self, id: &CommandDefinitionId) -> Option<CommandDefinition> {
        self.entries.remove(id).map(|(def, _)| def)
    }

    pub fn get(&self, id: &CommandDefinitionId) -> Option<&CommandDefinition> {
        self.entries.get(id).map(|(def, _)| def)
    }

    pub fn contains(&self, id: &CommandDefinitionId) -> bool {
        self.entries.contains_key(id)
    }

    pub fn list(&self) -> Vec<&CommandDefinition> {
        self.entries.values().map(|(def, _)| def).collect()
    }

    /// Internal accessor used by `CommandExecutor` to look up the handler.
    pub fn handler(&self, id: &CommandDefinitionId) -> Option<&dyn CommandHandler> {
        self.entries.get(id).map(|(_, h)| h.as_ref())
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for CommandRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandRegistry")
            .field("count", &self.entries.len())
            .field("ids", &self.entries.keys().collect::<Vec<_>>())
            .finish()
    }
}