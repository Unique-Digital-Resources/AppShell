# Docking Environment — Phase 3 Record: Validation and Structural Invariants

**Phase:** 3 of 12 (`current_state/plan.md` — Validation and structural invariants)
**Status:** ✅ Complete — full suite green, serde path green, no new warnings
**Verification:**
- `cargo test` → **850 app_engine + 435 domain_foundation (integration) + 26 in-crate lib tests pass**, 0 failures
  (23 docking lib tests are the Phase‑3 in-crate validation fixtures; 9 external validation tests included in the 435)
- `cargo test --features serde` → same suites green with serde-gated tests enabled (439 / 30 lib; docking contributes 4 imported-state fixtures)
- `cargo check --features serde` → clean; 10 pre-existing warnings, none from `docking/`
- `cargo doc --no-deps` → no new warnings from `validation.rs` (pre-existing Phase‑2 doc warnings in `embed_host.rs`/`model.rs` untouched)

---

## 1. What was built

New file `src/domain_foundation/environments/docking/validation.rs` + additions to
`error.rs`, `model.rs`, `sizing.rs`, `mod.rs`.

| File | Established |
|---|---|
| `validation.rs` | [`DockLayout::validate`] — whole-layout invariant check returning the first violated rule; [`DockLayout::is_valid`] convenience; private colour-DFS `walk_for_cycle`; `#[cfg(test)] mod validation_tests` (in-crate invalid fixtures) with a nested `#[cfg(all(test, feature = "serde"))] mod serde_import` |
| `error.rs` | 6 new layout-invariant variants: `OrphanDock { id }`, `DuplicatePlacement { id }`, `InvalidRoot { id }`, `MissingRoot { docks }`, `CycleDetected { id }`, `InvalidSizeState { split, index }` (+ `Display` arms) |
| `model.rs` | `pub(crate) fn placement_count(&self, id) -> usize` — placement-multiplicity accounting (root slot + all container references, duplicate list entries counted), mirroring the existing placement scan |
| `sizing.rs` | `#[cfg(test)] pub(crate) fn from_weights_unchecked(Vec<f64>)` — test-only constructor for invalid weight sets (the way imported state can carry them) |
| `mod.rs` | `pub mod validation;` (placeholder comment replaced) + owns-list doc line |

### Check order (documented rule precedence; exact `assert_eq!` tests lock it)

`validate` scans docks in **ascending `DockId` order** so the reported error is
deterministic for a given broken state (PRD §17.2), and reports the *first* violated rule:

1. **Root designation** — docks present but root `None` → `MissingRoot { docks }`;
   root pointer referencing a missing dock (also with zero docks) → `UnknownReference`.
2. **Reference validity + category parity**, per dock in id order — every structural
   reference (split children, tab members, host regions in fixed top/bottom/left/right
   order, then floating — the canonical `structural_children` edge set) must exist
   (`UnknownReference`, reused); then split steady state (≥ 2 children → `InsufficientChildren`,
   reuse; sizes/child count parity → `SizeCountMismatch`, reuse; finite > 0 weights →
   `InvalidSizeState`), tab steady state (≥ 1 member → `InsufficientChildren`, reuse;
   `active < members.len()` → `ActiveIndexOutOfRange`, reuse).
3. **Placement multiplicity**, per dock in id order — 0 placements → `OrphanDock`; ≥ 2 →
   `InvalidRoot` when the root is also owned by a container, else `DuplicatePlacement`
   (double-claimed across containers *and* duplicated within one parent's collection).
4. **Acyclicity** — colour DFS from every dock over all structural edge sets
   (tags: gray-on-stack back edge = witness) → `CycleDetected`.

Derivation noted in the module docs: rules 1–3 + acyclicity imply every dock is
*reachable from the root* (every dock has exactly one owner, no cycles, only the
root is ownerless). Reachability is asserted per valid fixture
(`descendants(root).len() + 1 == len()`) rather than re-implemented as a check.

## 2. Decisions made during implementation

1. **Steady-state authority.** `validate` intentionally rejects the *transient*
   mid-operation states the model permits (unplaced docks, rootless layouts) — PRD
   DM-07 explicitly allows only transient states between operations. Phase 4
   operations validate the committed result, not their build steps.
2. **Reuse over duplication (CHECKPOINT directive).** Unknown-reference, size-count,
   child-count, and active-index rules reuse the Phase‑2 creation-time variants; only
   genuinely new layout-level rules got new variants.
3. **`CycleDetected` name (not `LayoutCycle`).** The CHECKPOINT §3 exploration list
   suggested `LayoutCycle`, but the codebase precedent is `LayersError::CycleDetected`
   (`hierarchy.rs` reparent guard) and the Phase 1 convention table fixed "same variant
   naming" — convention wins (AGENTS.md golden rule 1). Payload is the cycle witness id.
4. **`InvalidSizeState { split, index }` — no f64 payload.** `DockingError` derives
   `Eq`; error variants are echo-locations (which split, which weight) rather than
   carrying the weight value itself.
5. **Root placement rule.** A dock owning the Root slot *and* a container slot is
   reported as `InvalidRoot` (the root "must have no owner") rather than the generic
   `DuplicatePlacement`, so the error names the violated rule.
6. **In-crate fixture strategy (per CHECKPOINT §3.3).** Public API cannot construct most
   invalid states — invalid fixtures are assembled through `pub(crate)` mutation
   (`put_dock`, `take_dock`, `get_mut`, `HostRegions::assign`, `set_root`) inside
   `#[cfg(test)] mod validation_tests` in `validation.rs`, plus the test-only
   `SplitSizes::from_weights_unchecked`. The `serde_import` sub-module proves the same
   rules fire for **publicly-reachable imported state** (raw JSON → `from_str` →
   `validate`), which is validation's production raison d'être (Ph.10 restoration).
7. **Determinism.** Sorted-id scan order for reporting (HashMap iteration order is not
   stable); neighbour order inside a dock is the fixed canonical edge order. Cycle
   witness is the first gray-on-stack node reached in DFS-from-smallest-unvisited order.
8. **`validate`/`is_valid` are pure** — asserted by a `validate_does_not_mutate` test.
9. **Precedence tests lock the documented order** — ref-before-placement (combined
   fixture), placement-before-cycle (combined fixture), and the earlier sub-fixtures
   that only pass because of precedence (e.g. import with an infinite weight *and*
   dangling refs reports the ref).

## 3. Completion criteria check (current_state/plan.md Phase 03)

- [x] Valid and invalid model fixtures are covered by tests — external suite
      (`docking_validation_test.rs`, 9 tests) covers constructible rules (empty, created
      layouts, transient `MissingRoot`, public `OrphanDock`, purity, `is_valid`
      coherence); in-crate suite (23 plain + 4 serde fixtures) covers every invariant
      including cycles through host regions and deterministic precedence.
      Existing `docking_test.rs` (14 tests) untouched and green.
- [x] Validation errors identify the violated rule — six new variants with targeted
      payloads; reused variants fire for the identical creation-time rules.
- [x] Validation is usable independently of the UI — headless, no coordinates, pure
      check, works on arbitrary imported state.

Public API surface: unchanged except `DockLayout::{validate, is_valid}` and six new
`DockingError` variants (plan: "public API unchanged except new checks + variants").

## 4. Files touched

```
src/domain_foundation/environments/docking/validation.rs          (new)
src/domain_foundation/environments/docking/error.rs               (+6 variants, +Display arms, doc)
src/domain_foundation/environments/docking/model.rs               (+pub(crate) placement_count)
src/domain_foundation/environments/docking/sizing.rs              (+#[cfg(test)] from_weights_unchecked)
src/domain_foundation/environments/docking/mod.rs                 (pub mod validation + doc line)
tests/domain_foundation_tests/main.rs                             (mod docking_validation_test;)
tests/domain_foundation_tests/docking_validation_test.rs          (new — 9 tests)
```

## 5. Next: Phase 04 — Basic structural operations

Per `current_state/plan.md` Phase 4 + CHECKPOINT §3 "Then (preview)":

1. `operations.rs`: `DockingOperation` / `DockingOutcome` value enums with
   `apply(self, &mut DockLayout) -> Result<DockingOutcome, DockingError>` — in the
   `LayersOperation`/`CanvasOperation` shape.
2. **Outcomes carry full inverse data from day one** (Ph.1 §4.2 R-A1/R-A2): removed
   subtrees in BFS order (`LayersOutcome::LayerRemoved` precedent), previous placements,
   previous sizes, previous active index — Phase 9's `ExecutedDockingOperation` will be
   the second environment with real undo.
3. Operations: create panel/host/split/tab, insert-beside (splits), move, tab
   grouping/moving/reorder/set-active, remove, re-place; candidate-build →
   `validate()` → commit, failure leaves state untouched (PRD OP-01…OP-04).
4. Guard cycles via existing `is_descendant` on candidate placements (checkpoint
   note); **occupied edge region policy: reject-by-default**; center-drop semantics =
   reject unless explicitly defined.
5. Tests: `tests/domain_foundation_tests/docking_operations_test.rs`, registered in
   `main.rs`; keep the Phase 2+3 suites green.
