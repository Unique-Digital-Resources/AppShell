//! Relation identity and top-level model.

use super::endpoint::Endpoint;
use std::fmt;

/// Stable, opaque identifier for a registered relation.
///
/// Issued by [`RelationRegistry`](super::registry::RelationRegistry) when a
/// definition is registered. Not constructible externally.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RelationId(u64);

impl RelationId {
    /// Construct from a raw value. Intended for tests and bridging from
    /// external identity schemes.
    pub fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    pub fn as_raw(self) -> u64 {
        self.0
    }
}

impl fmt::Display for RelationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rel#{}", self.0)
    }
}

/// A relation instance: an identity paired with its source and target
/// endpoints.
///
/// This is the lightweight, value-type view stored after a
/// [`RelationDefinition`](super::definition::RelationDefinition) is
/// registered. The full definition (including the effect closure) is kept
/// inside the [`RelationRegistry`](super::registry::RelationRegistry) and
/// looked up by id during evaluation.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Relation {
    pub id: RelationId,
    pub source: Endpoint,
    pub target: Endpoint,
}

impl Relation {
    pub fn new(id: RelationId, source: Endpoint, target: Endpoint) -> Self {
        Self { id, source, target }
    }
}