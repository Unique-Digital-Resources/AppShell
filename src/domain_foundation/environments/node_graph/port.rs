//! Port identity and model.

use super::node::NodeId;
use std::fmt;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PortId(u64);

impl PortId {
    pub fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    pub fn as_raw(self) -> u64 {
        self.0
    }
}

impl fmt::Display for PortId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "port{}", self.0)
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortDirection {
    Input,
    Output,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Port {
    pub id: PortId,
    pub node: NodeId,
    pub direction: PortDirection,
    pub name: String,
    pub type_name: String,
    /// Maximum number of input connections. 0 = unlimited.
    /// Only meaningful for input ports; ignored for output ports.
    pub max_inputs: u32,
}

impl Port {
    pub fn new(
        id: PortId,
        node: NodeId,
        direction: PortDirection,
        name: String,
        type_name: String,
    ) -> Self {
        Self {
            id,
            node,
            direction,
            name,
            type_name,
            max_inputs: 0,
        }
    }

    pub fn with_max_inputs(mut self, max: u32) -> Self {
        self.max_inputs = max;
        self
    }

    pub fn is_input(&self) -> bool {
        self.direction == PortDirection::Input
    }

    pub fn is_output(&self) -> bool {
        self.direction == PortDirection::Output
    }
}