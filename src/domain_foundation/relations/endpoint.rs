//! Endpoint addressing: which object and which property a relation touches.
//!
//! The foundation uses opaque identifiers so that the relation system does
//! not need to know what any property *means*. A Domain Engine issues
//! [`ObjectId`] and [`PropertyId`] values and passes them to the relation
//! registry; the foundation merely stores and matches them.

use std::fmt;

/// Stable, opaque identifier for an object within the relation system.
///
/// Objects are identified externally — the relation system does not own
/// object storage. A Domain Engine issues ids (typically derived from its
/// environment's identity, e.g. a Canvas2D [`ObjectId`](crate::environments::canvas_2d::ObjectId))
/// and passes them in.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectId(u64);

impl ObjectId {
    /// Construct from a raw value. Visible within the crate so Domain
    /// Engines can bridge from their own identity scheme.
    pub(crate) fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Construct from any u64 source.
    pub fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub fn as_raw(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "obj{}", self.0)
    }
}

/// Stable, opaque identifier for a property on an object.
///
/// A PropertyId is issued by the Domain Engine and has no inherent meaning
/// to the foundation. Two PropertyIds are equal only if they refer to the
/// same property on the same object.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PropertyId {
    pub object: ObjectId,
    pub property: u32,
}

impl PropertyId {
    pub fn new(object: ObjectId, property: u32) -> Self {
        Self { object, property }
    }

    pub fn object(self) -> ObjectId {
        self.object
    }

    pub fn property(self) -> u32 {
        self.property
    }
}

impl fmt::Display for PropertyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.p{}", self.object, self.property)
    }
}

/// One end of a relation: a property on an object, plus a role.
///
/// Source endpoints trigger evaluation when their property changes.
/// Target endpoints receive the effect produced by the relation's
/// [`EffectFn`](super::definition::EffectFn).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Endpoint {
    pub property: PropertyId,
    pub role: EndpointRole,
}

impl Endpoint {
    pub fn source(property: PropertyId) -> Self {
        Self { property, role: EndpointRole::Source }
    }

    pub fn target(property: PropertyId) -> Self {
        Self { property, role: EndpointRole::Target }
    }

    pub fn is_source(&self) -> bool {
        self.role == EndpointRole::Source
    }

    pub fn is_target(&self) -> bool {
        self.role == EndpointRole::Target
    }
}

/// The role an [`Endpoint`] plays in a relation.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EndpointRole {
    /// The property whose change triggers the relation.
    Source,
    /// The property that receives the relation's effect.
    Target,
}