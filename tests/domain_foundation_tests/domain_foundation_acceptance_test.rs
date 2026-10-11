//! Acceptance tests: two simulated Domain Engines compose different
//! subsets of the foundation, proving the architecture supports
//! selective composition.

use app_shell::domain_engine::DomainEngine;
use app_shell::domain_foundation::environments::canvas_2d::{
    Canvas, CanvasOperation, CanvasOutcome, Transform, Vec2,
};
use app_shell::domain_foundation::environments::layers::{
    Layers, LayersOperation, LayersOutcome,
};
use app_shell::domain_foundation::environments::timeline::{
    Duration, Time, Timeline, TimelineOperation, TimelineOutcome,
};
use app_shell::domain_foundation::environments::table::{
    CellValue, Table, TableOperation, TableOutcome,
};
use app_shell::domain_foundation::environments::text_editor::{
    Document, Position, Range, TextEditorOperation, TextEditorOutcome,
};
use app_shell::domain_foundation::relations::{
    Endpoint, ObjectId, PropagationMode, PropertyId, RelationDefinitionBuilder,
    RelationRegistry, Trigger,
};
use app_shell::domain_foundation::utilities::filtering::{criteria, filter};
use app_shell::domain_foundation::utilities::tagging::{Tag, TagSet};
use app_shell::domain_foundation::utilities::validation::{rule, ValidationResult, ValidationRule};

// ============================================================
// Simulated Domain Engine A: Vector Editor
// Composes: Canvas2D + Layers + Relations + Filtering + App Engine
// ============================================================

#[test]
fn vector_editor_composes_canvas_layers_and_relations() {
    // --- Canvas2D for spatial placement ---
    let mut canvas = Canvas::new();
    let rect = canvas.create_object(Transform::from_translation(Vec2::new(0.0, 0.0)));
    let circle = canvas.create_object(Transform::from_translation(Vec2::new(100.0, 0.0)));

    // --- Layers for hierarchy ---
    let mut layers = Layers::new();
    let bg = layers.create_layer("Background".to_string());
    let fg = layers.create_layer("Foreground".to_string());
    layers.create_child(fg, "Shapes".to_string()).unwrap();

    assert_eq!(layers.len(), 3);
    assert_eq!(layers.layer(fg).unwrap().child_count(), 1);

    // --- Relations for property propagation ---
    let mut registry = RelationRegistry::new();
    use app_shell::domain_foundation::relations::definition::{EffectContext, EffectOutcome};
    use std::sync::Arc;

    let src = Endpoint::source(PropertyId::new(ObjectId::new(rect.as_raw()), 0));
    let tgt = Endpoint::target(PropertyId::new(ObjectId::new(circle.as_raw()), 0));

    registry.register(
        RelationDefinitionBuilder::new(src, tgt)
            .trigger(Trigger::OnChange)
            .propagation(PropagationMode::Cascading)
            .effect(Arc::new(|ctx: &EffectContext| {
                EffectOutcome::Apply(ctx.source_value.clone())
            }))
            .build(),
    );
    assert_eq!(registry.len(), 1);

    // --- Filtering to select objects by position ---
    let objects: Vec<_> = canvas.iter().copied().collect();
    let right_half = filter(&objects, &criteria(|obj: &app_shell::domain_foundation::environments::canvas_2d::Object| {
        obj.transform.translation.x > 50.0
    }));
    assert_eq!(right_half.len(), 1); // only circle

    // --- App Engine for undo/redo ---
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
            translation: Vec2::new(10.0, 20.0),
        })
        .unwrap();

    engine.undo().unwrap();
    assert_eq!(
        engine.canvas().get(id).unwrap().transform.translation,
        Vec2::new(0.0, 0.0)
    );

    engine.redo().unwrap();
    assert_eq!(
        engine.canvas().get(id).unwrap().transform.translation,
        Vec2::new(10.0, 20.0)
    );
}

// ============================================================
// Simulated Domain Engine B: Spreadsheet
// Composes: Table + Relations + Tagging + Validation
// ============================================================

#[test]
fn spreadsheet_composes_table_tagging_and_validation() {
    // --- Table for tabular data ---
    let mut table = Table::new();
    let header = table.insert_row(Some("Header".to_string()));
    let data1 = table.insert_row(Some("Row 1".to_string()));
    let data2 = table.insert_row(Some("Row 2".to_string()));
    let col_a = table.insert_column(Some("A".to_string()));
    let col_b = table.insert_column(Some("B".to_string()));

    table.set_cell_value(header, col_a, CellValue::Text("Name".to_string())).unwrap();
    table.set_cell_value(data1, col_a, CellValue::Text("Alice".to_string())).unwrap();
    table.set_cell_value(data2, col_a, CellValue::Text("Bob".to_string())).unwrap();
    table.set_cell_value(data1, col_b, CellValue::Int(30)).unwrap();
    table.set_cell_value(data2, col_b, CellValue::Int(-5)).unwrap();

    assert_eq!(table.cell_count(), 5);

    // --- Tagging for classification ---
    let mut tags = TagSet::new();
    tags.add(Tag::new("validated"));
    tags.add(Tag::new("locked"));
    assert_eq!(tags.len(), 2);
    assert!(tags.contains(&Tag::new("validated")));

    // --- Validation: B column values must be non-negative ---
    let validate_age = rule(|v: &i64| {
        if *v >= 0 {
            ValidationResult::valid()
        } else {
            ValidationResult::invalid("age must be non-negative")
        }
    });

    let val1: i64 = match table.cell(data1, col_b) {
        CellValue::Int(v) => *v,
        _ => 0,
    };
    let val2: i64 = match table.cell(data2, col_b) {
        CellValue::Int(v) => *v,
        _ => 0,
    };

    assert!(validate_age.validate(&val1).is_valid());
    assert!(validate_age.validate(&val2).is_invalid());

    // --- Relations for cell dependencies ---
    let mut registry = RelationRegistry::new();
    let src = Endpoint::source(PropertyId::new(ObjectId::new(data1.as_raw()), col_b.as_raw() as u32));
    let tgt = Endpoint::target(PropertyId::new(ObjectId::new(data2.as_raw()), col_b.as_raw() as u32));

    registry.register(
        RelationDefinitionBuilder::new(src, tgt)
            .trigger(Trigger::OnChange)
            .propagation(PropagationMode::Direct)
            .effect(std::sync::Arc::new(|ctx: &app_shell::domain_foundation::relations::EffectContext| {
                app_shell::domain_foundation::relations::EffectOutcome::NoChange
            }))
            .build(),
    );
    assert_eq!(registry.len(), 1);
}

// ============================================================
// Simulated Domain Engine C: Simple Text Editor
// Composes: TextEditor + App Engine (via DomainEngine pattern)
// ============================================================

#[test]
fn text_editor_composes_document_and_operations() {
    let mut doc = Document::new();

    // Type some text.
    let outcome = TextEditorOperation::Insert {
        position: Position::new(0),
        text: "Hello".to_string(),
    }
    .apply(&mut doc)
    .unwrap();

    let range = match outcome {
        TextEditorOutcome::Inserted { range } => range,
        _ => unreachable!(),
    };
    assert_eq!(range, Range::new(Position::new(0), Position::new(5)));

    // Append more text.
    TextEditorOperation::Insert {
        position: Position::new(5),
        text: " World".to_string(),
    }
    .apply(&mut doc)
    .unwrap();

    assert_eq!(doc.text(), "Hello World");
    assert_eq!(doc.len(), 11);

    // Replace "Hello" with "Goodbye".
    let outcome = TextEditorOperation::Replace {
        range: Range::new(Position::new(0), Position::new(5)),
        text: "Goodbye".to_string(),
    }
    .apply(&mut doc)
    .unwrap();

    match outcome {
        TextEditorOutcome::Replaced { previous_text, .. } => {
            assert_eq!(previous_text, "Hello");
        }
        _ => unreachable!(),
    }
    assert_eq!(doc.text(), "Goodbye World");

    // Delete " World".
    TextEditorOperation::Delete {
        range: Range::new(Position::new(7), Position::new(13)),
    }
    .apply(&mut doc)
    .unwrap();

    assert_eq!(doc.text(), "Goodbye");

    // Clear.
    TextEditorOperation::Clear.apply(&mut doc).unwrap();
    assert!(doc.is_empty());
}

// ============================================================
// Timeline + Layers: a video-editor-style composition
// ============================================================

#[test]
fn video_editor_composes_timeline_and_layers() {
    // --- Timeline for temporal data ---
    let mut timeline = Timeline::new();
    let video_track = timeline.create_track();
    let audio_track = timeline.create_track();

    let clip1 = timeline
        .insert_clip(video_track, Time::new(0.0), Duration::new(10.0))
        .unwrap();
    let clip2 = timeline
        .insert_clip(video_track, Time::new(10.0), Duration::new(15.0))
        .unwrap();
    let _audio = timeline
        .insert_clip(audio_track, Time::new(0.0), Duration::new(25.0))
        .unwrap();

    assert_eq!(timeline.track_count(), 2);
    assert_eq!(timeline.clip_count(), 3);
    assert_eq!(timeline.end_time(), Time::new(25.0));

    // Move clip1 — should fail (overlaps clip2).
    let err = timeline.move_clip(clip1, Time::new(5.0)).unwrap_err();
    assert!(matches!(
        err,
        app_shell::domain_foundation::environments::timeline::TimelineError::ClipOverlap { .. }
    ));

    // --- Layers for visual hierarchy ---
    let mut layers = Layers::new();
    let root = layers.create_layer("Scene".to_string());
    let _child = layers.create_child(root, "Overlay".to_string()).unwrap();

    layers.set_visibility(root, false).unwrap();
    assert!(!layers.layer(root).unwrap().visible);

    // Timeline and Layers are independent — no shared types.
    assert_eq!(timeline.track_count(), 2);
    assert_eq!(layers.len(), 2);
}

// ============================================================
// Full undo/redo across multiple operation types
// ============================================================

#[test]
fn full_undo_redo_cycle_with_domain_engine() {
    let mut engine = DomainEngine::new();

    // Create + Move + Transform + Remove + Undo all + Redo all.
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
            translation: Vec2::new(3.0, 4.0),
        })
        .unwrap();

    let final_tf = Transform::from_translation(Vec2::new(3.0, 4.0)).with_rotation(0.5);
    engine
        .execute(CanvasOperation::Transform {
            id,
            transform: final_tf,
        })
        .unwrap();

    let original = engine.canvas().get(id).unwrap().transform;

    // Undo all 3.
    engine.undo().unwrap(); // undo transform
    engine.undo().unwrap(); // undo move
    engine.undo().unwrap(); // undo create
    assert!(!engine.canvas().contains(id));
    assert_eq!(engine.history().position(), 0);
    assert!(!engine.can_undo());

    // Redo all 3.
    engine.redo().unwrap(); // redo create
    engine.redo().unwrap(); // redo move
    engine.redo().unwrap(); // redo transform

    assert!(engine.canvas().contains(id));
    assert_eq!(engine.canvas().get(id).unwrap().transform, original);
    assert_eq!(engine.history().position(), 3);

    // 3 execute events + 3 undo events + 3 redo events = 9
    assert_eq!(engine.events().len(), 9);
}