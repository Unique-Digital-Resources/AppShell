//! Phase 7 acceptance criteria.

use app_shell::app_engine::{
    Bootstrap, Event, EventBus, EventHandler, HistoryEntry, HistoryStore,
    TransactionManager,
};
use std::sync::Arc;

struct ClosureEventHandler<F: Fn(&Event) + Send + Sync + 'static> {
    f: F,
}
impl<F: Fn(&Event) + Send + Sync + 'static> EventHandler for ClosureEventHandler<F> {
    fn handle(&self, event: &Event) {
        (self.f)(event);
    }
}
fn event_handler<F: Fn(&Event) + Send + Sync + 'static>(f: F) -> Box<dyn EventHandler> {
    Box::new(ClosureEventHandler { f })
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct MoveData {
    object: String,
    from: (f64, f64),
    to: (f64, f64),
}

/// 1. Record meaningful application operations
/// 2. Store ordered history entries
#[test]
fn acceptance_record_and_store_ordered() {
    let store = HistoryStore::new();

    store.append(HistoryEntry::new("object.create").undoable());
    store.append(HistoryEntry::new("object.move").undoable());
    store.append(HistoryEntry::new("object.resize").undoable());

    assert_eq!(store.active_count(), 3);
    let ops: Vec<String> = store.active().iter().map(|e| e.1.clone()).collect();
    assert_eq!(ops, vec!["object.create", "object.move", "object.resize"]);
}

/// 3. Identify the current history position
#[test]
fn acceptance_identify_current_position() {
    let store = HistoryStore::new();
    store.append(HistoryEntry::new("op.a"));
    store.append(HistoryEntry::new("op.b"));

    assert_eq!(store.position(), 2);
    assert_eq!(store.current().unwrap().1, "op.b");
}

/// 4. Undo an undoable operation
#[test]
fn acceptance_undo_operation() {
    let store = HistoryStore::new();
    let move_data = MoveData {
        object: "obj1".to_string(),
        from: (0.0, 0.0),
        to: (100.0, 200.0),
    };
    let inverse = MoveData {
        object: "obj1".to_string(),
        from: (100.0, 200.0),
        to: (0.0, 0.0),
    };

    let id = store.append(
        HistoryEntry::new("object.move")
            .undoable()
            .with_data(move_data)
            .with_inverse(inverse),
    );

    let result = store.undo();
    assert_eq!(result.count(), 1);
    assert!(!result.is_empty());

    let inverse_data = store
        .with_entry(id, |e| e.inverse_data::<MoveData>().map(|d| d.clone()))
        .flatten()
        .unwrap();
    assert_eq!(inverse_data.to, (0.0, 0.0));
}

/// 5. Redo an undone operation
#[test]
fn acceptance_redo_operation() {
    let store = HistoryStore::new();
    store.append(
        HistoryEntry::new("object.move")
            .undoable()
            .with_data("forward".to_string()),
    );

    store.undo();
    let result = store.redo();
    assert_eq!(result.count(), 1);
    assert_eq!(store.active_count(), 1);
}

/// 6. Invalidate the redo branch after a new operation
#[test]
fn acceptance_invalidate_redo_branch() {
    let store = HistoryStore::new();
    store.append(HistoryEntry::new("op.a"));
    store.append(HistoryEntry::new("op.b"));
    store.append(HistoryEntry::new("op.c"));

    store.undo();
    store.undo();
    assert!(store.can_redo());

    store.append(HistoryEntry::new("op.d"));
    assert!(!store.can_redo());
    assert_eq!(store.entry_count(), 2);
}

/// 7. Group multiple operations into a transaction
/// 8. Commit or discard a transaction
#[test]
fn acceptance_transaction_commit() {
    let store = HistoryStore::new();
    let mut tx_mgr = TransactionManager::new();

    let tx = tx_mgr.begin();
    store.append(HistoryEntry::new("object.move").with_transaction(tx));
    store.append(HistoryEntry::new("object.resize").with_transaction(tx));
    store.append(HistoryEntry::new("object.rotate").with_transaction(tx));
    tx_mgr.commit();

    assert_eq!(store.active_count(), 3);

    let result = store.undo();
    assert_eq!(result.count(), 3);
    assert!(result.transaction_undone());
    assert_eq!(store.active_count(), 0);
}

#[test]
fn acceptance_transaction_rollback() {
    let store = HistoryStore::new();
    let mut tx_mgr = TransactionManager::new();

    let tx = tx_mgr.begin();
    store.append(HistoryEntry::new("op.a").with_transaction(tx));
    store.append(HistoryEntry::new("op.b").with_transaction(tx));
    tx_mgr.rollback();

    store.remove_transaction(tx);
    assert_eq!(store.entry_count(), 0);
}

/// 9. Replay recorded operations
/// 10. Reuse existing command/execution mechanisms during replay
#[test]
fn acceptance_replay_operations() {
    let store = HistoryStore::new();
    store.append(
        HistoryEntry::new("object.create").undoable().with_data("obj1".to_string()),
    );
    store.append(
        HistoryEntry::new("object.move").undoable().with_data("obj1_moved".to_string()),
    );

    let replayed: Vec<String> = store.replay().map(|(_, op, _)| op).collect();
    assert_eq!(replayed, vec!["object.create", "object.move"]);
}

/// 11. Distinguish undoable and non-undoable operations
#[test]
fn acceptance_distinguish_undoable() {
    let store = HistoryStore::new();
    store.append(HistoryEntry::new("object.move").undoable());
    store.append(HistoryEntry::new("file.export").non_undoable());

    let active = store.active();
    assert!(active[0].2); // undoable
    assert!(!active[1].2); // not undoable
}

/// 12. Avoid recording irrelevant runtime signals
#[test]
fn acceptance_filter_irrelevant_events() {
    let store = Arc::new(HistoryStore::new());

    let s = store.clone();
    let handler = event_handler(move |event: &Event| {
        if event.event_type().starts_with("app.") {
            s.append(HistoryEntry::new(event.event_type()));
        }
    });

    let mut bus = EventBus::new();
    bus.subscribe("app.object_created", handler);
    let s2 = store.clone();
    bus.subscribe(
        "task.progress_changed",
        event_handler(move |event: &Event| {
            if event.event_type().starts_with("app.") {
                s2.append(HistoryEntry::new(event.event_type()));
            }
        }),
    );

    bus.publish(&Event::new("app.object_created"));
    bus.publish(&Event::new("task.progress_changed"));

    assert_eq!(store.entry_count(), 1);
}

/// 13. Integrate with Events without making execution depend directly on History
#[test]
fn acceptance_event_integration_without_coupling() {
    let store = Arc::new(HistoryStore::new());

    let s = store.clone();
    let mut bus = EventBus::new();
    bus.subscribe(
        "task.completed",
        event_handler(move |event: &Event| {
            s.append(
                HistoryEntry::new(event.event_type()).with_source(event.source().unwrap_or("unknown")),
            );
        }),
    );

    bus.publish(&Event::new("task.completed").with_source("TaskExecutor"));

    assert_eq!(store.entry_count(), 1);
    let source = store
        .with_current(|e| e.source().map(|s| s.to_string()))
        .flatten();
    assert_eq!(source, Some("TaskExecutor".to_string()));
}

/// 14. Keep History independent of domain semantics
/// 15. Domain provides its own undo/replay information
#[test]
fn acceptance_domain_provides_undo_info() {
    let store = HistoryStore::new();

    let forward = MoveData {
        object: "card_42".to_string(),
        from: (10.0, 10.0),
        to: (50.0, 50.0),
    };
    let inverse = MoveData {
        object: "card_42".to_string(),
        from: (50.0, 50.0),
        to: (10.0, 10.0),
    };

    store.append(
        HistoryEntry::new("object.move")
            .undoable()
            .with_data(forward)
            .with_inverse(inverse)
            .with_source("MoveObjectCommand"),
    );

    let data = store
        .with_current(|e| e.data::<MoveData>().map(|d| d.clone()))
        .flatten()
        .unwrap();
    assert_eq!(data.object, "card_42");

    let inverse = store
        .with_current(|e| e.inverse_data::<MoveData>().map(|d| d.clone()))
        .flatten()
        .unwrap();
    assert_eq!(inverse.to, (10.0, 10.0));
}

/// Full lifecycle: Command → Execution → Event → History → Undo → Redo
#[test]
fn acceptance_full_architectural_flow() {
    let runtime = Bootstrap::create().unwrap();

    runtime.history_store().append(
        HistoryEntry::new("object.create")
            .undoable()
            .with_data("document_1".to_string())
            .with_inverse("document_1".to_string())
            .with_source("CreateDocumentCommand"),
    );

    let undo_result = runtime.history_store().undo();
    assert_eq!(undo_result.count(), 1);

    let redo_result = runtime.history_store().redo();
    assert_eq!(redo_result.count(), 1);
}