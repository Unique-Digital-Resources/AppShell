//! Layers operations: value-type intents for hierarchy changes.

use super::error::LayersError;
use super::hierarchy::Layers;
use super::layer::{Layer, LayerId};

/// A layers operation: a value describing a single hierarchy-level change.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum LayersOperation {
    /// Create a root-level layer.
    CreateLayer { name: String },
    /// Create a child of an existing layer.
    CreateChild { parent: LayerId, name: String },
    /// Remove a layer and all its descendants.
    RemoveLayer { id: LayerId },
    /// Rename a layer.
    RenameLayer { id: LayerId, name: String },
    /// Show or hide a layer.
    SetVisibility { id: LayerId, visible: bool },
    /// Lock or unlock a layer.
    SetLocked { id: LayerId, locked: bool },
    /// Move a layer within its siblings.
    ReorderSibling { id: LayerId, new_index: usize },
    /// Move a layer to a different parent. `None` makes it a root.
    Reparent { id: LayerId, new_parent: Option<LayerId> },
    /// Remove all layers.
    Clear,
    /// Duplicate a layer as a new root.
    DuplicateLayer { id: LayerId },
}

/// Outcome of a successful [`LayersOperation`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum LayersOutcome {
    LayerCreated { id: LayerId },
    /// All removed layers (the layer plus descendants, in BFS order)
    /// for undo re-insertion.
    LayerRemoved { layers: Vec<Layer> },
    LayerRenamed { id: LayerId, previous_name: String },
    VisibilityChanged { id: LayerId, previous_visible: bool },
    LockedChanged { id: LayerId, previous_locked: bool },
    SiblingReordered { id: LayerId, previous_index: usize },
    Reparented { id: LayerId, previous_parent: Option<LayerId> },
    /// All layers removed.
    Cleared,
    /// Layer was duplicated. `new_id` is the new root layer's id.
    LayerDuplicated { source: LayerId, new_id: LayerId },
}

impl LayersOperation {
    pub fn apply(self, layers: &mut Layers) -> Result<LayersOutcome, LayersError> {
        match self {
            LayersOperation::CreateLayer { name } => {
                let id = layers.create_layer(name);
                Ok(LayersOutcome::LayerCreated { id })
            }
            LayersOperation::CreateChild { parent, name } => {
                let id = layers.create_child(parent, name)?;
                Ok(LayersOutcome::LayerCreated { id })
            }
            LayersOperation::RemoveLayer { id } => {
                let removed = layers.remove_layer(id)?;
                Ok(LayersOutcome::LayerRemoved { layers: removed })
            }
            LayersOperation::RenameLayer { id, name } => {
                let previous_name = layers.rename(id, name)?;
                Ok(LayersOutcome::LayerRenamed { id, previous_name })
            }
            LayersOperation::SetVisibility { id, visible } => {
                let previous_visible = layers.set_visibility(id, visible)?;
                Ok(LayersOutcome::VisibilityChanged { id, previous_visible })
            }
            LayersOperation::SetLocked { id, locked } => {
                let previous_locked = layers.set_locked(id, locked)?;
                Ok(LayersOutcome::LockedChanged { id, previous_locked })
            }
            LayersOperation::ReorderSibling { id, new_index } => {
                let previous_index = layers.reorder_sibling(id, new_index)?;
                Ok(LayersOutcome::SiblingReordered { id, previous_index })
            }
            LayersOperation::Reparent { id, new_parent } => {
                let previous_parent = layers.reparent(id, new_parent)?;
                Ok(LayersOutcome::Reparented { id, previous_parent })
            }
						LayersOperation::Clear => {
								layers.clear();
								Ok(LayersOutcome::Cleared)
						}
            LayersOperation::DuplicateLayer { id } => {
                let new_id = layers.duplicate_layer(id)?;
                Ok(LayersOutcome::LayerDuplicated { source: id, new_id })
            }
        }
    }
}