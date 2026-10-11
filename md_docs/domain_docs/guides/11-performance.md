# Performance Guide

This guide documents the time complexity of every key operation across all environments, caching strategies, and when to optimize.

---

## Complexity Reference

### Canvas2D

| Operation | Complexity | Notes |
|---|---|---|
| `create_object` | O(1) | HashMap insert + Vec push |
| `get(id)` | O(1) | HashMap lookup |
| `contains(id)` | O(1) | HashMap contains |
| `remove_object(id)` | O(n) | HashMap remove O(1) + `draw_order.retain()` O(n) |
| `set_transform` / `set_translation` | O(1) | HashMap get_mut + field write |
| `move_to_front` / `move_to_back` | O(n) | `draw_order.retain()` O(n) + push/insert O(1) |
| `move_forward` / `move_backward` | O(n) | `position()` O(n) + swap O(1) |
| `reorder` | O(n) | `remove` O(n) + `insert` O(n) |
| `objects_in_bounds` | O(n) | Linear scan |
| `iter()` | O(1) | Iterator creation |
| `clear()` | O(n) | HashMap clear + Vec clear |
| `duplicate_object` | O(1) | get O(1) + create O(1) |
| `insert_object` | O(n) | `draw_order.contains()` O(n) + HashMap insert O(1) |

### Timeline

| Operation | Complexity | Notes |
|---|---|---|
| `create_track` | O(1) | HashMap insert + Vec push |
| `insert_clip` | O(n) | Overlap check O(n) + `partition_point` O(log n) + Vec insert O(n) |
| `remove_clip` | O(n) | Linear search + Vec remove |
| `move_clip` | O(n) | Remove + overlap check + re-insert |
| `clips_at(time)` | O(n) | Scan all clips across all tracks |
| `end_time()` | O(n) | Scan all tracks for last clip |
| `move_track` | O(n) | `position()` O(n) + Vec remove + insert |
| `trim_clip` | O(1) | HashMap get_mut + field write |
| `split_clip` | O(n) | Remove + insert left + insert right |
| `clear()` | O(n) | HashMap clear + Vec clear |

### Layers

| Operation | Complexity | Notes |
|---|---|---|
| `create_layer` | O(1) | HashMap insert + Vec push |
| `create_child` | O(1) | HashMap insert + Vec push |
| `remove_layer` | O(n) | BFS collect descendants + remove each |
| `reparent` | O(n) | `is_descendant` O(n) + retain O(n) + push O(1) |
| `reorder_sibling` | O(n) | `position()` O(n) + swap O(1) |
| `move_up` / `move_down` | O(n) | `position()` O(n) + swap O(1) |
| `effective_visibility` | O(d) | d = depth of hierarchy (walk parent chain) |
| `effective_locked` | O(d) | d = depth of hierarchy |
| `layer_path` | O(d) | d = depth of hierarchy |
| `descendants` | O(n) | BFS traversal |
| `is_descendant` | O(n) | BFS traversal |
| `clear()` | O(n) | HashMap clear + Vec clear |

### NodeGraph

| Operation | Complexity | Notes |
|---|---|---|
| `create_node` | O(1) | HashMap insert + Canvas create O(1) |
| `remove_node` | O(n) | BFS collect ports + scan edges for each port + Canvas remove |
| `add_input_port` / `add_output_port` | O(1) | HashMap insert + Vec push |
| `remove_port` | O(n) | Scan all edges for connections |
| `connect` | O(n) | Validation: check direction, type, duplicate, max_inputs — each O(1), but `any()` scan is O(n) |
| `disconnect` | O(1) | HashMap remove |
| `topological_sort` | O(V + E) | Kahn's algorithm |
| `has_cycles` | O(V + E) | Calls topological_sort |
| `is_connected` | O(n) | Scan all edges |
| `edges_on_port` | O(n) | Filter all edges |
| `edges_between` | O(n) | Filter all edges + port lookups |
| `duplicate_node` | O(1) | HashMap get + Canvas create |
| `clear()` | O(n) | Clear HashMaps + Canvas clear |

### Viewport3D

| Operation | Complexity | Notes |
|---|---|---|
| `create_object` | O(1) | HashMap insert |
| `get(id)` | O(1) | HashMap lookup |
| `remove_object` | O(1) | HashMap remove |
| `set_parent` | O(n) | `is_descendant` O(n) cycle check |
| `world_transform` | O(d) | d = depth of parent chain |
| `children(id)` | O(n) | Scan all objects |
| `root_objects` | O(n) | Scan all objects |
| `objects_in_bounds` | O(n) | Linear scan |
| `clear()` | O(n) | HashMap clear |

### Table

| Operation | Complexity | Notes |
|---|---|---|
| `insert_row` / `insert_column` | O(1) | HashMap insert + Vec push |
| `remove_row` | O(n) | Scan all cells for matching row + remove |
| `remove_column` | O(n) | Scan all cells for matching column + remove |
| `set_cell_value` | O(1) | HashMap get_mut for row + column + HashMap insert |
| `clear_cell` | O(1) | HashMap remove |
| `move_row` / `move_column` | O(n) | `position()` O(n) + Vec remove + insert |
| `swap_rows` / `swap_columns` | O(1) | Vec swap |
| `merge_cells` | O(r×c) | r = rows in range, c = columns in range |
| `unmerge_cells` | O(n) | `retain()` on merged_cells HashMap |
| `clear()` | O(n) | Clear 4 HashMaps + 2 Vecs |

### TextEditor

| Operation | Complexity | Notes |
|---|---|---|
| `len()` | **O(1)** | Cached `char_count` |
| `insert(pos, text)` | O(n) | `char_to_byte` O(n) + String insert O(n) + cache update O(k) where k = inserted chars |
| `delete(range)` | O(n) | `char_to_byte` O(n) + String drain O(n) + cache update O(1) |
| `replace(range, text)` | O(n) | delete O(n) + insert O(n) |
| `find(needle, from)` | O(n) | `char_to_byte` O(n) + `str::find` O(n) |
| `find_all(needle)` | O(n·m) | n = text length, m = number of matches |
| `replace_all` | O(n) | `find_all` O(n·m) + `String::replace` O(n) + cache recompute O(n) |
| `line_count` | O(n) | `str::lines().count()` |
| `line_col_at(pos)` | O(n) | Linear scan to position |
| `clear()` | O(1) | `mem::take` + set char_count = 0 |
| `byte_count` | O(1) | `String::len()` |
| `char_at(pos)` | O(n) | `chars().nth(n)` |
| `lines()` | O(1) | Iterator creation |

### Relations

| Operation | Complexity | Notes |
|---|---|---|
| `register` | O(n) | Priority-sorted insert into `by_source` Vec O(n) |
| `unregister` | O(n) | Remove from 3 HashMaps + `retain` on Vecs |
| `get_definition` | O(1) | HashMap lookup |
| `relations_with_source` | O(1) | HashMap lookup returns Vec iterator |
| `propagate_from` | O(R × D) | R = relations in cascade, D = max depth (256) |
| `evaluate_one` | O(1) | HashMap get + effect call + HashMap insert |
| `begin_batch` / `commit_batch` | O(n) | n = number of pending changes (deduplicated) |
| `clear()` | O(n) | Clear 3 HashMaps + reset graph |

---

## Caching Strategies

### Document: Cached `char_count`

`Document::len()` is **O(1)** because it caches `char_count: u32`. All mutating methods update the cache:

| Method | Cache update |
|---|---|
| `insert(pos, text)` | `char_count += text.chars().count()` |
| `delete(range)` | `char_count -= range.len()` |
| `replace(range, text)` | `char_count = char_count - removed + added` |
| `clear()` | `char_count = 0` |
| `replace_all(...)` | `char_count = text.chars().count()` (full recompute) |

Without caching, every `len()` call would be O(n) via `text.chars().count()`.

### Canvas: `draw_order` Vec

The `draw_order: Vec<ObjectId>` provides O(1) draw-order access while `objects: HashMap` provides O(1) id lookup. Without the Vec, draw-order iteration would require sorting.

---

## HashMap vs BTreeSet Tradeoffs

| Collection | Used for | Iteration order | Lookup |
|---|---|---|---|
| `HashMap` | Object storage | Unspecified | O(1) |
| `Vec` | Draw order, row/column order, track order | Defined | O(n) for position, O(1) for index |
| `BTreeSet` | TagSet | Sorted (deterministic) | O(log n) |
| `HashMap` | Relation `by_source`/`by_target` | Unspecified | O(1) |
| `Vec` | Priority-sorted relations | Priority order | O(n) |

`BTreeSet` is used for `TagSet` to guarantee deterministic iteration order (important for tests and UI display).

---

## Clone Costs

Operations and outcomes are `Clone` (required for history recording). The cost depends on the data:

| Type | Clone cost | Notes |
|---|---|---|
| `CanvasOperation` | O(1) to O(n) | `Move` is O(1); `Create` copies a `Transform` (Copy); `Remove` copies nothing |
| `CanvasOutcome` | O(1) to O(n) | `Moved` copies a `Transform` (Copy); `Removed` clones an `Object` (Copy) |
| `TableOperation` | O(1) to O(n) | `SetCellValue` clones `CellValue` (O(1) to O(n) for `Text`) |
| `TimelineOperation` | O(1) to O(n) | `InsertClip` is O(1); `Move` is O(1) |

Most clones are cheap because:
- `ObjectId`, `Transform`, `Vec2`, `Vec3` are `Copy`
- `Object` and `Object3D` are `Copy`
- `String` clones are O(n) but typically short

---

## Arc<Mutex> vs Direct Ownership

| Pattern | Used by | Pros | Cons |
|---|---|---|---|
| Direct ownership | DomainEngine | Fast, no locking | Can't share with command handlers |
| `Arc<Mutex<Env>>` | Command handlers | Shareable across threads | Lock overhead, potential poisoning |

The DomainEngine owns environments directly. Command handlers wrap `Arc<Mutex<Env>>` so both the engine and handler see the same state.

For best performance, prefer the DomainEngine's `execute_*` methods over command handlers when you don't need the App Engine's command dispatch.

---

## Spatial Indexing

`objects_in_bounds()` on Canvas2D and Viewport3D is O(n) — a linear scan. For large scenes (1000+ objects), consider adding a spatial index:

| Index type | Best for | Complexity |
|---|---|---|
| Quadtree (2D) | Canvas2D with many static objects | O(log n) query |
| BVH (3D) | Viewport3D with many objects | O(log n) query |
| Grid (2D) | Uniform distribution | O(1) query (fixed cell size) |

The foundation doesn't include spatial indexes — they're application-specific. A Domain Engine wraps the environment and maintains an index alongside it.

---

## When to Optimize

| Scenario | Optimize? | Why |
|---|---|---|
| `len()` is O(n) | ✅ | Called frequently; cache the count |
| `objects_in_bounds` is O(n) with <100 objects | ❌ | Linear scan is fine for small N |
| `objects_in_bounds` is O(n) with 10K+ objects | ✅ | Add a spatial index |
| `remove_layer` is O(n) for deep hierarchies | Maybe | BFS is acceptable for typical depths (<20) |
| `topological_sort` is O(V+E) | ❌ | This is optimal; can't do better |
| `propagate_from` is O(R×D) | Maybe | If R is large (100+ relations on one source), consider batching |
| `find_all` is O(n×m) | Maybe | For very long documents, consider Boyer-Moore |

---

## Benchmarking

The foundation includes criterion benchmarks:

```sh
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench -- canvas_create_100_objects
cargo bench -- document_len_1000_chars
cargo bench -- timeline_insert_50_clips
cargo bench -- relation_register_100
```

### Benchmark files

```text
benches/
└── domain_foundation_bench.rs
```

| Benchmark | What it measures |
|---|---|
| `canvas_create_100_objects` | Creating 100 Canvas2D objects |
| `document_len_1000_chars` | `Document::len()` on 1000-char document (should be O(1)) |
| `document_insert_at_end` | Inserting at end of 1000-char document |
| `timeline_insert_50_clips` | Inserting 50 clips on one track |
| `relation_register_100` | Registering 100 relations |

---

## Checklist

- [ ] `len()` is O(1) (cache the count if needed)
- [ ] Hot-path operations are O(1) or O(log n)
- [ ] No O(n²) operations in common paths
- [ ] Clones are cheap (prefer `Copy` types)
- [ ] Benchmarks added for hot paths
- [ ] Stress tests verify performance at scale (100K+)

---

## Next Steps

- [01-creating-environment.md](01-creating-environment.md) — Creating environments with performance in mind.
- [02-adding-operations.md](02-adding-operations.md) — Adding operations (consider clone costs).
- [06-composition-patterns.md](06-composition-patterns.md) — Composition (separate concerns = separate performance).
- [17-testing.md](../17-testing.md) — Stress tests and benchmarks.