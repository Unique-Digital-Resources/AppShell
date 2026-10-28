# Project overview — what AppShell is

*Read when: first session in the repo, or whenever layer responsibilities or the
"do not read" boundary are unclear. Stateless — current work state lives in `current_state/`.*

## What AppShell is

AppShell is an application-shell platform: generic embedded infrastructure
(engine kernels) plus a library of reusable **headless domain environments** that the
Domain Engine composes into an application. It ships as a single Rust crate
`app_shell` (lib `src/lib.rs` + bin `src/main.rs`).

## The three-layer stack

```
src/
├── app_engine/          # generic infrastructure (commands, history, events, signals...)
├── domain_foundation/   # headless, reusable domain environments + relations + utilities
└── domain_engine/       # engine composing environments + app_engine (execute_*, undo/redo)
```

- **Dependency direction:** `domain_engine → domain_foundation → app_engine`.
  Never upward. `app_engine` must not know domain concepts.
- **`app_engine`** — the generic engine: commands, history/undo, events, signals,
  scheduling, concurrency, persistence, configuration, state, tasks. Environments
  never become mini App Engines: environments define *what an operation means*;
  `app_engine`/`domain_engine` provide the execution infrastructure.
- **`domain_foundation`** — environments themselves, plus `relations/`
  (definition → registry → evaluation → propagation) and `utilities/`
  (batch, filtering, querying, sorting, grouping, tagging, validation — verify the
  current set in `src/domain_foundation/utilities/mod.rs`). Shared test fixtures live
  in `src/domain_foundation/testing/fixtures.rs`.
- **`domain_engine`** — owns the `DomainEngine` struct: one owned field per
  environment, plus history/transaction machinery. Registration of operations and
  command handlers lives in `src/domain_engine/operations/`.

## Registered environments

`src/domain_foundation/environments/mod.rs` registers, in order:
`canvas_2d`, `timeline`, `layers`, `node_graph` (composes Canvas2D internally),
`viewport_3d`, `table`, `text_editor`, `docking`.

- Environment models are named per domain, not uniformly:
  `Canvas`, `Timeline`, `Layers`, `NodeGraph`, `Scene`, `Table`, `Document`,
  `DockLayout`.
- **No universal `Environment` trait** — intentional. Environments are duck-typed by
  convention; composition happens between concrete environments (NodeGraph uses
  Canvas2D for spatial placement while owning graph connectivity).
- Each environment declares in its `mod.rs` doc header **what it owns and what it
  deliberately does not own** (rendering, pointer/input, application data).

## Build layout

| Path | Purpose |
|---|---|
| `src/{app_engine,domain_foundation,domain_engine}/` | the three layers |
| `tests/app_engine_tests/`, `tests/domain_foundation_tests/` | integration suites, one `.rs` per feature area (`main.rs` declares `mod`s) |
| `benches/domain_foundation_bench.rs` | criterion bench (`harness = false`) |
| `md_docs/` + `mkdocs.yml` | documentation site (generated `site/` is ignored) |

Features in `Cargo.toml`: `serde = ["dep:serde"]`, `async-runtime = ["tokio"]` (both
optional; default = none). Dev-deps: `criterion`, `proptest`, `serde_json`.

## Do-not-read boundary

Do **not** read these `app_engine` subsystems unless the task is explicitly about
them: `plugins/`, `scheduling/`, `concurrency/`, `resources/`, `lifecycle/`,
`services/`, `tasks/`. They are large and unrelated to environment work.

Repo docs published at:
<https://unique-digital-resources.github.io/AppShell-Documentation/> ·
source repo: <https://github.com/Unique-Digital-Resources/AppShell>
