//! Benchmarks for Domain Foundation hot paths.
//! Run with: cargo bench

use app_shell::domain_foundation::environments::canvas_2d::{Canvas, Transform, Vec2};
use app_shell::domain_foundation::environments::text_editor::Document;
use app_shell::domain_foundation::environments::timeline::{Duration, Time, Timeline};
use app_shell::domain_foundation::relations::{
    Endpoint, ObjectId, PropagationMode, PropertyId, RelationDefinitionBuilder,
    RelationRegistry, Trigger,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_canvas_create(c: &mut Criterion) {
    c.bench_function("canvas_create_100_objects", |b| {
        b.iter(|| {
            let mut canvas = Canvas::new();
            for _ in 0..100 {
                canvas.create_object(Transform::identity());
            }
            black_box(&canvas);
        })
    });
}

fn bench_document_len(c: &mut Criterion) {
    c.bench_function("document_len_1000_chars", |b| {
        let text = "x".repeat(1000);
        let doc = Document::from_str(&text);
        b.iter(|| {
            black_box(doc.len());
        })
    });

    c.bench_function("document_insert_at_end", |b| {
        let text = "x".repeat(1000);
        b.iter(|| {
            let mut doc = Document::from_str(&text);
            doc.insert(
                app_shell::domain_foundation::environments::text_editor::Position::new(1000),
                "hello",
            )
            .unwrap();
            black_box(&doc);
        })
    });
}

fn bench_timeline_insert_clip(c: &mut Criterion) {
    c.bench_function("timeline_insert_50_clips", |b| {
        b.iter(|| {
            let mut timeline = Timeline::new();
            let track = timeline.create_track();
            for i in 0..50 {
                timeline
                    .insert_clip(track, Time::new(i as f64 * 5.0), Duration::new(5.0))
                    .unwrap();
            }
            black_box(&timeline);
        })
    });
}

fn bench_relation_register(c: &mut Criterion) {
    c.bench_function("relation_register_100", |b| {
        b.iter(|| {
            let mut registry = RelationRegistry::new();
            for i in 0..100u64 {
                let src = Endpoint::source(PropertyId::new(ObjectId::new(i), 0));
                let tgt = Endpoint::target(PropertyId::new(ObjectId::new(i + 1000), 0));
                registry.register(
                    RelationDefinitionBuilder::new(src, tgt)
                        .trigger(Trigger::OnChange)
                        .propagation(PropagationMode::Direct)
                        .build(),
                );
            }
            black_box(&registry);
        })
    });
}

criterion_group!(
    benches,
    bench_canvas_create,
    bench_document_len,
    bench_timeline_insert_clip,
    bench_relation_register,
);
criterion_main!(benches);