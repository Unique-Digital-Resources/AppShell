//! `CommandHandler` — contract for consuming commands.
//!
//! Phase 13: handlers now receive `&EngineRef` for controlled engine access.

use crate::app_engine::capabilities::engine_ref::EngineRef;
use crate::app_engine::commands::command::CommandInput;
use crate::app_engine::commands::command_context::CommandContext;
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