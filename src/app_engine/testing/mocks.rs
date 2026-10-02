//! Mock implementations for testing handlers without a full engine.

//! Mock implementations for testing handlers without a full engine.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::app_engine::capabilities::engine_ref::EngineRef;
use crate::app_engine::commands::command::Command;
use crate::app_engine::commands::command_definition::{CommandDefinition, CommandDefinitionId};
use crate::app_engine::commands::command_executor::CommandHandler;
use crate::app_engine::commands::command_result::CommandResult;
use crate::app_engine::errors::command_error::CommandError;
use crate::app_engine::events::event::Event;
use crate::app_engine::events::event_handler::EventHandler;
use crate::app_engine::signals::signal::Signal;
/// Mock command executor that records executed commands.
pub struct MockCommandExecutor {
    handlers: HashMap<CommandDefinitionId, Box<dyn CommandHandler>>,
    executed: Mutex<Vec<String>>,
}

impl MockCommandExecutor {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            executed: Mutex::new(vec![]),
        }
    }

    pub fn register(
        &mut self,
        definition: CommandDefinition,
        handler: Box<dyn CommandHandler>,
    ) -> Result<(), CommandError> {
        if self.handlers.contains_key(definition.id()) {
            return Err(CommandError::AlreadyRegistered {
                id: definition.id().as_str().to_string(),
            });
        }
        self.handlers.insert(definition.id().clone(), handler);
        Ok(())
    }

    pub fn execute(
        &self,
        command: &Command,
        engine: &EngineRef,
    ) -> Result<CommandResult, CommandError> {
        let def_id = command.definition_id();
        self.executed.lock().unwrap().push(def_id.as_str().to_string());

        let handler = self
            .handlers
            .get(def_id)
            .ok_or(CommandError::UnknownCommand {
                id: def_id.as_str().to_string(),
            })?;

        handler.execute(command.input(), command.context(), engine)
    }

    pub fn assert_executed(&self, id: &str) {
        let executed = self.executed.lock().unwrap();
        assert!(
            executed.iter().any(|e| e == id),
            "command not executed: {} (executed: {:?})",
            id,
            *executed
        );
    }

    pub fn assert_not_executed(&self, id: &str) {
        let executed = self.executed.lock().unwrap();
        assert!(
            !executed.iter().any(|e| e == id),
            "command should not have been executed: {}",
            id
        );
    }

    pub fn executed_count(&self) -> usize {
        self.executed.lock().unwrap().len()
    }

    pub fn contains(&self, id: &CommandDefinitionId) -> bool {
        self.handlers.contains_key(id)
    }

    pub fn list(&self) -> Vec<&CommandDefinitionId> {
        self.handlers.keys().collect()
    }
}

impl Default for MockCommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock event bus that records published events.
pub struct MockEventBus {
    published: Mutex<Vec<String>>,
    handlers: Mutex<HashMap<String, Vec<Box<dyn EventHandler>>>>,
}

impl MockEventBus {
    pub fn new() -> Self {
        Self {
            published: Mutex::new(vec![]),
            handlers: Mutex::new(HashMap::new()),
        }
    }

    pub fn subscribe(
        &self,
        event_type: &str,
        handler: Box<dyn EventHandler>,
    ) {
        self.handlers
            .lock()
            .unwrap()
            .entry(event_type.to_string())
            .or_default()
            .push(handler);
    }

    pub fn publish(&self, event: &Event) {
        // Record the event type (Event doesn't implement Clone)
        self.published.lock().unwrap().push(event.event_type().to_string());

        // Deliver to subscribers
        let handlers = self.handlers.lock().unwrap();
        if let Some(list) = handlers.get(event.event_type()) {
            for handler in list {
                handler.handle(event);
            }
        }
    }

    pub fn assert_published(&self, event_type: &str) {
        let published = self.published.lock().unwrap();
        assert!(
            published.iter().any(|e| e == event_type),
            "event not published: {} (published: {:?})",
            event_type,
            *published
        );
    }

    pub fn assert_not_published(&self, event_type: &str) {
        let published = self.published.lock().unwrap();
        assert!(
            !published.iter().any(|e| e == event_type),
            "event should not have been published: {}",
            event_type
        );
    }

    pub fn published_count(&self) -> usize {
        self.published.lock().unwrap().len()
    }

    pub fn published_events(&self) -> Vec<String> {
        self.published.lock().unwrap().clone()
    }
}

impl Default for MockEventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock signal bus that records emitted signals.
pub struct MockSignalBus {
    emitted: Mutex<Vec<String>>,
}

impl MockSignalBus {
    pub fn new() -> Self {
        Self {
            emitted: Mutex::new(vec![]),
        }
    }

    pub fn emit(&self, signal: &Signal) {
        self.emitted
            .lock()
            .unwrap()
            .push(signal.signal_type().to_string());
    }

    pub fn subscribe<F>(&self, _signal_type: &str, _callback: F)
    where
        F: Fn(&Signal) + Send + Sync + 'static,
    {
        // Mock doesn't deliver — just records
    }

    pub fn assert_emitted(&self, signal_type: &str) {
        let emitted = self.emitted.lock().unwrap();
        assert!(
            emitted.iter().any(|s| s == signal_type),
            "signal not emitted: {} (emitted: {:?})",
            signal_type,
            *emitted
        );
    }

    pub fn emitted_count(&self) -> usize {
        self.emitted.lock().unwrap().len()
    }
}

impl Default for MockSignalBus {
    fn default() -> Self {
        Self::new()
    }
}