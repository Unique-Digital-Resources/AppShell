# Docking Environment — Phase 6 Record: Sizing and Resizing

**Phase:** 6 of 12 (`current_state/plan.md` — Sizing and resizing)
**Status:** ✅ Complete — full suite green, serde path green, warning baselines unchanged
**Verification:**
- `cargo test` → **850 app_engine + 483 domain_foundation (integration) + 47 in-crate lib tests
  pass**, 0 failures (docking external total **71**: 14 model + 9 validation + 24 operation +
  8 normalization + **16 new sizing tests**; lib docking 47 = 23 validation fixtures + **8
  in-crate operation tests** (+3: two undo round-trips, one imported-garbage determinism) +
  **13 in-crate normalization fixtures** (+1: collapsed-member rule) + 4 serde-gated import
  fixtures, plus the small non-docking in-crate set; the "0 passed" empty benches harness
  line is pre-existing)
- `cargo test --features serde` → same suites green (**850 / 487 / 51 lib**); the serde gain
  is the additive `Panel.collapsed` field with `serde(default)` — JSON without the flag
  restores expanded (the existing §7.7 import fixtures pass unchanged)
- `cargo check --features serde` → clean; **10 pre-existing warnings**, none from `docking/`
- `cargo doc --no-deps` → **9 warnings, all pre-existing** Phase‑2/3 leftovers
  (`embed_host.rs`/`model.rs`×2/`slot.rs`/docking `mod.rs` + relations/`node_graph`); none
  from the Phase‑6 changes

---

## 1. What was built

Sizing and resizing (PRD §9.1–§9.4, §9.7): the stored-weight representation decision, a
pair-conserving splitter-resize operation with an operation-translated rejection policy, and
panel collapse/restore as persistent logical state.

| File | Established |
|---|---|
| `sizing.rs` | The **stored-representation decision** in the module doc (PRD §9.2's "initialized / updated / normalized / transferred" table): weights are stored **verbatim and never renormalized** — one finite, strictly positive weight per child is the only representation invariant; the rendering proportion `weightᵢ / Σweightⱼ` is a derived read, never stored |
| `operations.rs` | `DockingOperation::ResizeSiblings` + `apply_resize_siblings` — the pair-conserving resize with rejection-policy bounds; `DockingOperation::SetDockCollapsed` + `apply_set_dock_collapsed` — the panel collapse preference (no-op success when the value matches); outcomes `DockingOutcome::SiblingsResized { container, first, previous, inverse }` and `DockingOutcome::DockCollapsedSet { dock, collapsed, previous, inverse }`; `FormerCollection::Panel` undo record; module-doc gesture contract + non-structural-ops-skip-the-cascade note; `#[cfg(test)]` — two undo round-trips + the imported-garbage determinism fixture |
| `error.rs` | Three variants: `NotASplit { id }` (the `NotATabGroup` mirror), `SplitChildIndexOutOfRange { index, child_count }` (the pair-start argument echo), `SizeBoundViolation { split, index }` (the rejection policy; `index` names the ascending-first violating region of the pair) |
| `dock.rs` | `DockRole::Panel { content, collapsed: bool }` — the persistent collapse preference (PRD §9.7); `serde(default)` on the flag; the `collapsed()` read accessor (`None` for non-panels) |
| `model.rs` | `create_panel` starts panels expanded; `restore_collection` gained the `Panel` arm (undo replay) |
| `normalization.rs` | Docs: a collapsed panel is an **ordinary member** — the flag changes no placement or sizing data, it counts toward its container's minimum, and no rule rewrites it; the resize cross-reference (conservation, no rescaling); `#[cfg(test)]` collapsed-member-through-cascade fixture |
| `validation.rs` | Doc note: the collapse preference is deliberately non-structural and skipped — every collapsed panel is as valid as its expanded self |
| `mod.rs` | Owns-bullets updated (sizing representation, resize + collapse operations) |

### External test coverage (`docking_sizing_test.rs`, 16 tests)

Pair policy (two-child weights + outcome fields + inverse snapshot; four-child conservation
with verbatim untouched regions); the drift-free gesture contract (frames compose from the
gesture's initial state; stalled frames are no-ops; frame count irrelevant); the constraint
policy (below-minimum rejection + atomicity, inclusive bound endpoints, complement-math
second-region minimum, invalid values 0/negative/NaN/∞, deterministic ascending check order,
the infeasible-total case); preconditions (missing dock / not-a-split / pair-out-of-range);
collapse (toggle without touching allocation, exact restore, no-op success with identity
inverse, panels-only eligibility, never-unreachable through a removal cascade, live-allocation
retention across a resize-during-collapse); coherence (resize + insert-beside movement).
Imports use `serde(default)`-compatible layouts only; imported garbage sizing is exercised
in-crate via `SplitSizes::from_weights_unchecked`.

## 2. Decisions made during implementation

1. **Stored weights are verbatim — never renormalized** (PRD §9.2 representation decision,
   the Phase‑6 item the `sizing.rs` doc deferred). The codebase already treats the sum as
   free: `uniform(n)` sums to `n`, the insert baseline copies the target's weight (growing
   the sum), and Phase 5's `prune_weight` removes without rescaling. A sum ≈ 1 constraint
   would invalidate those settled policies and churn every untouched weight on each write.
   The normalized proportion is the derived read `weightᵢ / Σweightⱼ`, documented as
   frontend-side; write-time rescaling would also make undo/redo comparisons drift.
2. **Resize identifies the affected regions and calculates preferences by pair
   conservation** (PRD §9.4): the adjacent pair `children[first]` / `children[first + 1]` is
   fully rewritten — first := `requested`, second := pair total − `requested` — so the
   pair's total (and therefore every other child's share) is conserved; untouched children
   stay verbatim. The operation changes no membership, hence no below-minimum container can
   appear and the normalization cascade step is skipped (like the other non-structural
   operations).
3. **Gesture contract: absolute, drift-free frames.** The frontend measures pointer movement
   and sends, per frame, the pair-first region's *absolute* requested weight derived from the
   gesture's initial state and the total displacement so far (PRD §9.4 SHOULD). Frames
   therefore compose without accumulating rounded deltas (tested: repeated frames equal a
   single application). Cancelling the gesture is frontend interaction state (Phase 11);
   frames already applied roll back through the outcome's inverse data like any operation.
4. **Minimum bounds are operation-carried, not model state** (PRD §9.3 sanctions "a
   consistent operation or layout policy"): `first_min` / `second_min` are logical weights
   translated by the frontend from its available geometry. v1 supports minimums (the §9.3
   SHOULD); maxima remain a MAY with the identical extension path (same policy, same interval
   math). Bound endpoints are honored inclusively.
5. **The constraint policy is rejection — never clamping, never silent invalid sizing**
   (PRD §9.3). `SizeBoundViolation { split, index }` names the first-checked violating region
   of the pair (ascending child order — deterministic per PRD §17.2). The sizing invariant
   (finite, strictly positive) is always the implicit bound, so 0 / negative / NaN / ∞
   requests reject without carried bounds; a NaN/∞ carried minimum is unsatisfiable like any
   other bound. When both regions violate, the first checked region is named; a total
   insufficient for both minimums (§9.4 bullet) rejects every request the same way.
6. **Imported-garbage determinism.** A stored pair that cannot produce a finite complement
   rejects through the bound path; stored garbage *outside* the pair is named by the
   steady-state authority (`InvalidSizeState`, the `prune_weight` precedent) — the candidate
   is discarded atomically either way, mirroring how the insert flows funnel stored garbage
   to the Phase‑3 gate. Fixtured in-crate (Debug-format compares, NaN safety).
7. **Collapse = panels only, one persistent flag on the dock payload** (PRD §9.7). v1
   eligible category: panels (the dockable leaves; containers do not collapse). The flag is
   presentation meaning for the frontend: toggling changes **no placement, membership, or
   sizing data**, so *the dock's current weight is the retained restore state* and restoring
   is the flag clearing — versus, e.g., zeroing the member's weight, which the strictly
   positive invariant forbids and rescaling would violate decision 1. A resize applied while
   collapsed therefore changes what the restore brings back (documented, tested) — the
   environment never snapshot-allocates.
8. **Collapse never makes a dock unreachable** (PRD §9.7): the dock stays placed, counts
   toward its container's membership minimum, traverses normally (`descendants`, validation
   skips the flag), and survives cascade rewrites with the flag untouched (in-crate fixture
   through a one-child split; external test through a removal cascade).
9. **`SetDockCollapsed` mirrors the established operation style**: `IneligibleDock` for
   non-panels, matching-value sets are no-op successes with an identity inverse (the
   `ReorderTab` `from == to` precedent), and the flip is recorded as a new
   `FormerCollection::Panel { id, collapsed }` record so undo replay (and Phase 9) restores
   the exact pre-op preference.
10. **Scope routing of the remaining §9 items** (plan Phase 6 tasks cover the split-sizing
    baseline): §9.5 embedded-region sizing preferences and their coherent resizing route to
    **Phase 7** (they need the host lifecycle they persist with), §9.6 floating sizing to
    **Phase 8**, and §9.8 shared-junction resizing stays deferred by the PRD. All three are
    documented in the `sizing.rs` module doc so the deferral is explicit, not silent.

## 3. Completion criteria check (current_state/plan.md Phase 6)

- [x] Resizing preserves valid allocation data — pair conservation, invariant-checked
      writes, verbatim untouched regions; undo round-trips restore exactly (in-crate
      `resize_pair_round_trips_through_undo` / `collapsed_preference_round_trips_through_undo`,
      external movement test `resize_survives_structural_movement_of_the_pair`).
- [x] Constraints are handled predictably — the rejection policy with a typed error naming
      the deterministic violating region; inclusive endpoints; per-region minimums verified
      through the complement math; infeasible totals reject consistently
      (`resize_names_the_first_checked_region_deterministically`).
- [x] Layout changes do not produce invalid sizing values — every resize write passes the
      sizing invariant by construction; rejection paths leave the layout untouched;
      imported garbage states are rejected deterministically in both garbage positions
      (in-crate fixture); resize cannot create below-minimum structure.
- [x] The model remains independent of pixels and frontend rendering — logical weights
      only; pointer/geometry conversion and the absolute-request computation stay with the
      frontend (documented in the operation contract); the collapse flag is presentation
      meaning; no browser measurement anywhere.
- [x] Sizing after insertion/removal/movement stays coherent with the Phase‑5 policy — the
      resize adds to (never rewrites) the established baselines: collapse keeps the slot
      weight, removal prunes, insertion copies the target's weight, and resize touches only
      its pair (`resize_during_collapse_retains_the_live_allocation`,
      `resize_survives_structural_movement_of_the_pair`).

Phase 2–5 suites all green with zero regressions (numbers at top); operation outcomes and
undo coverage extend uniformly (`inverse()` arms added to the shared match).

## 4. Files touched

```
src/domain_foundation/environments/docking/sizing.rs                  (representation policy docs)
src/domain_foundation/environments/docking/operations.rs              (2 ops + 2 outcomes + apply fns +
                                                                      FormerCollection::Panel + docs +
                                                                      3 in-crate tests)
src/domain_foundation/environments/docking/error.rs                   (3 new variants + Display)
src/domain_foundation/environments/docking/dock.rs                    (Panel.collapsed field + accessor)
src/domain_foundation/environments/docking/model.rs                   (create_panel default;
                                                                      restore_collection Panel arm)
src/domain_foundation/environments/docking/normalization.rs           (collapsed-member docs + 1 in-crate test)
src/domain_foundation/environments/docking/validation.rs              (collapse-preference doc note)
src/domain_foundation/environments/docking/mod.rs                     (doc bullets)
tests/domain_foundation_tests/docking_sizing_test.rs                  (new — 16 tests)
tests/domain_foundation_tests/main.rs                                 (mod docking_sizing_test;)
```

## 5. Next: Phase 07 — Embedding and host lifecycle

Per `current_state/plan.md` Phase 7 + PRD §8.8, §8.9, §9.5 + the Phase-6 routing:

1. Host identity and lifecycle rules: the relationship between a host and its embedded
   structures — validators, transient import semantics, promote/demote or reconstruct
   (PRD §8.9), host content preserved per the lifecycle rules.
2. Valid host slots and occupancy: define embedding/removal operations (v1 placement
   categories exist — `HostEdge` / `Floating` slots are modeled but publicly unreachable);
   occupied-region policy stays reject-by-default unless an operation explicitly defines the
   handling (§8.6 settled baseline).
3. Host promotion/demotion: preserve intended identities and content when a host's own work
   area joins or leaves a broader arrangement.
4. Removal outcomes for embedded structures (host removed → what happens to its placements).
5. **Routed from Phase 6** — PRD §9.5: persistent sizing preferences for occupied host
   regions, and a coherent region-resize path that does not invalidate the host's other
   placements (weights live with the region lifecycle; decide host-region interaction with
   left/right vs top/bottom allocation).
6. Tests: `tests/domain_foundation_tests/docking_embedding_test.rs` (register in `main.rs`);
   in-crate fixtures via the established region/floating installer routes.

**Then (preview):** Ph.8 floating docks (PRD §8.11/§9.6 — floating-root representation,
position/size state, re-docking, host interactions), Ph.9 Domain Engine integration
(via `docking_command_handler.rs` per plan + the `docking.split.resized` event surface),
Ph.10 persistence/history (weights, collapse flags, floating position state are the new
serialized payload), Ph.11 interaction contract, Ph.12 acceptance.
