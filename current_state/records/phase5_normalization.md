# Docking Environment — Phase 5 Record: Normalization and Atomic Mutations

**Phase:** 5 of 12 (`current_state/plan.md` — Normalization and atomic mutations)
**Status:** ✅ Complete — full suite green, serde path green, warning baselines unchanged
**Verification:**
- `cargo test` → **850 app_engine + 467 domain_foundation (integration) + 43 in-crate lib tests
  pass**, 0 failures (docking external total 55: 14 model + 9 validation + 24 operation +
  **8 new normalization tests**; lib docking 43 = 23 validation fixtures + 5 in-crate operation
  tests + **12 in-crate normalization fixture tests — new file**; the "0 passed" empty benches
  harness line is pre-existing)
- `cargo test --features serde` → same suites green (471 / 47 lib); normalization adds no serde
  surface (a maintenance pass, not persisted state)
- `cargo check --features serde` → clean; **10 pre-existing warnings** (same as Phases 2–4
  baseline), none from `docking/`
- `cargo doc --no-deps` → **10 warnings, all pre-existing** Phase‑2/3 leftovers
  (`embed_host.rs`/`model.rs`/`slot.rs`/`mod.rs`); the two private-item doc links introduced
  during this phase were fixed during the sweep (same route as the Phase‑4 record)

---

## 1. What was built

New file `src/domain_foundation/environments/docking/normalization.rs` — the deterministic
structural-cleanup rules as **one rule core + two drivers** — plus the fold-in of Phase 4's
mandatory cascade and pipeline documentation in `operations.rs`.

| File | Established |
|---|---|
| `normalization.rs` | Module doc: the concretized rule table (PRD §8.12), the canonical pipeline (mutate → normalize → validate → commit), the sizing-redistribution policy, determinism/idempotence guarantees. [`DockLayout::normalize`](normalization) — public, undo-free whole-layout pass. [`collapse_cascade`](normalization) — `pub(crate)` rootward chain driver **with undo recording** (moved verbatim from `operations.rs`, generalized to continue past a resolved Root into the survivor). `resolve_below_minimum` — the single shared resolution step: record the parent collection → rewrite the slot (survivor replaces the reference / membership drops) → delete the container into `undo.removed`. `below_minimum` — the rule predicate. [`prune_weight`](normalization) — moved from `operations.rs` (the removal-side weight policy). `#[cfg(test)] normalization_tests` — 12 rule-core fixtures via `pub(crate)` installers |
| `operations.rs` | `collapse_cascade` + `prune_weight` bodies **moved** to `normalization.rs` (call sites unchanged — imported); `record_split`/`record_tab`/`record_region`/`record_floating` made `pub(crate)` (the normalization pass records into the operation's `UndoData` with them); module + `apply`/`detach_dock` docs now state the canonical pipeline order; "Cascade normalization" bullet rewritten to point at the shared pass |
| `mod.rs` | `pub mod normalization;` (placeholder comment removed); owns-bullet added; the "deliberately does not own" normalization placeholder removed |
| `sizing.rs` | Module doc: the Phase‑2–5 scope states the structural transfer baselines (insert copies target weight, removal prunes, collapse keeps the slot weight) with bounds/rescale/resize left for Phase 6 |

### Rule table (PRD §8.12, concretized)

| Container | Steady-state minimum | Below minimum → |
|---|---|---|
| Split | ≥ 2 children | collapses: survivor (first child) inherits the split's slot; zero-child split (defensive, imported state): no survivor |
| Tab group | ≥ 1 member | zero members: removed, nothing inherits |
| Panel | — | leaf; nothing to resolve |
| Embed host | — | no child minimum (PRD §8.8); empty regions/floating are valid steady state |

### External test coverage (`docking_normalization_test.rs`, 8 tests)

Pipeline cleanup through operations: collapse-with-weight-inheritance,
sequential chain collapse (two levels), four-child removal ordering + weight pruning, tab-member
removal ordering + active identity, root-emptying via last tab-member removal. Entry-point
guarantees: `normalize` no-op on valid layouts, idempotence after operations. Atomicity
re-assertion: a failed operation leaves the original state unchanged.

## 2. Decisions made during implementation

1. **One rule core, two drivers.** `normalization.rs` owns every cleanup rule;
   `resolve_below_minimum` is the single rewrite step; `collapse_cascade` (rootward chain with
   undo recording — every operation's cleanup step) and `DockLayout::normalize` (whole-layout
   fixpoint pass, no undo contract) are the only drivers. No rule lives in a third place (PRD
   §17.5 centralization criterion).
2. **Canonical pipeline order fixed:** mutate → normalize → validate → commit. Operations
   mutate the candidate freely, call `collapse_cascade` on the vacated owner chain, then the
   Phase‑3 gate validates, then the swap commits. Candidate→swap atomicity is unchanged from
   Phase 4 (the normalization runs *on the candidate*, so a failed normalization still
   discards everything).
3. **Below-minimum definitions** (table above) with the deliberate **split/tab asymmetry**: a
   split needs ≥ 2 children, a tab group ≥ 1 — a one-member tab group is a legitimate steady
   state (meaningful independent semantics, PRD §8.12 rule 3) and is never unwrapped or
   deleted by normalization (plan criterion: no silent discarding of valid tab groups).
4. **The root participates in the same rules.** A below-minimum root collapses to its
   survivor, so the *designation* never dangles while docks remain, and removing the last dock
   yields the explicit empty layout (PRD §7.4 baseline as settled in Phase 2: the root is the
   current top-level dock, not a permanent structural node). The chain driver now continues
   into the survivor after a Root-slot resolution — a fixpoint correction over the Phase‑4
   loop's shortcut; unreachable from valid candidates, so no operation behavior changed.
5. **Unplaced below-minimum containers are untouched** (and dangling references likewise):
   they are invalid state the Phase‑3 gate rejects; normalization does not guess slots or
   repair non-normalization violations. `normalize` returns `Ok` with such leftovers, and the
   selector skips them so the pass cannot loop.
6. **Ordering guarantee.** A survivor copies into the exact slot reference it inherits, and
   member removals shift only by the removed index (active-tab shift/clamp carried from Phase
   4 into the shared pass) — the relative order of surviving children is preserved everywhere
   (plan criterion, tested at four-child removal and double-collapse levels).
7. **Weight redistribution policy promoted** (PRD §9.2), now owned by `normalization.rs`:
   collapse keeps the parent's slot weight (survivor inherits the area); zero-survivor removal
   prunes the index's weight (`prune_weight` moved here) so survivors keep relative shares
   verbatim — never rescaled. Insertion/fresh-container baselines stay next to the insertion
   flows in `operations.rs`; bounds, rescaling, and resize-gesture semantics remain Phase 6
   (PRD §9.3–§9.4). The `sizing.rs` module doc states the established scope.
8. **`normalize()` records no inverse data.** Undo coverage belongs to operations and their
   outcomes (`UndoData`); the pass is a maintenance entry for imported/restored or
   direct-repair state (Phase 10 reads it). Each step *checks before it mutates*: a mid-pass
   error (`InvalidSizeState` surfaced by `prune_weight` for imported garbage sizing) leaves
   the already-completed steps applied and the failing step's state untouched — deterministic
   and repeatable (tested). Operations remain fully atomic.
9. **Determinism and idempotence as the pass shape.** `normalize` selects the lowest-id
   placed below-minimum container each round (deterministic ascending-id work order, PRD
   §17.2) and walks the collapse chain it reports, repeating the scan until a scan resolves
   nothing — a fixpoint. Repeated calls change nothing (in-crate + external tests). Cost note
   (PRD §17.4): the whole-layout entry point pays the O(n²)-scan price; the operation path
   stays the single rootward chain, so committed results never re-traverse the layout.
10. **Test conventions.** Rule-core fixtures (below-minimum states unreachable via the public
    API) use the sanctioned in-crate surgery: `put_split`/`put_tab_group` region `assign`, and
    `SplitSizes::from_weights_unchecked` for garbage sizing. Layouts carrying `f64::NAN`
    cannot be compared via `PartialEq` (NaN ≠ NaN) — the fixture compares `Debug` formatting
    instead (`prune_failure_propagates_without_rewriting_the_parent`).

## 3. Completion criteria check (current_state/plan.md Phase 5)

- [x] Structural cleanup is deterministic — lowest-id selection + fixed rule order + one
      shared rule core; the fixtures pin the exact resulting states
      (`chain_driver_and_full_pass_agree` additionally pairs the operation driver with the
      whole-layout pass on the same input).
- [x] Normalization never silently discards valid panels or tab groups — the rule table
      removes only below-minimum wrappers; a one-member tab group survives every rule
      (`zero_child_split_in_tab_slot_removed_and_active_adjusted` keeps the group; the
      four-child / tab-member removals keep every valid panel).
- [x] Failed operations do not leave partial mutations — candidate→normalize→validate→swap
      kept; external `failed_operation_leaves_original_state_unchanged` plus in-crate
      mid-pass failure (`prune_failure...`) asserting the failing step never mutates.
- [x] Repeated normalization does not keep changing an already normalized layout —
      `repeated_normalization_is_idempotent` + external
      `normalize_is_idempotent_after_operations` + `normalize_is_a_no_op_on_valid_layouts`.

Phase 2–4 suites all green with zero regressions (numbers at top); the Phase‑4 operations
tests run unchanged against the moved cascade (same names, same behavior).

## 4. Files touched

```
src/domain_foundation/environments/docking/normalization.rs          (new — rule core, 2 drivers,
                                                                     prune_weight, 12 in-crate tests)
src/domain_foundation/environments/docking/operations.rs             (cascade + prune moved out;
                                                                     record_* pub(crate); pipeline docs)
src/domain_foundation/environments/docking/mod.rs                    (pub mod normalization; +
                                                                     doc bullets updated)
src/domain_foundation/environments/docking/sizing.rs                 (module doc: transfer scope)
tests/domain_foundation_tests/main.rs                                (mod docking_normalization_test;)
tests/domain_foundation_tests/docking_normalization_test.rs          (new — 8 tests)
```

## 5. Next: Phase 06 — Sizing and resizing

Per `current_state/plan.md` Phase 6 + PRD §9.2–§9.7:

1. `sizing.rs` decision: whether stored weights are normalized to sum ≈ 1 on write or kept
   verbatim (the doc comment defers it — decide and document; PRD §9.2 MUST define how weights
   are "initialized, updated, normalized, and transferred").
2. Relative resizing: a value-type operation updating two adjacent split-child weights
   (`{env}.{noun}.{action}` naming precedent: `docking.split.resized` is Phase‑9's event);
   derive updates from the gesture's total displacement, not accumulated deltas (PRD §9.4);
   outcome carries the previous weights (undo precedent: `CanvasOutcome::Moved`).
3. Bounds (PRD §9.3): min/max logical constraints for panels/containers/regions —
   conflict policy must reject or apply a *defined* constraint policy, never silently produce
   invalid sizing (`InvalidSizeState` is the steady-state authority; a bounds violation is an
   operation rejection, not a validation fix).
4. Collapse state (PRD §9.7): persistent logical collapse for eligible docks with restore
   state; interacts with normalization (a collapsed-but-present member still owns its slot —
   never unreachable).
5. Multi-child containers: define which adjacent pair a resize touches and ≥2-children
   behavior (PRD §9.4 bullets).
6. Tests: `tests/domain_foundation_tests/docking_sizing_test.rs` (register in `main.rs`);
   in-crate fixtures for bound-violating states via `from_weights_unchecked` if needed.
