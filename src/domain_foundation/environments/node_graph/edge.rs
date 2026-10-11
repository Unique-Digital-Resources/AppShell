//! Edge identity and model.

use super::port::PortId;
use std::fmt;

/// Stable, opaque identifier for an edge.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EdgeId(u64);

impl EdgeId {
    pub fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    pub fn as_raw(self) -> u64 {
        self.0
    }
}

impl fmt::Display for EdgeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "edge{}", self.0)
    }
}

/// An edge: a directed connection from an output port to an input port.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pub id: EdgeId,
    pub source: PortId,
    pub target: PortId,
}

impl Edge {
    pub fn new(id: EdgeId, source: PortId, target: PortId) -> Self {
        Self { id, source, target }
    }
}