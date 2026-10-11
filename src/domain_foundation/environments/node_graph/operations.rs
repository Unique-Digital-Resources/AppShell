//! NodeGraph operations: value-type intents for graph changes.

use super::edge::Edge;
use super::error::NodeGraphError;
use super::graph::{NodeGraph, NodeRemovedData, PortRemovedData};
use super::node::NodeId;
use super::port::PortId;
use crate::domain_foundation::environments::canvas_2d::{Object, Transform, Vec2};
use super::edge::EdgeId;

/// A node graph operation.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum NodeGraphOperation {
    CreateNode { name: String, transform: Transform },
    RemoveNode { id: NodeId },
    RenameNode { id: NodeId, name: String },
    MoveNode { id: NodeId, translation: Vec2 },
    SetNodeTransform { id: NodeId, transform: Transform },
    AddInputPort { node: NodeId, name: String, type_name: String },
    AddOutputPort { node: NodeId, name: String, type_name: String },
    RemovePort { id: PortId },
    Connect { source: PortId, target: PortId },
    Disconnect { id: EdgeId },
    /// Rename a port.
    RenamePort { id: PortId, name: String },
    /// Duplicate a node (without ports or edges).
    DuplicateNode { id: NodeId },
    /// Remove all nodes, ports, edges, and canvas objects.
    Clear,
}

/// Outcome of a successful [`NodeGraphOperation`].
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum NodeGraphOutcome {
    NodeCreated { id: NodeId },
    NodeRemoved(NodeRemovedData),
    NodeRenamed { id: NodeId, previous_name: String },
    NodeMoved { id: NodeId, previous_transform: Transform },
    NodeTransformSet { id: NodeId, previous_transform: Transform },
    PortAdded { id: PortId },
    PortRemoved(PortRemovedData),
    Connected { id: EdgeId },
    Disconnected { edge: Edge },
    PortRenamed { id: PortId, previous_name: String },
    NodeDuplicated { source: NodeId, new_id: NodeId },
    Cleared { node_count: usize },
}

impl NodeGraphOperation {
    pub fn apply(self, graph: &mut NodeGraph) -> Result<NodeGraphOutcome, NodeGraphError> {
        match self {
            NodeGraphOperation::CreateNode { name, transform } => {
                let id = graph.create_node(name, transform);
                Ok(NodeGraphOutcome::NodeCreated { id })
            }
            NodeGraphOperation::RemoveNode { id } => {
                let data = graph.remove_node(id)?;
                Ok(NodeGraphOutcome::NodeRemoved(data))
            }
            NodeGraphOperation::RenameNode { id, name } => {
                let previous_name = graph.rename_node(id, name)?;
                Ok(NodeGraphOutcome::NodeRenamed { id, previous_name })
            }
            NodeGraphOperation::MoveNode { id, translation } => {
                let previous_transform = graph.move_node(id, translation)?;
                Ok(NodeGraphOutcome::NodeMoved { id, previous_transform })
            }
            NodeGraphOperation::SetNodeTransform { id, transform } => {
                let previous_transform = graph.set_node_transform(id, transform)?;
                Ok(NodeGraphOutcome::NodeTransformSet { id, previous_transform })
            }
            NodeGraphOperation::AddInputPort { node, name, type_name } => {
                let id = graph.add_input_port(node, name, type_name)?;
                Ok(NodeGraphOutcome::PortAdded { id })
            }
            NodeGraphOperation::AddOutputPort { node, name, type_name } => {
                let id = graph.add_output_port(node, name, type_name)?;
                Ok(NodeGraphOutcome::PortAdded { id })
            }
            NodeGraphOperation::RemovePort { id } => {
                let data = graph.remove_port(id)?;
                Ok(NodeGraphOutcome::PortRemoved(data))
            }
            NodeGraphOperation::Connect { source, target } => {
                let id = graph.connect(source, target)?;
                Ok(NodeGraphOutcome::Connected { id })
            }
            NodeGraphOperation::Disconnect { id } => {
                let edge = graph.disconnect(id)?;
                Ok(NodeGraphOutcome::Disconnected { edge })
            }
						NodeGraphOperation::RenamePort { id, name } => {
								let previous_name = graph.rename_port(id, name)?;
								Ok(NodeGraphOutcome::PortRenamed { id, previous_name })
						}
						NodeGraphOperation::DuplicateNode { id } => {
								let new_id = graph.duplicate_node(id)?;
								Ok(NodeGraphOutcome::NodeDuplicated { source: id, new_id })
						}
						NodeGraphOperation::Clear => {
								let count = graph.node_count();
								graph.clear();
								Ok(NodeGraphOutcome::Cleared { node_count: count })
						}
        }
    }
}