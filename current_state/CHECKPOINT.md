# CHECKPOINT — Docking Environment Work State

**Read this file first in any session that touches the docking environment, the
current planned feature, or `current_state/`. Update it at the end of every
completed phase** (protocol: `agents/recipes/run_plan_phase.md`).

- **Last updated:** 2026-10-18 — after **Phase 12** — **feature complete**
- **Current phase:** Phase 12 complete → **Docking Environment shipped**. No further
  docking phase; the next *feature* starts fresh (refresh `plan.md` + `PRD.md` there;
  records stay as history).
- **Plan of record:** `current_state/plan.md` (12 phases). Requirements: `current_state/PRD.md`.
- **Agent guide:** root `AGENTS.md` (routing) + `agents/` knowledge base
  (load only what the task triggers — see `agents/README.md`).

---

## 1. Status snapshot

| Phase (`current_state/plan.md`) | State | Record |
|---|---|---|
| 01 — Architecture & integration review | ✅ Done | `records/phase1_architecture_review.md` |
| 02 — Core domain model | ✅ Done, tests green | `records/phase2_model.md` |
| 03 — Validation & structural invariants | ✅ Done, tests green | `records/phase3_validation.md` |
| 04 — Basic structural operations | ✅ Done, tests green | `records/phase4_basic_structural_operations.md` |
| 05 — Normalization & atomic mutations | ✅ Done, tests green | `records/phase5_normalization.md` |
| 06 — Sizing and resizing | ✅ Done, tests green | `records/phase6_sizing.md` |
| 07 — Embedding and host lifecycle | ✅ Done, tests green | `records/phase7_embedding.md` |
| 08 — Floating docking structures | ✅ Done, tests green | `records/phase8_floating.md` |
| 09 — Domain Engine integration | ✅ Done, tests green | `records/phase9_domain_engine_integration.md` |
| 10 — History, persistence, and restoration | ✅ Done, tests green | `records/phase10_history_persistence_restoration.md` |
| 11 — Interaction contract and frontend handoff | ✅ Done, tests green | `records/phase11_interaction_contract.md` |
| 12 — Verification and release readiness | ✅ Done, feature complete | `records/phase12_verification_and_release_readiness.md` |

Phase 01→12 of PRD §20 map onto these 12 phases (see Ph.1 report §6). Phase 12 closed
the feature: acceptance audit (PRD §18/§19 all pinned), a 5-test engine-level sequence
suite, the public-API review (doc fixes only), and the docs deliverable
(`21-docking.md` + cross-reference/site updates). Details in the Phase-12 record.

**Build/verify state at checkpoint (end of feature):**

- `cargo test` — **884 app_engine (879 + the 5 Phase-12 sequence tests in
  `docking_sequence_test.rs`) + 510 domain_foundation (integration) + 66 in-crate lib
  tests pass**, 0 failures. Docking external coverage: 98 docking tests in
  `domain_foundation_tests` + 43 docking engine tests in `app_engine_tests`
  (Phases 9–10 + contract suite + sequence suite).
- `cargo test --features serde` — same suites green (**889 / 516 / 73**): the serde
  delta is the docking persistence tests (`docking_persistence_test.rs`), the two
  docking round-trips, and the serde-gated contract/persistence tests.
- `cargo check` / `cargo check --features serde` — clean. **10 pre-existing warnings**,
  none from `docking/` or the Phase-9–12 files (locations re-verified this session, incl.
  `app_engine` unused imports, `job.rs` `set_last_executed`, `relations/endpoint.rs`
  `ObjectId::from_raw`). `cargo doc --no-deps` — **9 warnings, all pre-existing
  Phase‑2/3/7 doc leftovers** (`embed_host.rs`, `model.rs` ×2, `slot.rs`, docking
  `mod.rs:27` + relations/`node_graph`); the Phase-12 doc/code edits add none.
- **Domain Engine owns docking + the unified history registry**: `execute_docking`,
  `docking()` reader, `DomainError::Docking`, real undo/redo/transaction-rollback via
  `ExecutedDockingOperation` (`domain_engine/operations/docking_operations.rs` +
  `docking_command_handler.rs`), **one position-indexed executed-step registry**
  (`executed_steps: Vec<Option<ExecutedStep>>` — the Phase-10 R-9-1 unification;
  undo/redo/rollback dispatch on it) and **`restore_docking`** (the validated-adopt
  persistence entry: non-undoable `docking.restored` entry, `mark_saved`, event+signal).

## 2. What is established (code reality)

**Environment `src/domain_foundation/environments/docking/` (10 files):**

| File | Established |
|---|---|
| `dock.rs` | `DockId(u64)` opaque counter id (`serde(transparent)`, `dock#{n}`, never renumbered); `ContentRef` (opaque host-supplied content); `SplitOrientation::Row/Column`; `DockKind` (Panel/Split/TabGroup/EmbedHost); `Dock { id, role }` + `DockRole` payloads — `Panel { content, collapsed }` (persistent logical collapse preference, PRD §9.7), `EmbedHost { content, regions, floating: Vec<FloatingEntry>, persistent }` (PRD §8.9: persistent vs temporary-promotion host; `serde(default)` = `true` so pre-Phase-7 JSON restores persistent; the floating collection is the Phase-8 typed-entry API — one `FloatingEntry { dock, rect }` per floated subtree root) — + read accessors (`children()`, `sizes()`, `active()`, `collapsed()`, `persistent()`, `host_regions()`, `floating()`, `structural_children()` sweeping all ref edges) |
| `slot.rs` | `EdgeRegion` (Top/Bottom/Left/Right); `DockSlot` = `Root` / `SplitChild{parent,index}` / `TabMember{parent,index}` / `HostEdge{host,edge}` / `Floating{host,index}`; `owner()` |
| `embed_host.rs` | `HostRegions` (per-edge `Option<DockId>` + one **slot-attached** persistent weight per edge, PRD §9.5 — verbatim, baseline `REGION_WEIGHT_BASELINE = 1.0` on fresh embed, carried by slot-inheriting rewrites, reset to baseline when the region clears); `on(edge)`, `owner_of(dock)`, `occupied()` fixed scan order, `weight_on` (occupied-only read), `is_empty()`; mutation `assign` (occupant+weight, clearing resets the baseline) / `set_weight` (the one-region rewrite) / `slot_weight` (carry-forward read) are `pub(crate)` only (operations route); module doc defines the **two-stage allocation read** (column top/core(1)/bottom, then row left/content(1)/right; `serde(default)` restores weight fields at the baseline) |
| `sizing.rs` | `SplitSizes` relative weights, **stored verbatim — never renormalized** (the PRD §9.2 representation decision, Phase 6: init = uniform / target-weight copy, update = pair rewrite via `ResizeSiblings`, normalize = never at rest (derived read), transfer = prune / slot-inheritance) + the **§9.5 region-weight table** (Phase 7: init = embedding baseline 1.0, update = `ResizeRegion` one-region write, transfer = slot inheritance carry, cleared = baseline reset) + the **§9.6 floating-rect table** (Phase 8: init = slot rect or the floating baseline, update = `SetFloatingRect` one-entry write, transfer = slot inheritance carry, cleared = entry removed with the slot); proportion is `weightᵢ / Σweightⱼ`, a frontend read; `uniform(count)` (None at 0), `from_weights` (finite/strictly-positive), `matches(child_count)`; `pub(crate) insert_weight(index, weight)` (Phase‑4 operation helper); test-only `#[cfg(test)] from_weights_unchecked` for invalid fixtures |
| `model.rs` | `DockLayout { docks: HashMap<DockId, Dock>, root: Option<DockId>, next_id }`; creation primitives (`create_panel`, `create_embed_host` (persistent, PRD §8.9), `create_split`, `create_tab_group`, `set_root`) with typed preconditions; placement **derived by scan** (no parent back-pointers); traversal `is_descendant`, `descendants` (BFS, all placement categories — host edges then floating entries), `path`, `depth`, `all_docks`; `put_dock`/`take_dock`/`get_mut` are `pub(crate)` (undo paths); `pub(crate) placement_count(id)` (multiplicity for validation); `restore_collection` arms: Split/Tab/Region (occupant *and* weight together)/Floating (full entries — dock *and* rect)/Panel/Promotion/Demotion |
| `error.rs` | `DockingError`: creation-time batch (`DockNotFound`, `UnknownReference`, `DuplicateMembership`, `AlreadyPlaced`, `InsufficientChildren`, `SizeCountMismatch`, `ActiveIndexOutOfRange`, `RootAlreadySet`) + Phase‑3 invariant variants (`OrphanDock`, `DuplicatePlacement`, `InvalidRoot`, `MissingRoot`, `CycleDetected`, `InvalidSizeState{split,index}`) + Phase‑4 operation variants (`IneligibleDock`, `NotSplittable`, `NotATabGroup`, `TabIndexOutOfRange{index,member_count}`) + Phase‑6 sizing variants (`NotASplit{id}`, `SplitChildIndexOutOfRange{index,child_count}`, `SizeBoundViolation{split,index}`) + Phase‑7 embedding variants (`NotAnEmbedHost{id}`, `RegionOccupied{host,edge}`, `RegionUnoccupied{host,edge}`, `RegionBoundViolation{host,edge}`, `InvalidRegionState{host,edge}`) + **Phase‑8 floating variants** (`FloatingIndexOutOfRange{index,entry_count}`, `FloatingBoundViolation{host,index}`, `InvalidFloatingState{host,index}`) (serde-gated) |
| `validation.rs` | `DockLayout::validate() -> Result<(), DockingError>` (first-error, deterministic ascending-id scan; documented precedence: root designation → reference validity + category parity (incl. the §9.5 occupied-region weight rule — an unoccupied region's stored value is inert — and the **Phase-8 floating rect rule**: every entry must hold a valid `FloatingRect`, scanned after the region weights, before multiplicity/acyclicity, `InvalidFloatingState` naming host+index) → placement multiplicity → colour-DFS acyclicity over all placement categories) + `is_valid()`; in-crate `#[cfg(test)] validation_tests` (invalid fixtures via `pub(crate)` mutation) + `serde_import` (JSON → validate; 7 fixtures incl. imported host-region sizing + the floating rect entries: valid baseline-restore accepted, explicit garbage → `InvalidFloatingState`) — full-layout checks callable on imported state (PRD §7.7) |
| `normalization.rs` | Deterministic structural cleanup (PRD §8.12): concretized rule table — split <2 collapses (survivor inherits slot **and** slot weight; zero-child split removed); zero-member tab group removed; ≥1-member groups preserved; **hosts impose no child minimum** (§8.8) and the **empty temporary host demotes** back to `Panel { content, collapsed: false }` (the §8.9 rule — identity/content/placement verbatim, in-place role change; persistent hosts and occupied hosts are steady state); a **floated survivor** copies into the inherited `DockSlot::Floating` reference keeping the entry's rect verbatim (the §9.6 slot inheritance — Phase 8) and a zero-survivor float leaves with its entry; one shared rewrite step (`resolve_below_minimum`) + two drivers — `pub(crate) collapse_cascade` (rootward chain with undo recording; the operation cleanup step) and `DockLayout::normalize` (public, undo-free whole-layout fixpoint pass, lowest-id placed selection); `prune_weight` (the removal-side weight policy); docs: resize cross-reference, collapsed-member, promotion/demotion, and floating-entry notes; `#[cfg(test)] normalization_tests` (19 fixtures via `pub(crate)` installers) |
| `mod.rs` | doc header (owns / not-owns, incl. embedding/lifecycle, region-sizing, and floating-structures bullets), re-exports (`EmbedSlot`, `FloatingEntry`, `FloatingRect`), `pub mod normalization;` |
| `operations.rs` | `DockingOperation` (CreatePanel / CreateEmbedHost / InsertBeside / TabJoin / ReorderTab / SetActiveTab / Remove / ResizeSiblings / SetDockCollapsed / Embed / PromoteHost / ResizeRegion / **SetFloatingRect**) + `DockingOutcome` (12 variants, every one carrying an `UndoData`) via `apply(self, &mut DockLayout)`; atomicity = candidate clone → normalize (shared cascade; structural ops only) → `validate()` gate → swap-commit; **`Embed { source, host, slot }`** values a panel or host (eligibility: host/panel; splits/tab groups reject `IneligibleDock`), rejects occupied regions (`RegionOccupied`), rejects host-inside-source cycles, detaches the source with cascade records, auto-promotes a panel target (temporary, `persistent: false`), takes the region (baseline weight 1.0) or appends floating last; **floating appends any source subtree intact** (source kind unrestricted — panel/tab group/split/host; the §8.11 case; `EmbedSlot::Floating { rect }` carries the drop's initial rect, `None` = the floating baseline; an invalid rect rejects upfront, naming the would-be entry index); **`PromoteHost`** is the explicit, intentionally *persistent* promotion (no-op success on an already-host dock); **`ResizeRegion { host, edge, requested, min }`** rewrites exactly one occupied region's weight (carried minimum, rejection policy, `RegionBoundViolation`; unoccupied regions reject `RegionUnoccupied`); **`SetFloatingRect { host, index, rect, min_w, min_h }`** rewrites exactly one entry's persistent rect (non-structural — no cascade; carried minimums inclusive, `FloatingBoundViolation`, outcome carries `previous`); wrap arms (region-occupant targets, tab-wraps in a host region or a floating slot) inherit the slot's stored state (occupant + weight; entry + rect); support types `SplitDirection`, `InsertSource`, `EmbedSlot` (`Region(edge)`/`Floating{rect}`); `UndoData` = former collection states (`FormerCollection` split/tab/region/floating-entries/panel/promotion/demotion), removed dock payloads (BFS), created ids, previous root + `pub(crate) replay_into` seam (Phase 9 undo); helpers `detach_dock`, `resolve_insertion_target` (tab-member chain up), `create_split_around`, `promote_to_host`, record_* snapshots (pub(crate); region/floating/promotion/demotion records shared with the normalization pass); `#[cfg(test)]` 21 fixtures (12 Phase-7/8: embed rejections/cycle, re-dock-in-own-host, promote/demote + undo round-trips, region + floating-rect bounds and undo, occupancy and initial-rect atomicity) via `structural_equal` |

**Registration:** `pub mod docking;` in `environments/mod.rs` (+ doc tree line), mention in `domain_foundation/mod.rs`.
**Tests:** `tests/domain_foundation_tests/docking_test.rs` (14) + `docking_validation_test.rs` (9) +
`docking_operations_test.rs` (24) + `docking_normalization_test.rs` (8) + `docking_sizing_test.rs` (16) +
`docking_embedding_test.rs` (12) + `docking_floating_test.rs` (15), registered in
`tests/domain_foundation_tests/main.rs`; in-crate `validation.rs` fixtures (23 + 6 serde) +
`operations.rs` fixtures (21) + `normalization.rs` fixtures (19).

**Engine integration (Phase 9) — `src/domain_engine/`:**

| File | Established |
|---|---|
| `operations/docking_operations.rs` | `ExecutedDockingOperation { operation, outcome, redo_record }` (the `ExecutedCanvasOperation` precedent): `execute(layout, op)` runs the pipeline once and builds the **forward replay record** from the committed post-state (see the settled engine decisions); `undo` replays `outcome.inverse()` via `UndoData::replay_into`, `redo` replays the forward record via the same driver, both `validate()` the restored layout; `history_entry()` names all 13 ops (`docking.create_panel` … `docking.set_floating_rect`) |
| `operations/docking_command_handler.rs` | App-engine bridge (`layers_command_handler` compressed template): `Arc<Mutex<DockLayout>>`, `CommandInput::get::<DockingOperation>()`, applies, appends `"docking.operation"` undoable entry + publishes `"docking.operation"`, `CommandResult::success_with(outcome)` (like its siblings it appends undoable without an engine-side inverse — the `DomainEngine` path is the real-undo route) |
| `operations/mod.rs` | `pub mod` both + re-exports beside the canvas/layers/timeline entries |
| `mod.rs` | `DomainError::Docking(DockingError)` + `From` + Display `"docking: {e}"`; engine field `docking: DockLayout` + `executed_docking: Vec<ExecutedDockingOperation>`; `docking()` reader; **`execute_docking`** (the `execute_canvas` shape 1:1: truncate → execute → transaction-tagged undoable entry → append → push → `enforce_history_limit` → publish); undo/redo/rollback_transaction docking branches (add-only — the canvas path untouched); helpers `docking_outcome_event` (13 arms — `docking.dock.created`, `docking.dock.inserted_beside`, `docking.tab.joined`, `docking.tab.reordered`, `docking.tab.active_selected`, `docking.dock.removed`, **`docking.split.resized`** (the plan-named shape), `docking.panel.collapsed_set`, `docking.dock.embedded`, `docking.host.promoted`, `docking.region.resized`, `docking.floating.rect_set`) + `docking_outcome_signal` (created/removed/resize/rect-set specifics, wildcard `"docking.changed"`) |
| `tests/app_engine_tests/docking_undo_redo_test.rs` | **17 docking engine tests** (registered in that suite's `main.rs`): execution basics + rejection/error plumbing, **id-preserving undo/redo cycles** (the forward-record redo pins same-ids), removal cascade, promotion round-trip, region weight / floating rect / collapse restores, transaction auto-rollback + committed-transaction unit undo, command handler shape |

**Engine additions (Phase 10) — history registry + persistence (`src/domain_engine/mod.rs`):**

| Established | Detail |
|---|---|
| `ExecutedStep` (private enum) + `executed_steps: Vec<Option<ExecutedStep>>` | **One position-indexed executed-step registry** replacing the per-env parallel vectors (the R-9-1 unification, compiled to engine code): `Some(step)` exactly when the history entry at that index carries a replay step, `None` gaps for plain-apply entries and non-replayable actions; `undo()`/`redo()`/`rollback_transaction()` walk history positions once and dispatch; `push_executed_step(pos, step)` pads gaps at execute time. Canvas/docking wrappers untouched; interleaved replay is pinned by `docking_history_registry_test.rs` (4 tests, ungated). |
| `restore_docking(layout) -> Result<(), DomainError>` | The **validated-adopt persistence entry**: `DockLayout::validate()` (the Phase-3 gate) before any engine mutation — invalid layouts declined wholesale (state/history/events untouched), valid ones adopted **verbatim incl. ids + the id counter** (Phase-1 §7.2 resolution: shipped, not re-issued), recorded as a **non-undoable** `"docking.restored"` entry, `mark_saved()`s, publishes `"docking.restored"`. The persisted form is the model's own serde derive; storage rides `StorageBackend` (no docking-specific framework). Pinned by `docking_persistence_test.rs` (4 tests, serde-gated): fresh-engine id-exact equivalence (`structural_equal` replica — the counter is monotone), gate rejection, two-layer malformed handling (parse vs. validate), backend round-trip. `serde_roundtrip_test.rs` +2: rich round-trip + the `"next_id"` carry via symmetric-mutation equality, and the empty layout. |

**Interaction contract (Phase 11) — specification + contract tests (no behavior code):**

| Established | Detail |
|---|---|
| `docking/mod.rs` doc section "Frontend handoff — the interaction contract (PRD §10, §11)" | Stage→API lifecycle table; region+mode → operation tables (split edges → `InsertBeside` directions; center∉split; tab center → `TabJoin`; embed edges → `EmbedSlot::Region`; embed body → `EmbedSlot::Floating { rect }`, `None` = baseline); advisory eligibility-discovery reads (kinds, `HostRegions::on`, `placement`, self-inclusive `is_descendant`) with the commit re-validating; the settled rejection-payload table; gesture commit boundaries (per-frame undo / transaction span). No drag-session file — the resolved plan warning. |
| `operations.rs` / `docking_command_handler.rs` doc cross-refs | The operation values are the contract (per-op gesture contracts are its resize/move clauses); the engine route `execute_docking` is the frontend-facing commit path (the command bridge stays the name-only sibling). |
| `tests/app_engine_tests/docking_interaction_contract_test.rs` (9 tests) | Pin the contract panels: mode/intent→op mapping (split edges, tab center, embed regions + baseline weight, floating drop rect/baseline), settled rejections with byte-equal-layout + zero-event/zero-history asserts (`IneligibleDock`, `RegionOccupied`, `NotSplittable`, `CycleDetected` self + host-in-source, `DockNotFound`), persisted form byte-equality across a rejection (serde-gated), gesture rollback frame-by-frame and via transaction spans (rollback whole / committed span undo-redo as one unit), and the full §10.1 walkthrough (source via `iter()`/`content()` read, eligibility discovery, one `TabJoin` commit, incompatible second drag clears cleanly). |


**Settled model decisions (do not relitigate; source: Ph.1/Ph.2 records):**
Root = current top-level dock, `None` = explicit empty layout; auto-root only from a fully empty
layout; one `Split` container with orientation (no separate row/column types); tab `active` is a
member index; collapse state (Phase 2 deferred it; delivered in Phase 6 — `Panel.collapsed`,
settled sizing block below); center-drop semantics + occupied-region policy =
reject-by-default; no parent back-pointers (placement by scan).
Undo integration will follow the `ExecutedCanvasOperation` pattern — see Ph.1 report §4.2 (R-A1).
Wrong path — restructure flow: `set_root(None)` → create → `set_root(new)`.

**Settled invariant decisions (Phase 3 — do not relitigate; source: `records/phase3_validation.md`):**
`validate()` is the steady-state authority (creation stays valid-by-construction); first-error
with deterministic ascending-`DockId` scan; rule precedence is root designation → ref validity +
category parity → placement multiplicity → colour-DFS acyclicity; reused creation-time variants
for identical rules; `CycleDetected` name follows `LayersError::CycleDetected` (not `LayoutCycle`);
root-owned-by-container reports `InvalidRoot`; transient unplaced/rootless states are *invalid
by design* when committed; reachability from root is derived (asserted in tests), not re-checked;
`InvalidSizeState` is index-based (error enum stays `Eq`).

**Settled operation decisions (Phase 4 — do not relitigate; source: `records/phase4_basic_structural_operations.md`):**
Value-type `DockingOperation`/`DockingOutcome` with `apply(self, &mut DockLayout)` (Layers
precedent); atomicity = candidate clone → `validate()` gate → swap (OP-01…OP-04); every outcome
carries full inverse `UndoData` (removed BFS payloads, former container collections, created ids,
previous root) replayed via the `pub(crate)` `replay_into` seam in Phase 9; mandatory cascade
only (split `<2` children collapses — survivor inherits the slot, slot weight untouched — empty
tab group deleted), full normalization is Phase 5; sizing baseline = new child copies the
target's weight, fresh containers uniform, detach deletes the weight (`prune_weight`); tab
baseline = joined member becomes active, reorder moves the selection with the member, member
removal shifts/clamps the active index, `from == to` reorder is a no-op success; insert-beside
resolves the target **up the tab-member chain** (splits at the outermost group so groups keep
every member), reuses compatible containers, wraps otherwise (root/split-child/host-edge slots);
floating insert targets are rejected `NotSplittable` (the settled Phase-8 baseline — it
stands); center-drop/swap is not a v1
operation (PRD §8.10); standalone "move" ops are unnecessary — new docks enter layouts as
`InsertSource::NewPanel` inside `InsertBeside`/`TabJoin`, existing docks re-place through the
same two ops; wraps need the `pub(crate)` `put_split`/`put_tab_group`/`set_root_unchecked`
installers (public creation preconditions reject placed children); undo round-trips compare
`structural_equal` (id counter monotone, like canvas undo); undo owns everything an op created
(`undo.created` includes `NewPanel` sources); wrap must capture the target slot **before**
installing the container (placement is derived by scan).

**Settled normalization decisions (Phase 5 — do not relitigate; source: `records/phase5_normalization.md`):**
One rule core (`resolve_below_minimum`), two drivers: `collapse_cascade` (rootward chain, undo
recording — the operation cleanup step) and `DockLayout::normalize` (public, undo-free
whole-layout fixpoint pass); canonical pipeline = mutate → normalize → validate → commit (the
normalization runs on the candidate; candidate→swap atomicity unchanged); below-minimum rules:
split <2 children collapses (survivor = first child inherits the slot; zero-child split removed,
nothing inherits), zero-member tab group removed, **≥1-member tab groups preserved** (tab
minimum is 1 — deliberate asymmetry with the split's 2; PRD §8.12 rule 3), panels are leaves,
embed hosts impose no child minimum; the root participates in the same rules (survivor becomes
root; last removal → explicit empty layout; Ph.2 root = current top-level dock); unplaced
below-minimum containers and dangling references are left for the Phase‑3 gate — normalization
does not guess slots; relative order of survivors preserved (a survivor copies into the exact
inherited reference; member removals shift by index only); weight policy: collapse keeps the
parent's slot weight, zero-survivor removal prunes the index's weight (`prune_weight` — survivors
keep shares verbatim, never rescaled), insert baselines stay with the insertion flows, bounds/
rescale/resize = Phase 6; `normalize()` selects the lowest-id placed below-minimum container
each round and rescans to fixpoint (deterministic + idempotent), records no inverse data, and
its steps check before mutating (mid-pass `InvalidSizeState` from garbage imported sizing leaves
completed steps applied, failing step untouched); layout fixtures carrying `f64::NAN` compare
via Debug format, not `PartialEq`.

**Settled sizing decisions (Phase 6 — do not relitigate; source: `records/phase6_sizing.md`):**
Stored split weights are **verbatim, never renormalized** (PRD §9.2 representation decision —
`uniform(n)` sums to `n`; the proportion `weightᵢ/Σweightⱼ` is a derived frontend read); the
resize operation is `ResizeSiblings { container, first, requested, first_min, second_min }`,
rewriting the **adjacent pair** `children[first]`/`children[first+1]` with pair conservation
(first := requested; second := pair total − requested; untouched children verbatim — no
membership change → no cascade step for this non-structural op); the gesture contract sends
**absolute requests** (the frontend derives each frame's requested weight from the gesture's
initial state + total displacement — frames compose drift-free; cancellation is frontend
state, rollback via the outcome inverse); minimum bounds (PRD §9.3) are
**operation-carried** (frontend-translated logical weights; v1 minimums only, maxima MAY),
bound endpoints inclusive; the conflict policy is **rejection** —
`SizeBoundViolation { split, index }` names the first-checked (ascending child index)
violating region, the sizing
invariant (finite, strictly positive) is always the implicit bound (0/negative/NaN/∞ reject
without carried bounds), infeasible totals reject every request deterministically, never
clamped; imported-garbage determinism: pair garbage → bound path, garbage outside the pair →
`InvalidSizeState` via the `prune_weight` precedent; collapse (PRD §9.7) is **panels only**,
one persistent `Panel.collapsed` flag (`serde(default)`) toggled by `SetDockCollapsed`
(matching-value sets are no-op successes with identity inverse; the flip is recorded as
`FormerCollection::Panel`) — collapse changes **no placement/membership/weighting**, the
dock's current weight is the retained restore state (a resize during collapse changes what
restore returns), collapsed members are ordinary members to normalization and never
unreachable; §9.5 region sizing routes to Phase 7, §9.6 floating sizing to Phase 8, §9.8
shared junctions stay deferred (all documented in the `sizing.rs` module doc).

**Settled embedding decisions (Phase 7 — do not relitigate; source: `records/phase7_embedding.md`):**
Hosts carry a **persistence dichotomy** (PRD §8.9): `EmbedHost.persistent` — `create_embed_host`
and the explicit `PromoteHost` produce **persistent** hosts (an empty persistent host is a valid
steady state, §8.8 — it stays a host when its last occupant leaves); `Embed`'s auto-promotion is
**temporary** (`persistent: false`, the panel was promoted solely to support that embed) and
**demotes** back to `Panel { content, collapsed: false }` through the shared normalization pass
(`needs_resolution`: empty regions *and* floating) — an in-place role change keeping identity,
content, and placement verbatim; no parent collection is rewritten; the panel-only collapse
preference does not carry across the boundary (v1 hosts are not a collapsible category) and the
`Promotion`/`Demotion` undo records rebuild the exact pre/post roles; serde default `true`
(pre-Phase-7 hosts restore persistent, never silently demote on import). `Embed` **rejects by
default** (`RegionOccupied` — no v1 replacement policy); host eligibility = embed host or panel
(auto-promoted inside the op), splits/tab groups reject `IneligibleDock` (no work area to
preserve); check order: host existence/eligibility → source existence/cycle → occupancy →
detach (cascade records) → promote → attach → normalize. The **cycle rule is asymmetric**: the
host inside the *source's* subtree (incl. source == host) rejects `CycleDetected { id: host }`;
a source inside the host's subtree re-parents into another slot of its own ancestor (region
occupant → floating) and is allowed. **Region weights attach to the slot, not the occupant**
(PRD §9.5): one verbatim weight per edge, establishes at the embedding baseline
`REGION_WEIGHT_BASELINE = 1.0` (uniform with the implicit core weight), carried forward by
slot-inheriting rewrites (wrap / collapse survivor), rewritten only by
`ResizeRegion { host, edge, requested, min }` (exactly one weight; each region resizes against
its inner boundary; gesture + constraint contracts mirror `ResizeSiblings` — absolute drift-free
requests, operation-carried minimums inclusive, **rejection** via `RegionBoundViolation`,
invariant implicit bound), and **reset to the baseline when the region clears** (weights live
with the region lifecycle; unoccupied regions have no preference and reject resizes
`RegionUnoccupied`). The **two-stage allocation read** (column top/core(1)/bottom, then row
left/content(1)/right inside the core band) is a derived frontend read — resizing one region
never rewrites another's stored weight; the core holds no stored weight in v1 (a content-side
minimum is a MAY maximum). **Removal outcomes** (PRD §8.11): removing a host removes the whole
arrangement (the BFS sweep already crosses host edges — regions in fixed order, then floating;
region weights go with the cleared slots); removing an occupant detaches its slot alone
(`FormerCollection::Region { host, edge, previous, weight }` restores occupant **and** weight
together on undo) and may demote a thereby-emptied temporary host through the cascade; content
lifetime stays with the host application either way. `EmbedSlot::Floating` appends last
(any source subtree intact — the Phase-8 settlement extends this) and floating docks stay
unreachable from directional insertion (`NotSplittable`; re-confirmed standing in Phase 8).
Imported-garbage region weights are the Phase-3 gate's steady-state
finding (`InvalidRegionState`; the `HostRegions` `serde(default)` restores absent weight fields
at the baseline so only explicit garbage imports); phase-7 serde default fns are
feature-gated (`default_effect` precedent).

**Settled floating decisions (Phase 8 — do not relitigate; source: `records/phase8_floating.md`):**
The floating layer is **host-anchored** (PRD §7.5, DM-07): every floated structure belongs to
exactly one host via `DockSlot::Floating { host, index }`; there is **no layout-level floating
collection** and no placement category beyond root/split-child/tab-member/host-edge/floating; a
frontend wanting a floating layer without a host creates one (`CreateEmbedHost`) or promotes
a panel (`PromoteHost`) first. **Floating preserves structure** (§7.6, §9.6 SHOULD): the
source detaches with its subtree intact — never flattened — and the source kind is
**unrestricted** (panel, tab group, split, host; only the host argument has eligibility
limits); the §8.11 "float from tab group" case is the member detach (the group keeps the rest
with the settled member-removal selection rule). **Position/size state** (PRD §9.6): one
`FloatingRect { x, y, w, h }` per floating entry — fractions of the host's **body** naming the
**dock body** rectangle (no decoration chrome; pixels reconstructed frontend-side), stored
verbatim under the sizing representation decision, host-bounded (`x`, `y` finite in `[0, 1]`;
`w`, `h` finite strictly positive; `x + w ≤ 1`, `y + h ≤ 1` — endpoints inclusive), attached
to the **slot, not the occupant** (the §9.5 decision extended): a rect-less append starts at
`FloatingRect::BASELINE = (0, 0, 0.5, 0.5)` (the `EmbedSlot::Floating { rect: None }` default
— the slot's `rect` field carries the drop interaction's initial rect, invalid initial rects
reject upfront naming the would-be index), `SetFloatingRect { host, index, rect, min_w, min_h }`
is the only update path (one entry, non-structural — no cascade; absolute drift-free gesture
contract; carried minimums inclusive; **rejection** via `FloatingBoundViolation`, never
clamped; outcome carries `previous`), slot-inheriting rewrites keep the entry's rect verbatim
(TabJoin wrap in a floating slot, a normalization survivor), member shifts carry each entry's
rect with the entry, and removal (or zero-survivor collapse) clears the rect with the slot.
Visual stacking/z-order stays **frontend-derived** (PRD §11.3 — collection order is placement
order, no v1 rearrangement op). **Re-docking** a floater runs through the same targeted ops
(`Embed` region target, `InsertBeside`, `TabJoin`) — the `NotSplittable` directional-target
baseline stands (tab-chain resolution to a floated group included). **Serde** (§16):
`FloatingRect` is struct-level `serde(default)` (absent fields restore the baseline), the
entry `rect` field defaults separately, and `FloatingRect` derives `Debug, Clone, Copy,
PartialEq` (f64 — no `Eq`/`Hash`, so `EmbedSlot` dropped those derives). Imported-garbage
floating rects are the Phase-3 gate's steady-state finding (`InvalidFloatingState`; regions
scan first, then floating entries by index).

**Settled engine-integration decisions (Phase 9 — do not relitigate; source: `records/phase9_domain_engine_integration.md`):**
Docking is the **second real-undo environment**, wired per the R-A1 mitigation as a
**parallel vector** `executed_docking: Vec<ExecutedDockingOperation>` with index-aligned
branches add-only in `undo()` / `redo()` / `rollback_transaction()` — the canvas replay path
untouched. The engine owns **one `DockLayout` instance** (`docking()` reader); the model's
mutators stay `pub(crate)`, so all external mutation flows through `execute_docking` → the
pipeline (mutate → normalize → validate → commit) — validation cannot be bypassed. Redo is
**NOT a re-apply of the operation** (dock ids are counter-issued, monotone, never renumbered —
re-apply drifts ids and corrupts the surrounding history): `ExecutedDockingOperation` carries
a **forward replay record** (`UndoData` — the environment's own record vocabulary) built at
execute time from the committed post-state — `removed` = full *post* payloads of created
**and** category-swapped (promotion/demotion) docks (whole-payload replacement is what
expresses an `EmbedHost` role verbatim), `containers` = *post* snapshots of every collection
the inverse names (deleted docks skip; an empty post region records `(None, None)` — the
replay's clearing path resolves the baseline), `previous_root` = the post root, `created` =
the inverse's removed ids; both undo and the forward record play through
`UndoData::replay_into` (a generic state-restore driver, used symmetrically) and both
`validate()` afterwards (exact replay of a committed state is valid by construction —
validation only surfaces replay drift). Redo id preservation is pinned by tests (same wrap
container / sibling id across cycles). Undo/redo publish `domain.undo` / `domain.redo` as for
canvas; per-operation events stay on the execute path (`docking.dock.*`,
`docking.split.resized`, …, wildcard signal `docking.changed`). The command handler uses the
compressed layers-family naming (`"docking.operation"`) and, like its siblings, appends an
`.undoable()` entry without a replay mechanism — acceptable for the bridge, never a
substitute for the wrapper path. **R-9-1 (inherited, Phase-10 scope):** per-env executed
vectors are position-aligned only per-env (name-only entries from *other* environments
already misalign canvas replay when interleaved — pre-existing); Phase 10 should unify the
alignment rather than stack a third vector. R-A2 held: outcomes carried complete inverse
data from day one — no model-snapshot fallback anywhere.

**Settled persistence/history decisions (Phase 10 — do not relitigate; source:
`records/phase10_history_persistence_restoration.md`):**
The persisted form is the **model's own serde derive** — the established serialization
convention (all sibling environments ship derive-only serde; `serde_json` stays a
dev-dependency; the engine never gains a format-specific save API): saving is
"clone the model and serialize it" at the host layer, restoring is
"parse the payload, hand the model to `restore_docking`". **Ids and the id counter ship
verbatim inside the form** (the Phase-1 §7.2 resolution, landed): never re-issued — a
restored layout is collision-free by construction (DM-02/DM-04) and continued mutation
issues fresh ids; the counter is monotone on undo, so **undo-round-trip comparisons after
restore are structural, not `PartialEq`** (the Phase-4 `structural_equal` shape; the
persistence suite ships a public-API replica via `iter()`). **`restore_docking` validates
before it adopts — rejection, never repair** (PRD §12.3): the Phase-3 gate *is* the
imported-state report, the gate runs before any engine mutation (rejections leave
layout/history/events untouched), and v1 defines **no** missing-/outdated-reference
recovery or migration. **No version envelope in v1** (PRD §12.2 strategy): the form's own
restore-compat design is the strategy — `serde(default)` baselines on absent optional
fields, serde tolerance of unknown fields, parse-reject for shape violations (the
caller-side boundary), validate-reject for invariants; no schema-version constant.
**A restore is a trusted import, not a reversible operation**: adopts append a
**non-undoable** `"docking.restored"` entry, `mark_saved()` after the append (the restored
file anchors the saved position; `is_dirty()` false post-restore), and publish
`"docking.restored"` (event + signal); the registry slot stays `None`; history before a
restore describes the pre-restore session, and a pre-restore undo whose replay drifts out
of validity surfaces `DomainError::Docking` from the Phase-9 replay validation guard; the
tested consumer posture is restore at document-open. **One position-indexed
executed-step registry replaces per-env vectors** (R-9-1 resolved):
`executed_steps: Vec<Option<ExecutedStep>>` with `Canvas`/`Docking` variants around the
untouched wrappers; `None` gaps for the plain-apply family and non-replayable actions;
undo/redo/rollback walk history positions once and dispatch — interleaved real-undo
environments replay in recorded order (this also fixes the pre-existing
canvas-under-layers misalignment); future real-undo environments add one registry
variant, never a vector. **R-9-2 stays a documented no-op** (deliberate): real history
trimming needs `HistoryStore` truncation semantics plus registry-offset handling, and
nothing consumes `with_max_history` today. **History persistence is metadata-only**:
`HistoryStore::persist_entry` records operation/source/undoable/tx through the optional
`StorageBackend`; inverse/replay data stays in-memory; full-state persistence rides the
environment's persisted form in a `StorageBackend` key-value entry (`docking.layout` in
tests) — no docking-specific persistence framework.

**Phase 10 delivered** (plan exit criteria, all met): docking changes undo/redo through
the existing history system (unified registry); a saved layout restores into an
equivalent valid domain state; missing/invalid references handled by explicit rules
(gate rejection); no docking-specific history/persistence framework introduced.

**Settled interaction-contract decisions (Phase 11 — do not relitigate; source:
`records/phase11_interaction_contract.md`):**
The contract is **specification, not new API** — no interaction/drag-session file, no
intent-translation type, no eligibility policy, no preview/observer surface; the
settled operation vocabulary, the public read API, `execute_docking`, engine
undo/transactions, and post-commit event publishing *are* the stages of PRD §10.1
(the stage→API table lives in the environment's module docs). **Mode + region →
operation is a pinned mapping** (§10.2–§10.3): split+edge = `InsertBeside` direction;
center is not a split intent; tab+center = `TabJoin` (wrap-or-join, joined member
active); embed+edge = `EmbedSlot::Region(edge)`; embed+body =
`EmbedSlot::Floating { rect }` (`None` = baseline, §9.6). An incompatible submitted
value rejects **as itself** — never silently executed as a different operation.
**Eligibility discovery is advisory** (§10.4): a frontend SHOULD pre-filter with the
read table (embed-capable kinds, `HostRegions::on`, non-floating placement,
panels-only v1 tab membership, self-inclusive `is_descendant`); the commit
re-validates everything and reports the settled typed payloads (frontend checks are
never a replacement for domain validation). **A rejection is the whole consequence**
(§10.5, §8.2): layout untouched, no history entry, no event, and the persisted form
byte-identical (pinned serde test — completion criterion 3). **Gesture commit
boundaries ride existing history** (§11.4): absolute-request frame commits; per-frame
`undo()` or a transaction span (`rollback_transaction` reverts whole in one call; a
committed span undoes/redoes as one unit through the executed-step registry). **The
vocabulary is input-method neutral** (§10.6): intent values only — no coordinates
anywhere.

**Phase 11 delivered** (plan exit criteria, all met): the frontend can express
docking intent with domain-level inputs (walkthrough test); the environment validates
and executes those requests without knowing about mouse coordinates; preview state is
not accidentally persisted as committed layout state. Behavior code added: none.

**Phase 12 delivered** (feature closed): the acceptance audit mapped PRD AC-01…AC-18
onto the existing suites (§3 record table) and found only the sequence-composition gap
— closed by `tests/app_engine_tests/docking_sequence_test.rs` (5 engine-level tests:
8-op split→move→tab→remove undo/redo round trip, persistent-promotion round trip,
auto-promotion + demotion trace, float + re-dock of a nested structure verbatim,
in-sequence rejections isolate). The public-API review fixed only stale docs
(`DockId` raw-mapping wording; `UndoData::replay_into` allow removed). The
documentation deliverable shipped: `md_docs/domain_docs/21-docking.md` (+ mkdocs nav,
overview/environments/public-api/error-reference sections, site-wide 8-environment
counts) — including the frontend-handoff section mirroring the Phase-11 contract.

## 3. What is next — the feature is complete; no further docking phase

Phase 12 delivered its plan scope: the acceptance audit (all ACs tested — the
record's §3 table), the missing sequence-level composition tests, the public-API
review, the docs/examples deliverable, and the full verification battery
(default + serde green; 10/9 pre-existing warning baselines unchanged). Exit
criteria (plan) — all met: all PRD acceptance criteria have tests, no structural
invariants violated, public operations have documented behavior + errors, the
environment works without a frontend and is ready for a UI adapter. Details in
§2 + `records/phase12_verification_and_release_readiness.md`.

**Starting a new feature?** Per the update protocol: refresh
`current_state/plan.md` + `current_state/PRD.md` for the new feature, reset this
checkpoint's status snapshot (and rewrite §§3–4 for the new work), and leave the
`records/` as history. If the new feature touches docking, verify the
Phase-12-established surface from `records/phase12_verification_and_release_readiness.md`
and `md_docs/domain_docs/21-docking.md` first — behavior code from Phases 2–12 needs
no re-derivation.

## 4. What needs to be checked (minimal reading list)

### 4.1 `current_state/` folder — role map

| Path | Role | When to read |
|---|---|---|
| `CHECKPOINT.md` | this file — state + next tasks | **always first** |
| `plan.md` | 12-phase plan; current phase's Tasks/Related files/Completion criteria | before each phase |
| `PRD.md` | requirements & acceptance criteria; Ph.1 resolved the 3 open architecture questions | on demand per feature |
| `records/phaseN_<name>.md` | completed-phase records (decisions, files, criteria check) | previous record before a new phase |
| `design/` | conceptual design basis for docking (browser docking spec + UI keypoints) | when a rule needs design justification |

Stateless knowledge (conventions, environment/engine recipes, testing, docs) lives in
`agents/` — routed by root `AGENTS.md`, maintenance in `agents/README.md`.

### 4.2 Code — for follow-up work on docking

The feature is closed. This reading list is the Phase-12 one, kept for continuity;
for a *new* feature start from its own plan/PRD (§3). Historical list:

1. `current_state/records/phase12_verification_and_release_readiness.md` and
   `records/phase11_interaction_contract.md` — the closing records (audit table,
   settled decisions; the Phase-12 record summarizes the whole surface).
2. `current_state/PRD.md` §17–§20 — the requirements/acceptance source.
3. The docking test files for the coverage map (start from
   `tests/domain_foundation_tests/main.rs` + `tests/app_engine_tests/main.rs`
   registrations and the per-file headers — behaviors are settled and pinned
   Phases 3–11).
4. `src/domain_foundation/environments/docking/mod.rs` (front-door docs incl. the
   interaction contract) — still the API-review front door.

**Not needed:** `sizing.rs`/`embed_host.rs`/normalization internals (settled), the
engine internals beyond `undo()`/`redo()` surface familiarity, windowing/rendering
(none exists by design).

### 4.3 Verification commands (run after every phase)

```bash
cargo test --test app_engine_tests -- docking                 # new docking engine tests
cargo test                                                    # full suite, 0 regressions
cargo check --features serde                                  # serde path compiles
cargo doc --no-deps                                           # no new doc warnings from docking
```

## 5. Update protocol (every phase)

1. Do the phase; run §4.3 commands (`agents/recipes/run_plan_phase.md` for the steps).
2. Write/extend `current_state/records/phaseN_<name>.md` (record: decisions, files, criteria check).
3. Update section 1 (status table + build state), section 3 (next phase tasks), section 4 as needed.
4. Bump "Last updated" + "Current phase" at top.
5. Fold any *new general convention* into the matching `agents/` file (stateless side).
