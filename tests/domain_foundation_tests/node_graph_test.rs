//! Model and query tests for the NodeGraph environment.

use app_shell::domain_foundation::environments::canvas_2d::{Transform, Vec2};
use app_shell::domain_foundation::environments::node_graph::{
    NodeGraph, NodeGraphError, PortDirection,
};

fn make_simple_graph() -> (
    NodeGraph,
    app_shell::domain_foundation::environments::node_graph::NodeId,
    app_shell::domain_foundation::environments::node_graph::NodeId,
    app_shell::domain_foundation::environments::node_graph::PortId,
    app_shell::domain_foundation::environments::node_graph::PortId,
) {
    use app_shell::domain_foundation::environments::node_graph::{NodeId, PortId};

    let mut graph = NodeGraph::new();
    let a = graph.create_node("A".to_string(), Transform::from_translation(Vec2::new(0.0, 0.0)));
    let b = graph.create_node("B".to_string(), Transform::from_translation(Vec2::new(100.0, 0.0)));
    let out_a = graph.add_output_port(a, "out".to_string(), "float".to_string()).unwrap();
    let in_b = graph.add_input_port(b, "in".to_string(), "float".to_string()).unwrap();
    (graph, a, b, out_a, in_b)
}

#[test]
fn new_graph_is_empty() {
    let graph = NodeGraph::new();
    assert_eq!(graph.node_count(), 0);
    assert_eq!(graph.port_count(), 0);
    assert_eq!(graph.edge_count(), 0);
}

#[test]
fn create_node_adds_to_graph_and_canvas() {
    let graph = NodeGraph::new();
    let mut graph = graph;
    let id = graph.create_node("Test".to_string(), Transform::identity());
    assert!(graph.contains_node(id));
    assert_eq!(graph.node_count(), 1);
    assert!(graph.canvas().len() == 1);
}

#[test]
fn add_ports_to_node() {
    let (graph, _a, _b, out_a, in_b) = make_simple_graph();
    let _ = (graph, out_a, in_b);
}

#[test]
fn connect_ports_creates_edge() {
    let (mut graph, _a, _b, out_a, in_b) = make_simple_graph();
    let edge_id = graph.connect(out_a, in_b).unwrap();
    assert_eq!(graph.edge_count(), 1);
    assert!(graph.edge(edge_id).is_some());
}

#[test]
fn connect_requires_output_source() {
    let (mut graph, a, _b, _out_a, in_b) = make_simple_graph();
    // Add another input on A and try to connect input→input.
    let in_a = graph.add_input_port(a, "in2".to_string(), "float".to_string()).unwrap();
    let err = graph.connect(in_a, in_b).unwrap_err();
    assert!(matches!(err, NodeGraphError::SourceNotOutput { .. }));
}

#[test]
fn connect_requires_input_target() {
    let (mut graph, _a, _b, out_a, _in_b) = make_simple_graph();
    // Add another output on B and try output→output.
    let out_b = graph.add_output_port(
        app_shell::domain_foundation::environments::node_graph::NodeId::from_raw(1),
        "out2".to_string(),
        "float".to_string(),
    ).unwrap();
    let err = graph.connect(out_a, out_b).unwrap_err();
    assert!(matches!(err, NodeGraphError::TargetNotInput { .. }));
}

#[test]
fn connect_rejects_same_node() {
    let mut graph = NodeGraph::new();
    let n = graph.create_node("N".to_string(), Transform::identity());
    let out_n = graph.add_output_port(n, "out".to_string(), "float".to_string()).unwrap();
    let in_n = graph.add_input_port(n, "in".to_string(), "float".to_string()).unwrap();
    let err = graph.connect(out_n, in_n).unwrap_err();
    assert!(matches!(err, NodeGraphError::SameNode));
}

#[test]
fn connect_rejects_type_mismatch() {
    let mut graph = NodeGraph::new();
    let a = graph.create_node("A".to_string(), Transform::identity());
    let b = graph.create_node("B".to_string(), Transform::identity());
    let out_a = graph.add_output_port(a, "out".to_string(), "float".to_string()).unwrap();
    let in_b = graph.add_input_port(b, "in".to_string(), "string".to_string()).unwrap();
    let err = graph.connect(out_a, in_b).unwrap_err();
    assert!(matches!(err, NodeGraphError::TypeMismatch { .. }));
}

#[test]
fn connect_rejects_duplicate() {
    let (mut graph, _a, _b, out_a, in_b) = make_simple_graph();
    graph.connect(out_a, in_b).unwrap();
    let err = graph.connect(out_a, in_b).unwrap_err();
    assert!(matches!(err, NodeGraphError::DuplicateEdge { .. }));
}

#[test]
fn remove_node_cascades() {
    let (mut graph, a, _b, out_a, _in_b) = make_simple_graph();
    let edge = graph.connect(out_a, _get_input(&graph, 1)).unwrap();

    let data = graph.remove_node(a).unwrap();
    assert_eq!(data.ports.len(), 1); // out_a
    assert_eq!(data.edges.len(), 1);
    assert!(!graph.contains_node(a));
    assert_eq!(graph.node_count(), 1);
    assert_eq!(graph.port_count(), 1); // in_b on B
    assert_eq!(graph.edge_count(), 0);
    assert!(graph.canvas().len() == 1); // only B's object
}

fn _get_input(graph: &NodeGraph, node_raw: u64) -> app_shell::domain_foundation::environments::node_graph::PortId {
    let node_id = app_shell::domain_foundation::environments::node_graph::NodeId::from_raw(node_raw);
    let node = graph.node(node_id).unwrap();
    node.input_ports()[0]
}

#[test]
fn remove_port_cascades_to_edges() {
    let (mut graph, _a, _b, out_a, _in_b) = make_simple_graph();
    graph.connect(out_a, _get_input(&graph, 1)).unwrap();
    assert_eq!(graph.edge_count(), 1);

    let data = graph.remove_port(out_a).unwrap();
    assert_eq!(data.edges.len(), 1);
    assert_eq!(graph.port_count(), 1); // in_b remains
    assert_eq!(graph.edge_count(), 0);
}

#[test]
fn edges_on_port_finds_connected() {
    let (mut graph, _a, _b, out_a, in_b) = make_simple_graph();
    graph.connect(out_a, in_b).unwrap();

    let on_source: Vec<_> = graph.edges_on_port(out_a).collect();
    assert_eq!(on_source.len(), 1);

    let on_target: Vec<_> = graph.edges_on_port(in_b).collect();
    assert_eq!(on_target.len(), 1);
}

#[test]
fn edges_between_finds_connections() {
    let (mut graph, a, b, out_a, in_b) = make_simple_graph();
    graph.connect(out_a, in_b).unwrap();

    let between: Vec<_> = graph.edges_between(a, b).collect();
    assert_eq!(between.len(), 1);
}

#[test]
fn topological_sort_returns_dependency_order() {
    use app_shell::domain_foundation::environments::canvas_2d::Transform;
    let mut graph = NodeGraph::new();
    let a = graph.create_node("A".to_string(), Transform::identity());
    let b = graph.create_node("B".to_string(), Transform::identity());
    let c = graph.create_node("C".to_string(), Transform::identity());

    let out_a = graph.add_output_port(a, "out".to_string(), "f".to_string()).unwrap();
    let in_b = graph.add_input_port(b, "in".to_string(), "f".to_string()).unwrap();
    let out_b = graph.add_output_port(b, "out".to_string(), "f".to_string()).unwrap();
    let in_c = graph.add_input_port(c, "in".to_string(), "f".to_string()).unwrap();

    graph.connect(out_a, in_b).unwrap();
    graph.connect(out_b, in_c).unwrap();

    let sorted = graph.topological_sort().unwrap();
    let pos_a = sorted.iter().position(|n| *n == a).unwrap();
    let pos_b = sorted.iter().position(|n| *n == b).unwrap();
    let pos_c = sorted.iter().position(|n| *n == c).unwrap();
    assert!(pos_a < pos_b);
    assert!(pos_b < pos_c);
}

#[test]
fn has_cycles_detects_cycles() {
    use app_shell::domain_foundation::environments::canvas_2d::Transform;
    let mut graph = NodeGraph::new();
    let a = graph.create_node("A".to_string(), Transform::identity());
    let b = graph.create_node("B".to_string(), Transform::identity());

    let out_a = graph.add_output_port(a, "out".to_string(), "f".to_string()).unwrap();
    let in_a = graph.add_input_port(a, "in".to_string(), "f".to_string()).unwrap();
    let out_b = graph.add_output_port(b, "out".to_string(), "f".to_string()).unwrap();
    let in_b = graph.add_input_port(b, "in".to_string(), "f".to_string()).unwrap();

    graph.connect(out_a, in_b).unwrap();
    graph.connect(out_b, in_a).unwrap();

    assert!(graph.has_cycles());
}

#[test]
fn connection_policy_enforces_max_inputs() {
    use app_shell::domain_foundation::environments::canvas_2d::Transform;
    use app_shell::domain_foundation::environments::node_graph::PortDirection;
    let mut graph = NodeGraph::new();
    let a = graph.create_node("A".to_string(), Transform::identity());
    let b = graph.create_node("B".to_string(), Transform::identity());
    let c = graph.create_node("C".to_string(), Transform::identity());

    let out_a = graph.add_output_port(a, "out".to_string(), "f".to_string()).unwrap();
    let out_c = graph.add_output_port(c, "out".to_string(), "f".to_string()).unwrap();
    let in_b = graph.add_input_port(b, "in".to_string(), "f".to_string()).unwrap();

    // Set max_inputs = 1 on in_b.
    graph.port_mut(in_b).unwrap().max_inputs = 1;

    graph.connect(out_a, in_b).unwrap();
    let err = graph.connect(out_c, in_b).unwrap_err();
    assert!(matches!(err, app_shell::domain_foundation::environments::node_graph::NodeGraphError::MaxInputsExceeded { .. }));
}

#[test]
fn node_graph_clear_empties() {
    use app_shell::domain_foundation::environments::canvas_2d::Transform;
    let mut graph = NodeGraph::new();
    graph.create_node("A".to_string(), Transform::identity());
    graph.create_node("B".to_string(), Transform::identity());
    assert_eq!(graph.node_count(), 2);
    
    graph.clear();
    assert_eq!(graph.node_count(), 0);
    assert_eq!(graph.port_count(), 0);
    assert_eq!(graph.edge_count(), 0);
    assert_eq!(graph.canvas().len(), 0);
}
#[test]
fn is_connected_checks_direct_edge() {
    use app_shell::domain_foundation::environments::canvas_2d::Transform;
    let mut graph = NodeGraph::new();
    let a = graph.create_node("A".to_string(), Transform::identity());
    let b = graph.create_node("B".to_string(), Transform::identity());

    let out_a = graph.add_output_port(a, "out".to_string(), "f".to_string()).unwrap();
    let in_b = graph.add_input_port(b, "in".to_string(), "f".to_string()).unwrap();

    // Not connected yet.
    assert!(!graph.is_connected(out_a, in_b));

    graph.connect(out_a, in_b).unwrap();

    // Now connected.
    assert!(graph.is_connected(out_a, in_b));
    // Reverse direction doesn't count.
    assert!(!graph.is_connected(in_b, out_a));
}