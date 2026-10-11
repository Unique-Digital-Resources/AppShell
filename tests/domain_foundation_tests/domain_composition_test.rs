//! Composition and boundary tests.
//!
//! Verifies that:
//! - Each environment works independently.
//! - NodeGraph composes Canvas2D without coupling.
//! - Relations operate over abstract properties without knowing environments.
//! - Utilities are optional and work over any data.
//! - The Domain Engine composes environments + App Engine.
//! - The App Engine doesn't depend on domain concepts.

use app_shell::domain_engine::DomainEngine;
use app_shell::domain_foundation::environments::canvas_2d::{
    Canvas, CanvasOperation, CanvasOutcome, Transform, Vec2,
};
use app_shell::domain_foundation::environments::timeline::{
    Duration, Time, Timeline,
};
use app_shell::domain_foundation::environments::layers::Layers;
use app_shell::domain_foundation::environments::node_graph::NodeGraph;
use app_shell::domain_foundation::environments::table::{Table, CellValue};
use app_shell::domain_foundation::environments::text_editor::Document;
use app_shell::domain_foundation::environments::viewport_3d::{Scene, Transform3D, Vec3};

// --- Each environment works standalone ---

#[test]
fn canvas_2d_works_without_other_environments() {
    let mut canvas = Canvas::new();
    let id = canvas.create_object(Transform::identity());
    assert!(canvas.contains(id));
    assert_eq!(canvas.len(), 1);
}

#[test]
fn timeline_works_without_canvas() {
    let mut timeline = Timeline::new();
    let track = timeline.create_track();
    let clip = timeline
        .insert_clip(track, Time::new(0.0), Duration::new(10.0))
        .unwrap();
    assert!(timeline.clip(clip).is_some());
}

#[test]
fn layers_works_without_timeline() {
    let mut layers = Layers::new();
    let root = layers.create_layer("Root".to_string());
    let child = layers.create_child(root, "Child".to_string()).unwrap();
    assert_eq!(layers.layer(root).unwrap().child_count(), 1);
    assert_eq!(layers.layer(child).unwrap().parent, Some(root));
}

#[test]
fn table_works_without_anything_else() {
    let mut table = Table::new();
    let row = table.insert_row(None);
    let col = table.insert_column(None);
    table.set_cell_value(row, col, CellValue::Int(42)).unwrap();
    assert_eq!(table.cell(row, col), &CellValue::Int(42));
}

#[test]
fn text_editor_works_standalone() {
    use app_shell::domain_foundation::environments::text_editor::Position;
    let mut doc = Document::new();
    doc.insert(Position::new(0), "hello").unwrap();
    assert_eq!(doc.text(), "hello");
}

#[test]
fn viewport_3d_works_standalone() {
    let mut scene = Scene::new();
    let id = scene.create_object(Transform3D::from_translation(Vec3::new(1.0, 2.0, 3.0)));
    assert_eq!(scene.len(), 1);
    assert_eq!(
        scene.get(id).unwrap().transform.translation,
        Vec3::new(1.0, 2.0, 3.0)
    );
}

// --- NodeGraph composes Canvas2D ---

#[test]
fn node_graph_and_canvas_are_composed_not_coupled() {
    let mut graph = NodeGraph::new();
    let n = graph.create_node("N".to_string(), Transform::identity());

    assert_eq!(graph.node_count(), 1);
    assert_eq!(graph.canvas().len(), 1);

    // Move the node — spatial only.
    graph.move_node(n, Vec2::new(50.0, 50.0)).unwrap();
    assert_eq!(graph.node_transform(n).unwrap().translation, Vec2::new(50.0, 50.0));

    // Graph connectivity is untouched by spatial operations.
    assert_eq!(graph.port_count(), 0);
    assert_eq!(graph.edge_count(), 0);
}

// --- Relations operate over abstract properties ---

#[test]
fn relations_operate_without_knowing_environments() {
    use app_shell::domain_foundation::relations::{
        Endpoint, ObjectId, PropagationMode, PropertyId, RelationDefinitionBuilder,
        RelationRegistry, Trigger,
    };
    use app_shell::domain_foundation::relations::definition::{EffectContext, EffectOutcome};
    use std::sync::Arc;

    let mut registry = RelationRegistry::new();

    let src = Endpoint::source(PropertyId::new(ObjectId::new(1), 0));
    let tgt = Endpoint::target(PropertyId::new(ObjectId::new(2), 0));

    registry.register(
        RelationDefinitionBuilder::new(src, tgt)
            .trigger(Trigger::OnChange)
            .propagation(PropagationMode::Direct)
            .effect(Arc::new(|ctx: &EffectContext| {
                EffectOutcome::Apply(ctx.source_value.clone())
            }))
            .build(),
    );

    assert_eq!(registry.len(), 1);
}

// --- Utilities are optional and work over any data ---

#[test]
fn utilities_work_over_environment_data() {
    use app_shell::domain_foundation::utilities::filtering::{criteria, filter};
    use app_shell::domain_foundation::environments::canvas_2d::Object;

    let mut canvas = Canvas::new();
    canvas.create_object(Transform::from_translation(Vec2::new(0.0, 0.0)));
    canvas.create_object(Transform::from_translation(Vec2::new(10.0, 0.0)));
    canvas.create_object(Transform::from_translation(Vec2::new(20.0, 0.0)));

    let objects: Vec<Object> = canvas.iter().copied().collect();
    let right_side = filter(&objects, &criteria(|obj: &Object| {
        obj.transform.translation.x > 5.0
    }));

    assert_eq!(right_side.len(), 2);
}

// --- Domain Engine composes everything ---

#[test]
fn domain_engine_composes_canvas_history_and_events() {
    let mut engine = DomainEngine::new();

    let id = match engine
        .execute(CanvasOperation::Create {
            transform: Transform::identity(),
        })
        .unwrap()
    {
        CanvasOutcome::Created { id } => id,
        _ => unreachable!(),
    };

    engine
        .execute(CanvasOperation::Move {
            id,
            translation: Vec2::new(5.0, 5.0),
        })
        .unwrap();

    // Undo the move.
    engine.undo().unwrap();
    assert_eq!(
        engine.canvas().get(id).unwrap().transform.translation,
        Vec2::new(0.0, 0.0)
    );

    // Redo the move.
    engine.redo().unwrap();
    assert_eq!(
        engine.canvas().get(id).unwrap().transform.translation,
        Vec2::new(5.0, 5.0)
    );

    // 2 execute events + 1 undo event + 1 redo event = 4
    assert_eq!(engine.events().len(), 4);
    assert_eq!(engine.history().position(), 2);
}

// --- App Engine is domain-agnostic ---

#[test]
fn app_engine_does_not_depend_on_domain_foundation() {
    use app_shell::app_engine::{Event, HistoryEntry, HistoryStore};

    let mut store = HistoryStore::new();
    store.append(HistoryEntry::new("generic.operation").undoable());
    assert_eq!(store.position(), 1);

    store.undo();
    assert_eq!(store.position(), 0);

    store.redo();
    assert_eq!(store.position(), 1);

    let _event = Event::new("generic.event");
}