# Playbook — establish a new environment

*Follow when the task is "create a new environment in domain_foundation"
(that is not already on the registry). Knowledge deps: `agents/project.md`,
`agents/conventions.md`, `agents/environments.md`, `agents/testing.md`.
State deps: `current_state/CHECKPOINT.md`, `current_state/plan.md`.*

## 0. Discover before designing (do not skip)

Inspect the real source of ≥2 existing environments (the most recent one is the
freshest convention reference; check `src/domain_foundation/environments/mod.rs` for
the registry order). Identify how they define: models, ids, operations/outcomes,
errors, pub(crate) encapsulation, and tests. Extract the *pattern*, not the code.

Deliverable: a short integration note (a phase record under
`current_state/records/`), naming conventions fixed for the new env
(module name, model struct name, op/outcome names, error names, event prefixes).

## 1. Commit to a phased plan

Split the work into sequential phases (the docking product used a 12-phase plan, see
`current_state/plan.md` for shape): architecture review → core model → validation →
operations → normalization → domain-specific semantics (sizing/embedding/floating…)
→ engine integration → history/persistence → interaction contract → verification.

Requirements first: write a PRD-style requirements doc (MUST/SHOULD + acceptance
criteria + resolved open questions) into `current_state/`. The plan references it by
section; code doc comments reference plan/PRD sections for traceability.

## 2. Core model phase

Per `agents/environments.md` — identity newtype, model struct with creation
primitives + typed preconditions, placement/hierarchy derivation, traversal
(`is_descendant`, `descendants`, `path`, `depth`), serde gating from day one,
explicit empty-state representation, `pub(crate)` mutation surface.

Decide and record (in the phase record) tradeoffs like parent back-pointers vs
scan-derived placement — the docking rule: collections are the single source of
truth, placement derived by scan.

## 3. Validation phase

Whole-layout invariant checking per `agents/conventions.md` ("Validation pattern") if
the env defines validity beyond creation preconditions: first-error deterministic
order, reuse creation-time error variants, in-crate invalid-state fixtures,
imported-state (serde) tests, derived properties asserted not re-checked.

## 4. Operations phases

Value-type `Operation`/`Outcome` enums with `apply()`; **outcomes carry full inverse
data from day one** (see `agents/engine_integration.md` — the undo reality); guards
(ctx: cycle checks, eligibility, reject-by-default occupancy policies specified
explicitly in the plan/PRD); normalization as a later, separate phase; failed
operations leave the model untouched (PRD-style OP rules copied into the requirements
doc).

## 5. Engine integration phase

Follow `agents/engine_integration.md` exactly: field + accessor + `execute_*` +
`DomainError` wrapper + name/event/signal helpers; real undo only if a
`Executed*Operation` wrapper + undo/redo/transaction branches are added.

## 6. History/persistence phase

Serialize via the serde feature; define restoration validation (imports go through
`validate()`-style checks with explicit repair/reject rules recorded in the PRD);
undo/redo through the existing history system — no new frameworks.

## 7. Verification phase

Each op tested independently; op sequences; failure/rollback; serialization;
undo/redo through the engine; PRD acceptance-criteria coverage; then docs
(`agents/docs_site.md`) and a public-API review pass.

## Checklists

- [ ] Registered: `environments/mod.rs` (`pub mod` + doc tree) + `domain_foundation/mod.rs`
- [ ] `mod.rs` owns/not-owns doc header; PRD/design § references in doc comments
- [ ] Serde feature compiles (`cargo check --features serde`)
- [ ] Test suite registered in `tests/domain_foundation_tests/main.rs`
- [ ] Per phase: record in `current_state/records/`, checkpoint updated
      (`agents/recipes/run_plan_phase.md` protocol)
