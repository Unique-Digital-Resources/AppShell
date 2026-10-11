# Serialization

The Domain Foundation supports optional serialization via `serde` behind a feature flag. All serializable types have conditional derives; non-serializable types (closures, `Mutex`, lifetimes) use `#[serde(skip)]`.

---

## Enabling Serialization

```toml
[dependencies]
app_shell = { path = "path/to/AppShell", features = ["serde"] }
```

The `serde` feature adds `serde` as a dependency and enables `Serialize`/`Deserialize` derives on all eligible types.

---

## What's Serializable

### Newtypes (with `serde(transparent)`)

Newtypes wrap a single value. `transparent` tells serde to serialize the inner value directly:

```rust
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct ObjectId(u64);
```

| Newtype | Inner type |
|---|---|
| `ObjectId` | `u64` |
| `TrackId` | `u64` |
| `ClipId` | `u64` (has `raw` + `track` fields — not transparent) |
| `KeyframeId` | `u64` |
| `LayerId` | `u64` |
| `NodeId` | `u64` |
| `PortId` | `u64` |
| `EdgeId` | `u64` |
| `RowId` | `u64` |
| `ColumnId` | `u64` |
| `ObjectId3D` | `u64` |
| `Position` | `u32` |
| `Time` | `f64` |
| `Duration` | `f64` |
| `RelationId` | `u64` |
| `Tag` | `String` |

### Structs and enums

```rust
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Transform {
    pub translation: Vec2,
    pub rotation: f64,
    pub scale: Vec2,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum CanvasOperation {
    Create { transform: Transform },
    Move { id: ObjectId, translation: Vec2 },
    Remove { id: ObjectId },
    // ...
}
```

### Environment containers

```rust
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Canvas {
    objects: HashMap<ObjectId, Object>,
    draw_order: Vec<ObjectId>,
    next_id: u64,
}
```

All fields are serializable because `HashMap`, `Vec`, and `u64` implement `Serialize`/`Deserialize`.

---

## What's NOT Serializable

| Type | Reason | Solution |
|---|---|---|
| `EffectFn` (`Arc<dyn Fn...>`) | Closures can't be serialized | `#[serde(skip, default = "default_effect")]` |
| `RelationRegistry.next_id` | `AtomicU64` doesn't implement `Serialize` | `#[serde(skip)]` |
| `HistoryStore` | Uses `Mutex<Vec<...>>` | Not serializable (App Engine type) |
| `EventBus` | Contains handler boxes | Not serializable (App Engine type) |
| `SignalBus` | Contains callback boxes | Not serializable (App Engine type) |
| `DomainEngine` | Contains `EventBus`, `HistoryStore`, etc. | Not serializable |
| `EvaluationEngine<'a>` | Lifetime-bound | Not serializable |
| `EvaluationContext<'a>` | Lifetime-bound | Not serializable |
| `SourcedValueStore<S>` | Generic, not concrete | Not serializable |
| `FnCriteria<F>` | Contains closure | Not serializable |
| `FnSortCriteria<F>` | Contains closure | Not serializable |
| `FnRule<F>` | Contains closure | Not serializable |
| `FnBatchOperation<F>` | Contains closure | Not serializable |

---

## RelationDefinition: Custom Default

`RelationDefinition` has an `effect: EffectFn` field that can't be serialized. Serde's `#[serde(skip)]` requires `Default`, but `Arc<dyn Fn...>` doesn't implement `Default`. The solution: a custom default function.

```rust
#[cfg(feature = "serde")]
fn default_effect() -> EffectFn {
    Arc::new(|_| EffectOutcome::NoChange)
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone)]
pub struct RelationDefinition {
    pub source: Endpoint,
    pub target: Endpoint,
    pub trigger: Trigger,
    pub propagation: PropagationMode,
    pub priority: i32,
    #[cfg_attr(feature = "serde", serde(skip, default = "default_effect"))]
    pub effect: EffectFn,
}
```

When deserializing a `RelationDefinition`, the `effect` field is filled with a no-op closure. The Domain Engine must re-register the real effect after deserialization.

---

## Round-Trip Tests

```rust
#[cfg(feature = "serde")]
#[test]
fn canvas_roundtrip() {
    let mut canvas = Canvas::new();
    canvas.create_object(Transform::from_translation(Vec2::new(1.0, 2.0)));
    canvas.create_object(Transform::from_translation(Vec3::new(3.0, 4.0)));

    let json = serde_json::to_string(&canvas).unwrap();
    let restored: Canvas = serde_json::from_str(&json).unwrap();

    assert_eq!(canvas, restored);
}
```

### What round-trip tests verify

| Test | What it checks |
|---|---|
| `canvas_roundtrip` | Canvas objects, draw_order, next_id survive |
| `timeline_roundtrip` | Tracks, clips, keyframes, interpolation survive |
| `table_structure_roundtrip` | Rows, columns survive (cells don't — see limitation) |
| `document_roundtrip` | Text and char_count survive |

---

## Known Limitations

| Limitation | Reason | Workaround |
|---|---|---|
| `Table`'s `HashMap<CellId, CellValue>` can't serialize to JSON | `serde_json` requires string keys; `CellId` is a struct | Custom serializer, or use a different format (bincode, msgpack) |
| `Table`'s `HashMap<CellId, CellId>` (merged_cells) | Same reason | Same workaround |
| `RelationDefinition::effect` deserializes as no-op | Closures can't be serialized | Re-register the effect closure after deserialization |
| `RelationRegistry::next_id` skipped on deserialize | `AtomicU64` can't be serialized | The registry re-issues ids from 0 after deserialization; existing `RelationId`s may collide if not remapped |

---

## Using with `serde_json`

```rust
#[cfg(feature = "serde")]
fn save_canvas(canvas: &Canvas) -> String {
    serde_json::to_string(canvas).unwrap()
}

#[cfg(feature = "serde")]
fn load_canvas(json: &str) -> Canvas {
    serde_json::from_str(json).unwrap()
}
```

Add `serde_json` as a dev-dependency for round-trip tests:

```toml
[dev-dependencies]
serde_json = "1"
```

---

## Using with Binary Formats

For formats that support non-string keys (like `bincode`), the `Table` limitation doesn't apply:

```toml
[dependencies]
bincode = "1"
```

```rust
let bytes = bincode::serialize(&table).unwrap();
let restored: Table = bincode::deserialize(&bytes).unwrap();
assert_eq!(table, restored);
```

---

## Next Steps

- [17-testing.md](17-testing.md) — Testing strategies.
- [13-domain-engine-integration.md](13-domain-engine-integration.md) — DomainEngine (not serializable, but contains serializable environments).
- [11-relations.md](11-relations.md) — Relations (effect closures are skipped on serialize).
- [18-public-api-reference.md](18-public-api-reference.md) — Complete type listing.