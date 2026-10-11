//! Layers-specific errors.

use super::layer::LayerId;
use std::fmt;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayersError {
    LayerNotFound { id: LayerId },
    InvalidSiblingIndex { index: usize, sibling_count: usize },
    /// Reparenting would create a cycle (new parent is the layer itself
    /// or a descendant of the layer being moved).
    CycleDetected { id: LayerId, new_parent: LayerId },
}

impl fmt::Display for LayersError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LayersError::LayerNotFound { id } => write!(f, "layer {id} not found"),
            LayersError::InvalidSiblingIndex { index, sibling_count } => {
                write!(f, "sibling index {index} out of range (have {sibling_count} siblings)")
            }
            LayersError::CycleDetected { id, new_parent } => {
                write!(f, "cannot reparent {id} under {new_parent}: would create a cycle")
            }
        }
    }
}

impl std::error::Error for LayersError {}