//! Public API smoke test: verify that all intended exports compile and
//! are accessible from outside the crate.

// --- Environments ---

#[test]
fn canvas_2d_exports_are_accessible() {
    use app_shell::domain_foundation::environments::canvas_2d::{
        Canvas, CanvasError, CanvasOperation, CanvasOutcome, Object, ObjectId,
        Transform, Vec2,
    };

    let mut canvas = Canvas::new();
    assert!(canvas.is_empty());

    let id = canvas.create_object(Transform::identity());
    assert!(canvas.contains(id));

    let _obj: &Object = canvas.get(id).unwrap();
    let _tf: Transform = Transform::from_translation(Vec2::new(1.0, 2.0));
    let _op: CanvasOperation = CanvasOperation::Create { transform: Transform::identity() };
    let _err: CanvasError = CanvasError::ObjectNotFound { id };
    let _outcome: CanvasOutcome = CanvasOutcome::Created { id };
}

#[test]
fn timeline_exports_are_accessible() {
    use app_shell::domain_foundation::environments::timeline::{
        Clip, ClipId, Duration, Keyframe, KeyframeId, KeyframeValue, Time,
        Timeline, TimelineError, TimelineOperation, TimelineOutcome, Track,
        TrackId,
    };

    let mut timeline = Timeline::new();
    assert_eq!(timeline.track_count(), 0);

    let track_id = timeline.create_track();
    let clip_id = timeline
        .insert_clip(track_id, Time::new(0.0), Duration::new(5.0))
        .unwrap();

    let _clip: &Clip = timeline.clip(clip_id).unwrap();
    let _track: &Track = timeline.track(track_id).unwrap();
    let _time: Time = Time::new(1.0);
    let _dur: Duration = Duration::new(2.0);
    let _err: TimelineError = TimelineError::TrackNotFound { id: track_id };
    let _op: TimelineOperation = TimelineOperation::CreateTrack;
    let _outcome: TimelineOutcome = TimelineOutcome::TrackCreated { id: track_id };
    let _kf_val: KeyframeValue = KeyframeValue::Float(1.0);
}

#[test]
fn layers_exports_are_accessible() {
    use app_shell::domain_foundation::environments::layers::{
        Layer, LayerId, Layers, LayersError, LayersOperation, LayersOutcome,
    };

    let mut layers = Layers::new();
    assert!(layers.is_empty());

    let id = layers.create_layer("Base".to_string());
    let _layer: &Layer = layers.layer(id).unwrap();
    let _err: LayersError = LayersError::LayerNotFound { id };
    let _op: LayersOperation = LayersOperation::CreateLayer { name: "X".to_string() };
    let _outcome: LayersOutcome = LayersOutcome::LayerCreated { id };
}

#[test]
fn node_graph_exports_are_accessible() {
    use app_shell::domain_foundation::environments::node_graph::{
        Edge, EdgeId, Node, NodeGraph, NodeGraphError, NodeGraphOperation,
        NodeGraphOutcome, NodeId, Port, PortDirection, PortId,
    };

    let mut graph = NodeGraph::new();
    assert_eq!(graph.node_count(), 0);

    let id = graph.create_node("N".to_string(), Default::default());
    let _node: &Node = graph.node(id).unwrap();
    let _dir: PortDirection = PortDirection::Input;
    let _err: NodeGraphError = NodeGraphError::NodeNotFound { id };
    let _op: NodeGraphOperation = NodeGraphOperation::CreateNode {
        name: "X".to_string(),
        transform: Default::default(),
    };
    let _outcome: NodeGraphOutcome = NodeGraphOutcome::NodeCreated { id };
}

#[test]
fn viewport_3d_exports_are_accessible() {
    use app_shell::domain_foundation::environments::viewport_3d::{
        Object3D, ObjectId3D, Scene, Transform3D, Vec3, Viewport3DError,
        Viewport3DOperation, Viewport3DOutcome,
    };

    let mut scene = Scene::new();
    assert!(scene.is_empty());

    let id = scene.create_object(Transform3D::identity());
    let _obj: &Object3D = scene.get(id).unwrap();
    let _v: Vec3 = Vec3::new(1.0, 2.0, 3.0);
    let _err: Viewport3DError = Viewport3DError::ObjectNotFound { id };
    let _op: Viewport3DOperation = Viewport3DOperation::Create {
        transform: Transform3D::identity(),
    };
    let _outcome: Viewport3DOutcome = Viewport3DOutcome::Created { id };
}

#[test]
fn table_exports_are_accessible() {
    use app_shell::domain_foundation::environments::table::{
        CellId, CellValue, Column, ColumnId, Row, RowId, Table, TableError,
        TableOperation, TableOutcome,
    };

    let mut table = Table::new();
    assert!(table.is_empty());

    let row = table.insert_row(None);
    let col = table.insert_column(None);
    let _cell_id: CellId = CellId { row, column: col };
    let _val: CellValue = CellValue::Int(42);
    let _err: TableError = TableError::RowNotFound { id: row };
    let _op: TableOperation = TableOperation::InsertRow { name: None };
    let _outcome: TableOutcome = TableOutcome::RowInserted { id: row };
}

#[test]
fn text_editor_exports_are_accessible() {
    use app_shell::domain_foundation::environments::text_editor::{
        Document, Position, Range, TextEditorError, TextEditorOperation,
        TextEditorOutcome,
    };

    let doc = Document::new();
    assert!(doc.is_empty());

    let mut doc = doc;
    doc.insert(Position::new(0), "hello").unwrap();
    assert_eq!(doc.len(), 5);

    let _range: Range = Range::new(Position::new(0), Position::new(3));
    let _err: TextEditorError = TextEditorError::PositionOutOfBounds {
        position: 99,
        length: 5,
    };
    let _op: TextEditorOperation = TextEditorOperation::Clear;
    let _outcome: TextEditorOutcome = TextEditorOutcome::Cleared {
        previous_text: "x".to_string(),
    };
}

// --- Relations ---

#[test]
fn relations_exports_are_accessible() {
    use app_shell::domain_foundation::relations::{
        EffectFn, Endpoint, ObjectId, PropagationMode, PropertyId,
        RelationDefinition, RelationDefinitionBuilder, RelationError,
        RelationId, RelationRegistry, Trigger,
    };

    let registry = RelationRegistry::new();
    assert!(registry.is_empty());

    let src = Endpoint::source(PropertyId::new(ObjectId::new(1), 0));
    let tgt = Endpoint::target(PropertyId::new(ObjectId::new(2), 0));

    let _def: RelationDefinition = RelationDefinitionBuilder::new(src, tgt).build();
    let _err: RelationError = RelationError::RelationNotFound {
        id: RelationId::from_raw(0),
    };
    let _mode: PropagationMode = PropagationMode::Direct;
    let _trigger: Trigger = Trigger::OnChange;
    let _effect: EffectFn = std::sync::Arc::new(|_| {
        app_shell::domain_foundation::relations::EffectOutcome::NoChange
    });
}

// --- Utilities ---

#[test]
fn filtering_exports_are_accessible() {
    use app_shell::domain_foundation::utilities::filtering::{
        criteria, And, FilterCriteria, FnCriteria, Not, Or, all, any, count,
        filter,
    };

    let items = vec![1, 2, 3];
    let c = criteria(|x: &i32| *x > 1);
    let _result: Vec<&i32> = filter(&items, &c);
    let _count: usize = count(&items, &c);
    assert!(any(&items, &c));
    assert!(!all(&items, &c));

    let _composed: And<FnCriteria<_>, FnCriteria<_>> =
        criteria(|x: &i32| *x > 0).and(criteria(|x| *x < 10));
    let _or: Or<FnCriteria<_>, FnCriteria<_>> =
        criteria(|x: &i32| *x == 0).or(criteria(|x| *x == 1));
    let _not: Not<FnCriteria<_>> = criteria(|x: &i32| *x == 0).not();
}

#[test]
fn tagging_exports_are_accessible() {
    use app_shell::domain_foundation::utilities::tagging::{Tag, TagSet};

    let mut set = TagSet::new();
    set.add(Tag::new("a"));
    assert!(set.contains(&Tag::new("a")));
    assert_eq!(set.len(), 1);
}

#[test]
fn querying_exports_are_accessible() {
    use app_shell::domain_foundation::utilities::filtering::{criteria, FilterCriteria};
    use app_shell::domain_foundation::utilities::querying::{evaluate, Query};

    let q = Query::new("test".to_string(), criteria(|x: &i32| *x > 0));
    let items = vec![1, -2, 3];
    let result = evaluate(&q, &items);
    assert_eq!(result.len(), 2);
}

#[test]
fn sorting_exports_are_accessible() {
    use app_shell::domain_foundation::utilities::sorting::{by, sort, sorted, SortCriteria};

    let mut items = vec![3, 1, 2];
    sort(&mut items, &by(|a: &i32, b: &i32| a.cmp(b)));
    assert_eq!(items, vec![1, 2, 3]);

    let items2 = vec![3, 1, 2];
    let _refs: Vec<&i32> = sorted(&items2, &by(|a: &i32, b: &i32| a.cmp(b)));
}

#[test]
fn grouping_exports_are_accessible() {
    use app_shell::domain_foundation::utilities::grouping::{group_by, GroupCriteria};
    use std::collections::HashMap;

    let items = vec![1, 2, 3, 4];
    let groups: HashMap<bool, Vec<&i32>> = group_by(&items, |x| *x % 2 == 0);
    assert_eq!(groups.get(&true).unwrap().len(), 2);

    struct ByParity;
    impl GroupCriteria<i32> for ByParity {
        type Key = bool;
        fn group_key(&self, item: &i32) -> bool {
            *item % 2 == 0
        }
    }
    use app_shell::domain_foundation::utilities::grouping::group;
    let groups2 = group(&items, &ByParity);
    assert_eq!(groups2.get(&false).unwrap().len(), 2);
}

#[test]
fn validation_exports_are_accessible() {
    use app_shell::domain_foundation::utilities::validation::{
        rule, validate_all, FnRule, ValidationRule, ValidationResult,
    };

    let r = rule(|x: &i32| {
        if *x >= 0 {
            ValidationResult::valid()
        } else {
            ValidationResult::invalid("negative")
        }
    });

    assert!(r.validate(&5).is_valid());
    assert!(r.validate(&-1).is_invalid());

    let invalids = validate_all(&[1, -2, 3, -4], &r);
    assert_eq!(invalids.len(), 2);
}

#[test]
fn batch_exports_are_accessible() {
    use app_shell::domain_foundation::utilities::batch::{
        apply_batch, operation, BatchOperation,
    };

    let mut items = vec![1, 2, 3];
    let op = operation(|items: &mut [i32]| {
        for i in items.iter_mut() {
            *i += 1;
        }
    });
    apply_batch(&op, &mut items);
    assert_eq!(items, vec![2, 3, 4]);
}

// --- Domain Engine ---

#[test]
fn domain_engine_exports_are_accessible() {
    use app_shell::domain_engine::DomainEngine;
    use app_shell::domain_foundation::environments::canvas_2d::{
        CanvasOperation, CanvasOutcome, Transform, Vec2,
    };

    let mut engine = DomainEngine::new();
    let outcome = engine
        .execute(CanvasOperation::Create {
            transform: Transform::from_translation(Vec2::new(1.0, 2.0)),
        })
        .unwrap();

    match outcome {
        CanvasOutcome::Created { id } => assert!(engine.canvas().contains(id)),
        _ => panic!("expected Created"),
    }

    assert_eq!(engine.events().len(), 1);
    assert_eq!(engine.history().position(), 1);
}