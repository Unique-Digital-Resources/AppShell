//! Operation tests for the NodeGraph environment.

use app_shell::domain_foundation::environments::canvas_2d::{Transform, Vec2};
use app_shell::domain_foundation::environments::node_graph::{
    NodeGraph, NodeGraphError, NodeGraphOperation, NodeGraphOutcome, NodeId, PortId,
};

fn setup() -> (NodeGraph, NodeId, NodeId, PortId, PortId) {
    let mut graph = NodeGraph::new();
    let a = graph.create_node("A".to_string(), Transform::identity());
    let b = graph.create_node("B".to_string(), Transform::identity());
    let out_a = graph.add_output_port(a, "out".to_string(), "float".to_string()).unwrap();
    let in_b = graph.add_input_port(b, "in".to_string(), "float".to_string()).unwrap();
    (graph, a, b, out_a, in_b)
}

#[test]
fn create_node_operation() {
    let mut graph = NodeGraph::new();
    let outcome = NodeGraphOperation::CreateNode {
        name: "N".to_string(),
        transform: Transform::identity(),
    }
    .apply(&mut graph)
    .unwrap();

    let id = match outcome {
        NodeGraphOutcome::NodeCreated { id } => id,
        other => panic!("expected NodeCreated, got {other:?}"),
    };
    assert!(graph.contains_node(id));
    assert_eq!(graph.canvas().len(), 1);
}

#[test]
fn remove_node_operation_returns_cascade_data() {
    let (mut graph, a, _b, out_a, in_b) = setup();
    graph.connect(out_a, in_b).unwrap();

    let outcome = NodeGraphOperation::RemoveNode { id: a }
        .apply(&mut graph)
        .unwrap();

    match outcome {
        NodeGraphOutcome::NodeRemoved(data) => {
            assert_eq!(data.ports.len(), 1);
            assert_eq!(data.edges.len(), 1);
            assert!(data.canvas_object.is_some());
        }
        other => panic!("expected NodeRemoved, got {other:?}"),
    }
    assert!(!graph.contains_node(a));
}

#[test]
fn move_node_operation() {
    let (mut graph, a, _b, _out_a, _in_b) = setup();

    let outcome = NodeGraphOperation::MoveNode {
        id: a,
        translation: Vec2::new(50.0, 50.0),
    }
    .apply(&mut graph)
    .unwrap();

    assert!(matches!(outcome, NodeGraphOutcome::NodeMoved { .. }));
    assert_eq!(graph.node_transform(a).unwrap().translation, Vec2::new(50.0, 50.0));
}

#[test]
fn connect_operation() {
    let (mut graph, _a, _b, out_a, in_b) = setup();

    let outcome = NodeGraphOperation::Connect {
        source: out_a,
        target: in_b,
    }
    .apply(&mut graph)
    .unwrap();

    let id = match outcome {
        NodeGraphOutcome::Connected { id } => id,
        other => panic!("expected Connected, got {other:?}"),
    };
    assert!(graph.edge(id).is_some());
}

#[test]
fn disconnect_operation_returns_edge() {
    let (mut graph, _a, _b, out_a, in_b) = setup();
    let edge_id = graph.connect(out_a, in_b).unwrap();

    let outcome = NodeGraphOperation::Disconnect { id: edge_id }
        .apply(&mut graph)
        .unwrap();

    match outcome {
        NodeGraphOutcome::Disconnected { edge } => {
            assert_eq!(edge.id, edge_id);
        }
        other => panic!("expected Disconnected, got {other:?}"),
    }
    assert_eq!(graph.edge_count(), 0);
}

#[test]
fn add_port_operations() {
    let mut graph = NodeGraph::new();
    let n = graph.create_node("N".to_string(), Transform::identity());

    let outcome = NodeGraphOperation::AddInputPort {
        node: n,
        name: "in".to_string(),
        type_name: "float".to_string(),
    }
    .apply(&mut graph)
    .unwrap();
    let in_id = match outcome {
        NodeGraphOutcome::PortAdded { id } => id,
        _ => unreachable!(),
    };
    assert!(graph.port(in_id).unwrap().is_input());

    let outcome = NodeGraphOperation::AddOutputPort {
        node: n,
        name: "out".to_string(),
        type_name: "float".to_string(),
    }
    .apply(&mut graph)
    .unwrap();
    let out_id = match outcome {
        NodeGraphOutcome::PortAdded { id } => id,
        _ => unreachable!(),
    };
    assert!(graph.port(out_id).unwrap().is_output());
}

#[test]
fn operations_on_missing_node_fail() {
    let mut graph = NodeGraph::new();
    let fake = NodeId::from_raw(999);

    let err = NodeGraphOperation::RemoveNode { id: fake }
        .apply(&mut graph).unwrap_err();
    assert!(matches!(err, NodeGraphError::NodeNotFound { .. }));

    let err = NodeGraphOperation::MoveNode { id: fake, translation: Vec2::zero() }
        .apply(&mut graph).unwrap_err();
    assert!(matches!(err, NodeGraphError::NodeNotFound { .. }));
}

#[test]
fn connect_validation_errors_propagate() {
    let (mut graph, a, _b, out_a, _in_b) = setup();
    // Add an input on A, try to connect input→input.
    let in_a = graph.add_input_port(a, "in2".to_string(), "float".to_string()).unwrap();

    let err = NodeGraphOperation::Connect { source: in_a, target: _get_in(&graph, 1) }
        .apply(&mut graph).unwrap_err();
    assert!(matches!(err, NodeGraphError::SourceNotOutput { .. }));

    let err = NodeGraphOperation::Connect { source: out_a, target: out_a }
        .apply(&mut graph).unwrap_err();
    assert!(matches!(err, NodeGraphError::TargetNotInput { .. }));
}

fn _get_in(graph: &NodeGraph, raw: u64) -> PortId {
    let nid = NodeId::from_raw(raw);
    graph.node(nid).unwrap().input_ports()[0]
}

#[test]
fn sequence_of_operations_composes() {
    let mut graph = NodeGraph::new();

    let outcome = NodeGraphOperation::CreateNode {
        name: "Source".to_string(),
        transform: Transform::from_translation(Vec2::new(0.0, 0.0)),
    }
    .apply(&mut graph).unwrap();
    let source = match outcome {
        NodeGraphOutcome::NodeCreated { id } => id,
        _ => unreachable!(),
    };

    let outcome = NodeGraphOperation::CreateNode {
        name: "Sink".to_string(),
        transform: Transform::from_translation(Vec2::new(200.0, 0.0)),
    }
    .apply(&mut graph).unwrap();
    let sink = match outcome {
        NodeGraphOutcome::NodeCreated { id } => id,
        _ => unreachable!(),
    };

    let outcome = NodeGraphOperation::AddOutputPort {
        node: source,
        name: "value".to_string(),
        type_name: "float".to_string(),
    }
    .apply(&mut graph).unwrap();
    let out_port = match outcome {
        NodeGraphOutcome::PortAdded { id } => id,
        _ => unreachable!(),
    };

    let outcome = NodeGraphOperation::AddInputPort {
        node: sink,
        name: "value".to_string(),
        type_name: "float".to_string(),
    }
    .apply(&mut graph).unwrap();
    let in_port = match outcome {
        NodeGraphOutcome::PortAdded { id } => id,
        _ => unreachable!(),
    };

    NodeGraphOperation::Connect {
        source: out_port,
        target: in_port,
    }
    .apply(&mut graph).unwrap();

    NodeGraphOperation::MoveNode {
        id: source,
        translation: Vec2::new(10.0, 10.0),
    }
    .apply(&mut graph).unwrap();

    assert_eq!(graph.node_count(), 2);
    assert_eq!(graph.port_count(), 2);
    assert_eq!(graph.edge_count(), 1);
    assert_eq!(graph.node_transform(source).unwrap().translation, Vec2::new(10.0, 10.0));
    assert_eq!(graph.node_transform(sink).unwrap().translation, Vec2::new(200.0, 0.0));
}

#[test]
fn rename_port_operation() {
    let mut graph = NodeGraph::new();
    use app_shell::domain_foundation::environments::canvas_2d::Transform;
    let n = graph.create_node("N".to_string(), Transform::identity());
    let port_id = graph.add_input_port(n, "old".to_string(), "f".to_string()).unwrap();

    let outcome = NodeGraphOperation::RenamePort { id: port_id, name: "new".to_string() }
        .apply(&mut graph)
        .unwrap();

    match outcome {
        NodeGraphOutcome::PortRenamed { previous_name, .. } => {
            assert_eq!(previous_name, "old");
        }
        other => panic!("expected PortRenamed, got {other:?}"),
    }
    assert_eq!(graph.port(port_id).unwrap().name, "new");
}

#[test]
fn duplicate_node_operation() {
    let mut graph = NodeGraph::new();
    use app_shell::domain_foundation::environments::canvas_2d::Transform;
    let n = graph.create_node("Original".to_string(), Transform::identity());

    let outcome = NodeGraphOperation::DuplicateNode { id: n }
        .apply(&mut graph)
        .unwrap();

    match outcome {
        NodeGraphOutcome::NodeDuplicated { source, new_id } => {
            assert_eq!(source, n);
            assert_ne!(new_id, n);
        }
        other => panic!("expected NodeDuplicated, got {other:?}"),
    }
    assert_eq!(graph.node_count(), 2);
    assert_eq!(graph.canvas().len(), 2);
}

#[test]
fn clear_node_graph_operation() {
    let mut graph = NodeGraph::new();
    use app_shell::domain_foundation::environments::canvas_2d::Transform;
    graph.create_node("A".to_string(), Transform::identity());
    graph.create_node("B".to_string(), Transform::identity());

    let outcome = NodeGraphOperation::Clear
        .apply(&mut graph)
        .unwrap();

    match outcome {
        NodeGraphOutcome::Cleared { node_count } => assert_eq!(node_count, 2),
        other => panic!("expected Cleared, got {other:?}"),
    }
    assert_eq!(graph.node_count(), 0);
}