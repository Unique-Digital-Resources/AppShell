//! Canvas domain operations: wrappers around Canvas2D operations that
//! carry undo/redo information and integrate with App Engine history
//! and events.

use crate::app_engine::{Event, HistoryEntry};
use crate::domain_foundation::environments::canvas_2d::{
    Canvas, CanvasError, CanvasOperation, CanvasOutcome, Object, ObjectId,
};
use crate::domain_foundation::environments::canvas_2d::Vec2;

#[derive(Debug, Clone)]
pub struct ExecutedCanvasOperation {
    operation: CanvasOperation,
    outcome: CanvasOutcome,
}

impl ExecutedCanvasOperation {
    pub fn execute(
        canvas: &mut Canvas,
        operation: CanvasOperation,
    ) -> Result<Self, CanvasError> {
        let outcome = operation.clone().apply(canvas)?;
        Ok(Self { operation, outcome })
    }

    pub fn outcome(&self) -> CanvasOutcome {
        self.outcome.clone()
    }

    pub fn operation(&self) -> &CanvasOperation {
        &self.operation
    }

    pub fn undo(&self, canvas: &mut Canvas) -> Result<(), CanvasError> {
        match &self.outcome {
            CanvasOutcome::Created { id } => {
                canvas.remove_object(*id)?;
            }
            CanvasOutcome::Moved { id, previous } => {
                canvas.set_transform(*id, *previous)?;
            }
            CanvasOutcome::Transformed { id, previous } => {
                canvas.set_transform(*id, *previous)?;
            }
            CanvasOutcome::Removed { object } => {
                canvas.insert_object(*object);
            }
						CanvasOutcome::MovedToFront { id, previous_index } => {
								canvas.reorder(*id, *previous_index)?;
						}
						CanvasOutcome::MovedToBack { id, previous_index } => {
								canvas.reorder(*id, *previous_index)?;
						}
            CanvasOutcome::MovedForward { id, previous_index } => {
                canvas.reorder(*id, *previous_index)?;
            }
            CanvasOutcome::MovedBackward { id, previous_index } => {
                canvas.reorder(*id, *previous_index)?;
            }
            CanvasOutcome::Reordered { id, previous_index } => {
                canvas.reorder(*id, *previous_index)?;
            }
            CanvasOutcome::Duplicated { new_id, .. } => {
                canvas.remove_object(*new_id)?;
            }
            CanvasOutcome::Cleared { .. } => {}
        }
        Ok(())
    }

    pub fn redo(&self, canvas: &mut Canvas) -> Result<CanvasOutcome, CanvasError> {
        match &self.operation {
            CanvasOperation::Create { transform } => {
                let id = match &self.outcome {
                    CanvasOutcome::Created { id } => *id,
                    _ => unreachable!(),
                };
                canvas.insert_object(Object::new(id, *transform));
                Ok(self.outcome.clone())
            }
            _ => self.operation.clone().apply(canvas),
        }
    }

    pub fn history_entry(&self) -> HistoryEntry {
        let name = match &self.operation {
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
        HistoryEntry::new(name).undoable()
    }

    pub fn event(&self) -> Event {
        let event_name = match &self.operation {
            CanvasOperation::Create { .. } => "canvas.object.created",
            CanvasOperation::Move { .. } => "canvas.object.moved",
            CanvasOperation::Transform { .. } => "canvas.object.transformed",
            CanvasOperation::Remove { .. } => "canvas.object.removed",
            CanvasOperation::MoveToFront { .. } => "canvas.object.moved_to_front",
            CanvasOperation::MoveToBack { .. } => "canvas.object.moved_to_back",
            CanvasOperation::MoveForward { .. } => "canvas.object.moved_forward",
            CanvasOperation::MoveBackward { .. } => "canvas.object.moved_backward",
            CanvasOperation::Reorder { .. } => "canvas.object.reordered",
            CanvasOperation::Duplicate { .. } => "canvas.object.duplicated",
            CanvasOperation::Clear => "canvas.cleared",
        };
        Event::new(event_name)
    }
}