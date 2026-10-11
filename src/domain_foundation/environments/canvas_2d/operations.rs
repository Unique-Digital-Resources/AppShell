//! Canvas operations: value-type intents that describe *what* a change means.

use super::canvas::Canvas;
use super::error::CanvasError;
use super::object::{Object, ObjectId};
use super::transform::{Transform, Vec2};

/// A canvas operation: a value describing a single canvas-level change.
#[derive(Debug, Clone, PartialEq)]
pub enum CanvasOperation {
    Create { transform: Transform },
    Move { id: ObjectId, translation: Vec2 },
    Transform { id: ObjectId, transform: Transform },
    Remove { id: ObjectId },
    MoveToFront { id: ObjectId },
    MoveToBack { id: ObjectId },
    /// Move an object forward by one position in the draw order.
    MoveForward { id: ObjectId },
    /// Move an object backward by one position in the draw order.
    MoveBackward { id: ObjectId },
    /// Reorder an object to a specific index in the draw order.
    Reorder { id: ObjectId, new_index: usize },
    /// Duplicate an object at the given offset.
    Duplicate { id: ObjectId, offset: Vec2 },
    /// Remove all objects.
    Clear,
}

/// Outcome of a successful [`CanvasOperation`].
#[derive(Debug, Clone, PartialEq)]
pub enum CanvasOutcome {
    Created { id: ObjectId },
    Moved { id: ObjectId, previous: Transform },
    Transformed { id: ObjectId, previous: Transform },
    Removed { object: Object },
    MovedToFront { id: ObjectId, previous_index: usize },
    MovedToBack { id: ObjectId, previous_index: usize },
    /// Moved forward by one position. `previous_index` is the index before the move.
    MovedForward { id: ObjectId, previous_index: usize },
    /// Moved backward by one position. `previous_index` is the index before the move.
    MovedBackward { id: ObjectId, previous_index: usize },
    /// Reordered to `new_index`. `previous_index` is the index before the move.
    Reordered { id: ObjectId, previous_index: usize },
    /// Duplicated. The new object's id is returned.
    Duplicated { source: ObjectId, new_id: ObjectId },
    /// All objects removed. `count` is the number removed.
    Cleared { count: usize },
}

impl CanvasOperation {
    pub fn apply(self, canvas: &mut Canvas) -> Result<CanvasOutcome, CanvasError> {
        match self {
            CanvasOperation::Create { transform } => {
                let id = canvas.create_object(transform);
                Ok(CanvasOutcome::Created { id })
            }
            CanvasOperation::Move { id, translation } => {
                let previous = canvas.set_translation(id, translation)?;
                Ok(CanvasOutcome::Moved { id, previous })
            }
            CanvasOperation::Transform { id, transform } => {
                let previous = canvas.set_transform(id, transform)?;
                Ok(CanvasOutcome::Transformed { id, previous })
            }
            CanvasOperation::Remove { id } => {
                let object = canvas.remove_object(id)?;
                Ok(CanvasOutcome::Removed { object })
            }
            CanvasOperation::MoveToFront { id } => {
                let current = canvas.draw_order().iter()
                    .position(|oid| *oid == id)
                    .ok_or(CanvasError::ObjectNotFound { id })?;
                canvas.move_to_front(id)?;
                Ok(CanvasOutcome::MovedToFront { id, previous_index: current })
            }
            CanvasOperation::MoveToBack { id } => {
                let current = canvas.draw_order().iter()
                    .position(|oid| *oid == id)
                    .ok_or(CanvasError::ObjectNotFound { id })?;
                canvas.move_to_back(id)?;
                Ok(CanvasOutcome::MovedToBack { id, previous_index: current })
            }
            CanvasOperation::MoveForward { id } => {
                let current = canvas.draw_order().iter()
                    .position(|oid| *oid == id)
                    .ok_or(CanvasError::ObjectNotFound { id })?;
                canvas.move_forward(id)?;
                Ok(CanvasOutcome::MovedForward { id, previous_index: current })
            }
            CanvasOperation::MoveBackward { id } => {
                let current = canvas.draw_order().iter()
                    .position(|oid| *oid == id)
                    .ok_or(CanvasError::ObjectNotFound { id })?;
                canvas.move_backward(id)?;
                Ok(CanvasOutcome::MovedBackward { id, previous_index: current })
            }
            CanvasOperation::Reorder { id, new_index } => {
                let current = canvas.draw_order().iter()
                    .position(|oid| *oid == id)
                    .ok_or(CanvasError::ObjectNotFound { id })?;
                canvas.reorder(id, new_index)?;
                Ok(CanvasOutcome::Reordered { id, previous_index: current })
            }
            CanvasOperation::Duplicate { id, offset } => {
                let new_id = canvas.duplicate_object(id, offset)?;
                Ok(CanvasOutcome::Duplicated { source: id, new_id })
            }
            CanvasOperation::Clear => {
                let count = canvas.len();
                canvas.clear();
                Ok(CanvasOutcome::Cleared { count })
            }
        }
    }
}