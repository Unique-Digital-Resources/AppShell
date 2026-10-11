//! Node identity and model.

use crate::domain_foundation::environments::canvas_2d::ObjectId;
use super::port::PortId;
use std::fmt;

/// Stable, opaque identifier for a node.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(u64);

impl NodeId {
    pub fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    pub fn as_raw(self) -> u64 {
        self.0
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "node{}", self.0)
    }
}

/// A node: a graph entity with optional spatial placement via Canvas2D.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub id: NodeId,
    pub name: String,
    /// Links this node to its spatial representation in the Canvas2D
    /// environment. `None` for purely logical nodes.
    pub canvas_object: Option<ObjectId>,
    pub(crate) input_ports: Vec<PortId>,
    pub(crate) output_ports: Vec<PortId>,
}

impl Node {
    pub fn new(id: NodeId, name: String, canvas_object: Option<ObjectId>) -> Self {
        Self {
            id,
            name,
            canvas_object,
            input_ports: Vec::new(),
            output_ports: Vec::new(),
        }
    }

    pub fn input_ports(&self) -> &[PortId] {
        &self.input_ports
    }

    pub fn output_ports(&self) -> &[PortId] {
        &self.output_ports
    }
}