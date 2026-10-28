# Docking Environment — Phase 4 Record: Basic Structural Operations

**Phase:** 4 of 12 (`current_state/plan.md` — Basic structural operations)
**Status:** ✅ Complete — full suite green, serde path green, warnings baseline unchanged
**Verification:**
- `cargo test` → **850 app_engine + 459 domain_foundation (integration) + 31 in-crate lib tests pass**, 0 failures
  (docking external total 47: 14 model + 9 validation + **24 new operation tests**; lib docking 28 = 23
  validation fixtures + **5 new in-crate operation tests**)
- `cargo test --features serde` → same suites green (463 / 35 lib); operations types all serde-gated
- `cargo check --features serde` → clean; 10 pre-existing warnings, none from `docking/`
- `cargo doc --no-deps` → no warnings from Phase‑4 files (two private-item doc links to
  `UndoData::replay_into` fixed during the sweep); remaining doc warnings are the pre-existing
  Phase‑2/3 leftovers in `embed_host.rs`/`model.rs`/`slot.rs`/`mod.rs` (untouched)

---

## 1. What was built

New file `src/domain_foundation/environments/docking/operations.rs` + additions to
`model.rs`, `sizing.rs`, `error.rs`, `mod.rs`. External test file
`tests/domain_foundation_tests/docking_operations_test.rs` (24 tests) registered in `main.rs`.

| File | Established |
|---|---|
| `operations.rs` | [`DockingOperation`] — value-type enum, one-method [`apply`](DockingOperation::apply) contract (the `LayersOperation` precedent): `CreatePanel`, `CreateEmbedHost`, `InsertBeside`, `TabJoin`, `ReorderTab`, `SetActiveTab`, `Remove` (PRD §8.3/§8.6/§8.7/§8.11). Support types: `SplitDirection` (Left/Right/Top/Bottom → Row/Column + `inserts_after`), `InsertSource` (`Existing(id)` moved / `NewPanel{content}` created-and-placed immediately). [`DockingOutcome`] — 6 variants (`Created`, `InsertedBeside`, `TabJoined`, `TabReordered`, `ActiveTabSelected`, `Removed`), each carrying [UndoData](UndoData). Atomicity: `apply` clones the layout into a **candidate**, mutates it freely (transient rootless/collapsed states allowed mid-flight), then `validate()`s (Phase 3) before committing by swap (OP-01…OP-04). `#[cfg(test)] mod tests`: 3 inverse round-trips (inserted-beside, remove-with-cascade, tab-wrap-reusing-parents) + region-occupant round-trip + floating rejection, via the `pub(crate)` `UndoData::replay_into` seam and a `structural_equal` comparator |
| `operations.rs` (undo) | `UndoData` — the inverse of every op: `containers: Vec<FormerCollection>` (per mutated container, in mutation order), `removed: Vec<Dock>` (payloads deleted: removed subtrees BFS + cascade-destroyed containers), `created: Vec<DockId>` (docks the op created, delete on undo), `previous_root`. `FormerCollection` = `Split{children,sizes}` / `Tab{members,active}` / `Region{host,edge,previous}` / `Floating{entries}`. `replay_into` is the `pub(crate)` seam for Phase 9's `ExecutedDockingOperation` (root restored before created docks deleted; containers restored newest-first) |
| `operations.rs` (helpers) | `detach_dock` (removes a placed dock from its slot, recording the former owner's collection), `collapse_cascade` (mandatory normalization, see §2.4), `prune_weight` (delete weight at index; `InvalidSizeState` only for imported state), `record_split`/`record_tab`/`record_region`/`record_floating` (undo bookkeeping), `resolve_insertion_target` (walk a tab member up to its group), `create_split_around` (uniform split wrapping target+source) |
| `model.rs` | `pub(crate)` installers for the operations: `put_split`/`put_tab_group` (create a *placed-children-wrapping* container without the creation-time preconditions — see §2.8), `set_root_unchecked` (candidate-phase root write; public `set_root` keeps its checks), `restore_collection(&FormerCollection)` (undo replay; private) |
| `sizing.rs` | `pub(crate) fn insert_weight(index, weight)` — insert into a split's weights at the child index (`children.insert` in lockstep); sibling weights untouched |
| `error.rs` | 4 new variants (+ `Display` arms): `IneligibleDock { id }` (a dock of the wrong category for the requested role — v1 tabs are panels only), `NotSplittable { id }` (a dock without a directional layout surface — floating targets in v1), `NotATabGroup { id }` (a tab op aimed at a non-group dock), `TabIndexOutOfRange { index, member_count }` (reorder index against the pre-removal member count — no f64 payload, enum stays `Eq`) |
| `mod.rs` | `pub mod operations;` + re-exports (`DockingOperation`, `DockingOutcome`, `SplitDirection`, `InsertSource`, `UndoData`, `FormerCollection`) + owns/not-owns doc bullets |

### Operation contract summary (pre → post, per PRD §8)

| Operation | Preconditions (check order) | Postconditions | New errors |
|---|---|---|---|
| `CreatePanel` / `CreateEmbedHost` | none (auto-root per PRD §7.4 on the fully empty layout; on a populated layout the created dock would be unplaced) | dock created, `Created { is_root }` or gate rejection | — (`validate` → `OrphanDock`) |
| `InsertBeside` | target exists; for `Existing` source — exists, resolved target not source/self-descendant (`is_descendant`, Layers precedent) | source beside target in a reused compatible container (weight copied) or a fresh split inheriting the target's slot (root / split-child / host-edge); former parent left valid by cascade | `DockNotFound`, `CycleDetected`, `NotSplittable` |
| `TabJoin` | target exists and is a panel; for `Existing` source — exists, is a panel, not source-or-in-its-subtree | source joins target's group (last member, becomes active) or a group is created around both inheriting the target's slot; already-member = no-op success | `DockNotFound`, `IneligibleDock`, `CycleDetected` |
| `ReorderTab` | group exists (`contains` → `kind_of`), `from`/`to` valid against the pre-removal member count; `from == to` no-op success | member moved, its selection follows it | `DockNotFound`, `NotATabGroup`, `TabIndexOutOfRange` |
| `SetActiveTab` | group exists and is a tab group; `index < members.len()` | active member selected | `DockNotFound`, `NotATabGroup`, `ActiveIndexOutOfRange` |
| `Remove` | dock exists | dock + its descendant subtree removed in BFS order (outcome `docks`), cascade keeps the layout valid; root removal yields the explicit empty layout | `DockNotFound` |

Composite errors (e.g. `OrphanDock` in `detach_dock`) fire only for defensive mid-candidate
states; the Phase-3 gate is the last authority for every committed result.

## 2. Decisions made during implementation

1. **Value-type enum pair, `apply(self, &mut layout)` contract.** Same shape as
   `LayersOperation`/`CanvasOperation` (CHECKPOINT directive; PRD §8.1): the operation value
   describes *what the change means*, not a call sequence, and is `Clone`/`PartialEq`-able.
2. **Candidate-clone atomicity.** `apply` runs the whole mutation pipeline on a clone,
   validates the result with the Phase‑3 gate, and swaps on success (PRD §8.2, OP-01…OP-04).
   Transient rootless/unplaced states are legal **on the candidate only** — committing one
   is impossible by construction.
3. **Full inverse data from day one** (Ph.1 §4.2 R-A1/R-A2). Every outcome's `UndoData` was
   round-tripped by an in-crate test through the `pub(crate)` `replay_into` seam; a
   `Removed`-with-cascade and a wrap-reusing-parents case cover both collection-rewrite
   kinds. Round-trips compare **structural equality, not whole-struct equality**: the id
   counter is monotone (never reset — the `Canvas::clear`/`Layers::clear` precedent), so
   undo of a *created* dock legitimately leaves a higher counter, exactly like undoing a
   canvas object's creation.
4. **Mandatory cascade is Phase‑4 minimal, Phase‑5 complete** (PRD §8.12 rules 1–2 only):
   after a detach, a split with fewer than two children collapses (survivor inherits the
   split's slot, the split's token is deleted, the parent slot keeps its weight) and a tab
   group that lost its last member is deleted with nothing inheriting. One deterministic
   rootward chain. Full normalization (ordering preservation guarantees, idempotence pass)
   is Phase 5's `normalization.rs`.
5. **Sizing baseline** (fine transfer policy stays Phase 5/6, PRD §9.2): a child inserted
   into a compatible container **copies the target's weight** (siblings unchanged — source
   and target share the target's former area); fresh containers are uniform; detaching
   deletes the moved child's weight (`prune_weight` — relative weights mean survivors keep
   their shares); a collapsing split's slot weight is untouched.
6. **Tab index semantics** (PRD §8.7): a joined member becomes the active one; `ReorderTab`
   follows the Layers `reorder_sibling` convention (`to` is an insertion index against the
   pre-removal count, `from == to` is a no-op) and the moved member's selection follows it;
   removing a member shifts/clamps the active index; `SetActiveTab` reuses the
   `ActiveIndexOutOfRange` variant.
7. **Insert-beside target resolution.** The target is resolved **up the tab-member chain**
   first (an insertion splits at the outermost tab boundary, so a group never loses
   members); a compatible container is reused, otherwise a new split *wraps* the target and
   inherits its slot. Occupied-region policy per plan: dropping beside a **host-edge
   occupant** splits that region explicitly (the region occupant becomes the split's
   child); dropping beside a **floating** dock has no v1 meaning — rejected
   `NotSplittable` (Phase 8); center-drop/swap is **not** an operation in v1 (PRD §8.10).
8. **`put_split`/`put_tab_group` installers.** The public creation primitives
   (`create_split`/`create_tab_group`) preconditio­nally reject *placed* children — which
   the wrap case always wraps. The `pub(crate)` installers drop those creation-time checks
   (the operation pipeline owns the preconditions; the Phase‑3 gate checks the committed
   result). `set_root_unchecked` exists for the same reason (the root write in operations
   is already justified by the pipeline).
9. **Undo owns everything an op created.** A `NewPanel` source inside `InsertBeside`/
   `TabJoin` is recorded in `undo.created` the moment it is allocated, so a replay deletes
   it — found and fixed during the verification sweep (the region-occupant round-trip
   initially left the created source orphaned after undo).
10. **Slot capture before wrap.** Because placement is *derived by scan*, a wrap must read
    the target's original slot **before** installing the container over it (reading after
    would see the new membership). Found and fixed during the sweep by the
    `tab_join_detached_source_collapses_former_parent` test; `InsertBeside` was already
    correct (it matches on the slot before mutating).

## 3. Completion criteria check (current_state/plan.md Phase 4)

- [x] Each operation has defined preconditions and postconditions — per-op doc comments
      enumerate the exact check order and post-state; the record table above summarizes.
- [x] Moving an element preserves its identity —
      `insert_beside_moves_existing_dock_preserving_identity` (same `DockId` after the
      move, subtree intact).
- [x] Invalid moves are rejected — per-op rejection tests
      (`create_on_populated_layout_is_rejected_atomically`,
      `insert_beside_rejects_missing_targets_and_sources`,
      `insert_beside_rejects_self_and_descendant_targets`,
      `tab_join_rejects_missing_ineligible_and_cyclic`,
      `insert_beside_floating_target_is_rejected`, reorder/set-active range rejections,
      `remove_unknown_dock_leaves_state_untouched`), each asserting `layout == before`
      (OP-04).
- [x] Basic operations maintain the Phase‑3 invariants — `apply` cannot commit without
      `validate()` passing; success-path tests assert `layout.is_valid()`; inverse
      round-trips restore `structural_equal` states.

Public API surface: `DockingOperation`/`DockingOutcome` (+ support types) applied on the
layout, four new `DockingError` variants; the model gained no *public* mutation surface
(the installers are `pub(crate)`).

## 4. Files touched

```
src/domain_foundation/environments/docking/operations.rs          (new — 7 ops, 6 outcomes,
                                                                  UndoData/FormerCollection,
                                                                  helpers, 5 in-crate tests)
src/domain_foundation/environments/docking/model.rs               (+4 pub(crate) installers:
                                                                  put_split, put_tab_group,
                                                                  set_root_unchecked,
                                                                  restore_collection)
src/domain_foundation/environments/docking/sizing.rs              (+insert_weight)
src/domain_foundation/environments/docking/error.rs               (+4 variants, +Display arms)
src/domain_foundation/environments/docking/mod.rs                 (pub mod operations +
                                                                  re-exports + doc bullets)
tests/domain_foundation_tests/main.rs                             (mod docking_operations_test;)
tests/domain_foundation_tests/docking_operations_test.rs          (new — 24 tests)
```

## 5. Next: Phase 05 — Normalization and atomic mutations

Per `current_state/plan.md` Phase 5 + PRD §8.12 + PRD §9.2 transfer rules:

1. `normalization.rs` with a `pub fn normalize(&mut self) -> Result<(), DockingError>`-style
   entry (name per conventions at implementation time): fold the mandatory cascade rules
   and add the full deterministic rules — empty-container removal, one-child split
   collapse, **relative ordering of surviving children preserved**.
2. Idempotence: repeated normalization must not keep changing an already normalized layout
   (plan criterion); normalization never silently discards valid panels or tab groups.
3. Establish the canonical pipeline order for every operation — mutate → normalize →
   validate → commit — moving the Phase‑4 cascade calls into the shared pass, and fold the
   Phase‑4 sizing baselines (weight copying/removal, slot-weight inheritance) into the
   documented redistribution policy.
4. Atomicity stays as built (candidate → validate → swap); a failed operation must leave
   the original state unchanged — keep the Phase‑4 suites green and add
   `tests/domain_foundation_tests/docking_normalization_test.rs` (name aligned with
   conventions).
