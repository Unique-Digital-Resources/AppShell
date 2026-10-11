//! `AsyncCommandHandler` — async counterpart to `CommandHandler`.

use std::pin::Pin;

use crate::app_engine::capabilities::engine_ref::EngineRef;
use crate::app_engine::commands::command::CommandInput;
use crate::app_engine::commands::command_context::CommandContext;
use crate::app_engine::commands::command_result::CommandResult;
use crate::app_engine::errors::command_error::CommandError;

pub type AsyncCommandFuture = Pin<Box<dyn std::future::Future<Output = Result<CommandResult, CommandError>> + Send>>;

pub trait AsyncCommandHandler: Send + Sync {
    fn execute(
        &self,
        input: &CommandInput,
        context: &CommandContext,
        engine: &EngineRef,
    ) -> AsyncCommandFuture;
}