# App Engine surfaces (only when required)

*Read when: a task explicitly touches `app_engine` — history entries, command
handlers, EngineRef — or an environment needs the app-engine bridge. Everything
here has its canonical names in `src/app_engine/mod.rs` (the re-export surface).*

## Canon names (from `app_engine/mod.rs` re-exports)

`EventBus`, `SignalBus`, `HistoryEntry`, `HistoryStore`, `TransactionManager`, …

## History

Files: `src/app_engine/history/{history_entry.rs, history_store.rs}`.

- `HistoryEntry::new(name)` → builder with `.undoable()` and `.with_transaction(tx_id)`;
  code that appends entries relies on the store's **position semantics**: entries are
  stack-based (a position cursor; undo/redo walk it; a new append after undo usually
  truncates the redo tail — verify in `history_store.rs` before relying).
- Name-only entries are the accepted pattern for environments without real undo;
  undoable entries without an inverse mechanism are a **lie** — an environment must
  pair `.undoable()` with an engine-side replay mechanism
  (see `agents/engine_integration.md`).

## Commands

Files: `src/app_engine/commands/{command.rs, command_context.rs,
command_definition.rs, command_executor.rs, command_result.rs}`.

- `CommandHandler` trait; extract typed payloads with `CommandInput::get::<T>()`;
  build results with `CommandResult::success_with(outcome)`.
- The optional per-environment bridge template lives in
  `domain_engine/operations/<env>_command_handler.rs`
  (see `agents/engine_integration.md`).

## EngineRef

`src/app_engine/capabilities/engine_ref.rs` — the handle handlers receive
(`engine.history_store`, `engine.event_bus`).

## Do-not-read boundary

Unless the task is explicitly about them, do not read: `plugins/`, `scheduling/`,
`concurrency/`, `resources/`, `lifecycle/`, `services/`, `tasks/` subsystems.
App-engine tests live in `tests/app_engine_tests/` (suite conventions:
`agents/testing.md`).
