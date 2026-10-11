//! Viewport3D operations: value-type intents for 3D spatial changes.

use super::error::Viewport3DError;
use super::object::{Object3D, ObjectId3D};
use super::scene::Scene;
use super::transform::{Transform3D, Vec3};

/// A viewport3d operation: a value describing a single 3D spatial change.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum Viewport3DOperation {
    /// Create a new object with the given transform.
    Create { transform: Transform3D },
    /// Set an object's translation, preserving rotation and scale.
    Move { id: ObjectId3D, translation: Vec3 },
    /// Set an object's rotation (Euler angles), preserving translation and scale.
    Rotate { id: ObjectId3D, rotation: Vec3 },
    /// Set an object's scale, preserving translation and rotation.
    Scale { id: ObjectId3D, scale: Vec3 },
    /// Replace an object's transform wholesale.
    Transform { id: ObjectId3D, transform: Transform3D },
    /// Remove an object.
    Remove { id: ObjectId3D },

    /// Set an object's parent for transform hierarchy.
    SetParent { id: ObjectId3D, parent: Option<ObjectId3D> },
    /// Remove all objects.
    Clear,
}

/// Outcome of a successful [`Viewport3DOperation`].
///
/// Each variant carries enough state to construct the inverse for undo.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum Viewport3DOutcome {
    Created { id: ObjectId3D },
    Moved { id: ObjectId3D, previous: Transform3D },
    Rotated { id: ObjectId3D, previous: Transform3D },
    Scaled { id: ObjectId3D, previous: Transform3D },
    Transformed { id: ObjectId3D, previous: Transform3D },
    Removed { object: Object3D },
    /// Parent was set. `previous_parent` is the old parent.
    ParentSet { id: ObjectId3D, previous_parent: Option<ObjectId3D> },
    /// All objects removed. `count` is the number removed.
    Cleared { count: usize },
}

impl Viewport3DOperation {
    pub fn apply(self, scene: &mut Scene) -> Result<Viewport3DOutcome, Viewport3DError> {
        match self {
            Viewport3DOperation::Create { transform } => {
                let id = scene.create_object(transform);
                Ok(Viewport3DOutcome::Created { id })
            }
            Viewport3DOperation::Move { id, translation } => {
                let previous = scene.set_translation(id, translation)?;
                Ok(Viewport3DOutcome::Moved { id, previous })
            }
            Viewport3DOperation::Rotate { id, rotation } => {
                let previous = scene.set_rotation(id, rotation)?;
                Ok(Viewport3DOutcome::Rotated { id, previous })
            }
            Viewport3DOperation::Scale { id, scale } => {
                let previous = scene.set_scale(id, scale)?;
                Ok(Viewport3DOutcome::Scaled { id, previous })
            }
            Viewport3DOperation::Transform { id, transform } => {
                let previous = scene.set_transform(id, transform)?;
                Ok(Viewport3DOutcome::Transformed { id, previous })
            }
            Viewport3DOperation::Remove { id } => {
                let object = scene.remove_object(id)?;
                Ok(Viewport3DOutcome::Removed { object })
            }

						Viewport3DOperation::SetParent { id, parent } => {
								let previous_parent = scene.get(id)
								    .ok_or(Viewport3DError::ObjectNotFound { id })?
								    .parent;
								scene.set_parent(id, parent)?;
								Ok(Viewport3DOutcome::ParentSet { id, previous_parent })
						}
						Viewport3DOperation::Clear => {
								let count = scene.len();
								scene.clear();
								Ok(Viewport3DOutcome::Cleared { count })
						}

        }
    }
}