# Docking Environment — Phase 9 Record: Domain Engine Integration

**Phase:** 9 of 12 (`current_state/plan.md` — Domain Engine integration)
**Status:** ✅ Complete — full suite green, serde path green, warning baselines unchanged
**Verification:**
- `cargo test` → **867 app_engine (+17 new docking engine tests) + 510 domain_foundation
  (integration) + 66 in-crate lib tests pass**, 0 failures (suites unchanged otherwise; the
  external docking coverage through the public API is exactly the Phase-8 state — 98 external
  docking tests)
- `cargo test --features serde` → same suites green (**867 / 514 / 73 lib**); no new
  serde-gated surface on the engine side (the wrapper follows `ExecutedCanvasOperation` —
  `Debug, Clone`, no serde derive; persistence rides Phase 10)
- `cargo check` / `cargo check --features serde` → clean; **10 pre-existing warnings**, none
  from the Phase-9 files (`docking_operations.rs`, `docking_command_handler.rs`, engine `mod.rs`
  additions)
- `cargo doc --no-deps` → **9 warnings, all pre-existing** Phase-2/3/7 leftovers (the one new
  draft warning — an intra-doc link to the private `redo_record` — was fixed before recording)

---

## 1. What was built

Docking wired into the Domain Engine following the established environment conventions
(Phase-1 §4.2 R-A1/R-A2 plan): engine-owned model, one execute entry, a real-undo wrapper per
outcome, error mapping, event/signal surfaces, and the app-engine command bridge. Callers
cannot bypass validation — execution goes exclusively through the environment's operation
pipeline (mutate → normalize → validate → commit); the model's mutators stay `pub(crate)`.

| File | Established |
|---|---|
| `domain_engine/operations/docking_operations.rs` | **New — `ExecutedDockingOperation`** (the `ExecutedCanvasOperation` precedent): holds `(operation, outcome, redo_record)`. `execute(layout, op)` runs the op once through `DockingOperation::apply` and computes the **forward replay record** from the committed post-state (see §2 decision 2). `undo` replays `outcome.inverse().clone().replay_into(layout)`; `redo` replays the forward record through the same driver; both validate the restored layout afterwards (`DockLayout::validate`). `history_entry()` names all 13 operations (`docking.create_panel` … `docking.set_floating_rect`). Accessors `outcome()` / `operation()`. |
| `domain_engine/operations/docking_command_handler.rs` | **New — `DockingCommandHandler`** (the layers-handler compressed template): holds `Arc<Mutex<DockLayout>>`, extracts `DockingOperation` from `CommandInput`, applies it, appends `HistoryEntry::new("docking.operation").undoable()` to `engine.history_store`, publishes `"docking.operation"` on `engine.event_bus`, returns `CommandResult::success_with(outcome)`. Definition `"docking.operation"` / `"Docking Operation"`. |
| `domain_engine/operations/mod.rs` | Registered both modules + re-exports (`DockingCommandHandler`, `ExecutedDockingOperation`) beside the canvas/layers/timeline entries. |
| `domain_engine/mod.rs` | `DomainError::Docking(DockingError)` + `From` + Display arm `"docking: {e}"`; engine **owns one `DockLayout` instance** (`docking: DockLayout`) with the parallel `executed_docking: Vec<ExecutedDockingOperation>`; `docking()` accessor; **`execute_docking`** (the `execute_canvas` shape 1:1: truncate → execute → tag entry with the active transaction → append → push → `enforce_history_limit` → publish); **undo() / redo() / rollback_transaction()** gain their index-aligned docking replay branches beside the canvas ones (added only — the canvas path is untouched, R-A1); bottom-of-file helpers `docking_outcome_event` (13 arms) + `docking_outcome_signal` (specifics incl. `docking.split.resized` — the plan-named shape — wildcard `"docking.changed"` for the rest). Doc header lists Docking. |
| `tests/app_engine_tests/docking_undo_redo_test.rs` | **New — 17 tests** (registered in that suite's `main.rs`), public API only: execution basics (state + history + event; rejection keeps state/history/events untouched; error surfaces as `DomainError::Docking`), undo/redo (create, insert with **id-preserving redo**, idempotent cycles with stable sizing, tab join, removal cascade, embed promotion/demotion round-trip, region-weight restore, floating-rect restore, collapse no-op entries, undo/redo events), transactions (failure auto-rollback reverts the created dock; committed transaction undoes/redoes as one unit), and the command handler (definition + shared-model application, canvas-test shape). |

### Event surface (`docking_outcome_event` / `docking_outcome_signal`)

`{env}.{noun}.{action}` recipe, snake actions (the `canvas.object.moved_to_front` style):
`docking.dock.created`, `docking.dock.inserted_beside`, `docking.tab.joined`,
`docking.tab.reordered`, `docking.tab.active_selected`, `docking.dock.removed`,
**`docking.split.resized`** (the plan's named shape), `docking.panel.collapsed_set`,
`docking.dock.embedded`, `docking.host.promoted`, `docking.region.resized`,
`docking.floating.rect_set`. Signals map the same specifics the frontend drives live state
from (created/removed/resizes/rect-set) and fall back to `"docking.changed"` otherwise.

## 2. Decisions made during implementation (do not relitigate)

1. **Docking is the second real-undo environment, wired as a parallel vector per the
   R-A1 mitigation** (Phase-1 §8): `executed_docking: Vec<ExecutedDockingOperation>` with
   branches added in `undo()`, `redo()`, and `rollback_transaction()`, index-aligned with
   history positions exactly like the canvas loop, *added only* — the canvas replay path is
   untouched. Outcomes already carry complete inverse data from day one (R-A2 satisfied:
   no model-snapshot fallback anywhere).
2. **Redo is a forward replay record, not a re-apply of the operation** (the key Phase-9
   decision): re-applying `DockingOperation` after an exact undo is **not**
   id-preserving — `DockId`s are counter-issued, monotone, never renumbered
   (Phase-1 §7.2), so every id-allocating operation (`CreatePanel`/`CreateEmbedHost`,
   `InsertSource::NewPanel` sources, wrap containers in `InsertBeside`/`TabJoin`/`Embed`)
   would allocate fresh ids and diverge from the ids recorded in the surrounding history
   (a later entry's `Remove { dock: id7 }` would fail after such a redo). The canvas
   wrapper's `Create` special case (redo re-inserts the *recorded* object) generalizes to
   docking as a **forward record**: `UndoData` — the environment's own record vocabulary —
   built at execution time from the committed post-state such that
   `redo_record.replay_into(layout)` restores the post state exactly:
   - `removed` = full post payloads of the docks the forward replay must re-place — the
     operation's *created* docks plus docks whose category swapped in place
     (promotions/demotions). Whole-payload placement is what a collection snapshot cannot
     express for a promoted host (the payload carries regions, floating collection, and the
     persistence flag verbatim).
   - `containers` = post snapshots of every collection the inverse names (the mirror of
     each record); deleted docks have no post state and skip. An empty post region records
     `(None, None)`, which the replay's clearing path resolves at the embedding baseline —
     the environment's own cleared-region lifecycle.
   - `previous_root` = the post root; `created` = the ids the operation removed, deleted
     again by the forward replay.
   The record is order-commuting (every entry carries a final value), so it feeds
   `replay_into`'s load-bearing order unchanged. `replay_into` is thereby used symmetrically
   as a generic state-restore driver: inverse → undo, forward record → redo.
3. **Undo and redo validate the restored layout** (`DockLayout::validate`): the exact
   replay of a committed state is valid by construction, so validation is cheap insurance —
   any replay drift surfaces as `DomainError::Docking` instead of propagating invalid state.
   Each intermediate state of a multi-entry undo/redo is a historical committed state, also
   valid.
4. **`execute_docking` mirrors `execute_canvas` exactly** (truncate the executed vec to the
   history position before applying — the redo-tail flush; wrapper supplies the history
   entry name; transaction tagging via `with_transaction`; `enforce_history_limit`;
   publish(event, signal)). The other environments' plain-apply shape (`execute_layers`) is
   *not* the model here because docking carries real inverse data.
5. **Event naming follows the established recipe** with per-noun.Docking nouns
   (dock/tab/split/region/floating/panel/host); `docking.split.resized` is the plan's named
   shape and is asserted in tests; the signal fn wildcards `"docking.changed"` on the
   low-signal outcomes (the canvas/layers signal precedent).
6. **The command handler exists because the plan and checkpoint name it as this phase's
   deliverable** (the per-environment bridge next to its three siblings), using the
   layers/timeline compressed naming style (`"docking.operation"`). Known caveat, consistent
   with the canvas/layers/timeline handlers: it appends an `.undoable()` entry on the
   app-engine-side store without an engine-side replay mechanism (the `agents/app_engine.md`
   "lie" pattern — inherited from the existing handlers, not made a docking-specific
   policy); the `DomainEngine::execute_docking` + wrapper path is the real-undo route.
7. **Discovered risks** (recorded, not fixed — both belong to Phase-10 history work):
   - **R-9-1 (inherited latent limitation):** the engine's per-env executed vectors are
     position-aligned only when the history contains just that environment's operations
     (`executed_canvas[i] ↔ history position i` holds because canvas was the only real-undo
     env today). Name-only entries from the other environments (layers, node graph, table,
     …) already break canvas alignment when interleaved — pre-existing. Now that docking is
     a second real-undo env, interleaved canvas+docking sequences miss-align **both** vectors
     the same way the canvas+layers case already does. Per the R-A1 mitigation the Phase-9
     shape follows the established canvas recipe (no unilateral engine redesign inside a
     docking phase); single-environment consumers (the docking app path) align perfectly.
     Phase 10, which owns the history system integration, should unify the alignment (e.g.
     one position-indexed executed-step registry) rather than stack a third vector.
   - **R-9-2 (note):** `enforce_history_limit()` is a no-op today (its truncation is
     documented-unimplemented); `execute_docking` calls it only to mirror `execute_canvas`.

## 3. Completion criteria check (current_state/plan.md Phase 9)

- [x] **Docking operations are accessible through the established AppShell integration** —
      `DomainEngine::execute_docking` + `docking()` reader + `DomainError::Docking`; the
      engine owns the model instance exactly like the other environments (no duplicated
      state ownership; the environment's mutators stay `pub(crate)`, so external mutation
      can only flow through the operation pipeline); the app-engine command bridge exists
      (`docking_command_handler.rs`).
- [x] **Invalid state changes are rejected consistently** — execution is exclusively the
      environment pipeline (mutate → normalize → validate → commit); failures surface as
      `DomainError::Docking(DockingError::…)`, map through the established
      subsystem-error conversion pattern, auto-roll back inside transactions, and leave
      state, history, and event counts untouched (pinned by tests); undo/redo validate
      restored states.
- [x] **Existing environments continue to work without regressions** — the canvas replay
      path is untouched (add-only branches); full suite green at the Phase-8 counts plus
      the new area (867 app_engine / 510 domain_foundation / 66 lib; serde 867 / 514 / 73);
      `engine_error_converts_from_subsystem_errors` untouched and green.
- [x] **Module declarations** — docking registered on both sides: the domain_foundation
      side has existed since Phase 1–8; Phase 9 adds the engine side (operations mod +
      re-exports).

## 4. Files touched

```
src/domain_engine/operations/docking_operations.rs            (new — ExecutedDockingOperation + forward-record redo)
src/domain_engine/operations/docking_command_handler.rs       (new — app-engine command bridge)
src/domain_engine/operations/mod.rs                           (2 registrations + re-exports)
src/domain_engine/mod.rs                                      (error variant + model/vec fields + execute_docking +
                                                               undo/redo/rollback branches + event/signal helpers)
tests/app_engine_tests/docking_undo_redo_test.rs              (new — 17 tests)
tests/app_engine_tests/main.rs                                (mod docking_undo_redo_test;)
```

The docking environment itself (`domain_foundation/environments/docking/`) is **untouched**
in Phase 9 — the `pub(crate)` `UndoData::replay_into` seam existed for exactly this
consumer since the operations landed (`#[allow(dead_code)]` now justified by use).

## 5. Next: Phase 10 — History, persistence, and restoration

Per `current_state/plan.md` Phase 10 + the Phase-9 record:

1. **Serialization of docking state** (PRD §16 groundwork, Phases 1–8 already gated):
   the environment types are `serde`-derive complete (`serde(transparent)` `DockId`,
   `serde(default)` restore-compat fields — weight baselines, collapse flag, persistent
   hosts, floating rects). Choose the persisted payload + layout-restore policy (ids may be
   shipped or re-issued — Phase-1 §7.2 deferred this to Phase 10) and where the persisted
   form lives (existing `app_engine/persistence/` + `serialization/` conventions).
2. **Restore + validate:** a saved layout must restore into an equivalent **valid** domain
   state — the Phase-3 gate (`DockLayout::validate`) already defines the imported-state
   report (first-error, deterministic scan incl. region/floating rect garbage fixtures);
   decide missing/outdated-reference rules (PRD §7.7) for unavailable panels/hosts.
3. **History integration:** docking mutations already integrate with undo/redo and
   transactions (Phase 9 wrapper + transaction tagging + rollback); Phase 10 defines how
   the history system records environment changes for persistence (its serialization
   story) and should resolve **R-9-1** (per-env executed-vector alignment under
   interleaving — unify rather than stack a third vector) and R-9-2 if the history limit
   becomes real.
4. **No docking-specific persistence framework** — ride the existing
   `app_engine/persistence/` + `serialization/` infrastructure.
5. **Tests:** serialization round-trips per `agents/testing.md` (`serde` feature-gated;
   `from_str → assert_eq!(original, restored)`; imported-garbage fixtures assert exact
   error variants; JSON cannot carry `f64::NaN`).
