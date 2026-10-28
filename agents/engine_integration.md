# Domain Engine integration recipe

*Read when: wiring an environment into `DomainEngine`, or touching history /
undo-redo / transactions / events in `src/domain_engine/`. Stateless.*
*Source of truth: `src/domain_engine/mod.rs` and
`src/domain_engine/operations/` — verify the current shape in source before relying
on any "what exists today" statement below.*

## The extension points (one per environment)

| Extension point | Convention | Example: layers → docking |
|---|---|---|
| Model field | owned struct per env | `layers: Layers` → `docking: DockLayout` |
| Read accessor | `pub fn <env>(&self) -> &<Model>` | `layers()` → `docking()` |
| Execute | `execute_<env>(op) -> Result<Outcome, DomainError>` | `execute_layers` → `execute_docking` |
| Error variant | `DomainError::<Env>(<EnvError>)` + `From` + `Display` arm `"{env}: {e}"` | `DomainError::Docking(DockingError)` |
| Name helpers | bottom-of-file free fns `<env>_op_name`, `<env>_outcome_event`, `<env>_outcome_signal` | `docking_op_name(&op)` |
| Event naming | `"{env}.{noun}.{action}"` (`canvas.object.moved`, `layers.layer.created`) | `docking.dock.created` |
| Signal naming | same, plus `"{env}.changed"` wildcard fallback | — |

## The execute recipe (value-type operation path)

```rust
pub fn execute_layers(&mut self, operation: LayersOperation)
    -> Result<LayersOutcome, DomainError>
{
    let outcome = operation.clone().apply(&mut self.layers)
        .map_err(|e| { let err: DomainError = e.into(); self.check_auto_rollback(&err); err })?;
    let name = layers_op_name(&operation);
    let mut entry = HistoryEntry::new(name).undoable();
    if let Some(tx_id) = self.active_transaction {
        entry = entry.with_transaction(tx_id);
    }
    self.history.append(entry);
    self.publish(layers_outcome_event(&outcome), layers_outcome_signal(&outcome));
    Ok(outcome)
}
```

Failures auto-roll back and surface as `DomainError`; success appends a name-only
or undoable `HistoryEntry` tagged with the active transaction, then publishes
event + signal. Callers cannot bypass validation: they go through `execute_*`
(the model's mutators are `pub(crate)`).

## Undo/redo — the critical asymmetry

Name-only history entries do **not** replay state. Real undo works through
`Executed<Env>Operation` wrappers (`canvas_operations.rs` is the reference):
a wrapper stores `(operation, outcome)` and implements
`undo(&mut <Model>)` / `redo(&mut <Model>)` from the outcome's inverse data.

Requirements for an environment with true undo:

1. **Outcomes must carry complete inverse data** (removed subtrees in BFS order,
   previous placements/sizes/selection state). Design this into every outcome —
   retrofitting is what forces a model-snapshot fallback.
2. Engine gets one `ExecutedStep` variant inside the **single position-indexed
   registry** `executed_steps: Vec<Option<ExecutedStep>>` — the step sits at the
   same vec index as its history entry (`None` gaps mark entries recorded
   without a replay mechanism); the engine's `undo()` / `redo()` /
   `rollback_transaction()` walk history positions once and dispatch on the
   step. Docking joined Canvas as a second real-undo environment in Phase 9 and
   the vectors were unified in Phase 10 (the R-9-1 unification — exactly this
   registry shape, replacing the per-env vectors). New real-undo environments
   add one registry variant, *never* a parallel vector.
3. Environments without a wrapper push name-only entries only — acceptable for a
   perimeter, never for user-state changes.

At the time of the docking Phase‑1 review, only Canvas had real undo; docking is
now the second real-undo environment. Verify the current list in
`src/domain_engine/mod.rs` (`undo()`/`redo()`) before planning integration.

### The redo recipe (counter-issued ids ⇒ forward record, not re-apply)

`ExecutedDockingOperation` (`domain_engine/operations/docking_operations.rs`) is
the reference for any real-undo integration:

- **Re-applying the operation on redo is unsound when ids are counter-issued**
  (`DockId`, like `ObjectId`/`LayerId`: monotone counter, never renumbered). An
  exact undo does not rewind the allocator, so a re-apply allocates *fresh* ids and
  diverges from the ids recorded in the surrounding history (a later entry's
  `Remove { dock: id7 }` then fails). Canvas escapes this only because its single
  allocating op (`Create`) is special-cased to re-insert the recorded object.
- The general answer: **record the forward direction too.** `UndoData::replay_into`
  is a generic state-restore driver (re-place dock payloads → restore collection
  snapshots → set root → delete docks); feed it a *forward record* built at execute
  time from the committed post-state — full post payloads of created and
  category-swapped docks (whole-payload replacement covers role swaps like
  promotion/demotion), post snapshots of every collection the inverse names, the
  post root, and the ids the op removed. `undo = inverse.replay_into`,
  `redo = forward_record.replay_into` — one driver both ways.
- Validate the restored layout after each replay (`Model::validate` where the
  environment has one): an exact replay of a committed state is valid by
  construction, so validation only surfaces replay drift as an error. Multi-step
  undo/redo walks a position range whose every intermediate state is a historical
  committed state, so per-entry validation stays sound.

### Registry alignment (settled in Phase 10)

The engine's undo/redo/rollback replay ran off per-environment executed vectors
until Phase 9; those were position-aligned **only while that environment's
operations were the only real-undo history** — any name-only entry from another
environment appends a history position without advancing that vector, so
interleaved real-undo environments misaligned their replay loops (R-9-1,
pre-existing for canvas under layers interleave). Phase 10 unified it: one
position-indexed registry (`executed_steps: Vec<Option<ExecutedStep>>`),
`Some` exactly when the matching history entry carries a replay step, `None` gaps for
the plain-apply family and non-replayable actions (e.g. a restore). Do not
reintroduce parallel vectors; see item 2 in the requirements list above.

### Persistence and restoration recipe (Phase 10)

- **The persisted form is the model's own serde derive** — the established
  serialization convention (all sibling environments ship derive-only serde;
  `serde_json` lives in the app/dev layer). The engine never gains a
  format-specific save API: saving = serialize the cloned model; restoring =
  parse the payload and hand the result to the engine.
- **The engine's restore entry validates before it adopts**:
  `restore_docking(layout)` runs the environment's `validate()` (the Phase-3
  gate / imported-state report) *first* — rejection, never repair (PRD §12.3);
  only then swaps the model in, appends a **non-undoable**
  `"{env}.restored"` history entry, marks the engine saved, and publishes
  `"{env}.restored"` (event + signal). The restore's registry slot stays
  `None` — it is not a replayable step.
- **Ids ride inside the form**: models whose ids are counter-issued serialize
  the counter too (docking: the whole `DockLayout` incl. `next_id`), so a
  restored layout is collision-free by construction and restored-state undo
  keeps the counter monotone (∴ undo round-trips compare *structural* state,
  not `PartialEq` — the counter is not equal after a restored-create was
  undone).
- **History persistence is metadata-only** (`HistoryStore::persist_entry`
  through the `StorageBackend`): inverse/replay data stays in-memory; full
  state restore rides the environment's persisted form, not recorded history.

## Transactions and compound operations

`begin_transaction / commit / rollback`, auto-rollback on error (see the
`map_err(... check_auto_rollback ...)` above), and `begin_compound /
commit_compound / rollback_compound` (depth-counted). Environments inherit
transaction tagging solely through `entry.with_transaction(tx_id)` — no extra
environment code. `rollback_transaction()` reverts only the environments with
real-undo wrappers (canvas and docking today, through the registry dispatch) —
extend it via requirement 2 above.

## Optional CommandHandler bridge

`domain_engine/operations/<env>_command_handler.rs` implements
`app_engine::commands::CommandHandler`:

- Hold `Arc<Mutex<Model>>`; extract the typed payload via `input.get::<XxxOperation>()`;
  apply it; append `HistoryEntry::new(name).undoable()` to `engine.history_store`;
  publish the event on `engine.event_bus`; return `CommandResult::success_with(outcome)`.

This is an app-engine-side bridge, separate from `DomainEngine::execute_*`. Add it
**only when an app-engine command consumer exists** (template:
`canvas_command_handler.rs`). See `agents/app_engine.md`.
