//! Stress tests: large-scale operations to verify performance and
//! correctness at scale.

use app_shell::domain_foundation::environments::canvas_2d::{
    Canvas, Transform, Vec2,
};
use app_shell::domain_foundation::environments::table::{CellValue, Table};
use app_shell::domain_foundation::environments::text_editor::Document;
use app_shell::domain_foundation::environments::timeline::{Duration, Time, Timeline};

#[test]
fn canvas_100k_objects() {
    let mut canvas = Canvas::new();
    for i in 0..100_000u64 {
        canvas.create_object(Transform::from_translation(Vec2::new(
            i as f64,
            i as f64,
        )));
    }
    assert_eq!(canvas.len(), 100_000);

    // Draw order should have all objects.
    assert_eq!(canvas.draw_order().len(), 100_000);

    // Iterate and count.
    let count = canvas.iter().count();
    assert_eq!(count, 100_000);
}

#[test]
fn table_10k_cells() {
    let mut table = Table::new();
    let rows: Vec<_> = (0..100).map(|_| table.insert_row(None)).collect();
    let cols: Vec<_> = (0..100).map(|_| table.insert_column(None)).collect();

    for (i, &row) in rows.iter().enumerate() {
        for (j, &col) in cols.iter().enumerate() {
            table
                .set_cell_value(row, col, CellValue::Int((i * 100 + j) as i64))
                .unwrap();
        }
    }

    assert_eq!(table.cell_count(), 10_000);
}

#[test]
fn document_100k_chars() {
    let text = "x".repeat(100_000);
    let doc = Document::from_str(&text);
    assert_eq!(doc.len(), 100_000);
    assert_eq!(doc.byte_count(), 100_000);

    // len() should be O(1) due to caching.
    let len = doc.len();
    assert_eq!(len, 100_000);
}

#[test]
fn timeline_10k_clips() {
    let mut timeline = Timeline::new();
    let track = timeline.create_track();
    for i in 0..10_000u64 {
        timeline
            .insert_clip(track, Time::new(i as f64 * 5.0), Duration::new(5.0))
            .unwrap();
    }
    assert_eq!(timeline.clip_count(), 10_000);
    assert_eq!(timeline.end_time(), Time::new(50_000.0));
}