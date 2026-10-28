# Docking Environment — Phase 2 Record: Core Domain Model

**Phase:** 2 of 12 (`current_state/plan.md` — Core domain model)
**Status:** ✅ Complete — model compiles, `serde` feature compiles, full suite green
**Verification:** `cargo test` → 426 domain-foundation tests pass (14 new docking model tests), 850+ other tests unaffected; `cargo check --features serde` clean; no warnings from the new module.

---

## 1. What was built

New environment `src/domain_foundation/environments/docking/`:

| File | Responsibility |
|---|---|
| `dock.rs` | [`DockId`] opaque u64 newtype (counter-issued, `serde(transparent)`, `dock#{n}`), [`ContentRef`] (opaque host-supplied content reference), [`SplitOrientation`] (Row/Column), [`DockKind`] category enum, [`Dock`] { id, role } + [`DockRole`] payloads (Panel / Split / TabGroup / EmbedHost) and read accessors (`children`, `sizes`, `active`, `host_regions`, `floating`, `structural_children`) |
| `embed_host.rs` | [`HostRegions`] — top/bottom/left/right, each `Option<DockId>`; `on(edge)`, `owner_of(dock)`, `occupied()` (fixed scan order), `assign` `pub(crate)` (operations-only) |
| `slot.rs` | [`EdgeRegion`] (Top/Bottom/Left/Right), [`DockSlot`] (`Root` / `SplitChild{parent,index}` / `TabMember{parent,index}` / `HostEdge{host,edge}` / `Floating{host,index}`), `owner()` |
| `sizing.rs` | [`SplitSizes`] — relative weights for split children; `uniform(count)`, `from_weights(vec)` (finite/positive validation), `matches(child_count)`; normalization policy deferred to Phase 6 by design |
| `model.rs` | [`DockLayout`] — `HashMap<DockId, Dock>` + `root: Option<DockId>` + `next_id` counter (never reset); inspections `get/contains/iter/kind_of`, creation primitives (`create_panel`, `create_embed_host`, `create_split`, `create_tab_group`, `set_root`), placement scan (`placement()`), traversal (`is_descendant`, `descendants` BFS across **all** placement categories, `path`, `depth`, `all_docks`) |
| `error.rs` | [`DockingError`] — `DockNotFound`, `UnknownReference`, `DuplicateMembership`, `AlreadyPlaced`, `InsufficientChildren`, `SizeCountMismatch`, `ActiveIndexOutOfRange`, `RootAlreadySet` (serde-gated; style matches all other environments) |
| `mod.rs` | doc header (owns / doesn't own), submodule declarations, re-exports, placeholders noting operations (Ph.4), validation (Ph.3), normalization (Ph.5) |

Registered in `environments/mod.rs` (`pub mod docking;` + doc tree), `domain_foundation/mod.rs` doc list.

**No parent back-pointers**: containers' collections are the single source of structural truth; placement is derived by scan (documented tradeoff: O(n) inquiry, zero desync risk — revisitable if benchmarks demand).

## 2. Decisions made during implementation

1. **Auto-rooting policy (refined):** the first dock created into a *completely empty* layout becomes root. Subsequent creates are unplaced. Auto-root never fires for a transient rootless (mid-operation) state.
2. **Creation preconditions:** split needs ≥ 2 children, tab group ≥ 1 member, active index within range, every referenced dock exists, unique, and is currently **unplaced** (`AlreadyPlaced`). Operations (Phase 4+) detach first, then create — keeping "create" pure.
3. **Root designation is explicit:** `set_root(Some)` requires no current root + unplaced existing dock; `set_root(None)` clears to the transient empty-root state. Operation re-rooting flow: `set_root(None)` → build → `set_root(new)`.
4. **Empty layout = `root: None` + zero docks** (PRD §7.4 "explicit empty-layout state", no fake containers).
5. **Tab `active` is an index** into `members` (canvas `Reorder{new_index}` precedent); validation will enforce `active < members.len()` (Phase 3).
6. **Collapse state deferred** to Phase 6 with sizing (kept out of `Panel` payload for now).

## 3. Completion criteria check (current_state/plan.md Phase 02)

- [x] A layout can be represented entirely as domain data (no rendering coordinates or widgets — see `nested_layout_representable`).
- [x] Nested structures can be inspected and traversed (`descendants`, `path`, `depth`, `placement`).
- [x] Identity and ownership rules are unambiguous (stable ids; exactly-one-placement derived by scan).
- [x] No UI required (headless tests only).

## 4. Files touched

```
src/domain_foundation/environments/docking/{mod,dock,embed_host,slot,sizing,model,error}.rs   (new)
src/domain_foundation/environments/mod.rs   (registration + doc tree)
src/domain_foundation/mod.rs                (doc list mentions Docking)
tests/domain_foundation_tests/main.rs       (mod docking_test;)
tests/domain_foundation_tests/docking_test.rs (new — 14 tests)
```

## 5. Next: Phase 03 — Validation and structural invariants

- `validation.rs` on the model: identity uniqueness, root validity (persistent empty root with docks = invalid), owner/slot compatibility, no cycles (operations already prevented by construction, but restoration & imports need checking), active-tab validity, size-count coherence, host-region/floating reference validity.
- Error variants for each violated rule (validation-result style).
- `tests/domain_foundation_tests/docking_validation_test.rs` + fixtures in `domain_foundation/testing/fixtures.rs` only if ≥ 2 suites share shapes.
