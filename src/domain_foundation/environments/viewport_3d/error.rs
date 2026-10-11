//! Viewport3D-specific errors.

use super::object::ObjectId3D;
use std::fmt;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Viewport3DError {
    /// No object exists in the scene with the given id.
    ObjectNotFound { id: ObjectId3D },
    /// Attempted to set an object as its own parent.
    CannotBeOwnParent { id: ObjectId3D },
    /// Attempted to set a parent that doesn't exist.
    ParentNotFound { id: ObjectId3D },
}

impl fmt::Display for Viewport3DError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Viewport3DError::ObjectNotFound { id } => {
                write!(f, "no scene object with id {id}")
            }
            Viewport3DError::CannotBeOwnParent { id } => {
                write!(f, "object {id} cannot be its own parent")
            }
            Viewport3DError::ParentNotFound { id } => {
                write!(f, "parent object {id} not found")
            }
        }
    }
}

impl std::error::Error for Viewport3DError {}