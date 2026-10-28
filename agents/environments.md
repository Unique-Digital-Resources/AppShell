# Environment authoring pattern

*Read when: creating or extending an environment under
`src/domain_foundation/environments/`. Stateless — pair with
`agents/conventions.md` (golden rules) and `agents/recipes/new_environment.md`
(the step-by-step playbook).*

## What an Environment is

From `src/domain_foundation/environments/mod.rs` (authoritative):

- A **reusable, headless model** for a particular kind of domain organisation or
  editing context. *"Headless"* = no DOM, CSS, rendering coordinates, pointer state,
  or windowing is required to represent, inspect, or change its state.
- It is **not** a UI component and **not** a mini App Engine: it defines *what an
  operation means*; `app_engine`/`domain_engine` provide execution infrastructure.
- **No universal trait** — each environment owns its semantics; composition between
  concrete environments is allowed when genuinely required (NodeGraph → Canvas2D).
- Application data is **not** stored on objects: domain engines layer it externally,
  keyed by the environment's id. The model keeps only an opaque reference if needed
  (e.g. `docking::ContentRef`).

## Module skeleton and registration

```
src/domain_foundation/environments/<name>/
├── mod.rs        # doc header (owns / not-owns), pub mods, re-exports, feature placeholders
├── <model>.rs    # identity type + model struct + invariants         (naming varies per env)
├── <types>.rs    # payloads, slots, sizing, regions, ...
├── operations.rs # value-type Operation/Outcome enums + apply()
└── error.rs      # XxxError
```

Registration: `pub mod <name>;` in `environments/mod.rs` + one line in its doc tree +
a mention in `src/domain_foundation/mod.rs` docs. Re-export the public surface from
`mod.rs`: `{ pub use model::…; pub use error::XxxError; pub use operations::{XxxOperation, XxxOutcome}; }`.

Actual file names vary per environment (conventions over uniformity):

| Env | Files |
|---|---|
| `canvas_2d` | `canvas.rs object.rs transform.rs operations.rs error.rs` |
| `timeline` | `timeline.rs track.rs clip.rs keyframe.rs operations.rs error.rs` |
| `layers` | `hierarchy.rs layer.rs operations.rs error.rs` |
| `node_graph` | `graph.rs node.rs port.rs edge.rs operations.rs error.rs` |
| `viewport_3d` | `scene.rs object.rs transform.rs operations.rs error.rs` |
| `table` | `table.rs row.rs column.rs cell.rs operations.rs error.rs` |
| `text_editor` | `document.rs position.rs range.rs operations.rs error.rs` |
| `docking` | `model.rs dock.rs slot.rs embed_host.rs sizing.rs validation.rs operations.rs error.rs` |

## Identity — the id newtype

Copy the block from `agents/conventions.md` (counter-issued opaque newtype,
`serde(transparent)`, `from_raw`/`as_raw` crate-visible, `"<noun>#{n}"`).
`external code cannot construct ids directly` — they come from the model's creation
methods or from outcomes.

## Model struct + method surface

Reference: `canvas_2d/canvas.rs`, `layers/hierarchy.rs` (docking `model.rs` for the
most recent full example).

- Fields: `HashMap<Id, T>` + ordered index / root pointer; `next_id` counter.
- Public: `new()`, `len()`, `is_empty()`, `contains(id)`, `get(id)`, iteration over
  values, `clear()` (counter not reset), accessors naming the payloads
  (`children() -> Option<&[Id]>` style).
- `pub(crate)`: mutation surface for operations and undo paths (`get_mut`,
  `insert_*/put_*` raising the counter, `take_*`), structural collections, and
  anything that must not bypass operation validation.
- Creation primitives with typed preconditions (every referenced id exists, is
  unique in this call, is unplaced/eligible); preconditions check before any
  mutation so failures leave the model unchanged.
- Traversal: `is_descendant`, `descendants` (BFS, every reference category),
  `path`, `depth`.

## Operations and outcomes

One public value-type enum pair per environment:

```rust
pub enum LayersOperation { CreateLayer { name: String }, RemoveLayer { id: LayerId }, Reparent { id: LayerId, new_parent: LayerId }, … }
pub enum LayersOutcome  { LayerCreated { id: LayerId }, LayerRemoved { layers: Vec<Layer> }, Reparented { id: LayerId, previous_parent: Option<LayerId> }, … }

impl LayersOperation {
    pub fn apply(self, layers: &mut Layers) -> Result<LayersOutcome, LayersError> { … }
}
```

Rules:

- The operation value describes what the change *means* (intent), not a sequence of
  model calls.
- Outcomes always carry **inverse data for undo** (previous transform, removed
  subtree in BFS order, previous placement/size/selection). This is the contract the
  engine-side `Executed*Operation` wrapper relies on — see
  `agents/engine_integration.md`.
- Cycle prevention: reject `new == self` and `is_descendant(new, self)` with
  `CycleDetected` (layers precedent).
- Atomicity: validate everything, mutate only when every precondition holds; a
  failed op leaves the model untouched. Multi-part structural ops may build a
  candidate state (clone or staged edits), validate, then commit — the outcome still
  records the inverse.

## Validation layer (when the env defines "valid state")

See `agents/conventions.md` ("Validation pattern"). The `docking::validation.rs`
implementation is the working reference: creation valid-by-construction +
steady-state `validate()` with documented rule precedence + deterministic first-error
scanning + in-crate invalid-state fixtures.

## Tests

`tests/domain_foundation_tests/<name>_test.rs` (+ `<name>_operations_test.rs` etc.),
registered in `tests/domain_foundation_tests/main.rs` as `mod <name>_test;`.
Integration tests use the public crate API only; invalid states are built in-crate.
Details: `agents/testing.md`.

## Docs

The per-environment documentation page (`md_docs/domain_docs/NN-<name>.md`) is a
docs-phase deliverable, not implementation work — see `agents/docs_site.md`.
