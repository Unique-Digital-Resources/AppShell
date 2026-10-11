//! Layer identity and model.

use std::fmt;

/// Stable, opaque identifier for a layer within a [`Layers`](super::hierarchy::Layers) hierarchy.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LayerId(u64);

impl LayerId {
    pub fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    pub fn as_raw(self) -> u64 {
        self.0
    }
}

impl fmt::Display for LayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "layer{}", self.0)
    }
}

/// A layer: a node in a hierarchy with name, visibility and lock state.
///
/// The foundation attaches no compositing-specific properties (opacity,
/// blend mode, masks). Domain Engines layer those externally, keyed by
/// [`LayerId`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    pub id: LayerId,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub parent: Option<LayerId>,
    /// Direct children of this layer, in sibling order.
    /// `pub(crate)` so the [`Layers`](super::hierarchy::Layers) manager
    /// can modify it; external code reads via [`children()`](Self::children).
    pub(crate) children: Vec<LayerId>,
}

impl Layer {
    pub fn new(id: LayerId, name: String) -> Self {
        Self {
            id,
            name,
            visible: true,
            locked: false,
            parent: None,
            children: Vec::new(),
        }
    }

    /// Direct children of this layer, in sibling order.
    pub fn children(&self) -> &[LayerId] {
        &self.children
    }

    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    pub fn child_count(&self) -> usize {
        self.children.len()
    }
}