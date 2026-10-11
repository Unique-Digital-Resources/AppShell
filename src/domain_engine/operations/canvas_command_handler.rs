//! Canvas command handler: bridges the App Engine's command system
//! with Canvas2D operations.

use crate::app_engine::capabilities::engine_ref::EngineRef;
use crate::app_engine::commands::command::CommandInput;
use crate::app_engine::commands::command_context::CommandContext;
use crate::app_engine::commands::command_definition::CommandDefinition;
use crate::app_engine::commands::command_executor::CommandHandler;
use crate::app_engine::commands::command_result::CommandResult;
use crate::app_engine::errors::command_error::CommandError;
use crate::app_engine::events::event::Event;
use crate::app_engine::history::history_entry::HistoryEntry;
use crate::domain_foundation::environments::canvas_2d::{
    Canvas, CanvasOperation, CanvasOutcome,
};
use std::sync::{Arc, Mutex};

pub struct CanvasCommandHandler {
    canvas: Arc<Mutex<Canvas>>,
}

impl CanvasCommandHandler {
    pub fn new(canvas: Arc<Mutex<Canvas>>) -> Self {
        Self { canvas }
    }

    pub fn definition() -> CommandDefinition {
        CommandDefinition::new(
            "canvas.operation",
            "Canvas Operation",
            "Applies a CanvasOperation to the 2D canvas environment.",
        )
    }
}

impl CommandHandler for CanvasCommandHandler {
    fn execute(
        &self,
        input: &CommandInput,
        _context: &CommandContext,
        engine: &EngineRef,
    ) -> Result<CommandResult, CommandError> {
        let operation = input
            .get::<CanvasOperation>()
            .ok_or_else(|| {
                CommandError::ExecutionFailed(
                    "expected CanvasOperation in CommandInput".to_string(),
                )
            })?
            .clone();

        let outcome = {
            let mut canvas = self
                .canvas
                .lock()
                .map_err(|e| CommandError::ExecutionFailed(format!("canvas lock: {e}")))?;
            operation
                .clone()
                .apply(&mut canvas)
                .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?
        };

        let entry_name = match &operation {
            CanvasOperation::Create { .. } => "canvas.create",
            CanvasOperation::Move { .. } => "canvas.move",
            CanvasOperation::Transform { .. } => "canvas.transform",
            CanvasOperation::Remove { .. } => "canvas.remove",
            CanvasOperation::MoveToFront { .. } => "canvas.move_to_front",
            CanvasOperation::MoveToBack { .. } => "canvas.move_to_back",
            CanvasOperation::MoveForward { .. } => "canvas.move_forward",
            CanvasOperation::MoveBackward { .. } => "canvas.move_backward",
            CanvasOperation::Reorder { .. } => "canvas.reorder",
            CanvasOperation::Duplicate { .. } => "canvas.duplicate",
            CanvasOperation::Clear => "canvas.clear",
        };
        engine.history_store.append(
            HistoryEntry::new(entry_name).undoable(),
        );

        let event_name = match &outcome {
            CanvasOutcome::Created { .. } => "canvas.object.created",
            CanvasOutcome::Moved { .. } => "canvas.object.moved",
            CanvasOutcome::Transformed { .. } => "canvas.object.transformed",
            CanvasOutcome::Removed { .. } => "canvas.object.removed",
            CanvasOutcome::MovedToFront { .. } => "canvas.object.moved_to_front",
            CanvasOutcome::MovedToBack { .. } => "canvas.object.moved_to_back",
            CanvasOutcome::MovedForward { .. } => "canvas.object.moved_forward",
            CanvasOutcome::MovedBackward { .. } => "canvas.object.moved_backward",
            CanvasOutcome::Reordered { .. } => "canvas.object.reordered",
            CanvasOutcome::Duplicated { .. } => "canvas.object.duplicated",
            CanvasOutcome::Cleared { .. } => "canvas.cleared",
        };
        engine.event_bus.publish(&Event::new(event_name).with_source("canvas_command_handler"));

        Ok(CommandResult::success_with(outcome))
    }
}