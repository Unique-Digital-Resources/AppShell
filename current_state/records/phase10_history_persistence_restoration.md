# Docking Environment — Phase 10 Record: History, Persistence, and Restoration

**Phase:** 10 of 12 (`current_state/plan.md` — History, persistence, and restoration)
**Status:** ✅ Complete — full suite green, serde path green, warning/doc baselines unchanged
**Verification:**
- `cargo test` → **871 app_engine (+4 Phase-10 registry tests) + 510 domain_foundation
  (integration) + 66 in-crate lib tests pass**, 0 failures. Docking external coverage
  unchanged from Phase 8 (98 external docking tests in `domain_foundation_tests`); the
  docking engine tests grow 867 → 871 (`docking_history_registry_test.rs`), and docking
  persistence tests are serde-gated (4 tests in `docking_persistence_test.rs`, below).
- `cargo test --features serde` → **875 app_engine (871 + 4 persistence) + 516 domain_foundation
  (510 + 6 serde round-trips incl. the 2 new docking ones) + 73 lib pass**, 0 failures.
- `cargo check` / `cargo check --features serde` → clean; **10 pre-existing warnings** on both
  paths (the `-cE "^warning"` count of 11 includes the summary line), none from the
  Phase-10 files (`domain_engine/mod.rs` edits are the timeline-import group untouched).
- `cargo doc --no-deps` → **9 warnings, all pre-existing** Phase-2/3 leftovers; the two
  Phase-10 draft warnings (an intra-doc link to the private `ExecutedStep`, and a
  `ContentRef` link from the restore doc) were fixed before recording.

---

## 1. What was built

Phase 10 delivers the history-integration unification and the persistence/restoration
boundary without introducing any docking-specific framework (the plan's completion
criterion). Two mechanisms land:

| File | Established |
|---|---|
| `src/domain_engine/mod.rs` | **One position-indexed executed-step registry** replacing the per-environment parallel vectors: `executed_steps: Vec<Option<ExecutedStep>>` with private `enum ExecutedStep { Canvas(ExecutedCanvasOperation), Docking(ExecutedDockingOperation) }` riding both wrappers unchanged — *the Phase-9 record's R-9-1 recommendation, compiled into the engine*. `execute_canvas` / `execute_docking` truncate the registry to the history position and `push_executed_step(pos, step)` pads `None` gaps for name-only positions then inserts the step at its history position; `undo()` / `redo()` / `rollback_transaction()` walk history positions once and dispatch on the step (`None` gaps skip). The canvas wrapper itself is untouched. **`restore_docking(&mut self, layout)`** — the trusted-adopt path: validates the submitted model with `DockLayout::validate()` *first* (the Phase-3 gate), declines invalid layouts wholesale, adopts a valid one verbatim, appends the **non-undoable** `"docking.restored"` history entry, `mark_saved()`s, and publishes `"docking.restored"` (event + signal). Doc header states the persisted-form convention. |
| `tests/app_engine_tests/docking_history_registry_test.rs` | **New — 4 tests** (ungated, no serde dependency): interleaved canvas/docking undo/redo replays in recorded order with id preservation on both sides (the misalignment regression pin), canvas-only undo/redo stays aligned after the loop rewrite, a committed transaction's mixed canvas+docking span undoes/redoes as one unit through the registry dispatch, and a rejected interleaved operation keeps the registry aligned for the surrounding steps. |
| `tests/app_engine_tests/docking_persistence_test.rs` | **New — 4 tests, `#[cfg(feature = "serde")]``-gated in the suite's `main.rs`** (the serde-gated `mod` precedent, `serde_roundtrip_test.rs`): restore into a fresh engine is id-exact + valid across hosts/regions/floating/weights (with the non-undoable entry, `mark_saved`, and event assertions; post-restore mutations undo/redo cleanly against the adopt via the public-API `structural_equal` replica — the id counter is monotone and intentionally not compared), invalid persisted state is declined wholesale with the first deterministic Phase-3 error and untouched engine state/history/events, malformed persisted state is rejected in two layers (caller-side parse for shape violations; the engine gate for a mechanically-parseable root naming no dock), and a saved layout round-trips through a `StorageBackend` key-value entry (the no-docking-specific-framework criterion). |
| `tests/domain_foundation_tests/serde_roundtrip_test.rs` | **+2 tests**: `docking_roundtrip` (rich layout built through the public operation API — root panel, split insert with non-uniform stored weights, tab join, collapse flag — validated, round-tripped, with the `"next_id"` carry asserted through continued *symmetric mutation equality*: the shipped counter issues the same fresh ids on both sides) and `docking_empty_layout_roundtrip` (the valid explicit-empty representation, PRD §7.4/§8.3). |
| `tests/app_engine_tests/main.rs` | Registered `docking_history_registry_test` (ungated) + `docking_persistence_test` (`#[cfg(feature = "serde")]`). |
| `agents/engine_integration.md` | **Stateless conventions update**: requirement 2 rewritten (one registry variant, never a parallel vector), the "Alignment caveat" section replaced by the settled Phase-10 registry statement, and a new "Persistence and restoration recipe" section (serde-derive persisted form; validate-before-adopt; ids + counter ride inside the form → undo comparisons are structural, not `PartialEq`; history persistence is metadata-only riding `HistoryStore::persist_entry` + `StorageBackend`). Transactions section wording refreshed (registry dispatch; canvas and docking). |

## 2. Decisions made during implementation (do not relitigate)

1. **The persisted form is the model's own serde derive — nothing more, nothing less**
   (the Phase-1 §7.2 id-policy resolution, landed): ids and the id counter **ship
   verbatim** inside the whole-`DockLayout` form; they are *not* re-issued. The
   warehouse decision rides the established serialization convention: every sibling
   environment ships derive-only serde with round-trip tests, `serde_json` stays a
   dev-dependency, and the engine never gains a format-specific save API — saving is
   "clone the model and serialize it" at the host layer. Consequences pinned by tests:
   a restored layout is collision-free by construction (DM-02/DM-04),
   continued mutation after restore issues fresh ids (the counter carried),
   and `ContentRef` rides as an opaque string — the host application stays
   responsible for restoring content (PRD §12.4).
2. **`restore_docking` validates before it adopts — rejection, never repair**
   (PRD §12.3, "prefer rejection with a meaningful error"): the Phase-3 gate
   (`DockLayout::validate`) *is* the imported-state report — first error,
   deterministic ascending-id scan, every placement category included (region-weight
   and floating-rect steady-state rules already carry the restore-compat fixtures).
   The gate runs on the submitted model **before** any engine mutation: a rejection
   leaves layout, history, and event counts untouched (pinned). v1 defines **no**
   missing-/outdated-reference recovery or migration: an unavailable panel or host is
   exactly an invalid reference to the gate (`UnknownReference` / `InvalidRoot` /
   `MissingRoot` per the first-rule precedence).
3. **No version envelope in v1** (PRD §12.2, the defined compatibility strategy):
   the form's own restore-compat design is the strategy — `serde(default)` fields
   restore absent optional values at their settled baselines (region weights,
   persistent hosts, the collapse flag, floating rects), serde tolerates unknown
   fields, shape violations fail the caller-side parse (required fields, non-dock
   junk — `serde_json` is the boundary), and anything structurally inside-but-wrong
   fails the engine's validate gate. No schema-version constant, no migration path;
   a future version can extend this contract without changing it.
4. **A restore is a trusted import, not a reversible operation**: the adopt appends
   a **non-undoable** `"docking.restored"` entry (name-only like the plain-apply
   family) and calls `mark_saved()` after appending — the restored file anchors the
   saved position (`is_dirty()` false post-restore), and the adopt publishes
   `"docking.restored"` (event + signal). The registry slot stays `None`.
   Post-restore docking operations replay against the restored layout and undo/redo
   cleanly; history recorded *before* a restore describes the pre-restore session,
   and an undo whose replay would drift out of validity surfaces
   `DomainError::Docking` from the existing Phase-9 replay-validate guard. The
   natural consumer posture (and the tested shape) is restore at document-open.
5. **R-9-1 is resolved by unification, as the Phase-9 record planned**: the engine's
   per-env executed vectors (`executed_canvas`, `executed_docking`) were
   position-aligned only while their environment's operations were the only
   real-undo history; the new single registry stores `Some(step)` exactly at the
   history position of each replayable entry and `None` gaps elsewhere, and the
   undo/redo/rollback loops walk positions once. This fixes the pre-existing
   canvas-under-layers misalignment as a side effect (intersection of the fix, not a
   docking special case) — pinned by the interleaved-history tests. The wrappers
   (`ExecutedCanvasOperation`, `ExecutedDockingOperation`) are untouched; future
   real-undo environments add one registry variant, never a vector.
6. **R-9-2 stays a documented no-op** (decided deliberately, not resolved):
   `enforce_history_limit` remains a no-op mirror of `execute_canvas`. Real trimming
   would need `HistoryStore` truncation semantics **plus** registry-offset handling
   (a trimmed entry shifts every later position), and no consumer sets a limit
   today. Making it real is its own decision when something consumes `with_max_history`.
7. **How the history system records environment changes for persistence** (plan
   task 1, the determination): `HistoryStore::persist_entry` writes entry *metadata
   only* (operation/source/undoable/tx pipe format) through the optional
   `StorageBackend`; inverse/replay data stays in-memory. Full-state persistence
   therefore rides each environment's persisted form with `StorageBackend`
   key-value entries (`"docking.layout"` in the tests) — the existing
   `app_engine/persistence/` + `serialization/` conventions with no docking-specific
   framework. `restore_docking`'s validate-before-adopt is what makes the imported
   state trustworthy on the way back.
8. **Undo round-trip comparisons after restore are structural, not `PartialEq`**
   (teaching the next agent): `DockLayout: PartialEq` includes `next_id`, and the
   counter is **monotone** — undoing a created dock leaves it incremented (the
   `Canvas::insert_object` convention, settled Phase 4). The Phase-4
   `structural_equal` in-crate helper can't be imported by external tests; the
   persistence suite ships a public-API replica (`iter()` + root, counter excluded)
   named accordingly. Pre-op post-restore equality (`PartialEq`) still holds and is
   asserted, because nothing allocated between adopt and undo.

## 3. Completion criteria check (current_state/plan.md Phase 10)

- [x] **Docking changes can be undone and redone using the existing history
      system** — the Phase-9 wrapper already provided real undo/redo/transactions
      (UR-01…UR-05); Phase 10 unified the replay into the engine's one
      position-indexed registry and pinned interleaved replay with id preservation
      across canvas+docking (the plan's "history integration" and the R-9-1
      resolution), with no third vector anywhere.
- [x] **A saved layout can be restored into an equivalent valid domain state** —
      `restore_docking` adopts a validated layout id-exactly (fresh-engine
      equivalence + `StorageBackend` + `mark_saved` pinned by tests); the persisted
      form represents logical layout state (ids, categories, relationships and
      ordering, active tab, region and floating placements, persistent weights,
      collapse) with no pixel geometry (PRD §12.1).
- [x] **Missing or invalid references are handled according to explicit rules** —
      PRD §12.3's rejection-with-error policy through the Phase-3 gate (first
      deterministic error, engine state untouched) at the engine level, plus the
      two-layer boundary (parse vs. validate); in-crate Phase-3 `serde_import`
      fixtures already pin the model-level report.
- [x] **No docking-specific history or persistence framework is introduced** — the
      registry is environment-generic; the persisted form is the model's own serde
      derive; storage rides `StorageBackend`; `serde_json` remains a dev-dependency;
      no new `DomainError` variant, feature, or dependency was needed.

## 4. Files touched

```
src/domain_engine/mod.rs                                       (registry enum + fields/loops rewritten;
                                                                restore_docking + push_executed_step; docs)
tests/app_engine_tests/docking_history_registry_test.rs        (new — 4 tests, R-9-1 unification)
tests/app_engine_tests/docking_persistence_test.rs             (new — 4 tests, serde-gated)
tests/app_engine_tests/main.rs                                 (2 registrations + serde gate)
tests/domain_foundation_tests/serde_roundtrip_test.rs          (+2 docking round-trip tests)
agents/engine_integration.md                                   (stateless conventions: registry + persistence recipe)
```

The docking environment itself (`domain_foundation/environments/docking/`) is
**untouched in Phase 10** — its serde surface was complete since Phases 1–8 and the
Phase-3 gate already defines the imported-state report.

## 5. Next: Phase 11 — Interaction contract and frontend handoff

Per `current_state/plan.md` Phase 11 (the interaction contract):

1. **Specify the contract, don't implement UI**: begin/cancel a drag; identify the
   dragged element or subtree; represent a proposed drop target; express
   top/bottom/left/right/center intent; distinguish split, tab-group, and embedding
   operations; determine eligibility; commit the confirmed operation; report
   validation failures to the frontend.
2. **The environment stays headless** (PRD §10, §11.2): hit-testing, pointer
   tracking, overlays, previews, animations, windowing remain frontend concerns;
   the domain knows *intent + eligibility + commit*, never coordinates.
3. **A separate interaction/drag-session file only if the domain API genuinely
   needs persistent interaction state** — the plan warns; pure visual/preview state
   must not become committed layout state (and must not be persisted — Phase 10's
   `restore_docking` gate would reject anything transient sneaking into the form).
4. Eligibility answers today: region drops on split/tab-group targets are
   ineligible (`IneligibleDock`), occupied regions reject (`RegionOccupied`),
   directional insertion into a floating dock rejects (`NotSplittable`),
   host-in-source cycles reject (`CycleDetected`) — the phase documents these as
   the contract rather than paving new policies.
5. Reading list for Phase 11: this record, `operations.rs` + `slot.rs` + `error.rs`
   (the eligibility surfaces), `docking_command_handler.rs` (the frontend-facing
   bridge), PRD §10/§11 — **not** the sizing/embed_host internals (settled).
