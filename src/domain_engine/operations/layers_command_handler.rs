use crate::app_engine::capabilities::engine_ref::EngineRef;
use crate::app_engine::commands::command::CommandInput;
use crate::app_engine::commands::command_context::CommandContext;
use crate::app_engine::commands::command_definition::CommandDefinition;
use crate::app_engine::commands::command_executor::CommandHandler;
use crate::app_engine::commands::command_result::CommandResult;
use crate::app_engine::errors::command_error::CommandError;
use crate::app_engine::events::event::Event;
use crate::app_engine::history::history_entry::HistoryEntry;
use crate::domain_foundation::environments::layers::{Layers, LayersOperation};
use std::sync::{Arc, Mutex};

pub struct LayersCommandHandler {
    layers: Arc<Mutex<Layers>>,
}

impl LayersCommandHandler {
    pub fn new(layers: Arc<Mutex<Layers>>) -> Self { Self { layers } }
    pub fn definition() -> CommandDefinition {
        CommandDefinition::new(
            "layers.operation", "Layers Operation",
            "Applies a LayersOperation to the layers environment.",
        )
    }
}

impl CommandHandler for LayersCommandHandler {
    fn execute(&self, input: &CommandInput, _ctx: &CommandContext, engine: &EngineRef)
        -> Result<CommandResult, CommandError>
    {
        let op = input.get::<LayersOperation>()
            .ok_or_else(|| CommandError::ExecutionFailed("expected LayersOperation".into()))?
            .clone();
        let outcome = {
            let mut l = self.layers.lock()
                .map_err(|e| CommandError::ExecutionFailed(format!("lock: {e}")))?;
            op.clone().apply(&mut l)
                .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?
        };
        engine.history_store.append(HistoryEntry::new("layers.operation").undoable());
        engine.event_bus.publish(&Event::new("layers.operation").with_source("layers_handler"));
        Ok(CommandResult::success_with(outcome))
    }
}