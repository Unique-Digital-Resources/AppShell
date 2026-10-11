//! 3D object identity and representation.

use super::transform::Transform3D;
use std::fmt;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectId3D(u64);

impl ObjectId3D {
    pub fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    pub fn as_raw(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ObjectId3D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "obj3d#{}", self.0)
    }
}

/// A 3D scene object: identity, spatial transform, and optional parent.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Object3D {
    pub id: ObjectId3D,
    pub transform: Transform3D,
    /// Parent object for transform hierarchy. `None` = root.
    pub parent: Option<ObjectId3D>,
}

impl Object3D {
    pub fn new(id: ObjectId3D, transform: Transform3D) -> Self {
        Self { id, transform, parent: None }
    }

    pub fn with_parent(mut self, parent: ObjectId3D) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }
}