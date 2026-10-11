//! Canvas-specific errors.

use super::object::ObjectId;
use std::fmt;

/// Errors produced by canvas operations.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanvasError {
    /// No object exists in the canvas with the given id.
    ObjectNotFound { id: ObjectId },
}

impl fmt::Display for CanvasError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CanvasError::ObjectNotFound { id } => {
                write!(f, "no canvas object with id {id}")
            }
        }
    }
}

impl std::error::Error for CanvasError {}