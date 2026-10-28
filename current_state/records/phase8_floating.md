# Docking Environment — Phase 8 Record: Floating Docking Structures

**Phase:** 8 of 12 (`current_state/plan.md` — Floating docking structures)
**Status:** ✅ Complete — full suite green, serde path green, warning baselines unchanged
**Verification:**
- `cargo test` → **850 app_engine + 510 domain_foundation (integration) + 66 in-crate lib tests
  pass**, 0 failures (docking external total **98**: 14 model + 9 validation + 24 operation +
  8 normalization + 16 sizing + 12 embedding + **15 new floating tests**; lib docking in-crate
  **63** = 23 validation fixtures + **19 in-crate operation tests** (+2 Phase-8) + **19
  normalization fixtures** (+1 Phase-8), plus the small non-docking in-crate set (3); the
  "0 passed" empty main/benches harness lines are pre-existing)
- `cargo test --features serde` → same suites green (**850 / 514 / 73 lib**); the serde gain is
  the `FloatingRect` struct-level `serde(default)` (absent fields restore the baseline), the
  entry `rect` field default (`None`-rect entries restore baseline), and the new
  imported-garbage floating fixture (explicit garbage trips `InvalidFloatingState` alongside
  the 6 pre-existing serde validation fixtures)
- `cargo check` / `cargo check --features serde` → clean; **10 pre-existing warnings**, none
  from `docking/`
- `cargo doc --no-deps` → **9 warnings, all pre-existing** Phase-2/3/7 leftovers
  (`embed_host.rs`/`model.rs`×2/`slot.rs`/docking `mod.rs` + relations/`node_graph`); the two
  Phase-8 redundant-link drafts were fixed before recording, none remain

---

## 1. What was built

Floating docking structures (PRD §7.5–§7.6, §8.11, §9.6, §11.3): the per-host floating
collection as the settled floating-root representation, one persistent logical
[`FloatingRect`] per floating entry (position *and* dimensions in host-body fractions), the
`SetFloatingRect` operation, structure-preserving floating of any eligible subtree (never
flattened), re-docking through the same targeted operations, and the removal/normalization
interactions extending the established slot-attached state rules.

| File | Established |
|---|---|
| `floating.rs` | **New module** — `FloatingRect { x, y, w, h }` (fractions of the host's body; the **dock body** rectangle — decoration is frontend-applied, PRD §9.1/§9.2): stored verbatim under the sizing representation decision; invariants `x`/`y` finite in `[0, 1]`, `w`/`h` finite strictly positive, `x + w ≤ 1`, `y + h ≤ 1` (host-bounded, PRD §9.6 SHOULD; bound endpoints inclusive); `FloatingRect::new` (validating constructor returning `Option`), `is_valid`, part getters `x()`/`y()`/`w()`/`h()`, `BASELINE = (0, 0, 0.5, 0.5)` with `Default` = baseline; `FloatingEntry { dock, rect }` — one entry per floated subtree root, `rect()` read and `pub(crate)` `set_rect()` (operation territory); module doc carries the §9.6 transfer table (initialized / updated / transferred / cleared) mirroring the §9.2/§9.5 tables |
| `dock.rs` | `EmbedHost { floating: Vec<FloatingEntry> }` — the floating collection is now the typed-entry API (each entry = floated root **+** its persistent rect); `floating()` read accessor (`Option<&[FloatingEntry]>`); `structural_children` sweeps
placement categories (regions fixed order, then floating entries) unchanged |
| `operations.rs` | `EmbedSlot::Floating { rect: Option<FloatingRect> }` (was the unit variant) — the slot carries the drop interaction's initial rect, `None` = baseline; invalid initial rects reject the embed upfront (`FloatingBoundViolation`, naming the would-be entry index — before any detach); `SetFloatingRect { host, index, rect, min_w, min_h }` + `apply_set_floating_rect`: exactly one entry's rect rewritten (dock, placement, and sibling entries verbatim — the non-structural `ResizeRegion` write shape; no cascade); outcome `FloatingRectSet { host, index, previous, inverse }`; `FormerCollection::Floating { host, entries }` replaces the Phase-7 floating record (entries restore together); `Embed`'s floating attach arm pushes `FloatingEntry::new(source_id, rect)`; `TabJoin`'s wrap-in-floating-slot rewrites the entry's dock (rect inherited — the §9.6 slot inheritance); the carried minimums reject inclusively (rejection policy — NaN/∞ minimums unsatisfiable); module docs extended (floating bullets in the scope section + `EmbedSlot::Floating`/`SetFloatingRect` docs) |
| `error.rs` | Three variants: `FloatingIndexOutOfRange { index, entry_count }` (the floating-side `TabIndexOutOfRange` sibling), `FloatingBoundViolation { host, index }` (the rejection policy — the `RegionBoundViolation` mirror), `InvalidFloatingState { host, index }` (imported-garbage steady-state finding — the `InvalidRegionState` mirror); Display strings follow the established style |
| `model.rs` | `restore_collection`'s Floating arm restores the full entries (dock + rect) verbatim; the placement scan / `placement_count` / descendants BFS float routes unchanged (`DockSlot::Floating { host, index }` indexes the entry list) |
| `normalization.rs` | The floating survivor arm: a survivor replacing a collapsed container copies into the inherited `DockSlot::Floating` reference **keeping the entry's rect verbatim** (the slot-attached §9.6 state); a zero-survivor removal deletes the entry with the slot (`rects live with the slot lifecycle`); the demotion rule unchanged (a temporary host demotes when regions *and* floating are empty — removing the last floater is one of the routes); docs extended; `#[cfg(test)]` +1 fixture |
| `validation.rs` | The floating rect rule (the PRD §9.6 + §7.7 gate): every floating entry must hold a valid `FloatingRect` — scanned after the region weights, before the placement multiplicity / acyclicity passes (check order documented); `InvalidFloatingState` for explicit garbage; a rect-less import (`serde(default)`) restores baseline and validates; the fixtures were updated to the entry API and a new garbage-rect import fixture added (serde-gated) |
| `mod.rs` | Re-exports `FloatingEntry` / `FloatingRect`; doc header gains the Phase-8 ownership bullet (floating structures) and the not-owned note (window decoration and visual stacking stay frontend-derived, PRD §9.6/§11.3) |
| `tests/domain_foundation_tests/docking_floating_test.rs` | **New — 15 tests** (registered in `main.rs`): below |
| `tests/domain_foundation_tests/docking_embedding_test.rs` | Migrated to the entry API (unit `EmbedSlot::Floating {}` matches → `{ rect: … }`, `floating()[i]` slices compared entry-wise) |

### External test coverage (`docking_floating_test.rs`, 15 tests)

Entries carry explicit rects / baseline by default and place by index; floating a split
subtree preserves orientation, children, order, and sizing verbatim (never flattened) and
clears the vacated region; "float from tab group" = the member detach (group survives,
selection rule clamps when the active member floats); floating a whole tab group preserves
members and re-docks through `TabJoin` (the entry shifts index *with* its persistent rect);
re-docking a floater through an `Embed` region rebuilds the attachment at the baseline weight
and shifts the collection; re-docking through `InsertBeside` wraps into the region slot and
retires the entry (no rect carry into split sizing); floating docks stay rejected as
directional targets (`NotSplittable` — the floated root and, via tab-chain resolution, any
dock inside the floater); `TabJoin` wrapping a floater inherits the entry's rect verbatim;
`SetFloatingRect` rewrites exactly one entry and reports `previous` (sibling entries,
occupants, and placements verbatim; the inverse owns the whole former collection; a repeated
set reports the installed value as its previous); carried minimums reject exclusively
(below-min, NaN, ∞ — untouched layouts) while the endpoint honors inclusively; index/host
validation (`FloatingIndexOutOfRange` naming the count, `NotAnEmbedHost`, `DockNotFound`
checked first); bound endpoints inclusive (full-body and corner-touching rects accepted);
floater removal clears its entry and shifts survivors by index with their rects (the inverse
holds the prior collection); removing a floated subtree takes its descendants in BFS order
and the persistent host stands; removing the last floater demotes the thereby-emptied
temporary host (in-place panel restore, fixpoint idempotent).

All fixtures go through the public operation API; the pub(crate) routes (invalid `FloatingRect`
construction, undo replay, imported garbage) are covered in-crate (`operations.rs` /
`validation.rs`) — the established conventions (agents/testing.md).

## 2. Decisions made during implementation

1. **The floating layer is host-anchored — no layout-level floating collection** (PRD §7.5,
   DM-07): every floated structure belongs to exactly one host via
   `DockSlot::Floating { host, index }`, the way its edge regions anchor docked ones. The
   closed placement-category list stands (there is no placement category beside
   root/split-child/tab-member/host-edge/floating), and the layout keeps no floating
   collection beside the per-host ones. A frontend that wants a floating layer without a
   host creates one (`CreateEmbedHost`) or promotes a panel (`PromoteHost`) first — both
   existing operations; nothing new is needed and no new root concept is introduced
   ("valid independently of the main root" is already how floats work: they are reachable
   through their anchor host, not through the root designation).
2. **Floating is `Embed` with a slot — the source detaches with its subtree intact** (PRD
   §7.6, §9.6 SHOULD: never flattened). The source kind is **unrestricted**: a panel, a tab
   group, a split, or another host all float as one entry naming their subtree root (only
   the *host* argument has eligibility limits, unchanged from Phase 7). The PRD §8.11
   "float from tab group" interaction is precisely the member detach: floating one member
   appends that member as its own entry while the group keeps the rest under the settled
   member-removal selection rule (same-position clamp).
3. **Position/size state is one `FloatingRect` per floating entry, attached to the slot,
   not the occupant** (PRD §9.6; the PRD §9.5 region-weight decision extended): the rect
   stores **fractions of the host's body** (0.0–1.0 per axis) naming the **dock body** (no
   decoration chrome — pixel geometry, including the chrome, is reconstructed frontend-side
   from the fractions mapped onto the host's actual work area). Containment in the host's
   logical bounds is a stored-state invariant the environment validates (§9.6 SHOULD;
   endpoints inclusive). A rect-less append starts at the `FloatingRect::BASELINE`
   (0, 0, 0.5, 0.5), the `EmbedSlot::Floating` default. `SetFloatingRect { host, index,
   rect, min_w, min_h }` is the only update path — one entry, absolute drift-free gesture
   contract (`ResizeSiblings`/`ResizeRegion` precedent), carried minimums honored
   inclusively, **rejection** (`FloatingBoundViolation`, never clamped). The write is
   non-structural (no cascade — no membership changes). Visual stacking / z-order stays
   frontend-derived (§11.3); the collection order is placement order and v1 defines no
   floating-rearrangement operation.
4. **Attach/detach interactions reuse the existing operations**: re-docking a floater drives
   it through `Embed` (region target), `InsertBeside`, or `TabJoin` as the source — the same
   detach/cascade/attach pipeline as any dock move, so a re-docked floater rebuilds a valid
   target attachment by construction. The Phase-4 directional baseline **stands**: a
   floating dock as an `InsertBeside` target rejects `NotSplittable` (tab-chain resolution
   to a floated container included), and a floating slot stays a wrap-inheritable slot for
   `TabJoin` (the new group takes over the entry **with its rect**). A floated dock is never
   a resize or directional-rearrangement surface.
5. **Removal/normalization rules extend verbatim** (PRD §8.11, §8.12): an entry removal
   clears the slot's rect with it (rects live with the slot lifecycle — the next float
   starts from the baseline or an explicit rect, never the previous occupant's move
   history); members shift by index and each entry's rect follows the entry (survivor order
   preservation); a survivor replacing a below-minimum floater copies into the entry keeping
   its rect (the slot inheritance); removing a floater can empty the temporary host that
   anchors it and the shared cascade demotes it; removing a host removes the whole
   arrangement — the BFS sweep crossed floating edges since Phase 7, unchanged.
6. **Three new error variants, named by slot category** (PRD §14 style):
   `FloatingIndexOutOfRange { index, entry_count }` (the entry-existence check — the
   `TabIndexOutOfRange` sibling), `FloatingBoundViolation { host, index }` (the rejection
   policy for invalid rects / carried-minimum conflicts — the `RegionBoundViolation`
   mirror), `InvalidFloatingState { host, index }` (imported-garbage steady-state finding —
   the `InvalidRegionState` mirror; the Phase-3 gate scans regions first, then floating
   entries by index, keeping the deterministic ascending-id + fixed-rule-order report).
7. **Serde policy** (§16): `FloatingRect` is struct-level `#[serde(default)]` (absent
   fields restore the whole baseline — a legacy rect-less import is valid state, matching
   the §7.7 restore story), the entry's `rect` field is `serde(default)` field-level,
   `FloatingRect` derives `Debug, Clone, Copy, PartialEq` only (f64 — no `Eq`/`Hash`, so
   `EmbedSlot` drops those derives it carried in Phase 7; layout types using slots as keys
   stay possible via `PartialEq` matching), and the imported-garbage fixture proves only
   *explicit* garbage trips `InvalidFloatingState`.

## 3. Completion criteria check (current_state/plan.md Phase 8)

- [x] **Floating structures remain valid independently of the main root** — floats are
      host-anchored explicit entries placed at `DockSlot::Floating` (not root-dependent);
      every committed state with floats validates; the Phase-3 gate re-checks imported
      floating rect state (`InvalidFloatingState`).
- [x] **Floating and re-docking preserve the intended internal structure** — a floated
      subtree keeps orientation, children, order, sizing, and selection verbatim (split and
      tab-group floats fixtured; the member detach keeps the group); re-docking through
      `Embed`/`InsertBeside`/`TabJoin` rebuilds valid attachments with the undo records
      owning every intermediate state.
- [x] **The domain model does not assume a particular window manager or UI toolkit** — float
      state is pure host-fraction logical geometry (no pixels, no decoration, no window/OS
      types); stacking/z-order is frontend-derived; the environment records the dock body
      rect only.
- [x] **Position/size preferences persist** (the §9.6 task) — one `FloatingRect` per entry,
      stored verbatim, rewritten only by `SetFloatingRect`, carried forward by slot
      inheritance (wrap / survivor), cleared with the slot on removal, restored through the
      `serde(default)` path.

Phase 2–7 suites all green with zero regressions (numbers at top); embedding's own tests
migrated to the entry API, operation outcomes and undo coverage extend uniformly
(`FloatingRectSet` arm in `inverse()`, dispatch and `restore_collection` arms for the
floating record).

## 4. Files touched

```
src/domain_foundation/environments/docking/floating.rs               (new — FloatingRect + FloatingEntry + §9.6 transfer docs)
src/domain_foundation/environments/docking/dock.rs                   (EmbedHost.floating: Vec<FloatingEntry> + floating() accessor + docs)
src/domain_foundation/environments/docking/operations.rs             (EmbedSlot::Floating { rect }, SetFloatingRect + outcome + record +
                                                                      initial-rect pre-check, TabJoin float wrap arm, docs, +2 in-crate tests)
src/domain_foundation/environments/docking/error.rs                  (3 new variants + Display)
src/domain_foundation/environments/docking/model.rs                  (restore_collection Floating arm → entries)
src/domain_foundation/environments/docking/normalization.rs          (floating survivor keeps the entry rect + docs + 1 test)
src/domain_foundation/environments/docking/validation.rs             (floating rect steady-state rule + garbage-rect import fixture)
src/domain_foundation/environments/docking/mod.rs                    (re-exports + doc bullets)
tests/domain_foundation_tests/docking_floating_test.rs               (new — 15 tests)
tests/domain_foundation_tests/docking_embedding_test.rs              (migrated to the entry API)
tests/domain_foundation_tests/main.rs                                (mod docking_floating_test;)
```

## 5. Next: Phase 09 — Domain Engine integration

Per `current_state/plan.md` Phase 9 + PRD §15/§16 routes + the Ph.1 R-A1/R-A2 undo plan:

1. **Expose the environment through the appropriate module declarations** — the module
   wiring exists (Phases 1–8 already registered `docking` in
   `domain_foundation/environments/mod.rs` / `domain_foundation/mod.rs`); Phase 9 wires the
   *engine* side (`domain_engine/operations/*`).
2. **Add docking operation handling following the existing environment conventions** —
   `domain_engine` currently knows nothing about docking; follow the established
   `Canvas`/`Layers`/etc. integration: a `DockError` (or the analogous) variant on the engine
   error enum, an `execute_docking` entry, and the `DockingOperation → ExecutedDockingOperation`
   wrapper whose undo/redo is built on `UndoData::replay_into` (the `pub(crate)` seam that
   every outcome already carries from day one).
3. **Connect operation execution to the Docking Environment model** — engine owns a
   `DockLayout` instance the way it owns the other environments' models (no duplicated
   state ownership; external data keyed by `DockId`, the Canvas2D/Layers precedent).
4. **Map domain errors into the established error/result pattern** — `DockingError` wraps
   into the engine's subsystem-error conversion (`engine_error_converts_from_subsystem_errors`
   is the existing shape to mirror).
5. **Callers cannot bypass validation** — execution goes exclusively through the operation
   pipeline (mutate → normalize → validate → commit) so the Phase-3 gate and the checks from
   Phases 4–8 hold.
6. **Event surfaces** — the established per-environment events gain their docking analogues
   (the plan names `docking.split.resized` as the shape to define; other operation outcomes
   follow the same event recipe).
7. Tests: `tests/app_engine_tests/` gains a docking area registered in that suite's
   `main.rs` (agents/testing.md); keep the external docking coverage through the public API
   as-is.

**Then (preview):** Ph.10 persistence/history (weights, collapse flags, region/floating
rects are the new serialized payload; `serde(default)` restore compat), Ph.11 interaction
contract, Ph.12 acceptance.
