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

### Docking

```text
domain_foundation::environments::docking::DockingError
```

A rejected request is the *whole* consequence: the layout is untouched (candidate-clone atomicity), and no history entry or event is recorded (PRD §8.2, §10.5).

#### Identity & creation (Phase 2)

| Variant | Fields | When | How to handle |
|---|---|---|---|
| `DockNotFound` | `{ id }` | Operation on a non-existent dock | Check `layout.contains(id)` |
| `UnknownReference` | `{ id }` | A created placement references a dock missing from the layout | Create the referenced dock first |
| `DuplicateMembership` | `{ id }` | The same dock appears twice in one container collection | Use distinct docks |
| `AlreadyPlaced` | `{ id }` | An already-placed dock used as if unplaced (PRD DM-07: one placement per dock) | Detach it (an operation) before re-placing, or use a different dock |
| `InsufficientChildren` | `{ minimum, found }` | A split (≥ 2 children) or tab group (≥ 1 member) created below its minimum | Pass enough children/members |
| `SizeCountMismatch` | `{ sizes, children }` | Weight count ≠ child count (PRD §9.2) | Match `SplitSizes` to the child list |
| `ActiveIndexOutOfRange` | `{ index, member_count }` | `active` (or `SetActiveTab`) outside `members` | Check `members.len()` first |
| `RootAlreadySet` | `{ current }` | `set_root(Some(id))` while a root is designated | `set_root(None)` first — re-rooting stays explicit |

#### Layout invariants (Phase 3)

| Variant | Fields | When | How to handle |
|---|---|---|---|
| `OrphanDock` | `{ id }` | A dock exists but occupies no placement | Keep docks placed or remove them; transient only mid-operation |
| `DuplicatePlacement` | `{ id }` | A dock occupies more than one placement | Never build such state; on import the gate rejects it |
| `InvalidRoot` | `{ id }` | The designated root is also placed inside a container | The root's only correct slot is `DockSlot::Root` |
| `MissingRoot` | `{ docks }` | The layout holds docks but designates no root | Designate a root, or `clear()` for the explicit empty layout |
| `CycleDetected` | `{ id }` | A structural containment cycle (split/tab/regions/floating all count) | Pre-check with the self-inclusive `layout.is_descendant(target, source)` |
| `InvalidSizeState` | `{ split, index }` | Imported weight not finite / not strictly positive (PRD §9.2) | Fix weights before import — restore goes through the validate gate |

#### Operations (Phase 4)

| Variant | Fields | When | How to handle |
|---|---|---|---|
| `IneligibleDock` | `{ id }` | A tab embedding/operation on a non-panel (v1 membership is panels only); embed/promote targets that are splits or tab groups (they cannot host or promote) | Operation on a panel, or pick a host/panel target |
| `NotSplittable` | `{ id }` | Directional insert beside a floating dock (no row/column meaning in the floating collection) | Re-dock the floater first, or embed/tab-target it |
| `NotATabGroup` | `{ id }` | A tab-group argument names a non-tab-group dock | Pass the group's id (read it from the `TabJoined` outcome) |
| `TabIndexOutOfRange` | `{ index, member_count }` | `ReorderTab`/`SetActiveTab` argument outside the member list | Check `members.len()` |

#### Splitter sizing (Phase 6)

| Variant | Fields | When | How to handle |
|---|---|---|---|
| `NotASplit` | `{ id }` | A resize names a non-split container | Pass the split container's id |
| `SplitChildIndexOutOfRange` | `{ index, child_count }` | The resize pair `[first, first + 1]` runs past the child list | `first + 1 < children.len()` |
| `SizeBoundViolation` | `{ split, index }` | The request violates a carried minimum or the sizing invariant — the rejection policy: never clamped | Send a reachable pair weight within the bounds (bound endpoints are honored inclusively) |

#### Embedding & host lifecycle (Phase 7)

| Variant | Fields | When | How to handle |
|---|---|---|---|
| `NotAnEmbedHost` | `{ id }` | A host argument names a split or tab group (remember: `Embed`/`PromoteHost` auto-promote *panels*; splits/tab groups cannot host) | Embed into a host, or a promotable panel |
| `RegionOccupied` | `{ host, edge }` | Embedding into an occupied edge region (an occupant is never silently replaced) | Choose a free region (check `host_regions().on(edge)`) |
| `RegionUnoccupied` | `{ host, edge }` | Resizing an empty edge region (no inner boundary, no stored weight) | Resize only occupied regions |
| `RegionBoundViolation` | `{ host, edge }` | The requested region weight violates its bounds (PRD §9.3 minimum / finite-positive invariant) | Send a reachable weight within bounds |
| `InvalidRegionState` | `{ host, edge }` | Imported region weight not finite/positive | Fix region state before import |

#### Floating state (Phase 8)

| Variant | Fields | When | How to handle |
|---|---|---|---|
| `FloatingIndexOutOfRange` | `{ index, entry_count }` | `SetFloatingRect` names an index outside the floating collection | Check `dock.floating().len()` |
| `FloatingBoundViolation` | `{ host, index }` | The rect violates the min sizes, the rect invariant (finite, `x`/`y` ≥ 0, `w`/`h` > 0), or the host-bounds rule (`x + w ≤ 1`, `y + h ≤ 1`) | Send a rect within bounds (endpoints inclusive) |
| `InvalidFloatingState` | `{ host, index }` | An imported rect violates the rect invariant or host-bounds rule | Fix floating state before import |

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DockingError {
    // identity & creation
    DockNotFound { id: DockId },
    UnknownReference { id: DockId },
    DuplicateMembership { id: DockId },
    AlreadyPlaced { id: DockId },
    InsufficientChildren { minimum: usize, found: usize },
    SizeCountMismatch { sizes: usize, children: usize },
    ActiveIndexOutOfRange { index: usize, member_count: usize },
    RootAlreadySet { current: DockId },
    // layout invariants
    OrphanDock { id: DockId },
    DuplicatePlacement { id: DockId },
    InvalidRoot { id: DockId },
    MissingRoot { docks: usize },
    CycleDetected { id: DockId },
    InvalidSizeState { split: DockId, index: usize },
    // operations
    IneligibleDock { id: DockId },
    NotSplittable { id: DockId },
    NotATabGroup { id: DockId },
    TabIndexOutOfRange { index: usize, member_count: usize },
    // splitter sizing
    NotASplit { id: DockId },
    SplitChildIndexOutOfRange { index: usize, child_count: usize },
    SizeBoundViolation { split: DockId, index: usize },
    // embedding & host lifecycle
    NotAnEmbedHost { id: DockId },
    RegionOccupied { host: DockId, edge: EdgeRegion },
    RegionUnoccupied { host: DockId, edge: EdgeRegion },
    RegionBoundViolation { host: DockId, edge: EdgeRegion },
    InvalidRegionState { host: DockId, edge: EdgeRegion },
    // floating state
    FloatingIndexOutOfRange { index: usize, entry_count: usize },
    FloatingBoundViolation { host: DockId, index: usize },
    InvalidFloatingState { host: DockId, index: usize },
}
```

**Display strings** (each names the offending dock/region/index so a drag can be re-scoped, PRD §10.5):

| Variant | Display |
|---|---|
| `DockNotFound` | `"no dock with id {id}"` |
| `UnknownReference` | `"dock reference {id} does not exist in the layout"` |
| `DuplicateMembership` | `"dock {id} appears more than once in one placement"` |
| `AlreadyPlaced` | `"dock {id} already occupies a placement"` |
| `InsufficientChildren` | `"structure expects at least {minimum} children, found {found}"` |
| `SizeCountMismatch` | `"size count {sizes} does not match child count {children}"` |
| `ActiveIndexOutOfRange` | `"active tab index {index} out of range (member count {member_count})"` |
| `RootAlreadySet` | `"root is already set to {current} (clear the root first)"` |
| `OrphanDock` | `"dock {id} has no placement (orphan)"` |
| `DuplicatePlacement` | `"dock {id} occupies more than one placement"` |
| `InvalidRoot` | `"dock {id} is the root but is also placed in a container"` |
| `MissingRoot` | `"layout holds {docks} docks but designates no root"` |
| `CycleDetected` | `"structural cycle detected at {id}"` |
| `InvalidSizeState` | `"sizing weight {index} of {split} is invalid (must be finite and strictly positive)"` |
| `IneligibleDock` | `"dock {id} does not qualify for this operation (v1 tab membership is panels only)"` |
| `NotSplittable` | `"dock {id} is in a floating placement: directional insertion is not defined for floating docks"` |
| `NotATabGroup` | `"dock {id} is not a tab group"` |
| `TabIndexOutOfRange` | `"tab index {index} out of range (member count {member_count})"` |
| `NotASplit` | `"dock {id} is not a split container"` |
| `SplitChildIndexOutOfRange` | `"split child index {index} out of range (child count {child_count})"` |
| `SizeBoundViolation` | `"resize rejected: weight {index} of {split} would violate its logical size bounds"` |
| `NotAnEmbedHost` | `"dock {id} is not an embed host"` |
| `RegionOccupied` | `"{edge:?} region of {host} is already occupied (embedding never replaces an occupant)"` |
| `RegionUnoccupied` | `"{edge:?} region of {host} is unoccupied and cannot be resized"` |
| `RegionBoundViolation` | `"region resize rejected: the {edge:?} region of {host} would violate its logical size bounds"` |
| `InvalidRegionState` | `"sizing weight of the {edge:?} region of {host} is invalid (must be finite and strictly positive)"` |
| `FloatingIndexOutOfRange` | `"floating index {index} out of range (entry count {entry_count})"` |
| `FloatingBoundViolation` | `"floating rect rejected: entry {index} of {host} would violate its logical bounds"` |
| `InvalidFloatingState` | `"logical rect of floating entry {index} of {host} is invalid (must be finite, non-empty, and within the host's bounds)"` |

!!! note "Rejection, never clamping"
    The three bound-violation variants express the environment's defined constraint policy (PRD §9.3): a conflicting resize/rect request is refused with exactly this error — never silently clamped, never resolved into invalid stored state.

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
    Docking(DockingError),
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
| Docking | `DockingError` | 29 | ✅ |
| Relations | `RelationError` | 3 | ✅ |
| Domain Engine | `DomainError` | 8 (wraps all above) | ❌ (contains non-Eq types) |

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