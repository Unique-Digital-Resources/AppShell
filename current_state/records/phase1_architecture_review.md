# Docking Environment — Phase 1 Report: Architecture & Integration Review

**Project:** AppShell
**Subsystem:** Domain Foundation — new built-in environment `docking`
**Phase:** 1 of 12 (per `current_state/plan.md`)
**Status:** ✅ Complete
**Sources reviewed:** actual repository source code (as of 2026-10-17), plus the planning documents (now under `current_state/`).

---

## 1. Phase 1 Goals

Per `current_state/plan.md`, Phase 1 must:

1. Inspect the existing `canvas_2d`, `layers`, and `node_graph` environments (and the four others that share the same pattern).
2. Identify how each environment defines models, operations, errors, and public exports.
3. Inspect `domain_engine/operations` to understand command handlers and operation execution.
4. Identify the established patterns for state changes, history, persistence, and serialization.
5. Finalize the docking model's terminology and public boundaries.
6. Decide how docking IDs and references will be represented using existing conventions.

**Completion criteria (checked off):**

- [x] The proposed docking model is documented (§5–§7).
- [x] Integration points are identified from actual source code (§3–§4).
- [x] No duplicate command, history, or persistence mechanism is planned (§8).

---

## 2. Environments that exist in the source (ground truth)

`src/domain_foundation/environments/mod.rs` registers — in this order:

```rust
pub mod canvas_2d;   // Canvas, Object, ObjectId, Transform, Vec2, CanvasOperation/Outcome, CanvasError
pub mod timeline;    // Timeline, Track, Clip, Keyframe, Duration, Time, …
pub mod layers;      // Layers, Layer, LayerId, …
pub mod node_graph;  // NodeGraph, Node, NodeId, Port, Edge, … (composes Canvas2D internally)
pub mod viewport_3d; // Scene, Object3D, …
pub mod table;       // Table, Row, Column, Cell, …
pub mod text_editor; // Document, …
```

All seven follow one identical shape. There is **no universal `Environment` trait** — that omission is documented as intentional in `environments/mod.rs` ("No `Environment` trait is declared here. Intentional."). Docking will follow the same duck-typed pattern: no shared base, just matching conventions.

Environment model structs are named per domain, not uniformly: `Canvas`, `Timeline`, `Layers`, `NodeGraph`, `Table`, `Document`, `Scene`.

---

## 3. The environment authoring pattern (from actual code)

Every environment consists of the same responsibilities; file names vary slightly.

### 3.1 Identity — opaque counter-backed newtype

Reference: `canvas_2d/object.rs`, `layers/layer.rs`, `timeline/*`, `node_graph/node.rs`.

```rust
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectId(u64);

impl ObjectId {
    pub fn from_raw(raw: u64) -> Self { Self(raw) }
    pub fn as_raw(self) -> u64 { self.0 }
}
impl fmt::Display for ObjectId { /* "object#{}" */ }
```

- IDs are issued by the environment's model from a `next_id: u64` counter.
- Counter is **not reset** by `clear()`; `insert_*` (undo path) raises the counter if needed (`canvas.rs:85-94`, `hierarchy.rs:78-99`).
- Documented as "not array indices; removing one object does not renumber the others" — exactly the docking stable-identity requirement (PRD DM-01…DM-05).
- IDs are instance-scoped: "Not stable across canvases or sessions — do not persist or compare across instances." Note for docking persistence: restoration must re-issue/renumber ids or define an id policy of its own (see §7).

### 3.2 Value-type operation + outcome enums with `apply()`

Reference: `canvas_2d/operations.rs`, `layers/operations.rs`, and identically in node_graph / table / text_editor / timeline / viewport_3d.

```rust
pub enum LayersOperation { CreateLayer { name: String }, RemoveLayer { id: LayerId }, Reparent { id, new_parent }, … }
pub enum LayersOutcome { LayerCreated { id }, LayerRemoved { layers: Vec<Layer> }, Reparented { id, previous_parent }, … }

impl LayersOperation {
    pub fn apply(self, layers: &mut Layers) -> Result<LayersOutcome, LayersError> { … }
}
```

Key rules observed:

- Application data is **not** attached to objects; domain engines layer it externally keyed by id (canvas_2d/mod.rs). Docking must do the same for panel content.
- Operation is a *value-type intent describing what the change means*.
- Outcome carries **previous state for invertibility**:
  - `CanvasOutcome::Moved { previous: Transform }`
  - `CanvasOutcome::Removed { object }` (returns the removed object for undo — `canvas.rs:127-135`)
  - `LayersOutcome::LayerRemoved { layers: Vec<Layer> }` (whole subtree in BFS order for undo re-insertion — `hierarchy.rs:101-143`). This is the precedent the docking `Remove` outcome must follow.
- Errors are specific per environment (`LayersError::{LayerNotFound, InvalidSiblingIndex, CycleDetected}` — the last is exactly docking's cycle invariant).

### 3.3 Model struct + method surface

Reference: `canvas_2d/canvas.rs`, `layers/hierarchy.rs`, `node_graph/graph.rs`.

- `HashMap<Id, T>` for storage + explicit order vector/list for ordering (`draw_order: Vec<ObjectId>`, `roots: Vec<LayerId>`, `children: Vec<LayerId>`).
- `children` on `Layer` is `pub(crate)` — mutated only through the hierarchy manager, read-only externally. Docking containers must use the same encapsulation.
- Cycle prevention precedent: `Layers::reparent()` rejects `np == id || is_descendant(np, id)` with `CycleDetected` (`hierarchy.rs:223-267`, `271-288`). Docking moves need the same check.
- Traversal precedents exist: `is_descendant`, `descendants` (BFS), `depth`, `layer_path`.

### 3.4 Errors

Reference: every `error.rs`. Shape:

```rust
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XxxError { … heterogeneous payloads … }

impl fmt::Display for XxxError { … }   // lowercase human message
impl std::error::Error for XxxError {}
```

There is no shared error type in `domain_foundation`; each environment owns its own. The Domain Engine wraps them.

---

## 4. Domain Engine integration pattern (`src/domain_engine/mod.rs`)

`DomainEngine` composes all environments **as owned fields**, plus history and event infrastructure.

### 4.1 Field, accessor, execution method — the pattern a new environment must add

```rust
pub struct DomainEngine {
    canvas: Canvas, timeline: Timeline, layers: Layers, node_graph: NodeGraph,
    table: Table, document: Document, scene: Scene,
    history: HistoryStore, …
}

pub fn layers(&self) -> &Layers { &self.layers }

pub fn execute_layers(&mut self, operation: Layers_operation)
    -> Result<LayersOutcome, DomainError>
{
    let outcome = operation.clone().apply(&mut self.layers)
        .map_err(|e| { let err: DomainError = e.into(); self.check_auto_rollback(&err); err })?;
    let name = layers_op_name(&operation);
    let mut entry = HistoryEntry::new(name).undoable();
    if let Some(tx_id) = self.active_transaction { entry = entry.with_transaction(tx_id); }
    self.history.append(entry);
    self.publish(layers_outcome_event(&outcome), layers_outcome_signal(&outcome));
    Ok(outcome)
}
```

So the docking additions are, exactly:

| Extension point | Actual convention (source) | Docking equivalent |
|---|---|---|
| Field | `layers: Layers` | `docking: DockLayout` |
| Accessor | `pub fn layers(&self) -> &Layers` | `pub fn docking(&self) -> &DockLayout` |
| Execute | `execute_layers(op) -> Result<Outcome, DomainError>` | `execute_docking(op) -> Result<DockingOutcome, DomainError>` |
| Error variant | `DomainError::Layers(LayersError)` + `From<LayersError>` + `Display` arm `"layers: {e}"` | `DomainError::Docking(DockingError)` + `From` + `"docking: {e}"` |
| Name helpers | `layers_op_name`, `layers_outcome_event`, `layers_outcome_signal` as bottom-of-file free fns | `docking_op_name`, `docking_outcome_event`, `docking_outcome_signal` |
| Event naming | `"{env}.{noun}.{action}"` e.g. `layers.layer.created`, `canvas.object.moved` | `docking.dock.created`, `docking.dock.moved`, `docking.host.embedded`, … |
| Signal naming | same as event, with `_ => "{env}.changed"` wildcard fallback | same policy |

### 4.2 Undo/redo reality — **the most important Phase 1 finding**

`DomainEngine::undo()` / `redo()` (mod.rs:467-497) only **replay state** for Canvas:

```rust
for i in (pos_after..pos_before).rev() {
    if let Some(executed) = self.executed_canvas.get(i) { executed.undo(&mut self.canvas)?; }
}
```

All other environments only push name-only history entries. Consequences for docking:

1. If docking operations only produced name-only outcomes, **docking undo would be a lie** (badly violating PRD UR-01…UR-05).
2. The established mechanism for *real* undo is `ExecutedCanvasOperation` (`domain_engine/operations/canvas_operations.rs`): a wrapper that stores `(operation, outcome)` and implements `undo(&mut Canvas)` / `redo(&mut Canvas)` from the outcome's inverse data.
3. Docking must therefore be the **second environment** with real undo: an `ExecutedDockingOperation` in `domain_engine/operations/docking_operations.rs`, plus a parallel `executed_docking` vector on `DomainEngine`, and docking branches in `undo()`, `redo()`, and `rollback_transaction()` (which currently only reverts canvas ops, mod.rs:405-424).

This means the *environment's* `DockingOutcome` must carry complete inverse information (removed subtrees, previous placements, previous sizes, previous active tabs) — consistent with what CanvasOutcome already does.

### 4.3 Command handlers (optional bridge)

`domain_engine/operations/{canvas,layers,timeline}_command_handler.rs` implement `app_engine::commands::CommandHandler`:

- Hold `Arc<Mutex<Model>>`; extract typed payload via `input.get::<XxxOperation>()`; apply; append `HistoryEntry::new(name).undoable()` to `engine.history_store`; publish event on `engine.event_bus`; return `CommandResult::success_with(outcome)`.
- These are a separate, *optional* app-engine bridge (not used by `DomainEngine::execute_*`). PRD §6.3 forbids a competing engine; a `docking_command_handler.rs` may be added **later, only if needed**, following this exact template. It is not required for Phase 2–10.

### 4.4 Transactions & compound operations

`begin_transaction / commit / rollback`, auto-rollback on error, and `begin_compound / commit_compound / rollback_compound` (depth-counted, mod.rs:391-463). Docking operations automatically inherit transaction tagging via `entry.with_transaction(tx_id)` — no extra docking code needed beyond following §4.1.

---

## 5. Established conventions that docking must follow (summary table)

| Aspect | Convention (source of truth) | Docking application |
|---|---|---|
| Module registration | `pub mod docking;` in `environments/mod.rs` (after `text_editor`) — doc comment header "What this env owns / deliberately does not own" | `docking/` under environments |
| Public exports | `pub use model::…; pub use error::XxxError; pub use operations::{XxxOperation, XxxOutcome};` | `DockLayout`, `Dock`, `DockId`, `DockKind`/categories, `DockingOperation`, `DockingOutcome`, `DockingError`, slots, sizing |
| Operation semantics | value enum + `apply(self, &mut Model)` | `DockingOperation::apply(self, &mut DockLayout)` |
| Atomicity | operations return `Result`; partial failure = Err before commit; undo via outcome inverses (PRD OP-01…OP-04) | candidate-then-commit for multi-part structural ops |
| Cycle prevention | `LayersError::CycleDetected` precedent | same variant naming for docking |
| Removal outcome | `LayersOutcome::LayerRemoved { layers: Vec<Layer> }` BFS for undo | `DockRemoved { removed: Vec<Dock>, previous_placement, … }` |
| Serde | `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]` everywhere; `serde(transparent)` on ids; feature `serde = ["dep:serde"]` in Cargo.toml | identical gating |
| Error contract | per-env enum; Display; std::error::Error | identical |
| History | `HistoryEntry::new("<env>.<op>").undoable()` + `with_transaction` | `"docking.<op>"` names |
| Events/Signals | `publish(event_name, signal_name)`; `"{env}.{noun}.{verb}"` | identical |
| Tests | `tests/domain_foundation_tests/main.rs`: one `mod xxx_test;` per file; integration tests use the public crate API; `domain_foundation/testing/fixtures.rs` only for ≥2 shared shapes | `docking_test`, `docking_operations_test` (+ validation/normalization/atomics in later phases) |
| Docs | `md_docs/domain_docs/03-environments.md` + per-env docs 04–10 + guides; nav in `mkdocs.yml` | add `21-docking.md` in Phase 12 (docs phase), update `mkdocs.yml` nav |
| Benches | `benches/domain_foundation_bench.rs` (criterion, harness=false) | optional later |

---

## 6. How the granular 12-phase plan maps to the PRD's 7 phases

The PRD (§20) uses 7 coarse phases; `current_state/plan.md` splits them into 12 fine phases. Mapping (for traceability):

| PRD phase | current_state/plan.md phase(s) |
|---|---|
| Phase 1 — Repository alignment & model spec | Phase 01 (this report) |
| Phase 2 — Model & validation | Phases 02–03 (core model; invariants) |
| Phase 3 — Structural operations | Phases 04–05 (ops; normalization/atomicity) |
| Phase 4 — Sizing & state transitions | Phases 06–08 (sizing; embedding/lifecycle; floating) ← *floating split out* |
| Phase 5 — Domain Engine integration | Phase 09 |
| Phase 6 — Persistence & history verification | Phases 10, 12 |
| Phase 7 — Frontend adapter prototype | Phase 11 (interaction contract; adapter is out of this repo's core scope) |

No conflict; the 12-phase plan remains authoritative.

---

## 7. Decisions resolved (Phase 1 deliverables)

### 7.1 Terminology & public boundary (finalized, aligned to code conventions)

- **Environment name:** `docking` (`domain_foundation::environments::docking`).
- **Model struct:** `DockLayout` — headless layout model owning root, docks, sizing, placements. Naming mirrors `Canvas`/`Layers`/`Scene` being domain nouns rather than "XManager".
- **Stable identity:** `DockId(u64)` opaque newtype — counter issued by `DockLayout`, `serde(transparent)`, `Copy/Eq/Hash/Order`, `Display` as `dock#{n}`. Application content stays external, keyed by `DockId` (PRD §7.3; canvas_2d precedent).
- **Dock categories:** `Panel`, `Split { orientation: Row | Column }` (one container type instead of separate row/column structs — per `structure.md` §5 refinement and to keep normalization simple), `TabGroup { active, children }`, `EmbedHost { content, top/bottom/left/right regions, floating set }`. A single `Dock` representation with per-kind payloads (or `enum Dock`) is a Phase 2 implementation choice; **names and responsibilities are now fixed**.
- **Root:** the layout itself owns the root node ("root is structural, never removed", PRD DM-06). `DockLayout` fields will follow the `Layers` pattern: `HashMap<DockId, Dock>` + structural index. Empty-layout state is represented explicitly (no fake panel — PRD §7.4).
- **Operations:** one public value enum `DockingOperation` with an `apply(self, &mut DockLayout) -> Result<DockingOutcome, DockingError>` entry (same one-method contract as all seven existing environments). Optional ergonomics wrapper can come later via `domain_engine::operations`.
- **Outcomes:** `DockingOutcome` — must always carry full inverse data (precedents: `CanvasOutcome::Moved{previous}`, `LayersOutcome::LayerRemoved{layers}`), because docking will be the second environment with true undo (§4.2).
- **Errors:** `DockingError` with variants covering PRD §14 (NotFound, DuplicateId, CycleDetected, InvalidRole, OccupiedRegion, HostIneligible, InvalidSizing, ActiveTabInvalid, …). Exact variant list is finalized in Phase 3 alongside operations; *style* (serde-gated, Display, std::error::Error) is fixed now.
- **Engine API:** `docking()`, `execute_docking(op) -> Result<DockingOutcome, DomainError>`, `DomainError::Docking(..)`, `"docking.dock.*"` events. `docking_command_handler.rs` deferred until an app-engine command consumer exists.

### 7.2 ID & reference representation

Docking uses **environment-issued `DockId` counters**, exactly like `ObjectId`/`LayerId`: private constructor, `from_raw` for the environment itself, counter never reset, raised by the restoration path. Cross-session persistence therefore must restore *structure* and re-issue ids (or ship them inside the serialized form and validate collisions — decision recorded for Phase 10, not needed for Phase 2).

### 7.3 The three open architecture questions (from `new_domain_foundation.md` §8) — settled direction

1. **Layout representation:** hierarchical tree for split/tab relationships + **explicit non-tree placement collections** on `EmbedHost` (edge regions, floating set). Confirmed by PRD §7.6 and avoids forcing floats into child semantics. Validates the `Layers` + `Canvas` storage pattern (map + ordered index).
2. **Atomicity & undo:** operations compute against the live model but validate before commit; on success they produce a complete-inverse `DockingOutcome` consumed by `ExecutedDockingOperation` (canvas_2d pattern, §4.2). No separate undo framework (PRD §23.8–23.9). Engine-side transaction support is inherited.
3. **Sizing semantics:** relative weights per split container's children (PRD §9.2 baseline), stored as logical state, with defined transfer/normalization rules delegated to Phase 6 tasks. Embedded regions carry their own size property; floating docks carry logical position+size within host bounds (PRD §9.6).

### 7.4 No duplicate mechanisms (criterion 3)

- No new history store, no new transaction manager, no new event/signal system, no new persistence backend, no universal `Environment` trait, no plugin/behaviour **registry** in the core (the design spec's behaviour registry is an interaction-layer concept; the core exposes a closed operation enum — open questions sharing/registry are deferred with the frontend phases per PRD §22).
- Serialization rides on the `serde` feature; restoration validation lives in the environment (`DockingError::MalformedLayout`, etc.).

---

## 8. Risks discovered during review (adds to PRD §21)

- **R-A1 (new):** `DomainEngine::undo/redo` currently replay **only** Canvas. Integrating docking undo requires touching `undo()`, `redo()`, and `rollback_transaction()` — regression risk for existing tests (`canvas_history_integration_test`). Mitigation: separate `executed_docking: Vec<ExecutedDockingOperation>` and index-aligned replay like canvas, keep canvas path untouched.
- **R-A2 (new):** `HistoryEntry` stores the inverse only implicitly, by convention (operation + outcome kept on the engine). Docking outputs must be designed for invertibility *from day one*, or Phase 10 undo will require a model-snapshot fallback (larger memory, different behavior from canvas). Mitigation: enforce "outcome carries previous state" as an acceptance test in Phase 3.
- **R-A3 (new):** name-only history is an accepted pattern in this codebase; a reviewer might wrongly reuse it for docking. Mitigation: this report plus Phase 9 test requirements (`docking_undo_redo_test` re-applying real state).

---

## 9. Inputs for Phase 2 (ready-to-use)

Phase 02 — Core domain model — can now start directly with:

1. `src/domain_foundation/environments/docking/mod.rs` (doc header, `pub mod …`, re-exports)
2. `dock.rs` — `DockId`, `DockCategory`/`DockKind`, shared dock info (label, collapsed state stub)
3. `container.rs` — split container (orientation, children, weights), tab group (members, active)
4. `embed_host.rs` — host content + edge regions + floating set
5. `slot.rs` — `DockSlot` (child position / tab membership / host edge / floating)
6. `sizing.rs` — `SizingWeights` + bounds types (logic in Phase 6)
7. `error.rs` — `DockingError`
8. `layout.rs` (`model.rs`) — `DockLayout{ docks: HashMap<DockId, Dock>, root, next_id }`, `new/len/contains/get`, root access, traversal (`is_descendant`, `descendants`, `path` — copy Layer/hierarchy precedents)
9. validation in Phase 03, operations in Phase 04–05.
10. Tests: `tests/domain_foundation_tests/docking_test.rs` (model) and `docking_operations_test.rs` (Phase 4), registered in `tests/domain_foundation_tests/main.rs`.

---

*End of Phase 1 report. Next milestone: Phases 02–05 as the first release slice (valid, testable headless docking model + core operations), per the plan's recommendation.*
