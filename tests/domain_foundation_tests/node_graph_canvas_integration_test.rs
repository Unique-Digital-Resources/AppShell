//! Composition tests: NodeGraph + Canvas2D integration.
//!
//! Verifies that the graph's connectivity semantics and the canvas's
//! spatial semantics are separate. Graph operations don't touch spatial
//! state; spatial operations don't touch graph connectivity.

use app_shell::domain_foundation::environments::canvas_2d::{Transform, Vec2};
use app_shell::domain_foundation::environments::node_graph::{
    NodeGraph, NodeId,
};

#[test]
fn create_node_also_creates_canvas_object() {
    let mut graph = NodeGraph::new();
    let id = graph.create_node("N".to_string(), Transform::from_translation(Vec2::new(10.0, 20.0)));

    // Canvas should have one object at the given transform.
    assert_eq!(graph.canvas().len(), 1);

    let transform = graph.node_transform(id).unwrap();
    assert_eq!(transform.translation, Vec2::new(10.0, 20.0));
}

#[test]
fn move_node_moves_canvas_object_not_graph_state() {
    let mut graph = NodeGraph::new();
    let a = graph.create_node("A".to_string(), Transform::identity());
    let b = graph.create_node("B".to_string(), Transform::identity());
    let out_a = graph.add_output_port(a, "out".to_string(), "float".to_string()).unwrap();
    let in_b = graph.add_input_port(b, "in".to_string(), "float".to_string()).unwrap();
    graph.connect(out_a, in_b).unwrap();

    // Move A.
    graph.move_node(a, Vec2::new(100.0, 200.0)).unwrap();

    // Spatial state changed.
    assert_eq!(graph.node_transform(a).unwrap().translation, Vec2::new(100.0, 200.0));
    // B didn't move.
    assert_eq!(graph.node_transform(b).unwrap().translation, Vec2::new(0.0, 0.0));

    // Graph connectivity unchanged.
    assert_eq!(graph.edge_count(), 1);
    assert_eq!(graph.port_count(), 2);
}

#[test]
fn connect_nodes_does_not_affect_spatial_state() {
    let mut graph = NodeGraph::new();
    let a = graph.create_node("A".to_string(), Transform::from_translation(Vec2::new(0.0, 0.0)));
    let b = graph.create_node("B".to_string(), Transform::from_translation(Vec2::new(100.0, 0.0)));
    let out_a = graph.add_output_port(a, "out".to_string(), "float".to_string()).unwrap();
    let in_b = graph.add_input_port(b, "in".to_string(), "float".to_string()).unwrap();

    let canvas_len_before = graph.canvas().len();
    let transform_a_before = graph.node_transform(a).unwrap();
    let transform_b_before = graph.node_transform(b).unwrap();

    graph.connect(out_a, in_b).unwrap();

    // Spatial state unchanged.
    assert_eq!(graph.canvas().len(), canvas_len_before);
    assert_eq!(graph.node_transform(a).unwrap(), transform_a_before);
    assert_eq!(graph.node_transform(b).unwrap(), transform_b_before);

    // Graph state changed.
    assert_eq!(graph.edge_count(), 1);
}

#[test]
fn remove_node_removes_canvas_object() {
    let mut graph = NodeGraph::new();
    let a = graph.create_node("A".to_string(), Transform::identity());
    let b = graph.create_node("B".to_string(), Transform::identity());

    assert_eq!(graph.canvas().len(), 2);

    graph.remove_node(a).unwrap();

    assert_eq!(graph.canvas().len(), 1);
    assert_eq!(graph.node_count(), 1);
    assert!(graph.contains_node(b));
}

#[test]
fn multiple_nodes_have_independent_spatial_placement() {
    let mut graph = NodeGraph::new();
    let a = graph.create_node("A".to_string(), Transform::from_translation(Vec2::new(0.0, 0.0)));
    let b = graph.create_node("B".to_string(), Transform::from_translation(Vec2::new(100.0, 200.0)));
    let c = graph.create_node("C".to_string(), Transform::from_translation(Vec2::new(-50.0, 50.0)));

    assert_eq!(graph.canvas().len(), 3);
    assert_eq!(graph.node_transform(a).unwrap().translation, Vec2::new(0.0, 0.0));
    assert_eq!(graph.node_transform(b).unwrap().translation, Vec2::new(100.0, 200.0));
    assert_eq!(graph.node_transform(c).unwrap().translation, Vec2::new(-50.0, 50.0));

    // Move B independently.
    graph.move_node(b, Vec2::new(999.0, 999.0)).unwrap();
    assert_eq!(graph.node_transform(a).unwrap().translation, Vec2::new(0.0, 0.0));
    assert_eq!(graph.node_transform(b).unwrap().translation, Vec2::new(999.0, 999.0));
    assert_eq!(graph.node_transform(c).unwrap().translation, Vec2::new(-50.0, 50.0));
}

#[test]
fn set_node_transform_replaces_whole_transform() {
    let mut graph = NodeGraph::new();
    let id = graph.create_node("N".to_string(), Transform::identity());

    let new = Transform::from_translation(Vec2::new(5.0, 5.0))
        .with_rotation(0.5)
        .with_scale(Vec2::splat(2.0));

    graph.set_node_transform(id, new).unwrap();

    let current = graph.node_transform(id).unwrap();
    assert_eq!(current.translation, Vec2::new(5.0, 5.0));
    assert_eq!(current.rotation, 0.5);
    assert_eq!(current.scale, Vec2::splat(2.0));
}

#[test]
fn insert_node_restores_canvas_object() {
    let mut graph = NodeGraph::new();
    let id = graph.create_node("N".to_string(), Transform::from_translation(Vec2::new(7.0, 8.0)));

    let data = graph.remove_node(id).unwrap();
    assert_eq!(graph.canvas().len(), 0);

    // Re-insert node and canvas object.
    graph.insert_node(data.node, data.canvas_object);

    assert!(graph.contains_node(id));
    assert_eq!(graph.canvas().len(), 1);
    assert_eq!(graph.node_transform(id).unwrap().translation, Vec2::new(7.0, 8.0));
}