# AppShell — Agent Routing Manifest

Root [`AGENTS.md`](../AGENTS.md) is the auto-loaded entry point. It holds the golden
rules and points here. **This knowledge base follows the skill pattern: nothing below is
loaded automatically — an agent opens only the file(s) its current task triggers.**

## Routing table — READ WHEN triggers

| File | READ WHEN |
|---|---|
| `project.md` | First session in the repo, or when unsure what a layer/crate is responsible for, or what NOT to read |
| `conventions.md` | Writing or reviewing Rust in this repo (style, serde, errors, IDs, operations) |
| `environments.md` | Creating or extending an environment under `src/domain_foundation/environments/` |
| `engine_integration.md` | Integrating an environment into `DomainEngine` (`execute_*`, history, undo/redo, transactions, events) |
| `app_engine.md` | A task explicitly touches `app_engine` (history entries, CommandHandler bridge, EngineRef, tasks) |
| `testing.md` | Writing or running tests, adding fixtures, or checking for regressions |
| `docs_site.md` | Documentation-site work (`md_docs/`, `mkdocs.yml`, published docs, repo links) |
| `recipes/new_environment.md` | Playbook for establishing a **brand-new** environment end-to-end |
| `recipes/run_plan_phase.md` | Executing one phase of a planned feature (record + checkpoint protocol) |
| `reference/new_domain_foundation.md` | Original env-creation guide notes (Q&A transcripts); most items are reconciled into `environments.md` |
| `reference/structure.md` | Historical proposed repo tree (Q&A transcript); reconciled by the Phase‑1 review — verify against the real tree |

**Stateful knowledge lives elsewhere:** `current_state/` (checkpoint, plan, PRD,
records, design inputs). Never move project state into `agents/` — files here must
stay independent of the current phase.

## Maintenance rules for this folder

1. **One file = one load trigger.** If a task would need a file only sometimes, it stays
   its own file; if a file would always be needed alongside another, merge them.
2. **Stateless only.** No "phase X is next", no test counts, no current-phase tasks —
   that is `current_state/` content. Facts with timestamps ("currently only Canvas has
   real undo") must carry a *verify-in-source* instruction instead of relying on them.
3. **Derived knowledge, not copies.** Prefer distilled rules + pointers to real source
   files (`src/...:` line refs). Duplicated explanations drift; the source does not.
4. **Updating.** When a phase establishes a new convention or recipe, fold it into the
   matching file here and (if it belongs to the plan phase) into
   `current_state/checkpoint` decisions, keeping both sides stateless/stateful.
5. **Never renamе silently.** Moving/renaming any file here requires updating the
   root `AGENTS.md` routing entry and every cross-reference.
