# Conventions — golden rules and code style

*Read when: writing or reviewing any Rust in this repo. Stateless.*
*When unsure between a convention and external naming, match the nearest existing code.*

## Golden rules (always in force)

1. **Match existing code conventions over any external proposal.** Every environment
   is headless: no DOM, no CSS, no rendering, no pointer/input, no windowing.
2. **No universal `Environment` trait** — environments are duck-typed by convention
   (intentional; see `src/domain_foundation/environments/mod.rs` doc header).
3. **Operations are value-type enums** with
   `apply(self, &mut Model) -> Result<Outcome, EnvError>`; outcomes carry
   **previous state for undo** (precedents: `CanvasOutcome::Moved { previous: Transform }`,
   `LayersOutcome::LayerRemoved { layers: Vec<Layer> }` — removed subtree in BFS order).
4. **Serde is feature-gated**, never unconditional:
   `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]`
   (add `serde(transparent)` for counter-backed id newtypes). Additive fields on existing
   payloads take `#[cfg_attr(feature = "serde", serde(default))]` so state persisted before
   the field existed restores with its neutral value (docking `Panel.collapsed`, Ph.6). A
   named default (`serde(default = "path")`) needs its helper fn behind
   `#[cfg(feature = "serde")]` or the flag-free build warns it dead (the
   `relations/definition.rs::default_effect` precedent); a **struct-level**
   `#[serde(default)]` fills missing fields from the struct's `Default` impl — put the
   neutral values there, not field `Default::default()`s (docking `HostRegions`, Ph.7).
5. **Don't skim the whole project** — use the minimal file maps
   (`agents/environments.md`, `agents/engine_integration.md`) and the
   do-not-read boundary (`agents/project.md`).

## Identity (counter-issued opaque newtypes)

```rust
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectId(u64);

impl ObjectId {
    pub fn from_raw(raw: u64) -> Self { Self(raw) }   // crate-visible, not for external callers
    pub fn as_raw(self) -> u64 { self.0 }
}
impl fmt::Display for ObjectId { /* "object#{}" */ }
```

- Issued by the model from a `next_id: u64` counter; **the counter is never reset by
  `clear()`**; the undo-path `insert_*`/`put_*` raises the counter if needed.
- Ids are not array indices; removal must not renumber survivors. Not stable across
  model instances/sessions — persistence restores the *structure* and re-acknowledges
  ids by an explicit policy.
- `Display` format: `"<noun>#{n}"`.

## Model storage pattern

- `HashMap<Id, T>` for storage + an explicit ordered collection for ordering
  (`draw_order`, `roots`, `children` on `Layer`).
- Structural collections are `pub(crate)` (or mutated only via `pub(crate)` model
  methods) so external code cannot bypass operation validation — public read
  accessors only. Undo/redo paths and test fixtures use the `pub(crate)` mutators
  (`put_*`, `take_*`, `get_mut`).
- No parent back-pointers: containers' collections are the single source of
  structural truth; placement is *derived by scan* (zero desync risk, O(n) inquiry —
  document the tradeoff when adopting it).
- Traversal precedents to copy: `is_descendant`, `descendants` (BFS), `path`,
  `depth`/`subtree_height` (recursive).
- Payments pick a **deterministic scan order** (e.g. fixed edge order, or ascending
  sorted ids for error reporting) — never rely on HashMap iteration order.

## Validation pattern (whole-layout phase)

Established by the `docking` environment — reuse for any environment that needs a
"valid state" definition beyond creation preconditions:

- Creation paths stay valid-by-construction; a `validate(&self) -> Result<(), Error>`
  is the *steady-state* authority, used on imported/restored state and as the commit
  gate for operations. It deliberately rejects transient mid-operation states.
- First-error semantics with a documented rule precedence, scanning in a
  deterministic order (ascending sorted ids).
- **Reuse creation-time error variants** for identical rules; add layout-invariant
  variants only for genuinely new rules. Error enums keep `Eq` — echo locations
  (ids, indexes), not payloads that break `Eq` (e.g. f64 values).
- Derivable properties (e.g. reachability from the root) are asserted in tests rather
  than re-implemented as redundant checks.
- Cycles are rejected by name following the `LayersError::CycleDetected` precedent,
  detected over *all* reference categories, not only ordinary children.

## Errors (per environment)

```rust
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XxxError { /* heterogeneous payloads */ }

impl fmt::Display for XxxError { /* lowercase human message, each variant */ }
impl std::error::Error for XxxError {}
```

- No shared error type in `domain_foundation`; each environment owns its enum and the
  Domain Engine wraps it (`DomainError::Xxx(E)`.
- Variant naming follows prior art (e.g. `CycleDetected`, `NotFound`-style ids,
  `{Noun}CountMismatch`).

## History, events, signals

- `HistoryEntry::new("<env>.<op>").undoable()` (+ `.with_transaction(tx_id)` when
  inside a transaction) is appended by `DomainEngine::execute_*`.
- Events: `"{env}.{noun}.{action}"` (e.g. `layers.layer.created`,
  `docking.dock.moved`); signals the same with a `"{env}.changed"` wildcard fallback.
- Keep the outcome→inverse contract: every outcome must contain whatever undo needs,
  without a model snapshot.

## Other style anchors

- Environment module doc header: "What this environment owns / deliberately does
  not own", with PRD/design-spec section references in doc comments (e.g. `PRD §8.2`).
- Doc comments reference planning documents by section (PRD §n.n, Ph.1/Ph.2 records)
  so decisions stay traceable.
