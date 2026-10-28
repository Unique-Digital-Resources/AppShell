# AGENTS.md — AppShell

Working guide for AI agents on **AppShell** (Rust, single crate `app_shell`).
This file is the auto-loaded bootstrap; everything else is **skill-style knowledge
loaded only when needed** (see `agents/README.md` for the knowledge-base rules).

## Session start — always

1. **Read `current_state/CHECKPOINT.md` first** — current established state,
   settled decisions, next step, and what to check. It is the one source for
   "where are we".
2. **Take current tasks from `current_state/plan.md`** (the active feature's
   phase plan) and `current_state/PRD.md` (requirements/acceptance criteria).
3. Then load **only** the knowledge files your task triggers (table below).
   Do not skim the whole project.

## Routing table — load knowledge on demand

| File | READ WHEN |
|---|---|
| `agents/project.md` | What AppShell/layers are; do-not-read boundaries |
| `agents/conventions.md` | Writing/reviewing Rust (style, serde, errors, IDs, ops) |
| `agents/environments.md` | Creating/extending an environment in `domain_foundation` |
| `agents/engine_integration.md` | Wiring an env into `DomainEngine` (execute_*, undo, events) |
| `agents/app_engine.md` | Tasks explicitly on `app_engine` (history, CommandHandler, EngineRef) |
| `agents/testing.md` | Writing/running tests, fixtures, regressions |
| `agents/docs_site.md` | Documentation site work (`md_docs/`, `mkdocs.yml`) |
| `agents/recipes/new_environment.md` | Playbook: establish a brand-new environment |
| `agents/recipes/run_plan_phase.md` | Playbook: execute one planned phase (+ record/checkpoint protocol) |
| `agents/reference/*.md` | Historical source-guide Q&A notes (verify against live code) |
| `current_state/design/…` | Design-basis documents for the current feature |

Unit tests in `tests/domain_foundation_tests/`, `tests/app_engine_tests/` (one `.rs`
per feature area, registered in the suite's `main.rs`); benches in `benches/`.
Verification: `cargo test` (full), per-area: `cargo test --test domain_foundation_tests -- <name>`,
serde path: `cargo check --features serde` / `cargo test --features serde`.

## Golden rules

1. Match existing code conventions over any external proposal. Every environment
   is headless: no DOM, CSS, rendering, pointer/input, or windowing.
2. **No universal `Environment` trait** — environments are duck-typed by
   convention (this is intentional).
3. Operations are value-type enums with
   `apply(self, &mut Model) -> Result<Outcome, EnvError>`; outcomes carry
   **previous state for undo** (see `CanvasOutcome`, `LayersOutcome::LayerRemoved`).
4. Serde is feature-gated:
   `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]`.
5. Don't skim the whole project — use the minimal file maps in the `agents/` files.

## Update protocol (who updates what)

- **Finished a planned phase** → follow `agents/recipes/run_plan_phase.md`:
  new `current_state/records/phase<N>_<name>.md` + update `current_state/CHECKPOINT.md`.
- **Reached a new codebase-level convention/recipe** → fold it into the matching
  `agents/` file (keep it stateless) and, if it changes routing, root `AGENTS.md`
  + `agents/README.md` together.
- **Finished the whole feature / switched feature** → refresh
  `current_state/plan.md` + `PRD.md` for the new feature and reset the checkpoint's
  status snapshot (records stay as history).
