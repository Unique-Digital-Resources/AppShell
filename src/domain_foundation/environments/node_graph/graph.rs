//! NodeGraph model: owns nodes, ports, edges, and a Canvas2D for spatial
//! placement.
//!
//! This is where environment composition happens. The graph delegates
//! spatial state to [`Canvas`](crate::domain_foundation::environments::canvas_2d::Canvas)
//! and owns graph connectivity itself.

use super::edge::{Edge, EdgeId};
use super::error::NodeGraphError;
use super::node::{Node, NodeId};
use super::port::{Port, PortDirection, PortId};
use crate::domain_foundation::environments::canvas_2d::{
    Canvas, CanvasError, Object, ObjectId, Transform, Vec2,
};
use std::collections::{HashMap, HashSet};

/// Headless node graph with spatial placement via Canvas2D.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NodeGraph {
    nodes: HashMap<NodeId, Node>,
    ports: HashMap<PortId, Port>,
    edges: HashMap<EdgeId, Edge>,
    canvas: Canvas,
    next_node_id: u64,
    next_port_id: u64,
    next_edge_id: u64,
}

/// Data returned by [`NodeGraph::remove_node`] for undo.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct NodeRemovedData {
    pub node: Node,
    pub ports: Vec<Port>,
    pub edges: Vec<Edge>,
    pub canvas_object: Option<Object>,
}

/// Data returned by [`NodeGraph::remove_port`] for undo.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct PortRemovedData {
    pub port: Port,
    pub edges: Vec<Edge>,
}

impl NodeGraph {
    pub fn new() -> Self {
        Self::default()
    }

    // --- Access ---

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn port_count(&self) -> usize {
        self.ports.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn contains_node(&self, id: NodeId) -> bool {
        self.nodes.contains_key(&id)
    }

    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    pub fn port(&self, id: PortId) -> Option<&Port> {
        self.ports.get(&id)
    }

    pub fn edge(&self, id: EdgeId) -> Option<&Edge> {
        self.edges.get(&id)
    }

    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }

    pub fn ports(&self) -> impl Iterator<Item = &Port> {
        self.ports.values()
    }

    pub fn edges(&self) -> impl Iterator<Item = &Edge> {
        self.edges.values()
    }

    /// Borrow the internal Canvas2D (read-only).
    pub fn canvas(&self) -> &Canvas {
        &self.canvas
    }

    // --- Node management ---

    /// Create a node with spatial placement. A canvas object is created
    /// with the given transform and linked to the node.
    pub fn create_node(&mut self, name: String, transform: Transform) -> NodeId {
        let id = NodeId::from_raw(self.next_node_id);
        self.next_node_id += 1;

        let canvas_id = self.canvas.create_object(transform);
        let node = Node::new(id, name, Some(canvas_id));
        self.nodes.insert(id, node);
        id
    }

    /// Insert an existing node (for undo of RemoveNode). Restores the
    /// node, its canvas object, ports, and edges must be re-inserted
    /// separately via [`insert_port`](Self::insert_port) and
    /// [`insert_edge`](Self::insert_edge).
    pub fn insert_node(&mut self, node: Node, canvas_object: Option<Object>) -> NodeId {
        let id = node.id;
        if id.as_raw() >= self.next_node_id {
            self.next_node_id = id.as_raw() + 1;
        }
        if let Some(obj) = canvas_object {
            self.canvas.insert_object(obj);
        }
        self.nodes.insert(id, node);
        id
    }

    /// Remove a node and all its ports and connected edges. Also removes
    /// the canvas object. Returns all removed data for undo.
    pub fn remove_node(&mut self, id: NodeId) -> Result<NodeRemovedData, NodeGraphError> {
        let node = self.nodes.get(&id).ok_or(NodeGraphError::NodeNotFound { id })?;

        // Collect port ids.
        let port_ids: Vec<PortId> = node.input_ports.iter()
            .chain(node.output_ports.iter())
            .copied()
            .collect();
        let port_set: HashSet<PortId> = port_ids.iter().copied().collect();

        // Collect connected edges.
        let edge_ids: Vec<EdgeId> = self.edges.iter()
            .filter(|(_, e)| port_set.contains(&e.source) || port_set.contains(&e.target))
            .map(|(eid, _)| *eid)
            .collect();

        // Remove canvas object.
        let canvas_object = match node.canvas_object {
            Some(obj_id) => Some(self.canvas.remove_object(obj_id)?),
            None => None,
        };

        // Remove edges.
        let mut removed_edges = Vec::new();
        for eid in &edge_ids {
            if let Some(e) = self.edges.remove(eid) {
                removed_edges.push(e);
            }
        }

        // Remove ports.
        let mut removed_ports = Vec::new();
        for pid in &port_ids {
            if let Some(p) = self.ports.remove(pid) {
                removed_ports.push(p);
            }
        }

        // Remove node.
        let removed_node = self.nodes.remove(&id)
            .ok_or(NodeGraphError::NodeNotFound { id })?;

        Ok(NodeRemovedData {
            node: removed_node,
            ports: removed_ports,
            edges: removed_edges,
            canvas_object,
        })
    }

    pub fn rename_node(&mut self, id: NodeId, name: String) -> Result<String, NodeGraphError> {
        let node = self.nodes.get_mut(&id).ok_or(NodeGraphError::NodeNotFound { id })?;
        let previous = node.name.clone();
        node.name = name;
        Ok(previous)
    }

    /// Move a node's canvas object by setting its translation.
    pub fn move_node(&mut self, id: NodeId, translation: Vec2) -> Result<Transform, NodeGraphError> {
        let node = self.nodes.get(&id).ok_or(NodeGraphError::NodeNotFound { id })?;
        let obj_id = node.canvas_object.ok_or(NodeGraphError::NoCanvasObject { id })?;
        Ok(self.canvas.set_translation(obj_id, translation)?)
    }

    /// Replace a node's canvas transform wholesale.
    pub fn set_node_transform(&mut self, id: NodeId, transform: Transform) -> Result<Transform, NodeGraphError> {
        let node = self.nodes.get(&id).ok_or(NodeGraphError::NodeNotFound { id })?;
        let obj_id = node.canvas_object.ok_or(NodeGraphError::NoCanvasObject { id })?;
        Ok(self.canvas.set_transform(obj_id, transform)?)
    }

    /// Read a node's current spatial transform from the canvas.
    pub fn node_transform(&self, id: NodeId) -> Result<Transform, NodeGraphError> {
        let node = self.nodes.get(&id).ok_or(NodeGraphError::NodeNotFound { id })?;
        let obj_id = node.canvas_object.ok_or(NodeGraphError::NoCanvasObject { id })?;
        let obj = self.canvas.get(obj_id)
            .ok_or(NodeGraphError::from(CanvasError::ObjectNotFound { id: obj_id }))?;
        Ok(obj.transform)
    }

    // --- Port management ---

    pub fn add_input_port(
        &mut self,
        node: NodeId,
        name: String,
        type_name: String,
    ) -> Result<PortId, NodeGraphError> {
        if !self.nodes.contains_key(&node) {
            return Err(NodeGraphError::NodeNotFound { id: node });
        }
        let id = PortId::from_raw(self.next_port_id);
        self.next_port_id += 1;
        let port = Port::new(id, node, PortDirection::Input, name, type_name);
        self.nodes.get_mut(&node).unwrap().input_ports.push(id);
        self.ports.insert(id, port);
        Ok(id)
    }

    pub fn add_output_port(
        &mut self,
        node: NodeId,
        name: String,
        type_name: String,
    ) -> Result<PortId, NodeGraphError> {
        if !self.nodes.contains_key(&node) {
            return Err(NodeGraphError::NodeNotFound { id: node });
        }
        let id = PortId::from_raw(self.next_port_id);
        self.next_port_id += 1;
        let port = Port::new(id, node, PortDirection::Output, name, type_name);
        self.nodes.get_mut(&node).unwrap().output_ports.push(id);
        self.ports.insert(id, port);
        Ok(id)
    }

    /// Insert an existing port (for undo of RemovePort).
    pub fn insert_port(&mut self, port: Port) -> PortId {
        let id = port.id;
        let node_id = port.node;
        if id.as_raw() >= self.next_port_id {
            self.next_port_id = id.as_raw() + 1;
        }
        match port.direction {
            PortDirection::Input => {
                if let Some(n) = self.nodes.get_mut(&node_id) {
                    if !n.input_ports.contains(&id) {
                        n.input_ports.push(id);
                    }
                }
            }
            PortDirection::Output => {
                if let Some(n) = self.nodes.get_mut(&node_id) {
                    if !n.output_ports.contains(&id) {
                        n.output_ports.push(id);
                    }
                }
            }
        }
        self.ports.insert(id, port);
        id
    }

    /// Remove a port and all connected edges.
    pub fn remove_port(&mut self, id: PortId) -> Result<PortRemovedData, NodeGraphError> {
        let port = self.ports.get(&id).ok_or(NodeGraphError::PortNotFound { id })?;
        let node_id = port.node;
        let direction = port.direction;

        // Find connected edges.
        let edge_ids: Vec<EdgeId> = self.edges.iter()
            .filter(|(_, e)| e.source == id || e.target == id)
            .map(|(eid, _)| *eid)
            .collect();

        // Remove edges.
        let mut removed_edges = Vec::new();
        for eid in &edge_ids {
            if let Some(e) = self.edges.remove(eid) {
                removed_edges.push(e);
            }
        }

        // Remove port from node's list.
        if let Some(n) = self.nodes.get_mut(&node_id) {
            match direction {
                PortDirection::Input => n.input_ports.retain(|p| *p != id),
                PortDirection::Output => n.output_ports.retain(|p| *p != id),
            }
        }

        let removed_port = self.ports.remove(&id)
            .ok_or(NodeGraphError::PortNotFound { id })?;

        Ok(PortRemovedData {
            port: removed_port,
            edges: removed_edges,
        })
    }


    pub fn port_mut(&mut self, id: PortId) -> Option<&mut Port> {
        self.ports.get_mut(&id)
    }


    // --- Edge management ---

    /// Create an edge from an output port to an input port. Validates
    /// direction, type match, prevents self-loops, duplicates, and
    /// enforces connection limits.
    pub fn connect(
        &mut self,
        source: PortId,
        target: PortId,
    ) -> Result<EdgeId, NodeGraphError> {
        let src_port = self.ports.get(&source)
            .ok_or(NodeGraphError::PortNotFound { id: source })?;
        let tgt_port = self.ports.get(&target)
            .ok_or(NodeGraphError::PortNotFound { id: target })?;

        // Source must be an output.
        if src_port.direction != PortDirection::Output {
            return Err(NodeGraphError::SourceNotOutput { id: source });
        }
        // Target must be an input.
        if tgt_port.direction != PortDirection::Input {
            return Err(NodeGraphError::TargetNotInput { id: target });
        }
        // Different nodes.
        if src_port.node == tgt_port.node {
            return Err(NodeGraphError::SameNode);
        }
        // Type match.
        if src_port.type_name != tgt_port.type_name {
            return Err(NodeGraphError::TypeMismatch {
                source_type: src_port.type_name.clone(),
                target_type: tgt_port.type_name.clone(),
            });
        }
        // No duplicate.
        if self.edges.values().any(|e| e.source == source && e.target == target) {
            return Err(NodeGraphError::DuplicateEdge { source, target });
        }

        // Connection limit on the target input port.
        if tgt_port.max_inputs > 0 {
            let existing = self.edges.values()
                .filter(|e| e.target == target)
                .count() as u32;
            if existing >= tgt_port.max_inputs {
                return Err(NodeGraphError::MaxInputsExceeded {
                    port: target,
                    max: tgt_port.max_inputs,
                });
            }
        }

        let id = EdgeId::from_raw(self.next_edge_id);
        self.next_edge_id += 1;
        self.edges.insert(id, Edge::new(id, source, target));
        Ok(id)
    }

    /// Insert an existing edge (for undo of Disconnect).
    pub fn insert_edge(&mut self, edge: Edge) -> EdgeId {
        let id = edge.id;
        if id.as_raw() >= self.next_edge_id {
            self.next_edge_id = id.as_raw() + 1;
        }
        self.edges.insert(id, edge);
        id
    }

    pub fn disconnect(&mut self, id: EdgeId) -> Result<Edge, NodeGraphError> {
        self.edges.remove(&id)
            .ok_or(NodeGraphError::EdgeNotFound { id })
    }

    /// All edges connected to a port (as source or target).
    pub fn edges_on_port(&self, port: PortId) -> impl Iterator<Item = &Edge> {
        self.edges.values().filter(move |e| e.source == port || e.target == port)
    }

    /// All edges from `source_node` to `target_node`.
    pub fn edges_between(&self, source_node: NodeId, target_node: NodeId) -> impl Iterator<Item = &Edge> {
        self.edges.values().filter(move |e| {
            let src_node = self.ports.get(&e.source).map(|p| p.node);
            let tgt_node = self.ports.get(&e.target).map(|p| p.node);
            src_node == Some(source_node) && tgt_node == Some(target_node)
        })
    }

    // --- Gap 9: Graph traversal ---

    /// Topologically sort nodes by edge dependency.
    /// Returns node ids in order: sources first, sinks last.
    /// Returns `CycleInGraph` error if the graph has a cycle.
    pub fn topological_sort(&self) -> Result<Vec<NodeId>, NodeGraphError> {
        use std::collections::{HashMap, VecDeque};

        let mut in_degree: HashMap<NodeId, usize> = HashMap::new();
        let mut adj: HashMap<NodeId, Vec<NodeId>> = HashMap::new();

        for node in self.nodes.values() {
            in_degree.entry(node.id).or_insert(0);
            adj.entry(node.id).or_default();
        }

        for edge in self.edges.values() {
            let source_node = self.ports.get(&edge.source).map(|p| p.node);
            let target_node = self.ports.get(&edge.target).map(|p| p.node);
            if let (Some(src), Some(tgt)) = (source_node, target_node) {
                adj.entry(src).or_default().push(tgt);
                *in_degree.entry(tgt).or_insert(0) += 1;
            }
        }

        let mut queue: VecDeque<NodeId> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut result = Vec::new();
        while let Some(node) = queue.pop_front() {
            result.push(node);
            if let Some(neighbors) = adj.get(&node) {
                for &neighbor in neighbors {
                    if let Some(deg) = in_degree.get_mut(&neighbor) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
        }

        if result.len() != self.nodes.len() {
            return Err(NodeGraphError::CycleInGraph);
        }
        Ok(result)
    }

    /// Whether the graph contains any cycles.
    pub fn has_cycles(&self) -> bool {
        self.topological_sort().is_err()
    }

    // --- Gap 10: Connection policy ---

    /// Count existing edges connected to a port as target.
    pub fn input_edge_count(&self, port: PortId) -> usize {
        self.edges.values().filter(|e| e.target == port).count()
    }

    /// Remove all nodes, ports, edges, and canvas objects.
    /// Id counters are not reset.
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.ports.clear();
        self.edges.clear();
        self.canvas.clear();
    }

    /// Rename a port.
    pub fn rename_port(&mut self, id: PortId, name: String) -> Result<String, NodeGraphError> {
        let port = self.ports.get_mut(&id)
            .ok_or(NodeGraphError::PortNotFound { id })?;
        let previous = port.name.clone();
        port.name = name;
        Ok(previous)
    }

    /// Duplicate a node (without ports or edges). The new node gets a new id
    /// and its own canvas object at the same transform as the source.
    pub fn duplicate_node(&mut self, id: NodeId) -> Result<NodeId, NodeGraphError> {
        let node = self.nodes.get(&id)
            .ok_or(NodeGraphError::NodeNotFound { id })?;
        let new_id = NodeId::from_raw(self.next_node_id);
        self.next_node_id += 1;

        let name = node.name.clone();
        let canvas_id = match node.canvas_object {
            Some(obj_id) => {
                let obj = self.canvas.get(obj_id)
                    .ok_or(NodeGraphError::CanvasError {
                        source: crate::domain_foundation::environments::canvas_2d::CanvasError::ObjectNotFound { id: obj_id },
                    })?;
                Some(self.canvas.create_object(obj.transform))
            }
            None => None,
        };

        let new_node = Node::new(new_id, name, canvas_id);
        self.nodes.insert(new_id, new_node);
        Ok(new_id)
    }

    /// Whether a direct edge exists from `source` to `target`.
    pub fn is_connected(&self, source: PortId, target: PortId) -> bool {
        self.edges
            .values()
            .any(|e| e.source == source && e.target == target)
    }

}