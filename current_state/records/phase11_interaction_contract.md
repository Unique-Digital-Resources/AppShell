# Docking Environment — Phase 11 Record: Interaction Contract and Frontend Handoff

**Phase:** 11 of 12 (`current_state/plan.md` — Interaction contract and frontend handoff)
**Status:** ✅ Complete — full suite green, serde path green, warning/doc baselines unchanged
**Verification:**
- `cargo test` → **879 app_engine (+8 Phase-11 contract tests) + 510 domain_foundation
  (integration) + 66 in-crate lib tests pass**, 0 failures. Docking external coverage
  unchanged in `domain_foundation_tests` (98 docking tests); the docking engine tests add
  `docking_interaction_contract_test.rs` (8 ungated tests + 1 serde-gated).
- `cargo test --features serde` → **884 app_engine (879 + 4 persistence + the serde-gated
  persisted-form contract test) + 516 domain_foundation + 73 lib pass**, 0 failures.
- `cargo check` / `cargo check --features serde` → clean; **10 pre-existing lib warnings**
  on both paths, none from the Phase-11 files (initial `split` fixture binding was tightened;
  the file now compiles warning-free — verified 0 warnings citing the new file).
- `cargo doc --no-deps` → **9 warnings, all pre-existing Phase-2/3 leftovers** (locations
  verified: `embed_host.rs`, `model.rs`, `slot.rs`, `docking/mod.rs:27` — the
  `[`validation`](validation)` link predates this phase and sits outside the added section —
  plus relations/`node_graph`); the three Phase-11 doc edits introduce no warnings.

---

## 1. What was built

Phase 11 specifies the frontend handoff without implementing UI: the contract is
**the settled operation API itself**, formalized as documentation + contract tests.
No interaction machinery, no new state, no listener/preview API (the plan's
drag-session-file warning resolved: the domain needs no persistent interaction
state, PRD §11.2).

| File | Established |
|---|---|
| `src/domain_foundation/environments/docking/mod.rs` | New module-doc section **"Frontend handoff — the interaction contract (PRD §10, §11)"**: the §10.1 stage-by-stage lifecycle → API mapping table; the §10.2–§10.3 region+mode → operation tables (split edges → `InsertBeside` directions; center is not a split intent; tab center → `TabJoin`; embed edges → `EmbedSlot::Region`; embed body drop → `EmbedSlot::Floating { rect }`, `None` = baseline); the §10.4 eligibility-discovery read table (kinds, `HostRegions::on`, `placement`, self-inclusive `is_descendant`) with "advisory only, the commit re-validates"; the settled rejection-payload table; gesture commit boundaries (§11.4: per-frame undo vs. transaction span). States the no-drag-session resolution and the §10.6 input-method neutrality. The "deliberately does not own" bullet now points at the section. |
| `src/domain_foundation/environments/docking/operations.rs` | Module-doc cross-reference section "Frontend interaction contract (PRD §10–§11, Phase 11)": these operation values *are* the contract; the per-op gesture contracts (already documented since Phases 6–8) are its resize/move clauses; the tables live in the environment's module docs. |
| `src/domain_engine/operations/docking_command_handler.rs` | Module doc added (matches the sibling handlers' one-line style + the contract pointer): the engine route `DomainEngine::execute_docking` is the frontend-facing commit path; the bridge appends a name-only undoable entry like its siblings and is never a substitute for real undo. |
| `tests/app_engine_tests/docking_interaction_contract_test.rs` | **New — 9 tests** (8 ungated + 1 serde-gated), pinned contract-level: (1) each split-mode edge intent maps to the right `SplitDirection` (orientation + child order + placement index); (2) tab-mode center intent joins-or-creates the group (joined member active; existing group reused as last member); (3) embed-mode edge intents land in the named regions with the §9.5 baseline weight, no auto-promotion of an explicit host; (4) the embed body drop carries the drop's logical rect (or the floating baseline with `None`); (5) the settled rejection payloads — `IneligibleDock` (split & tab targets), `RegionOccupied { host, edge }`, `NotSplittable` (floater target), `CycleDetected` (self-target **and** host-inside-source), `DockNotFound` (removed id) — each followed by a byte-equal layout assert, with zero events / zero history appended across all rejections; (6, serde-gated) the persisted form serializes byte-identical across a rejection — previews never reach §11.1 state; (7) committed gesture frames roll back frame-by-frame (`SetFloatingRect` frames, per-frame `undo()`, `previous` chains baseline→frame1); (8) a transaction span reverts whole on cancel (`rollback_transaction` removes span entries; not redoable) and undoes/redoes as one unit when committed (one `undo()`/`redo()` call each through the registry); (9) the full §10.1 walkthrough in stage order — source identified via the public `iter()`/`content()` read, eligibility discovery reads, one confirmed `TabJoin` commit, outcome/slot assertions, transient clear (frontend posture), and a second drag whose incompatible mode rejects cleanly. |
| `tests/app_engine_tests/main.rs` | Registered `docking_interaction_contract_test` (ungated; the serde test gates itself in-file). |

Behavior code changed: **none** — no operation, eligibility policy, error variant, or
engine semantics were added; the docking environment and engine are untouched apart
from the doc sections above.

## 2. Decisions made during implementation (do not relitigate)

1. **The contract is specification, not new API — no drag-session file** (the plan's
   warning is resolved the negative way): PRD §10–§11's stages map entirely onto the
   existing surfaces — [`DockingOperation`] values (intent), the public read API
   (eligibility discovery), `execute_docking` (commit; `DockingOutcome` / typed
   `DockingError` carry the response), engine undo/transactions (gesture rollback),
   and the engine event publishing (notification after commit, §11.4 commit
   boundary). The environment owns **no** drag state and cannot observe one; a
   purely-visual drag is frontend-only state, and the Phase-10 restore gate would
   reject anything transient that leaked into the persisted form anyway.
2. **Mode + region → operation is a pinned mapping, not a re-interpretation policy**
   (PRD §10.2–§10.3): split mode + the four edge intents = `InsertBeside` with the
   matching `SplitDirection`; center is *not* a split intent (the frontend submits
   the tab or embed mode instead); tab mode + center = `TabJoin` (wraps or joins,
   joined member active); embed mode + edge intents = `Embed` with
   `EmbedSlot::Region(edge)`; embed mode + body drop = `Embed` with
   `EmbedSlot::Floating { rect }` (the drop's logical rect; `None` = baseline).
   An incompatible submitted value rejects *as itself* — the environment never
   silently executes a different operation (PRD §10.3 MUST).
3. **Eligibility discovery is advisory read-side, and validation stays authoritative**
   (PRD §10.4): the documented discovery table (embed-capable kinds, free regions via
   `HostRegions::on`, non-floating placement for directional inserts, panels-only v1
   tab membership, self-inclusive `is_descendant` cycle guard) is enough to avoid
   presenting invalid targets, but the commit re-validates every precondition — a
   missed pre-check surfaces as the settled typed rejection, never an executed
   equivalent operation. The settled rejection payloads (`DockNotFound`,
   `CycleDetected` incl. self-targeting, `IneligibleDock`, `RegionOccupied`,
   `NotSplittable`, category/index/bound violations) name ids, hosts, edges, and
   indexes so the frontend can re-scope the drag.
4. **A rejection is the whole consequence** (PRD §10.5 + §8.2 atomicity, pinned):
   the layout is untouched, no history entry is appended, no event published — the
   serde test additionally pins the persisted form is byte-identical after a
   rejection (completion criterion: preview state is not accidentally persisted as
   committed layout state).
5. **Gesture commit boundaries ride existing history conventions** (PRD §11.4): a
   resize/float-move gesture commits absolute-request frame operations (the per-op
   gesture contracts, Phases 6–8); two commit-boundary forms are pinned — per-frame
   `undo()` walk-back for unwrapped gestures, and a transaction span
   (`begin_transaction` → frames → `rollback_transaction` reverts whole in one call /
   a committed span undoes and redoes as one unit through the executed-step
   registry). No new gesture/preview/snapshot machinery exists.
6. **Input-method neutrality is structural** (PRD §10.6): the vocabulary carries
   intent (ids, regions, direction enums, logical rects) and no coordinates or input
   device names — the same values are submitted by keyboard, touch, or programmatic
   callers, so no separate API path is possible or needed.

## 3. Completion criteria check (current_state/plan.md Phase 11)

- [x] **The frontend can express docking intent using domain-level inputs** — the
      walkthrough test exercises §10.1's stages in order using only
      `DockId` / `InsertSource` / mode + region intent values and the public read
      API (even the source discovery step is a `content()` lookup); the
      stage→API table lives in the environment's module docs.
- [x] **The environment can validate and execute those requests without knowing
      about mouse coordinates** — nothing in the operation vocabulary, read API,
      or engine path names coordinates; the contract tests submit intent-only
      values exclusively (hit-testing/pointer tracking stay frontend per §10,
      §11.3 — confirmation by construction, documented + pinned).
- [x] **Preview state is not accidentally persisted as committed layout state** —
      the environment holds no interaction state to confuse with layout state
      (decision 1), and the serde-gated test pins that a rejected request leaves
      the serialized payload byte-identical (§11.1, §12.1 boundary).

## 4. Files touched

```
src/domain_foundation/environments/docking/mod.rs               (contract section + ownership-bullet pointer)
src/domain_foundation/environments/docking/operations.rs        (contract cross-reference section)
src/domain_engine/operations/docking_command_handler.rs         (module doc: engine route = frontend commit path)
tests/app_engine_tests/docking_interaction_contract_test.rs     (new — 9 tests, 1 serde-gated)
tests/app_engine_tests/main.rs                                  (1 registration)
```

## 5. Next: Phase 12 — Verification and release readiness

Per `current_state/plan.md` Phase 12 (the final phase — environment readiness for
applications):

1. **Test each structural operation independently** — largely already covered by the
   Phase-4–8 suites; the phase's job is the *gap audit* against PRD §18 (Model /
   Structural operation / Atomicity / Sizing / Embedding / Floating / Normalization
   / Undo / Persistence tests), not re-pin what exists.
2. **Operation sequences** (split → move → tab group → remove; host promotion and
   demotion; floating and re-docking of nested structures) — sequence-level
   integration tests through the engine, likely an
   `docking_sequence_test.rs` (cross-check what Phase 9–11 suites already compose).
3. **Sizing constraints and normalization; invalid operations and rollback;
   serialization/restoration; undo/redo** — acceptance gap fill (the previous
   phases pinned most).
4. **Public API review** — remove unnecessary abstractions; confirm every public
   operation has documented behavior + errors (PRD §15). The Phase-11 contract
   section is the front door for the review.
5. **Documentation and examples** — the docs/demos delivery the plan reserves for
   this phase (candidates: a public-API example module or docs page walking a
   frontend through the Phase-11 contract).

Reading list: this record + `CHECKPOINT.md`, PRD §17–§20 (NFRs + acceptance
criteria), the record of each phase for settled behaviors, and the test files
themselves for the coverage map.
