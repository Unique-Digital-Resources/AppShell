use crate::app_engine::capabilities::engine_ref::EngineRef;
use crate::app_engine::commands::command::CommandInput;
use crate::app_engine::commands::command_context::CommandContext;
use crate::app_engine::commands::command_definition::CommandDefinition;
use crate::app_engine::commands::command_executor::CommandHandler;
use crate::app_engine::commands::command_result::CommandResult;
use crate::app_engine::errors::command_error::CommandError;
use crate::app_engine::events::event::Event;
use crate::app_engine::history::history_entry::HistoryEntry;
use crate::domain_foundation::environments::timeline::{Timeline, TimelineOperation};
use std::sync::{Arc, Mutex};

pub struct TimelineCommandHandler {
    timeline: Arc<Mutex<Timeline>>,
}

impl TimelineCommandHandler {
    pub fn new(timeline: Arc<Mutex<Timeline>>) -> Self {
        Self { timeline }
    }
    pub fn definition() -> CommandDefinition {
        CommandDefinition::new(
            "timeline.operation", "Timeline Operation",
            "Applies a TimelineOperation to the timeline environment.",
        )
    }
}

impl CommandHandler for TimelineCommandHandler {
    fn execute(&self, input: &CommandInput, _ctx: &CommandContext, engine: &EngineRef)
        -> Result<CommandResult, CommandError>
    {
        let op = input.get::<TimelineOperation>()
            .ok_or_else(|| CommandError::ExecutionFailed("expected TimelineOperation".into()))?
            .clone();
        let outcome = {
            let mut tl = self.timeline.lock()
                .map_err(|e| CommandError::ExecutionFailed(format!("lock: {e}")))?;
            op.clone().apply(&mut tl)
                .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?
        };
        engine.history_store.append(HistoryEntry::new("timeline.operation").undoable());
        engine.event_bus.publish(&Event::new("timeline.operation").with_source("timeline_handler"));
        Ok(CommandResult::success_with(outcome))
    }
}