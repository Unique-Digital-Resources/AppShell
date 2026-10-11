# Error Reference

All error types in the Domain Foundation, organized by module.

---

## Environments

### Canvas2D

```text
domain_foundation::environments::canvas_2d::CanvasError
```

| Variant | Fields | When | How to handle |
|---|---|---|---|
| `ObjectNotFound` | `{ id: ObjectId }` | Lookup, mutation, or z-order operation on a non-existent object | Check `canvas.contains(id)` before operating |

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanvasError {
    ObjectNotFound { id: ObjectId },
}
```

**Display:** `"no canvas object with id object#0"`

---

### Timeline

```text
domain_foundation::environments::timeline::TimelineError
```

| Variant | Fields | When | How to handle |
|---|---|---|---|
| `TrackNotFound` | `{ id: TrackId }` | Operation on a non-existent track | Check `timeline.contains_track(id)` |
| `ClipNotFound` | `{ id: ClipId }` | Operation on a non-existent clip | Check `timeline.clip(id).is_some()` |
| `KeyframeNotFound` | `{ id: KeyframeId }` | Remove or modify a non-existent keyframe | Check clip's keyframes before operating |
| `ClipOverlap` | `{ id, conflicting_with }` | Insert or move a clip that would overlap an existing clip on the same track | Check overlap with `clip.overlaps(other)` or catch the error and inform the user |
| `InvalidDuration` | `{ seconds: f64 }` | A clip with zero or negative duration was provided | Validate duration before inserting |
| `InvalidSplitPoint` | `{ time: Time }` | Split point is outside the clip's range or at its boundary | Ensure `clip.start < at_time < clip.end()` |
| `InvalidTrackIndex` | `{ index, track_count }` | `move_track` with an out-of-range index | Check `track_ids().len()` before moving |

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TimelineError {
    TrackNotFound { id: TrackId },
    ClipNotFound { id: ClipId },
    KeyframeNotFound { id: KeyframeId },
    ClipOverlap { id: ClipId, conflicting_with: ClipId },
    InvalidDuration { seconds: f64 },
    InvalidSplitPoint { time: Time },
    InvalidTrackIndex { index: usize, track_count: usize },
}
```

!!! note "No `Eq` derive"
    `TimelineError` derives `PartialEq` but not `Eq` because `InvalidDuration` and `InvalidSplitPoint` contain `f64` values, which don't implement `Eq`.

---

### Layers

```text
domain_foundation::environments::layers::LayersError
```

| Variant | Fields | When | How to handle |
|---|---|---|---|
| `LayerNotFound` | `{ id: LayerId }` | Operation on a non-existent layer | Check `layers.contains(id)` |
| `InvalidSiblingIndex` | `{ index, sibling_count }` | `reorder_sibling` with an out-of-range index | Check `children().len()` or `roots().len()` before reordering |
| `CycleDetected` | `{ id, new_parent }` | `reparent` would create a cycle (new parent is a descendant of the layer) | Check `is_descendant(new_parent, id)` before reparenting |

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayersError {
    LayerNotFound { id: LayerId },
    InvalidSiblingIndex { index: usize, sibling_count: usize },
    CycleDetected { id: LayerId, new_parent: LayerId },
}
```

---

### NodeGraph

```text
domain_foundation::environments::node_graph::NodeGraphError
```

| Variant | Fields | When | How to handle |
|---|---|---|---|
| `NodeNotFound` | `{ id: NodeId }` | Operation on a non-existent node | Check `graph.contains_node(id)` |
| `PortNotFound` | `{ id: PortId }` | Operation on a non-existent port | Check `graph.port(id).is_some()` |
| `EdgeNotFound` | `{ id: EdgeId }` | Disconnect a non-existent edge | Check `graph.edge(id).is_some()` |
| `NoCanvasObject` | `{ id: NodeId }` | Spatial operation (move/transform) on a node without a canvas object | Check `node.canvas_object.is_some()` before spatial operations |
| `SourceNotOutput` | `{ id: PortId }` | `connect` where the source port is not `Output` direction | Ensure the source port has `PortDirection::Output` |
| `TargetNotInput` | `{ id: PortId }` | `connect` where the target port is not `Input` direction | Ensure the target port has `PortDirection::Input` |
| `SameNode` | — | `connect` where source and target are on the same node | Use different nodes for source and target |
| `TypeMismatch` | `{ source_type, target_type }` | `connect` where port type names don't match | Ensure both ports have the same `type_name` |
| `DuplicateEdge` | `{ source, target }` | `connect` where an edge already exists between these ports | Check `graph.is_connected(source, target)` before connecting |
| `CanvasError` | `{ source: CanvasError }` | A Canvas2D operation failed (e.g., canvas object not found) | Check the inner `CanvasError` |
| `CycleInGraph` | — | `topological_sort` detected a cycle in the graph | Remove edges that form cycles before sorting |
| `MaxInputsExceeded` | `{ port, max }` | `connect` would exceed the target port's `max_inputs` limit | Check `graph.input_edge_count(port) < port.max_inputs` before connecting |

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeGraphError {
    NodeNotFound { id: NodeId },
    PortNotFound { id: PortId },
    EdgeNotFound { id: EdgeId },
    NoCanvasObject { id: NodeId },
    SourceNotOutput { id: PortId },
    TargetNotInput { id: PortId },
    SameNode,
    TypeMismatch { source_type: String, target_type: String },
    DuplicateEdge { source: PortId, target: PortId },
    CanvasError { source: CanvasError },
    CycleInGraph,
    MaxInputsExceeded { port: PortId, max: u32 },
}
```

---

### Viewport3D

```text
domain_foundation::environments::viewport_3d::Viewport3DError
```

| Variant | Fields | When | How to handle |
|---|---|---|---|
| `ObjectNotFound` | `{ id: ObjectId3D }` | Operation on a non-existent object | Check `scene.contains(id)` |
| `CannotBeOwnParent` | `{ id: ObjectId3D }` | `set_parent` where the object is its own parent or would create a cycle | Check `is_descendant(parent, id)` before setting parent |
| `ParentNotFound` | `{ id: ObjectId3D }` | `set_parent` with a non-existent parent | Check `scene.contains(parent_id)` before setting parent |

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Viewport3DError {
    ObjectNotFound { id: ObjectId3D },
    CannotBeOwnParent { id: ObjectId3D },
    ParentNotFound { id: ObjectId3D },
}
```

---

### Table

```text
domain_foundation::environments::table::TableError
```

| Variant | Fields | When | How to handle |
|---|---|---|---|
| `RowNotFound` | `{ id: RowId }` | Operation on a non-existent row | Check `table.contains_row(id)` |
| `ColumnNotFound` | `{ id: ColumnId }` | Operation on a non-existent column | Check `table.contains_column(id)` |
| `InvalidRowIndex` | `{ index, row_count }` | `move_row` with an out-of-range index | Check `row_ids().len()` before moving |
| `InvalidColumnIndex` | `{ index, column_count }` | `move_column` with an out-of-range index | Check `column_ids().len()` before moving |

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableError {
    RowNotFound { id: RowId },
    ColumnNotFound { id: ColumnId },
    InvalidRowIndex { index: usize, row_count: usize },
    InvalidColumnIndex { index: usize, column_count: usize },
}
```

---

### TextEditor

```text
domain_foundation::environments::text_editor::TextEditorError
```

| Variant | Fields | When | How to handle |
|---|---|---|---|
| `PositionOutOfBounds` | `{ position: u32, length: u32 }` | Insert/delete at a position beyond `document.len()` | Check `pos <= doc.len()` before operating |
| `InvalidRange` | `{ start: u32, end: u32 }` | Delete or replace where `start > end` | Ensure `start <= end` before operating |

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextEditorError {
    PositionOutOfBounds { position: u32, length: u32 },
    InvalidRange { start: u32, end: u32 },
}
```

---

## Relations

```text
domain_foundation::relations::RelationError
```

| Variant | Fields | When | How to handle |
|---|---|---|---|
| `RelationNotFound` | `{ id: RelationId }` | Evaluate a relation that's not in the registry | Check `registry.contains(id)` before evaluating |
| `SourceMismatch` | `{ relation_id, expected, actual }` | Evaluate a relation with a source property that doesn't match its definition | Ensure the correct source `PropertyId` is passed to `propagate_from` |
| `MaxPropagationDepthReached` | `{ depth, property }` | Cascading propagation exceeded 256 hops (cycle or very deep chain) | Check for cycles in the relation graph; increase `max_depth` if legitimate |

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationError {
    RelationNotFound { id: RelationId },
    SourceMismatch {
        relation_id: RelationId,
        expected: PropertyId,
        actual: PropertyId,
    },
    MaxPropagationDepthReached {
        depth: usize,
        property: PropertyId,
    },
}
```

---

## Domain Engine

```text
domain_engine::DomainError
```

Combined error type wrapping all environment errors:

```rust
#[derive(Debug)]
pub enum DomainError {
    Canvas(CanvasError),
    Timeline(TimelineError),
    Layers(LayersError),
    NodeGraph(NodeGraphError),
    Table(TableError),
    TextEditor(TextEditorError),
    Viewport3D(Viewport3DError),
}
```

### Conversions

Each environment error implements `From<EnvError> for DomainError`:

```rust
let err: DomainError = CanvasError::ObjectNotFound { id }.into();
```

### Auto-rollback

When `execute_*` fails while a transaction is active, the DomainEngine **automatically rolls back** the transaction. The error is still returned to the caller.

### Matching on DomainError

```rust
match engine.execute_canvas(op) {
    Ok(outcome) => { /* success */ },
    Err(DomainError::Canvas(CanvasError::ObjectNotFound { id })) => {
        eprintln!("Object not found: {}", id);
    },
    Err(e) => {
        eprintln!("Other error: {}", e);
    },
}
```

### Backward-compatible `execute()`

The `execute()` method (alias for `execute_canvas()`) returns `Result<CanvasOutcome, CanvasError>` (not `DomainError`) for backward compatibility with existing tests:

```rust
let err = engine.execute(CanvasOperation::Remove { id: fake_id }).unwrap_err();
assert!(matches!(err, CanvasError::ObjectNotFound { .. }));
```

---

## Error Handling Patterns

### Check before operating

```rust
// Pattern 1: Check before operating
if canvas.contains(id) {
    canvas.set_translation(id, Vec2::new(10.0, 20.0))?;
}

// Pattern 2: Operate and handle error
match canvas.set_translation(id, Vec2::new(10.0, 20.0)) {
    Ok(previous) => { /* success */ },
    Err(CanvasError::ObjectNotFound { id }) => {
        // Object was removed — skip or recreate
    },
}
```

### DomainEngine auto-rollback

```rust
engine.begin_transaction();
match engine.execute_canvas(CanvasOperation::Move { id, translation }) {
    Ok(_) => { /* continue */ },
    Err(e) => {
        // Transaction was auto-rolled back by check_auto_rollback
        // Canvas is back to its pre-transaction state
        eprintln!("Failed: {}", e);
    },
}
```

---

## All Error Types Summary

| Module | Error type | Variants | `Eq`? |
|---|---|---|---|
| Canvas2D | `CanvasError` | 1 | ✅ |
| Timeline | `TimelineError` | 7 | ❌ (`f64` in `InvalidDuration`) |
| Layers | `LayersError` | 3 | ✅ |
| NodeGraph | `NodeGraphError` | 12 | ✅ |
| Viewport3D | `Viewport3DError` | 3 | ✅ |
| Table | `TableError` | 4 | ✅ |
| TextEditor | `TextEditorError` | 2 | ✅ |
| Relations | `RelationError` | 3 | ✅ |
| Domain Engine | `DomainError` | 7 (wraps all above) | ❌ (contains non-Eq types) |

All error types implement:
- `Debug`
- `Clone`
- `PartialEq`
- `Display` (via `fmt::Display`)
- `std::error::Error`

---

## Next Steps

- [18-public-api-reference.md](18-public-api-reference.md) — Complete type listing.
- [20-quick-start.md](20-quick-start.md) — 5-minute getting started.
- [13-domain-engine-integration.md](13-domain-engine-integration.md) — DomainEngine error handling.
- [15-operations-and-undo.md](15-operations-and-undo.md) — Auto-rollback on error