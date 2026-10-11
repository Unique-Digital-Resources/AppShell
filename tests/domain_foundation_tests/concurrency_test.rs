//! Thread-safety tests: verify that environment types are Send + Sync
//! and can be shared across threads via Arc<Mutex<>>.

use app_shell::domain_foundation::environments::canvas_2d::{Canvas, Transform, Vec2};
use app_shell::domain_foundation::environments::timeline::{Duration, Time, Timeline};
use app_shell::domain_foundation::environments::layers::Layers;
use app_shell::domain_foundation::environments::table::Table;
use app_shell::domain_foundation::environments::text_editor::Document;
use app_shell::domain_foundation::environments::viewport_3d::{Scene, Transform3D, Vec3};
use app_shell::domain_foundation::relations::RelationRegistry;
use std::sync::{Arc, Mutex};
use std::thread;

// --- Compile-time Send + Sync bounds ---

#[test]
fn canvas_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Canvas>();
}

#[test]
fn timeline_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Timeline>();
}

#[test]
fn layers_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Layers>();
}

#[test]
fn table_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Table>();
}

#[test]
fn document_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Document>();
}

#[test]
fn scene_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Scene>();
}

#[test]
fn relation_registry_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<RelationRegistry>();
}

// --- Runtime: shared mutation across threads ---

#[test]
fn canvas_shared_across_threads() {
    let canvas = Arc::new(Mutex::new(Canvas::new()));

    let handles: Vec<_> = (0..4)
        .map(|i| {
            let canvas = Arc::clone(&canvas);
            thread::spawn(move || {
                let mut c = canvas.lock().unwrap();
                c.create_object(Transform::from_translation(Vec2::new(
                    i as f64,
                    i as f64,
                )));
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(canvas.lock().unwrap().len(), 4);
}

#[test]
fn document_shared_reads_across_threads() {
    let doc = Arc::new(Mutex::new(Document::from_str("hello world")));
    let doc_clone = Arc::clone(&doc);

    let handle = thread::spawn(move || {
        let d = doc_clone.lock().unwrap();
        assert_eq!(d.len(), 11);
    });

    handle.join().unwrap();
    assert_eq!(doc.lock().unwrap().len(), 11);
}

#[test]
fn timeline_concurrent_track_creation() {
    let timeline = Arc::new(Mutex::new(Timeline::new()));

    let handles: Vec<_> = (0..4)
        .map(|_| {
            let tl = Arc::clone(&timeline);
            thread::spawn(move || {
                let mut t = tl.lock().unwrap();
                let track = t.create_track();
                t.insert_clip(track, Time::new(0.0), Duration::new(5.0)).unwrap();
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(timeline.lock().unwrap().track_count(), 4);
    assert_eq!(timeline.lock().unwrap().clip_count(), 4);
}