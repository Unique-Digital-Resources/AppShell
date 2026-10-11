//! NodeGraph-specific errors.

use crate::domain_foundation::environments::canvas_2d::CanvasError;
use super::edge::EdgeId;
use super::node::NodeId;
use super::port::PortId;
use std::fmt;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeGraphError {
    NodeNotFound { id: NodeId },
    PortNotFound { id: PortId },
    EdgeNotFound { id: EdgeId },
    /// The node has no canvas object (no spatial placement).
    NoCanvasObject { id: NodeId },
    /// The source port is not an output port.
    SourceNotOutput { id: PortId },
    /// The target port is not an input port.
    TargetNotInput { id: PortId },
    /// Cannot connect ports on the same node.
    SameNode,
    /// Port type names don't match.
    TypeMismatch { source_type: String, target_type: String },
    /// An edge already exists between these ports.
    DuplicateEdge { source: PortId, target: PortId },
    /// A Canvas2D operation failed.
    CanvasError { source: CanvasError },
    /// The graph contains a cycle; topological sort failed.
    CycleInGraph,
    /// The target input port already has the maximum number of connections.
    MaxInputsExceeded { port: PortId, max: u32 },
}

impl From<CanvasError> for NodeGraphError {
    fn from(source: CanvasError) -> Self {
        Self::CanvasError { source }
    }
}

impl fmt::Display for NodeGraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeGraphError::NodeNotFound { id } => write!(f, "node {id} not found"),
            NodeGraphError::PortNotFound { id } => write!(f, "port {id} not found"),
            NodeGraphError::EdgeNotFound { id } => write!(f, "edge {id} not found"),
            NodeGraphError::NoCanvasObject { id } => write!(f, "node {id} has no canvas object"),
            NodeGraphError::SourceNotOutput { id } => write!(f, "source port {id} is not an output"),
            NodeGraphError::TargetNotInput { id } => write!(f, "target port {id} is not an input"),
            NodeGraphError::SameNode => write!(f, "cannot connect ports on the same node"),
            NodeGraphError::TypeMismatch { source_type, target_type } => {
                write!(f, "type mismatch: {source_type} ≠ {target_type}")
            }
            NodeGraphError::DuplicateEdge { source, target } => {
                write!(f, "edge already exists: {source} → {target}")
            }
            NodeGraphError::CanvasError { source } => write!(f, "canvas error: {source}"),
            NodeGraphError::CycleInGraph => write!(f, "graph contains a cycle"),
            NodeGraphError::MaxInputsExceeded { port, max } => {
                write!(f, "input port {port} already has {max} connections")
            }
        }
    }
}

impl std::error::Error for NodeGraphError {}