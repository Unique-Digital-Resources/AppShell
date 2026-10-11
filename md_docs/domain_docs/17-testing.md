# Testing

The Domain Foundation has **414+ tests** across 12 categories, covering model behavior, operation semantics, integration, composition, acceptance, concurrency, property-based, stress, serde round-trip, and App Engine lifecycle.

---

## Test Organization

All Domain Foundation tests live under `tests/domain_foundation_tests/` as submodules of a single test binary:

```text
tests/
└── domain_foundation_tests/
    ├── main.rs                            ← entry point, declares all submodules
    ├── canvas_2d_test.rs                 ← model tests
    ├── canvas_2d_operations_test.rs       ← operation tests
    ├── canvas_history_integration_test.rs ← undo/redo, transactions
    ├── canvas_command_handler_test.rs     ← command handler
    ├── relation_definition_test.rs        ← relation builder, criteria
    ├── relation_registry_test.rs         ← registry, lookup, clear
    ├── relation_evaluation_test.rs       ← evaluation, value store
    ├── relation_propagation_test.rs       ← cascading, batch, cycles
    ├── relation_value_source_test.rs      ← PropertyValueSource/Sink
    ├── timeline_test.rs
    ├── timeline_operations_test.rs
    ├── layers_test.rs
    ├── layers_operations_test.rs
    ├── node_graph_test.rs
    ├── node_graph_operations_test.rs
    ├── node_graph_canvas_integration_test.rs
    ├── viewport_3d_test.rs
    ├── viewport_3d_operations_test.rs
    ├── table_test.rs
    ├── table_operations_test.rs
    ├── text_editor_test.rs
    ├── text_editor_operations_test.rs
    ├── filtering_test.rs
    ├── querying_test.rs
    ├── tagging_test.rs
    ├── sorting_test.rs
    ├── grouping_test.rs
    ├── validation_test.rs
    ├── batch_test.rs
    ├── concurrency_test.rs               ← Send + Sync, threads
    ├── proptest_test.rs                  ← property-based
    ├── stress_test.rs                    ← 100K objects
    ├── serde_roundtrip_test.rs           ← serialize → deserialize → assert_eq
    ├── app_engine_lifecycle_test.rs      ← full AppEngine lifecycle
    ├── domain_foundation_test.rs         ← public API smoke
    ├── domain_composition_test.rs        ← boundaries, composition
    └── domain_foundation_acceptance_test.rs ← end-to-end acceptance
```

### Entry point (`main.rs`)

```rust
//! Phase 2–12 integration tests.

mod canvas_2d_test;
mod canvas_2d_operations_test;
mod relation_definition_test;
// ... all modules ...
mod domain_foundation_test;
mod domain_composition_test;
mod domain_foundation_acceptance_test;

#[cfg(feature = "serde")]
mod serde_roundtrip_test;
```

Run tests with:

```sh
# Regular tests
cargo test --test domain_foundation_tests

# With serde (enables round-trip tests)
cargo test --features serde --test domain_foundation_tests

# Specific module
cargo test --test domain_foundation_tests canvas_2d_test

# Stress tests only
cargo test --test domain_foundation_tests stress

# Benchmarks
cargo bench
```

---

## Test Categories

### Model Tests

Verify each environment's types, queries, and constraints:

```rust
#[test]
fn create_object_returns_unique_ids() {
    let mut canvas = Canvas::new();
    let a = canvas.create_object(Transform::identity());
    let b = canvas.create_object(Transform::identity());
    assert_ne!(a, b);
}
```

| Test file | Tests | What it covers |
|---|---|---|
| `canvas_2d_test.rs` | 15+ | Create, contains, iter, clear, z-order, spatial queries, Vec2 math |
| `timeline_test.rs` | 18+ | Tracks, clips, keyframes, overlap, clips_at, move_track, split |
| `layers_test.rs` | 18+ | Hierarchy, visibility, locked, reparent, move_up/down, layer_path |
| `node_graph_test.rs` | 15+ | Connect, validation, topo sort, cycles, max_inputs, is_connected |
| `viewport_3d_test.rs` | 17+ | Transforms, parent-child, world_transform, spatial queries, Vec3 math |
| `table_test.rs` | 18+ | Cells, merge, swap, ordering, IntoIterator |
| `text_editor_test.rs` | 30+ | Unicode, find/replace, line/col, ranges, Position arithmetic |

### Operation Tests

Verify each operation variant, error cases, and composition:

```rust
#[test]
fn create_operation_adds_an_object() {
    let mut canvas = Canvas::new();
    let outcome = CanvasOperation::Create { transform: Transform::identity() }
        .apply(&mut canvas)
        .unwrap();
    assert!(matches!(outcome, CanvasOutcome::Created { .. }));
    assert_eq!(canvas.len(), 1);
}
```

| Test file | Tests |
|---|---|
| `canvas_2d_operations_test.rs` | 11 operations + z-order undo |
| `timeline_operations_test.rs` | 11 operations (trim, split, set keyframe) |
| `layers_operations_test.rs` | 10 operations (duplicate, clear) |
| `node_graph_operations_test.rs` | 13 operations (rename port, duplicate, clear) |
| `viewport_3d_operations_test.rs` | 8 operations (set parent, clear) |
| `table_operations_test.rs` | 13 operations (merge, unmerge, clear) |
| `text_editor_operations_test.rs` | 7 operations (append, delete line, replace line) |

### Integration Tests

Verify DomainEngine + App Engine (undo/redo, events, transactions):

```rust
#[test]
fn full_undo_redo_cycle() {
    let mut engine = DomainEngine::new();
    // Create + Move + Transform
    engine.execute(Create { ... }).unwrap();
    engine.execute(Move { ... }).unwrap();
    engine.execute(Transform { ... }).unwrap();

    // Undo all 3
    engine.undo().unwrap();
    engine.undo().unwrap();
    engine.undo().unwrap();
    assert!(!engine.canvas().contains(id));

    // Redo all 3
    engine.redo().unwrap();
    engine.redo().unwrap();
    engine.redo().unwrap();
    assert!(engine.canvas().contains(id));
}
```

| Test file | Tests |
|---|---|
| `canvas_history_integration_test.rs` | Undo/redo, transactions, rollback, compound, dirty tracking |
| `domain_operation_integration_test.rs` | Execute through DomainEngine for each environment |
| `canvas_command_handler_test.rs` | CanvasCommandHandler definition and execution |

### Composition Tests

Verify architectural boundaries:

```rust
#[test]
fn canvas_2d_works_without_other_environments() {
    let mut canvas = Canvas::new();
    let id = canvas.create_object(Transform::identity());
    assert!(canvas.contains(id));
}

#[test]
fn node_graph_and_canvas_are_composed_not_coupled() {
    let mut graph = NodeGraph::new();
    let n = graph.create_node("N".into(), Transform::identity());
    graph.move_node(n, Vec2::new(50.0, 50.0)).unwrap();
    assert_eq!(graph.node_count(), 1);   // Graph unchanged
    assert_eq!(graph.canvas().len(), 1); // Canvas changed
}
```

| Test file | Tests |
|---|---|
| `domain_composition_test.rs` | Each environment standalone, NodeGraph→Canvas2D, Relations independence, utilities orthogonality, App Engine domain-agnostic |

### Acceptance Tests

End-to-end tests simulating real Domain Engines:

```rust
#[test]
fn vector_editor_composes_canvas_layers_and_relations() {
    // Simulates a vector editor using Canvas2D + Layers + Relations + Filtering
    let mut canvas = Canvas::new();
    let mut layers = Layers::new();
    let mut registry = RelationRegistry::new();
    // ... full setup ...
}
```

| Test file | Simulated Domain Engine | Environments used |
|---|---|---|
| `domain_foundation_acceptance_test.rs` | Vector Editor | Canvas2D + Layers + Relations + Filtering |
| | Spreadsheet | Table + Tagging + Validation + Relations |
| | Text Editor | TextEditor only |
| | Video Editor | Timeline + Layers |

### Concurrency Tests

Verify `Send + Sync` bounds and thread-safe shared mutation:

```rust
#[test]
fn canvas_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Canvas>();
}

#[test]
fn canvas_shared_across_threads() {
    let canvas = Arc::new(Mutex::new(Canvas::new()));
    let handles: Vec<_> = (0..4).map(|i| {
        let canvas = Arc::clone(&canvas);
        thread::spawn(move || {
            let mut c = canvas.lock().unwrap();
            c.create_object(Transform::from_translation(Vec2::new(i as f64, i as f64)));
        })
    }).collect();
    for h in handles { h.join().unwrap(); }
    assert_eq!(canvas.lock().unwrap().len(), 4);
}
```

| Test file | Tests |
|---|---|
| `concurrency_test.rs` | Send + Sync for all environments + RelationRegistry, shared mutation across threads |

### Property-Based Tests

Use `proptest` to verify invariants across random inputs:

```rust
proptest! {
    #[test]
    fn insert_then_delete_restores_text(
        original in "[a-z]{0,50}",
        pos in 0u32..50,
        inserted in "[a-z]{0,20}"
    ) {
        let mut doc = Document::from_str(&original);
        let len = doc.len();
        if pos <= len {
            doc.insert(Position::new(pos), &inserted).unwrap();
            let inserted_len = inserted.chars().count() as u32;
            doc.delete(Range::new(
                Position::new(pos),
                Position::new(pos + inserted_len),
            )).unwrap();
            prop_assert_eq!(doc.text(), original);
        }
    }

    #[test]
    fn undo_redo_preserves_canvas_state(ops in prop::collection::vec(canvas_op_strategy(), 1..10)) {
        let mut engine = DomainEngine::new();
        // Execute, undo all, redo all, verify same state
    }
}
```

| Test file | Properties tested |
|---|---|
| `proptest_test.rs` | Insert-then-delete restores text, replace preserves length, undo-redo preserves canvas state, canvas count invariant, table set-then-clear returns None |

### Stress Tests

Large-scale operations to verify performance and correctness at scale:

```rust
#[test]
fn canvas_100k_objects() {
    let mut canvas = Canvas::new();
    for i in 0..100_000u64 {
        canvas.create_object(Transform::from_translation(Vec2::new(i as f64, i as f64)));
    }
    assert_eq!(canvas.len(), 100_000);
    assert_eq!(canvas.draw_order().len(), 100_000);
}

#[test]
fn document_100k_chars() {
    let text = "x".repeat(100_000);
    let doc = Document::from_str(&text);
    assert_eq!(doc.len(), 100_000);  // O(1) due to caching
    assert_eq!(doc.byte_count(), 100_000);
}
```

| Test | Scale |
|---|---|
| `canvas_100k_objects` | 100,000 objects |
| `table_10k_cells` | 10,000 cells (100 rows × 100 columns) |
| `document_100k_chars` | 100,000 characters |
| `timeline_10k_clips` | 10,000 clips on one track |

### Serde Round-Trip Tests

Serialize → Deserialize → `assert_eq`:

```rust
#[cfg(feature = "serde")]
#[test]
fn canvas_roundtrip() {
    let mut canvas = Canvas::new();
    canvas.create_object(Transform::from_translation(Vec2::new(1.0, 2.0)));
    let json = serde_json::to_string(&canvas).unwrap();
    let restored: Canvas = serde_json::from_str(&json).unwrap();
    assert_eq!(canvas, restored);
}
```

| Test | What survives round-trip |
|---|---|
| `canvas_roundtrip` | Objects, draw_order, next_id |
| `timeline_roundtrip` | Tracks, clips, keyframes, interpolation |
| `table_structure_roundtrip` | Rows, columns (cells don't — JSON limitation) |
| `document_roundtrip` | Text, char_count |

### App Engine Lifecycle Test

Full `AppEngine` lifecycle with DomainEngine:

```rust
#[test]
fn app_engine_lifecycle_works() {
    let mut engine = AppEngine::new().unwrap();
    assert_eq!(engine.state(), LifecycleState::Created);

    engine.initialize().unwrap();
    assert_eq!(engine.state(), LifecycleState::Initialized);

    engine.start().unwrap();
    assert_eq!(engine.state(), LifecycleState::Running);

    assert!(engine.history_store().is_empty());

    engine.stop().unwrap();
    assert_eq!(engine.state(), LifecycleState::Stopped);

    engine.dispose().unwrap();
    assert_eq!(engine.state(), LifecycleState::Disposed);
}
```

---

## Testing Conventions

### Struct literal in match

Never write `match Enum::Variant { fields }.method() { ... }` — the parser can't disambiguate the `{`. Bind to a variable first:

```rust
// WRONG — won't compile
let id = match CanvasOperation::Create { transform }.apply(&mut canvas).unwrap() { ... };

// CORRECT
let outcome = CanvasOperation::Create { transform }.apply(&mut canvas).unwrap();
let id = match outcome {
    CanvasOutcome::Created { id } => id,
    _ => unreachable!(),
};
```

### Import conventions

```rust
// Test files import via the crate name
use app_shell::domain_foundation::environments::canvas_2d::{
    Canvas, CanvasError, Object, ObjectId, Transform, Vec2,
};

// For traits that provide methods (like FilterCriteria), import the trait
use app_shell::domain_foundation::utilities::filtering::{criteria, FilterCriteria};
```

### Test file naming

Test files use `_test` suffix (e.g., `canvas_2d_test.rs`) matching the `_test` suffix on the module name in `main.rs`.

---

## Test Count by Category

| Category | Test files | Tests |
|---|---|---|
| Model | 7 | ~120 |
| Operations | 7 | ~80 |
| Integration | 3 | ~25 |
| Composition | 1 | ~10 |
| Acceptance | 1 | ~5 |
| API | 1 | ~15 |
| Concurrency | 1 | ~10 |
| Proptest | 1 | ~5 |
| Stress | 1 | 4 |
| Serde | 1 | ~4 |
| Lifecycle | 1 | 1 |
| Relations | 4 | ~30 |
| Utilities | 7 | ~35 |
| **Total** | **35+** | **414+** |

---

## Next Steps

- [18-public-api-reference.md](18-public-api-reference.md) — Complete type listing.
- [19-error-reference.md](19-error-reference.md) — All error types.
- [20-quick-start.md](20-quick-start.md) — 5-minute getting started.
- [13-domain-engine-integration.md](13-domain-engine-integration.md) — DomainEngine (tested in integration tests).