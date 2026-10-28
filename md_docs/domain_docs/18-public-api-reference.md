# Public API Reference

Complete listing of all public types, traits, and functions in the Domain Foundation, organized by module.

---

## Environments

### Canvas2D

```text
domain_foundation::environments::canvas_2d
```

| Type | Kind | Description |
|---|---|---|
| `Canvas` | struct | 2D canvas container (objects + draw_order + next_id) |
| `Object` | struct | Canvas object (id + transform) |
| `ObjectId` | newtype | Opaque object identifier (u64) |
| `Transform` | struct | Decomposed 2D transform (translation, rotation, scale) |
| `Vec2` | struct | 2D vector (x, y) |
| `CanvasOperation` | enum | 11 variants: Create, Move, Transform, Remove, MoveToFront/Back, MoveForward/Backward, Reorder, Duplicate, Clear |
| `CanvasOutcome` | enum | 11 variants with undo data |
| `CanvasError` | enum | ObjectNotFound |

**Methods on `Canvas`:** `new`, `len`, `is_empty`, `contains`, `get`, `get_mut`, `iter`, `iter_ordered`, `draw_order`, `create_object`, `insert_object`, `set_transform`, `set_translation`, `remove_object`, `duplicate_object`, `clear`, `move_to_front`, `move_to_back`, `move_forward`, `move_backward`, `reorder`, `objects_in_bounds`

**Methods on `Vec2`:** `new`, `zero`, `splat`, `dot`, `length`, `normalize`, `lerp`, `rotate`, `angle_to` + `Add`, `Sub`, `Mul<f64>` operators

**Methods on `Transform`:** `identity`, `from_translation`, `from_rotation`, `from_scale`, `with_translation`, `with_rotation`, `with_scale`

**Traits:** `IntoIterator for &Canvas`, `Extend<Object>`

---

### Timeline

```text
domain_foundation::environments::timeline
```

| Type | Kind | Description |
|---|---|---|
| `Timeline` | struct | Temporal container (tracks + track_order + id counters) |
| `Track` | struct | Independent lane (id + clips) |
| `Clip` | struct | Time-bounded segment (id + start + duration + keyframes) |
| `Keyframe` | struct | Animated value (id + time + value + interpolation) |
| `Time` | newtype | Absolute time position (f64) |
| `Duration` | newtype | Relative time length (f64) |
| `Interpolation` | enum | Step, Linear, Smooth |
| `KeyframeValue` | enum | Float, Int, Bool, Text |
| `TrackId` | newtype | Opaque track identifier (u64) |
| `ClipId` | struct | Composite id (raw + track) |
| `KeyframeId` | newtype | Opaque keyframe identifier (u64) |
| `TimelineOperation` | enum | 11 variants: CreateTrack, RemoveTrack, InsertClip, RemoveClip, MoveClip, AddKeyframe, RemoveKeyframe, TrimClip, SplitClip, ClearTimeline, SetKeyframeValue |
| `TimelineOutcome` | enum | 11 variants with undo data |
| `TimelineError` | enum | TrackNotFound, ClipNotFound, KeyframeNotFound, ClipOverlap, InvalidDuration, InvalidSplitPoint, InvalidTrackIndex |

**Methods on `Timeline`:** `new`, `create_track`, `insert_track`, `remove_track`, `track`, `track_mut`, `tracks`, `track_ids`, `track_count`, `contains_track`, `insert_clip`, `reinsert_clip`, `remove_clip`, `clip`, `clip_mut`, `clip_count`, `move_clip`, `add_keyframe`, `remove_keyframe`, `end_time`, `clear`, `clips_at`, `move_track`, `trim_clip`, `split_clip`, `set_keyframe_value`

**Methods on `Clip`:** `new`, `end`, `keyframes`, `keyframes_mut`, `add_keyframe`, `remove_keyframe`, `overlaps`, `contains`, `keyframes_in_range`, `set_duration`, `split`

**Traits:** `IntoIterator for &Timeline`

---

### Layers

```text
domain_foundation::environments::layers
```

| Type | Kind | Description |
|---|---|---|
| `Layers` | struct | Hierarchy container (layers + roots + next_id) |
| `Layer` | struct | Hierarchy node (id + name + visible + locked + parent + children) |
| `LayerId` | newtype | Opaque layer identifier (u64) |
| `LayersOperation` | enum | 10 variants: CreateLayer, CreateChild, RemoveLayer, RenameLayer, SetVisibility, SetLocked, ReorderSibling, Reparent, DuplicateLayer, Clear |
| `LayersOutcome` | enum | 10 variants with undo data |
| `LayersError` | enum | LayerNotFound, InvalidSiblingIndex, CycleDetected |

**Methods on `Layers`:** `new`, `len`, `is_empty`, `contains`, `layer`, `roots`, `root_count`, `create_layer`, `create_child`, `insert_layer`, `remove_layer`, `duplicate_layer`, `set_visibility`, `set_locked`, `rename`, `reorder_sibling`, `reparent`, `is_descendant`, `descendants`, `depth`, `effective_visibility`, `effective_locked`, `layer_path`, `move_up`, `move_down`, `clear`

---

### NodeGraph

```text
domain_foundation::environments::node_graph
```

| Type | Kind | Description |
|---|---|---|
| `NodeGraph` | struct | Graph container (nodes + ports + edges + canvas + id counters) |
| `Node` | struct | Graph entity (id + name + canvas_object + input/output ports) |
| `Port` | struct | Connection point (id + node + direction + name + type_name + max_inputs) |
| `Edge` | struct | Directed connection (id + source + target) |
| `PortDirection` | enum | Input, Output |
| `NodeId` | newtype | Opaque node identifier (u64) |
| `PortId` | newtype | Opaque port identifier (u64) |
| `EdgeId` | newtype | Opaque edge identifier (u64) |
| `NodeRemovedData` | struct | Undo data (node + ports + edges + canvas_object) |
| `PortRemovedData` | struct | Undo data (port + edges) |
| `NodeGraphOperation` | enum | 13 variants: CreateNode, RemoveNode, RenameNode, MoveNode, SetNodeTransform, AddInputPort, AddOutputPort, RemovePort, Connect, Disconnect, RenamePort, DuplicateNode, Clear |
| `NodeGraphOutcome` | enum | 13 variants with undo data |
| `NodeGraphError` | enum | NodeNotFound, PortNotFound, EdgeNotFound, NoCanvasObject, SourceNotOutput, TargetNotInput, SameNode, TypeMismatch, DuplicateEdge, CanvasError, CycleInGraph, MaxInputsExceeded |

**Methods on `NodeGraph`:** `new`, `node_count`, `port_count`, `edge_count`, `contains_node`, `node`, `port`, `port_mut`, `nodes`, `ports`, `edges`, `canvas`, `create_node`, `insert_node`, `remove_node`, `duplicate_node`, `rename_node`, `move_node`, `set_node_transform`, `node_transform`, `add_input_port`, `add_output_port`, `insert_port`, `remove_port`, `rename_port`, `connect`, `insert_edge`, `disconnect`, `edges_on_port`, `edges_between`, `is_connected`, `input_edge_count`, `topological_sort`, `has_cycles`, `clear`

---

### Viewport3D

```text
domain_foundation::environments::viewport_3d
```

| Type | Kind | Description |
|---|---|---|
| `Scene` | struct | 3D container (objects + next_id) |
| `Object3D` | struct | 3D entity (id + transform + parent) |
| `Transform3D` | struct | Decomposed 3D transform (translation, rotation, scale) |
| `Vec3` | struct | 3D vector (x, y, z) |
| `ObjectId3D` | newtype | Opaque 3D object identifier (u64) |
| `Viewport3DOperation` | enum | 8 variants: Create, Move, Rotate, Scale, Transform, Remove, SetParent, Clear |
| `Viewport3DOutcome` | enum | 8 variants with undo data |
| `Viewport3DError` | enum | ObjectNotFound, CannotBeOwnParent, ParentNotFound |

**Methods on `Scene`:** `new`, `len`, `is_empty`, `contains`, `get`, `get_mut`, `iter`, `create_object`, `insert_object`, `set_transform`, `set_translation`, `set_rotation`, `set_scale`, `remove_object`, `clear`, `set_parent`, `world_transform`, `children`, `root_objects`, `objects_in_bounds`

**Methods on `Vec3`:** `new`, `zero`, `splat`, `dot`, `cross`, `length`, `normalised`, `lerp`, `distance_to` + operators

---

### Table

```text
domain_foundation::environments::table
```

| Type | Kind | Description |
|---|---|---|
| `Table` | struct | Tabular container (rows + columns + cells + merged_cells + ordering + id counters) |
| `Row` | struct | Row definition (id + name) |
| `Column` | struct | Column definition (id + name) |
| `CellId` | struct | Composite cell address (row + column) |
| `CellValue` | enum | None, Text, Int, Float, Bool |
| `RowId` | newtype | Opaque row identifier (u64) |
| `ColumnId` | newtype | Opaque column identifier (u64) |
| `TableOperation` | enum | 13 variants: InsertRow, InsertColumn, RemoveRow, RemoveColumn, SetCellValue, ClearCell, MoveRow, MoveColumn, RenameRow, RenameColumn, MergeCells, UnmergeCells, ClearTable |
| `TableOutcome` | enum | 13 variants with undo data |
| `TableError` | enum | RowNotFound, ColumnNotFound, InvalidRowIndex, InvalidColumnIndex |

**Methods on `Table`:** `new`, `row_count`, `column_count`, `cell_count`, `is_empty`, `contains_row`, `row`, `rows`, `row_ids`, `insert_row`, `restore_row`, `remove_row`, `rename_row`, `move_row`, `swap_rows`, `contains_column`, `column`, `columns`, `column_ids`, `insert_column`, `restore_column`, `remove_column`, `rename_column`, `move_column`, `swap_columns`, `set_cell_value`, `clear_cell`, `cell`, `cell_by_id`, `merge_cells`, `unmerge_cells`, `is_merged`, `master_cell`, `clear`

**Traits:** `IntoIterator for &Table`

---

### TextEditor

```text
domain_foundation::environments::text_editor
```

| Type | Kind | Description |
|---|---|---|
| `Document` | struct | Text container (text + char_count cached) |
| `Position` | newtype | Unicode scalar value offset (u32) |
| `Range` | struct | Half-open span [start, end) |
| `TextEditorOperation` | enum | 7 variants: Insert, Delete, Replace, Clear, Append, DeleteLine, ReplaceLine |
| `TextEditorOutcome` | enum | 7 variants with undo data |
| `TextEditorError` | enum | PositionOutOfBounds, InvalidRange |

**Methods on `Document`:** `new`, `from_str`, `text`, `len` (O(1)), `is_empty`, `byte_count`, `char_at`, `substring`, `lines`, `line_count`, `line_col_at`, `insert`, `append`, `delete`, `replace`, `clear`, `find`, `find_all`, `replace_all`, `delete_line`, `replace_line`

**Methods on `Range`:** `new`, `at`, `len`, `is_empty`, `contains`, `overlaps`, `intersect`, `shift` + `From<(Position, Position)>`

**Traits:** `IntoIterator for &Document` (yields `char`), `Add<u32> for Position`, `Sub<u32> for Position`, `Ord for Range`

---

### Docking

```text
domain_foundation::environments::docking
```

| Type | Kind | Description |
|---|---|---|
| `DockLayout` | struct | Dock-container (docks + designated root + id counter) |
| `Dock` | struct | Identity + category payload |
| `DockRole` | enum | Panel{content, collapsed}, Split{orientation, children, sizes}, TabGroup{members, active}, EmbedHost{content, regions, floating, persistent} |
| `DockKind` | enum | Panel, Split, TabGroup, EmbedHost (category without payload) |
| `DockId` | newtype | Opaque dock identifier (u64; ids never renumber; `from_raw`/`as_raw` for host keying) |
| `ContentRef` | type alias | `String` — opaque, host-supplied content reference |
| `DockSlot` | enum | Root, SplitChild{parent, index}, TabMember{parent, index}, HostEdge{host, edge}, Floating{host, index} |
| `EdgeRegion` | enum | Top, Bottom, Left, Right |
| `SplitOrientation` | enum | Row, Column |
| `SplitSizes` | struct | One relative weight per split child (finite, strictly positive) |
| `HostRegions` | struct | Edge-region occupancy + per-region weight |
| `FloatingEntry` / `FloatingRect` | struct | Floated subtree root + its host-fraction body rect (`BASELINE` constant) |
| `InsertSource` | enum | Existing(DockId), NewPanel{content} |
| `EmbedSlot` | enum | Region(EdgeRegion), Floating{rect: Option<FloatingRect>} |
| `SplitDirection` | enum | Left, Right, Top, Bottom |
| `FormerCollection` | enum | Split, Tab, Region, Floating, Panel, Promotion, Demotion (the slot a detached dock is restored to) |
| `UndoData` | enum | Full inverse/replay payload carried on every outcome |
| `DockingOperation` | enum | 13 variants: CreatePanel, CreateEmbedHost, InsertBeside, TabJoin, ReorderTab, SetActiveTab, Remove, ResizeSiblings, SetDockCollapsed, Embed, PromoteHost, ResizeRegion, SetFloatingRect |
| `DockingOutcome` | enum | 12 variants with undo data (Created serves both creation operations) |
| `DockingError` | enum | 29 variants (see [19-error-reference.md](19-error-reference.md)) |

**Methods on `DockLayout`:** `new`, `len`, `is_empty`, `root`, `root_dock`, `contains`, `get`, `iter`, `create_panel`, `create_embed_host`, `create_split`, `create_tab_group`, `set_root`, `clear`, `placement`, `is_descendant`, `descendants`, `path`, `depth`, `all_docks`, `kind_of`, `validate`, `normalize`

**Creation preconditions:** `create_split` needs ≥ 2 *unplaced* children and matching weight count (`InsufficientChildren` / `SizeCountMismatch`); `create_tab_group` needs ≥ 1 member and a valid `active` index; `set_root(Some(id))` refuses while a root is designated (`RootAlreadySet`).

**Slots are operation-owned:** `DockSlot` has no public constructor; placement changes happen only through `DockingOperation::apply(&mut DockLayout)` (a rejected request leaves the layout, history, and events untouched).

**Traits:** `Default for DockLayout`

---

## Relations

```text
domain_foundation::relations
```

| Type | Kind | Description |
|---|---|---|
| `PropertyId` | struct | Opaque address (object + property index) |
| `Endpoint` | struct | Source or target endpoint (property + role) |
| `EndpointRole` | enum | Source, Target |
| `ObjectId` | newtype | Opaque object identifier (u64) |
| `RelationId` | newtype | Opaque relation identifier (u64) |
| `Relation` | struct | Lightweight view (id + source + target) |
| `RelationDefinition` | struct | Full definition (endpoints + trigger + propagation + priority + effect) |
| `RelationDefinitionBuilder` | builder | Constructs `RelationDefinition` |
| `Trigger` | enum | OnChange, OnInitialSet, OnInvalidation |
| `PropagationMode` | enum | Direct, Cascading |
| `EffectFn` | type alias | `Arc<dyn Fn(&EffectContext) -> EffectOutcome + Send + Sync>` |
| `EffectContext` | struct | Read-only context for effect closures |
| `EffectOutcome` | enum | Apply(PropertyValue), NoChange, Invalidate |
| `PropertyValue` | enum | None, Bool, Int, Float, Text, Bytes |
| `RelationRegistry` | struct | Stores definitions, priority-sorted |
| `RelationRegistryStats` | struct | Statistics (total, unique sources, unique targets) |
| `ValueStore` | struct | Mutable property value store |
| `EvaluationEngine` | struct | Evaluates relations, applies outcomes |
| `EvaluationContext` | struct | External handle (set values, propagate, batch) |
| `Propagator` | struct | Drives cascading propagation |
| `PropagationStep` | struct | One step in a cascade (relation_id + target + outcome) |
| `DependencyGraph` | struct | Fast source/target lookup |
| `InvalidationTracker` | struct | Tracks dirty properties |
| `PropertyValueSource` | trait | Bridge Domain Engine state to evaluation |
| `PropertyValueSink` | trait | Receive updates from evaluation |
| `SourcedValueStore<S>` | struct | Adapter wrapping a `PropertyValueSource` |
| `RelationError` | enum | RelationNotFound, SourceMismatch, MaxPropagationDepthReached |

**Key methods:**
- `RelationRegistry`: `register`, `unregister`, `get_definition`, `relations_with_source`, `relations_with_target`, `relations_between`, `relations`, `dependency_graph`, `len`, `is_empty`, `stats`, `contains`, `clear`
- `EvaluationContext`: `new`, `set_value`, `get_value`, `propagate_from`, `begin_batch`, `commit_batch`, `rollback_batch`, `is_in_batch`
- `RelationDefinitionBuilder`: `new`, `trigger`, `propagation`, `priority`, `effect`, `build`

---

## Utilities

```text
domain_foundation::utilities
```

### Filtering (`utilities::filtering`)

| Type | Description |
|---|---|
| `FilterCriteria<T>` | Trait: `matches(&self, &T) -> bool` + `and()`, `or()`, `not()` |
| `FnCriteria<F>` | Closure-based criteria |
| `And<C1, C2>` | Logical AND |
| `Or<C1, C2>` | Logical OR |
| `Not<C>` | Logical NOT |

**Functions:** `criteria(f)`, `filter(items, &criteria)`, `count(items, &criteria)`, `any(items, &criteria)`, `all(items, &criteria)`

### Querying (`utilities::querying`)

| Type | Description |
|---|---|
| `Query<T>` | Named query with boxed criteria |

**Functions:** `evaluate(&query, items)`

### Tagging (`utilities::tagging`)

| Type | Description |
|---|---|
| `Tag` | String newtype (Ord, Hash) |
| `TagSet` | BTreeSet wrapper (deterministic order) |

**Methods on `TagSet`:** `new`, `from_tags`, `add`, `remove`, `contains`, `len`, `is_empty`, `iter`, `intersection`, `union`, `contains_any`, `contains_all`

**Traits:** `IntoIterator for &TagSet`

### Sorting (`utilities::sorting`)

| Type | Description |
|---|---|
| `SortCriteria<T>` | Trait: `compare(&self, &T, &T) -> Ordering` |
| `FnSortCriteria<F>` | Closure-based criteria |

**Functions:** `by(f)`, `sort(&mut items, &criteria)`, `sorted(&items, &criteria)`

### Grouping (`utilities::grouping`)

| Type | Description |
|---|---|
| `GroupCriteria<T>` | Trait with associated `Key` type |

**Functions:** `group_by(items, f)` → `HashMap`, `group_sorted(items, f)` → `BTreeMap`, `group(items, &criteria)`

### Validation (`utilities::validation`)

| Type | Description |
|---|---|
| `ValidationResult` | Valid, Invalid { reason } |
| `ValidationRule<T>` | Trait: `validate(&self, &T) -> ValidationResult` |
| `FnRule<F>` | Closure-based rule |

**Functions:** `rule(f)`, `validate_all(items, &rule)`, `validate_with_rules(item, rules)`

### Batch (`utilities::batch`)

| Type | Description |
|---|---|
| `BatchOperation<T>` | Trait: `apply(&self, &mut [T])` |
| `FnBatchOperation<F>` | Closure-based operation |

**Functions:** `operation(f)`, `apply_batch(&op, &mut items)`

---

## Domain Engine

```text
domain_engine
```

| Type | Description |
|---|---|
| `DomainEngine` | Composition root (8 environments + App Engine integration) |
| `DomainError` | Combined error type (Canvas, Timeline, Layers, NodeGraph, Table, TextEditor, Viewport3D) |
| `ExecutedCanvasOperation` | Operation + outcome pair with undo/redo |
| `CanvasCommandHandler` | App Engine `CommandHandler` for Canvas2D |
| `TimelineCommandHandler` | App Engine `CommandHandler` for Timeline |
| `LayersCommandHandler` | App Engine `CommandHandler` for Layers |

**Methods on `DomainEngine`:**
- Environment getters: `canvas()`, `timeline()`, `layers()`, `node_graph()`, `table()`, `document()`, `scene()`
- App Engine access: `history()`, `event_bus()`, `events()`, `transaction_manager()`
- Execute: `execute_canvas()`, `execute_timeline()`, `execute_layers()`, `execute_node_graph()`, `execute_table()`, `execute_text_editor()`, `execute_viewport_3d()`, `execute()` (alias)
- Transactions: `begin_transaction()`, `commit_transaction()`, `rollback_transaction()`, `in_transaction()`
- Compound: `begin_compound()`, `commit_compound()`, `rollback_compound()`
- Undo/redo: `undo()`, `redo()`, `can_undo()`, `can_redo()`
- Dirty: `mark_saved()`, `is_dirty()`
- Construction: `new()`, `with_signal_bus()`, `with_max_history()`

---

## Serde Feature

All types listed above have `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]` except:
- `EffectFn` (skipped on `RelationDefinition`, defaults to no-op)
- `RelationRegistry::next_id` (`AtomicU64`, skipped)
- Lifetime-bound types (`EvaluationEngine`, `EvaluationContext`)
- Closure wrappers (`FnCriteria`, `FnSortCriteria`, `FnRule`, `FnBatchOperation`)

---

## Next Steps

- [19-error-reference.md](19-error-reference.md) — All error types and variants.
- [20-quick-start.md](20-quick-start.md) — 5-minute getting started.
- [17-testing.md](17-testing.md) — Testing strategies.