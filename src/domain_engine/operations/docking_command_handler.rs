//! Docking command handler: bridges the App Engine's command system
//! with Docking operations. Like its siblings it appends a name-only
//! undoable entry without an engine-side replay mechanism — the
//! frontend-facing commit path for docking intent is the engine route
//! (`DomainEngine::execute_docking`), which carries real undo/redo and
//! publishes the per-operation events (Phase-9 note; the interaction
//! contract lives in the environment's module docs, PRD §10–§11).

use crate::app_engine::capabilities::engine_ref::EngineRef;
use crate::app_engine::commands::command::CommandInput;
use crate::app_engine::commands::command_context::CommandContext;
use crate::app_engine::commands::command_definition::CommandDefinition;
use crate::app_engine::commands::command_executor::CommandHandler;
use crate::app_engine::commands::command_result::CommandResult;
use crate::app_engine::errors::command_error::CommandError;
use crate::app_engine::events::event::Event;
use crate::app_engine::history::history_entry::HistoryEntry;
use crate::domain_foundation::environments::docking::{DockLayout, DockingOperation};
use std::sync::{Arc, Mutex};

pub struct DockingCommandHandler {
    docking: Arc<Mutex<DockLayout>>,
}

impl DockingCommandHandler {
    pub fn new(docking: Arc<Mutex<DockLayout>>) -> Self { Self { docking } }
    pub fn definition() -> CommandDefinition {
        CommandDefinition::new(
            "docking.operation", "Docking Operation",
            "Applies a DockingOperation to the docking environment.",
        )
    }
}

impl CommandHandler for DockingCommandHandler {
    fn execute(&self, input: &CommandInput, _ctx: &CommandContext, engine: &EngineRef)
        -> Result<CommandResult, CommandError>
    {
        let op = input.get::<DockingOperation>()
            .ok_or_else(|| CommandError::ExecutionFailed("expected DockingOperation".into()))?
            .clone();
        let outcome = {
            let mut d = self.docking.lock()
                .map_err(|e| CommandError::ExecutionFailed(format!("docking lock: {e}")))?;
            op.clone().apply(&mut d)
                .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?
        };
        engine.history_store.append(HistoryEntry::new("docking.operation").undoable());
        engine.event_bus.publish(&Event::new("docking.operation").with_source("docking_handler"));
        Ok(CommandResult::success_with(outcome))
    }
}
