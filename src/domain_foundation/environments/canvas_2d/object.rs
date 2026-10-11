//! Canvas object identity and representation.

use super::transform::Transform;
use std::fmt;

/// Stable, opaque identifier for an object within a
/// [`Canvas`](super::canvas::Canvas).
///
/// Identifiers are issued by the canvas on object creation and remain valid
/// for the lifetime of the object within that canvas. They are **not** array
/// indices: removing one object does not renumber the others.
///
/// External code cannot construct an `ObjectId` directly. It must obtain one
/// from `Canvas::create_object` or from a [`CanvasOutcome`](super::operations::CanvasOutcome).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectId(u64);

impl ObjectId {
    /// Construct an id from a raw counter value.
    ///
    /// Visible only within the `canvas_2d` environment. External callers
    /// must obtain ids from `Canvas::create_object`.
    pub fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Read the underlying counter value.
    ///
    /// Stable within a single canvas instance. Not stable across canvases
    /// or sessions — do not persist or compare across instances.
    pub fn as_raw(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "object#{}", self.0)
    }
}

/// A canvas object: an identity paired with a spatial [`Transform`].
///
/// The foundation attaches **no application-specific data** to objects.
/// Domain Engines layer their own data externally, keyed by [`ObjectId`].

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Object {
    pub id: ObjectId,
    pub transform: Transform,
}

impl Object {
    pub fn new(id: ObjectId, transform: Transform) -> Self {
        Self { id, transform }
    }
}