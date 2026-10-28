# Docking Environment — Phase 12 Record: Verification and Release Readiness

**Phase:** 12 of 12 (`current_state/plan.md` — verification and release readiness; **final phase**)
**Status:** ✅ Complete — feature shipped. Full suite green, serde path green, warning/doc baselines unchanged,
acceptance audit closed, docs published.
**Verification:**
- `cargo test` → **884 app_engine (879 + the 5 Phase-12 sequence tests) + 510 domain_foundation +
  66 lib tests pass**, 0 failures.
- `cargo test --features serde` → **889 app_engine + 516 domain_foundation (the 5 sequence tests +
  same-file serde-gated pins) + 73 lib tests pass**, 0 failures — including the docking
  `docking_roundtrip` / `docking_empty_layout_roundtrip` serde pins.
- `cargo check` / `cargo check --features serde` → clean; **10 pre-existing lib warnings**
  (locations verified this session: `app_engine` unused-import set, `scheduling/job.rs`
  `set_last_executed`, `relations/endpoint.rs` `ObjectId::from_raw` — none from `docking/`).
- `cargo doc --no-deps` → **9 warnings, all pre-existing Phase-2/3/7 leftovers**
  (same locations recorded since Phase 6: `embed_host.rs`, `model.rs` ×2, `slot.rs`,
  `docking/mod.rs:27`, plus relations/`node_graph`); the Phase-12 doc edits add none.

---

## 1. What was built

The closing phase: acceptance audit against PRD §18/§19, sequence-level integration
tests through the engine, a public-API review, and the documentation deliverable.

| File | Established |
|---|---|
| `tests/app_engine_tests/docking_sequence_test.rs` | **New — 5 tests** (engine-level, through `DomainEngine::execute_docking`): (1) `split_move_tab_group_remove_sequence_round_trips_history` — 8 ops (create, insert, move, tab join, reorder, set-active, resize, remove), position + outcome + event-count asserts, forward undo walk of all 8 and full redo replay (the §18.3 atomicity part 5/6 closes where earlier phases pinned the per-op case); (2) `explicit_persistent_promotion_undo_round_trip_keeps_identity_and_slot` — `PromoteHost` → embed → undo keeps id + slot + persisted host; redo reproduces; (3) `temporary_auto_promotion_demotes_when_its_last_occupant_leaves` — build trace shell→editor→inspector→output(wrap inner col)→TabJoin notes→resize inner `[1.5, 0.5]`→PromoteHost shell→float inner(rect Some)→insert host_b→PromoteHost host_b→Embed inner→host_b Region(Top)→float palette(None), re-docking the floater leaves the emptied *persistent* host valid (`promoted: false` asserts because both hosts were pre-promoted); (4) `floating_and_redocking_a_nested_structure_preserves_it_verbatim` — float a split+tab subtree (rect Some), `SetFloatingRect`, re-dock through `Embed`+`Region` — structure, weights, members, active member all byte-equal before/after; (5) `invalid_requests_inside_a_sequence_reject_without_touching_anything` — two invalid ops inside the composed sequence (self-target move on a non-tab-member dock; floating directional insert) reject with the typed error while layout/history/events stay untouched. |
| `tests/app_engine_tests/main.rs` | Registered `docking_sequence_test` (after `docking_undo_redo_test`). |
| `src/domain_foundation/environments/docking/dock.rs` | API-review doc fix: `DockId` struct + `from_raw`/`as_raw` docs no longer claim ids are non-constructible — the raw mapping is public so hosts can key content stores against serialized ids; ids are environment-issued for *new* docks and ship verbatim through persistence. |
| `src/domain_foundation/environments/docking/operations.rs` | API-review fix: removed the now-stale `#[allow(dead_code)]` on `UndoData::replay_into` (engine-consumed since Phase 9). |
| `md_docs/domain_docs/21-docking.md` | **New — the docs deliverable**: format-consistent sibling of `06-layers.md` — overview mermaid; key-types table; dock categories (host persistence, collapse); `DockLayout` method tables (construction / access / placement & discovery / validate–normalize); the five `DockSlot`s; the 13-operation outcome/undo table; insert-beside, tab, splitter (conserved pair total, rejection policy), embedding (temporary vs persistent promotion), floating (host-fraction body rect, re-dock paths) semantics; normalize/validate pipeline; the interaction-contract summary (mode→op + eligibility + gesture boundaries); engine integration (events table, serde note); a full build-walkthrough example with reachable numbers (`[1.0, 1.0] → insert copies weight → resize [1.5, 0.5] → tab-wrap → float → re-dock`) plus the engine-path undo example; "does not own" table; test-suite map. |
| `mkdocs.yml` | Nav entry `Docking: domain_docs/21-docking.md` in the Domain Foundation → Environments section. |
| `md_docs/domain_docs/01-overview.md` | Intro sentence, environments mermaid node, environment table row, environment-composition diagram node, comparison table row, denominator fixes (7→8), next-steps link. |
| `md_docs/domain_docs/03-environments.md` | Mermaid node, glance-table row (types/operations/queries), environments tree text block, traits-on-environments exception row (Docking has no `IntoIterator` — iterate via `iter()`/`all_docks()`), next-steps link. |
| `md_docs/domain_docs/18-public-api-reference.md` | **Docking section** (after TextEditor): all 19 public types with kinds/fields, the 21 `DockLayout` methods in one line, creation preconditions, "slots are operation-owned" note, `Default for DockLayout` trait line, denominator fix in the `DomainEngine` row. |
| `md_docs/domain_docs/19-error-reference.md` | **DockingError section**: full 29-variant listing grouped by phase (identity & creation / invariants / operations / splitter sizing / embedding & floating), each with fields + when + how-to-handle; the `Display` strings table (verbatim from `error.rs`); the rejection-not-clamping note; the atomicity note; `DomainError` enum gains `Docking(DockingError)`; summary table row (29 variants, `Eq` ✅) + wrapped-count 8. |
| `md_docs/index.md`, `md_docs/domain_docs/02-architecture.md`, `13-domain-engine-integration.md`, `20-quick-start.md` | Environment-count and engine-environment-node updates (7→8) so no site page claims fewer environments than exist. |

Behavior code changed beyond the two doc-comment fixes: **none** — no operation, rejection
policy, state type, or engine semantics were touched.

## 2. Decisions made during implementation (do not relitigate)

1. **The acceptance audit closed clean: every PRD §18 requirement / §19 criterion was
   already pinned by the Phase 2–11 suites.** The one genuine gap was *sequence-level
   composition* — earlier phases pinned each operation in isolation plus the §18.3
   atomicity checklist per op, but nothing exercised the split → move → tab-group →
   remove full-lifecycle UndoData chain, host promote/demote inside a composed trace,
   or float + re-dock of a nested structure through the engine. That gap is what the
   new suite covers — not a re-test of suite content.
2. **Sequence tests go through the engine, not `apply`** (PRD §18.7): the five tests
   drive `DomainEngine::execute_docking` so undo/redo positions, event streams, and
   history counts are asserted the way a UI adapter would observe them. They live in
   `tests/app_engine_tests/` next to the other engine suites.
3. **Semantics used while writing the tests, verified against live code**: same-row
   `InsertBeside` reuses the container and the new child *copies the target's weight*
   (`operations.rs` attach step); tab-member targets resolve up the member chain, so
   self-target rejection tests need a non-tab-member dock; pair-total conservation
   means a 2-child split can never reach `[3.0, 1.0]` (used `[1.5, 0.5]`); tab members
   are read via `Dock::structural_children()`, not `children()`; unresolved rejections
   leave layout, history, and events untouched (re-pinned once for the composite case).
4. **The docs deliverable is a standalone page + cross-reference updates**, following
   the established per-environment format (`06-layers.md` sibling), with the plan's
   example captured as a numerically-verified build walkthrough. Cross-references
   touched: overview, environments, public-api reference, error reference, nav, plus
   environment-count denominators on index/architecture/engine-integration/quick-start.
5. **Error reference counts 29 variants** (grouped by the phase that introduced them),
   each with its verbatim `Display` string — richer than the sibling pages because the
   settled rejection payloads are part of the frontend contract (PRD §10.5).
6. **Warning baselines are pre-existing and stay**: 10 lib/compile warnings and 9 doc
   warnings, all located outside Phase-12 files and recorded unchanged since Phases 5–8.
   The Phase-12 rule "new files add zero warnings" holds (the sequence suite and docs
   add none). Fixing the nine old doc warnings was ruled out of scope — they predate
   this phase and keep the baseline-comparison discipline meaningful.
7. **Public-API review findings were doc-fixes only**: (a) stale
   `#[allow(dead_code)]` on `UndoData::replay_into` removed (engine consumes replay
   since Phase 9); (b) `DockId` raw-mapping docs corrected (host keying is a
   documented, sanctioned use; ids ship verbatim on restore). The doc-comment audit
   confirms every public type, and all 21 `DockLayout` methods carry behavior +
   rejected-error documentation; operations.rs documents every variant with its
   constraint/rejection policy and gesture contract.

## 3. Acceptance-criteria audit (PRD §19 → suites)

| AC | Criterion | Pinned by |
|---|---|---|
| AC-01 | Built-in environment integration | `docking_persistence_test`, `docking_undo_redo_test`, `docking_history_registry_test` (engine accessors + `execute_docking` + events/transactions; serde gating per §13) |
| AC-02 | Valid logical layout, created + inspected headless | `docking_test` (empty layout explicit, first panel auto-roots, all slot categories reported), `docking_validation_test` |
| AC-03 | Stable identity through structural ops | `docking_test::identities_are_stable_and_not_renumbered`, `docking_sequence_test` (id-keyed asserts across 8 ops), `docking_persistence_test` |
| AC-04 | Row/column splits: valid structure, defined order + sizing | `docking_test::create_split_*`, `docking_operations_test::insert_beside_*` (compatible-reuse, wrap, root-wrap, direction order), `docking_sizing_test` |
| AC-05 | Tab grouping + valid active selection after membership changes | `docking_operations_test::tab_join_*`, `reorder_tab_*`, `set_active_tab_*`, `remove_tab_member_*`; `docking_floating_test::floating_a_tab_member_*` |
| AC-06 | Embedding without replacing the host's work area | `docking_embedding_test` (region placement with the work-area content preserved, occupied-region rejection, removal keeps host, re-dock within own host) |
| AC-07 | Floating assignment preserving identity + internal structure | `docking_floating_test::floating_a_subtree_preserves_its_structure_verbatim`, `floating_a_whole_tab_group_preserves_members_and_redocks_through_tab_join`, `docking_sequence_test::floating_and_redocking_a_nested_structure_preserves_it_verbatim` |
| AC-08 | Safe movement: descendants preserved, no cycles | `docking_operations_test::insert_beside_moves_whole_container_beside_target` / `..._moves_existing_dock_preserving_identity` / `..._rejects_self_and_descendant_targets`, `tab_join_rejects_missing_ineligible_and_cyclic`, `docking_embedding_test::embed_into_a_split_or_tab_group_is_ineligible_and_a_cycle_rejects` |
| AC-09 | Normalization preserves meaningful groups + root | `docking_normalization_test` (cascades, survivor weight inheritance, idempotence, no-op on valid layouts, explicit-empty result), `docking_floating_test::removing_the_last_floater_demotes_the_temporary_host`, `docking_sequence_test::temporary_auto_promotion_demotes_when_its_last_occupant_leaves` |
| AC-10 | Sizing as coherent logical state | `docking_sizing_test` (pair conservation, complements, endpoint-inclusive bounds, frames compose, collapse retains live allocation, restore returns pre-collapse layout), region sizing in `docking_embedding_test`, rect sizing in `docking_floating_test`, persistence pins in `docking_persistence_test` |
| AC-11 | Atomic failure leaves valid layout unchanged | `docking_operations_test::create_on_populated_layout_is_rejected_atomically`, `remove_unknown_dock_leaves_state_untouched`, `docking_normalization_test::failed_operation_leaves_original_state_unchanged`, `docking_sizing_test::resize_rejects_a_bound_violation_...`, `docking_embedding_test::occupied_region_is_rejected_and_layout_untouched`, `docking_interaction_contract_test` (byte-equal layout + zero events/history per rejection), `docking_sequence_test::invalid_requests_inside_a_sequence_reject_without_touching_anything` |
| AC-12 | Validation detects approved violations | `docking_validation_test` (pure check; orphans, root rules, cycles, storage gating), `docking_sizing_test::resize_preconditions_are_reported`, restore-gate tests in `docking_persistence_test` |
| AC-13 | Persistence readiness, incl. imported-state validation | `docking_persistence_test` (weights/region/floating/collapse/ids/order verbatim; malformed-payload rejection), serde `docking_roundtrip` + `docking_empty_layout_roundtrip` |
| AC-14 | Reversible operations work with established undo/redo | `docking_undo_redo_test`, `docking_history_registry_test` (transactions), `docking_sequence_test::split_move_tab_group_remove_sequence_round_trips_history` (+ explicit-promotion round trip) |
| AC-15 | Headless testability | all 9+ docking suites run browser-free/DOM-free (plus the whole-crate no-UI rules) |
| AC-16 | No required rendering/pointer machinery | mod.rs boundaries ("deliberately does not own"), `docking_interaction_contract_test` pins intent+eligibility+commit without pointer coordinates and the frozen persisted payload |
| AC-17 | Public API consistency with sibling environments | Phase-12 review (§2.7): value-type ops with `apply`, outcomes carry inverse, serde-gated derives, env-owned error with `Display`+`std::error::Error`, documented preconditions — all matching the `Canvas2D`/`Layers`/`Timeline` precedents |
| AC-18 | The full verification battery passes | this record's **Verification** block (default + serde, both green) |

§18.1–§18.8 model/operation/atomicity/sizing/persistence/history/integration/contract
requirements are covered by the same suites (the contract prototype requirements of
§18.8 by `docking_interaction_contract_test`, which walks the §10.1 lifecycle end to
end).

## 4. Exit-criteria checklist (checkpoint §3)

- [x] **All PRD acceptance criteria have tests** — §3 audit table, AC-01…AC-18 all mapped.
- [x] **No invariants violated** — default + serde suites green; no new warnings on either gate.
- [x] **Public operations documented with behavior + errors** — rustdoc audit clean; docs site has the full API/error reference sections.
- [x] **Environment ready for the UI adapter** — the Phase-11 interaction contract is pinned and now mirrored in `21-docking.md` (Frontend Handoff section); a frontend needs no domain-side drag state, and every rejection names what to re-scope.

## 5. Feature-complete state

The 12-phase Docking Environment plan is exhausted. Summary of the shipped surface:
`DockLayout` (identity, four dock categories, five slot kinds, traversal, validate/normalize),
13 structural/sizing operations with candidate-validated atomicity and full inverse data,
embed-host lifecycle (temporary vs persistent promotion, demotion via normalization),
floating collections with host-fraction persistent rects, serde persistence with an
imported-state gate, engine integration (`execute_docking`, undo/redo, transactions,
events/signals), the settled frontend interaction contract, and this documentation set
(1 new page + 4 enriched reference pages + site-wide denominator fixes).

Next feature starts fresh: refresh `current_state/plan.md` + `PRD.md` for the new feature
and reset the checkpoint's status snapshot at that time (records stay as history).
