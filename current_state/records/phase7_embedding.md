# Docking Environment — Phase 7 Record: Embedding and Host Lifecycle

**Phase:** 7 of 12 (`current_state/plan.md` — Embedding and host lifecycle)
**Status:** ✅ Complete — full suite green, serde path green, warning baselines unchanged
**Verification:**
- `cargo test` → **850 app_engine + 495 domain_foundation (integration) + 63 in-crate lib tests
  pass**, 0 failures (docking external total **83**: 14 model + 9 validation + 24 operation +
  8 normalization + 16 sizing + **12 new embedding tests**; lib docking in-crate **60** =
  23 validation fixtures + **19 in-crate operation tests** (+11 Phase-7) + **18 normalization
  fixtures** (+5 demotion/host-lifecycle), plus the small non-docking in-crate set (3); the
  "0 passed" empty benches harness line is pre-existing)
- `cargo test --features serde` → same suites green (**850 / 499 / 69 lib**); the serde gain is
  the additive `EmbedHost.persistent` field (`serde(default) = true`) and two new
  imported-region-state fixtures (`HostRegions` field-weight defaults restore at the §9.5
  baseline)
- `cargo check` / `cargo check --features serde` → clean; **10 pre-existing warnings**, none
  from `docking/`
- `cargo doc --no-deps` → **9 warnings, all pre-existing** Phase‑2/3 leftovers
  (`embed_host.rs`/`model.rs`×2/`slot.rs`/docking `mod.rs` + relations/`node_graph`); none
  from the Phase‑7 changes

---

## 1. What was built

Embedding and the host lifecycle (PRD §8.8–§8.9, §9.5, §8.11): the `Embed` operation (edge
regions + the floating appender, occupied-region rejection, auto-promotion), the explicit
`PromoteHost` operation, the automatic demotion of emptied temporary hosts through the shared
normalization pass, per-edge-region persistent sizing weights with `ResizeRegion`, and the
defined removal outcomes for embedded structures.

| File | Established |
|---|---|
| `dock.rs` | `EmbedHost { content, regions, floating, persistent }` — the persistence distinction (PRD §8.9): `persistent: true` = an explicitly configured host that remains a host when empty; `false` = a temporary promotion demoted by normalization once its last occupant leaves. `serde(default)` = `true` (pre-Phase-7 hosts restore persistent and never silently demote on import); the read accessor `persistent()` (`None` for non-hosts) |
| `embed_host.rs` | `HostRegions` carries one **slot-attached** sizing weight per edge (PRD §9.5): stored verbatim under the `sizing` representation decision, baseline `REGION_WEIGHT_BASELINE = 1.0` (uniform with the implicit core weight) on a fresh embed; `weight_on` (occupied-only read — the stored value of an empty region is inert), `assign` (clearing resets the slot to the baseline — weights live with the region lifecycle), `set_weight` (the one-region rewrite), `slot_weight` (the carry-forward read for slot-inheriting rewrites); the module doc defines the **two-stage allocation read** (column top/core(1)/bottom, then row left/content(1)/right inside the core band; `serde(default)` restores absent weight fields at the baseline) |
| `operations.rs` | `DockingOperation::Embed { source, host, slot }` (`EmbedSlot::Region(edge)` / `EmbedSlot::Floating`) + `apply_embed`; `PromoteHost` + `apply_promote_host` (explicit = **persistent**; no-op success on an already-host dock); `ResizeRegion { host, edge, requested, min }` + `apply_resize_region`; outcomes `Embedded { source, host, slot, promoted }`, `HostPromoted { dock, promoted }`, `RegionResized { host, edge, previous }`; `FormerCollection` gains `Region { host, edge, previous, weight }` (occupant and preference restore together), `Floating`, `Promotion { id, collapsed }`, `Demotion { id }`; the shared `promote_to_host` helper (records the panel's dropped collapse preference); detach_dock's region arm clears through `assign` (baseline reset); the wrap arms (host-region target in `InsertBeside`/`TabJoin`) carry the slot's weight with the occupant; `#[cfg(test)]` +10 Phase-7 fixtures (rejections, cycles, promotion semantics, undo round-trips) |
| `error.rs` | Five variants: `NotAnEmbedHost { id }` (the host-side `NotATabGroup`/`NotASplit` sibling), `RegionOccupied { host, edge }` (the §8.8 reject-by-default baseline), `RegionUnoccupied { host, edge }` (an empty region holds no resizable boundary/preference), `RegionBoundViolation { host, edge }` (the rejection policy, as `SizeBoundViolation`), `InvalidRegionState { host, edge }` (imported-garbage steady-state finding — the `InvalidSizeState` mirror) |
| `model.rs` | `create_embed_host` starts hosts persistent; `restore_collection` gained the `Region`/`Promotion`/`Demotion` arms (undo replay: the region record restores occupant **and** weight together; `Promotion`/`Demotion` rebuild the exact pre/post role states) |
| `normalization.rs` | The **host demotion rule** (the automatic §8.9 side): an empty (regions *and* floating) `persistent: false` host demotes in place to `Panel { content, collapsed: false }` (the panel-only collapse preference does not carry across the boundary — v1 hosts are not a collapsible category); persistent hosts and hosts with any occupant are valid steady state (§8.8) and never demoted; the rule runs through `needs_resolution` / `lowest_to_resolve` so both drivers (operation cascade, whole-layout fixpoint pass) honor it; the host-region collapse arm forwards the slot weight with the survivor; docs extended; `#[cfg(test)]` +5 fixtures (demotion through the cascade / explicit-promotion persistence / never-demote + in-place role change / whole-pass idempotence) |
| `validation.rs` | The occupied-region weight rule (PRD §9.5): every occupied edge must hold a finite, strictly positive stored weight; the stored value of an unoccupied region is inert and skipped; first-error report order unchanged (deterministic ascending-id scan; region weights check in the fixed top/bottom/left/right order). serde import fixtures: valid host-region sizing accepted; explicit garbage rejected as `InvalidRegionState` (the `invalid_size_state` mirror) |
| `sizing.rs` | The §9.5 transfer table in the module doc (region-weight initialized/updated/transferred/cleared policies mirror the split's); the resize cross-reference (one-region boundary write, absolute drift-free requests) |
| `mod.rs` | Doc header: embedding/`Embed`/`PromoteHost`/lifecycle + region-sizing ownership bullets |
| `tests/domain_foundation_tests/docking_embedding_test.rs` | **New — 12 tests** (registered in `main.rs`): below |
| `tests/domain_foundation_tests/main.rs` | `mod docking_embedding_test;` |

### External test coverage (`docking_embedding_test.rs`, 12 tests)

Embed lifecycle (region + floating placements with the `1.0` baselines and outcome fields);
floating appends last and rejects directional insertion (`NotSplittable`, the §8.6 v1
baseline); the occupied-region rejection keeps the layout untouched while other edges accept
their own embeds; eligibility (splits/tab groups are `IneligibleDock`) and the cycle rule
(host-inside-source-subtree rejected `CycleDetected`); re-docking a source inside its own host
(region occupant → floating, allowed); auto-promotion is **temporary** and demotes through a
removal cascade to the same expanded panel (identity, content, root preserved; normalize
idempotent); explicit `PromoteHost` is **persistent** and survives emptiness (empty persistent
host = valid steady state; `normalize()` a no-op); promotion/demotion in a split-child slot
keeps placement and the parent split verbatim; `ResizeRegion` rewrites exactly one stored
weight (siblings and occupants verbatim) and reports `previous`; occupancy + exclusive bounds
(0/NaN/∞/below-min reject deterministically, layout untouched; inclusive endpoint honors);
host removal takes the **whole arrangement** — BFS crossings cover the host edges (regions in
fixed order, then floating), layout empty; occupant removal clears exactly one slot (weights
lifecycle reset) with the host and its other placements verbatim.

All fixtures go through the public operation API; imported-garbage region state is
serde-gated in-crate (`validation.rs`), undo replay is the `pub(crate)` seam (in-crate
`operations.rs` fixtures) — the established conventions (agents/testing.md).

## 2. Decisions made during implementation

1. **The persistence dichotomy** (PRD §8.9's "temporary hosting relationships from persistent
   ones where required"): hosts carry a `persistent` flag — `create_embed_host` and the
   explicit `PromoteHost` produce **persistent** hosts (an empty persistent host is a valid
   steady state, §8.8 — an explicitly configured host area survives even with nothing
   embedded); `Embed`'s auto-promotion is **temporary** (`persistent: false`): the panel was
   promoted solely to support that embed (the §8.9 "when a valid embedding operation requires
   it" case), so it returns to a panel once nothing is embedded in it. The flag is
   `serde(default)`-`true`: layouts saved before the distinction existed restore as
   persistent hosts (they must not silently demote on import).
2. **Demotion runs through the shared normalization pass** (the automatic half of §8.9) by
   the `needs_resolution` predicate: an empty temporary host is "below minimum" in the same
   sense as a one-child split. Demotion is an **in-place role change** to
   `Panel { content, collapsed: false }` — identity, content, and placement survive verbatim;
   no parent collection is rewritten and the cascade chain ends there. The panel-only
   collapse preference does not carry across the category boundary (v1: hosts are not a
   collapsible category), and the undo records (`Promotion` / `Demotion`) rebuild the exact
   pre/post roles so replay restores even that dropped flag. Unplaced hosts are not demoted
   (left for the Phase-3 gate — the no-slot-guessing rule of every normalization).
3. **Embed rejects by default** (the settled §8.8 baseline): an occupied edge region rejects
   with `RegionOccupied` — the existing occupant is never silently discarded and v1 defines
   no replacement/exchange policy. Host eligibility: an embed host, or a panel (auto-promoted
   inside the operation); splits and tab groups hold no work area and reject `IneligibleDock`
   (they can neither host nor promote). Check order: host existence/eligibility → source
   existence/cycle → occupancy → detach (cascade recording) → promote → attach → normalize.
4. **The cycle rule is asymmetric on purpose**: the *host inside the source's* subtree
   (including source == host) is the only hazard — the attach would make the source its own
   descendant — so it rejects `CycleDetected { id: host }`. A source *inside* the host's
   subtree re-parents into another slot of its own ancestor (a region occupant undocked to
   floating) and is allowed.
5. **Region weights attach to the slot, not the occupant** (PRD §9.5; the `Split { children,
   sizes }` parent-side precedent): one verbatim weight per edge, established at the §9.5
   embedding baseline `1.0` (uniform with the implicit core weight), carried forward
   unchanged by slot-inheriting rewrites (an `InsertBeside`/`TabJoin` wrap of a region
   occupant, a collapsing survivor), rewritten *only* by `ResizeRegion`, and reset to the
   baseline when the region is cleared (`weights live with the region lifecycle` — the next
   embed starts from the default, not the previous occupant's preference).
6. **`ResizeRegion` rewrites exactly one weight** (the §9.5 coherence rule): each region
   resizes against its inner boundary, so no neighbour state changes and no other placement
   can be invalidated. Gesture and constraint contracts mirror `ResizeSiblings` (§9.4/§9.3):
   absolute drift-free requests, operation-carried minimums honored inclusively, and the
   **rejection** policy — `RegionBoundViolation`, never clamping; the finite/strictly
   positive invariant is always the implicit bound. Empty regions reject
   `RegionUnoccupied` (no boundary, no preference). The write covers the whole weight field,
   so there is no stored-garbage rebuild step — imported garbage is caught by the Phase-3
   gate (`InvalidRegionState`), deterministically, like `prune_weight` governs splits.
7. **The two-stage allocation read** (§9.5's top/bottom ↔ left/right interaction) is a
   derived frontend read, documented in `embed_host.rs`/`sizing.rs`: the column stage divides
   the host's height between occupied top/bottom weights + the implicit core weight 1.0; the
   row stage divides the core band's width between occupied left/right weights + the content
   cell's implicit 1.0. Resizing one region therefore never rewrites another's stored weight
   (the interaction is read-side, not a neighbour rewrite), and the core holds no stored
   weight in v1 — a content-side minimum expresses as the region's (MAY) maximum, which the
   v1 one-sided bounds omit deliberately.
8. **Removal of embedded structures has defined outcomes** (PRD §8.11): removing a host
   removes the **whole arrangement** — the existing BFS subtree sweep already crosses host
   edges (`structural_children` = regions in fixed top/bottom/left/right order, then
   floating), so subtree, occupants, floating entries, and (with them) the region weights go
   together; removing an occupant detaches it from its slot alone (the region's
   record restores occupant+weight together on undo, the cleared slot resets to the baseline)
   and may demote the thereby-emptied temporary host through the shared cascade. Content
   lifetime stays with the host application in both cases.
9. **The floating appender is v1-last** (§9.6 is Phase 8): `EmbedSlot::Floating` appends to
   the collection's end, carries no sizing state, and docks floating remain unreachable from
   the directional insertion pathways (`InsertBeside` beside a floating dock stays
   `NotSplittable`).
10. **Serde field-policy discipline**: the `EmbedHost.persistent` default fn is
    feature-gated (`#[cfg(feature = "serde")]`, the `relations/definition.rs::default_effect`
    precedent) so the flag-free build stays warning-clean; `HostRegions` uses a struct-level
    `serde(default)` so absent weight fields restore from the struct `Default` (the
    baseline), and the imported-garbage fixture proves only explicit garbage trips
    `InvalidRegionState`.

## 3. Completion criteria check (current_state/plan.md Phase 7)

- [x] **Host relationships are explicit and valid** — edge regions / floating collections are
      reachable through `Embed` (previously modeled-but-unreachable slots), persisted as
      `HostEdge` / `Floating` placements, and every committed state validates (the PRD §8.8
      empty-regions steady state included); validation carries the §9.5 occupied-region
      weight rule.
- [x] **Embedding does not create cycles or duplicate ownership** — the host-inside-source
      cycle check plus the Phase-3 acyclicity/multiplicity gate; the allowed-direction case
      (re-docking within one's own host) and the rejection case are both fixtured externally
      and round-trip through undo in-crate.
- [x] **Promotion and demotion preserve the intended identities and content** — auto-promotion
      (temporary) and explicit promotion (persistent) keep the dock id, content reference, and
      placement; demotion lands the exact prior panel (expanded); promotion/demotion round-trip
      through undo records, in a root slot and a split-child slot alike.
- [x] **Removing a host has a defined outcome for its embedded structures** — whole-arrangement
      removal (BFS crossing the host edges, the region weights going with the cleared slots);
      occupant removal detaches one slot; the emptied temporary host demotes automatically; the
      host application keeps content lifetime either way.

Phase 2–6 suites all green with zero regressions (numbers at top); operation outcomes and undo
coverage extend uniformly (`inverse()` arms added to the shared match; dispatch and
`restore_collection` arms added for the new record kinds).

## 4. Files touched

```
src/domain_foundation/environments/docking/dock.rs                  (EmbedHost.persistent + cfg'd serde default + accessor)
src/domain_foundation/environments/docking/embed_host.rs            (slot weights, assign/set_weight/slot_weight, §9.5 allocation docs)
src/domain_foundation/environments/docking/operations.rs            (Embed/PromoteHost/ResizeRegion + outcomes + records +
                                                                     host-region wrap arms + docs + 10 in-crate tests)
src/domain_foundation/environments/docking/error.rs                 (5 new variants + Display)
src/domain_foundation/environments/docking/model.rs                 (create_embed_host persistent; restore_collection Region/Promotion/Demotion)
src/domain_foundation/environments/docking/normalization.rs         (demotion rule + weight-carrying survivor + 5 tests)
src/domain_foundation/environments/docking/validation.rs            (occupied-region weight rule + 2 serde import fixtures)
src/domain_foundation/environments/docking/sizing.rs                (§9.5 transfer table + allocation cross-reference)
src/domain_foundation/environments/docking/mod.rs                   (doc bullets)
tests/domain_foundation_tests/docking_embedding_test.rs             (new — 12 tests)
tests/domain_foundation_tests/main.rs                               (mod docking_embedding_test;)
```

## 5. Next: Phase 08 — Floating docking structures

Per `current_state/plan.md` Phase 8 + PRD §8.11 (floating semantics), §9.6 (position/size
state) + the §8.6/§9.6 v1 deferrals recorded in Phases 4–7:

1. **Floating-root representation**: how floated structures exist outside the main root —
   today `Floating` lists hang off hosts' `EmbedHost { floating }` (placed but root-independent);
   decide what a floated *root-level* structure is and whether a floating host root is a
   separate arrangement or reuses the embed-host attachment.
2. **Floating a panel, tab group, or eligible subtree** while preserving internal structure
   (the `Embed` operation already accepts one source per append — groups/subtrees need the
   eligibility decision §8.11 sanctions).
3. **Position/size preferences** (PRD §9.6): the persistent per-floating-dock state the v1
   embed left out (the §9.6 defs live in `embed_host.rs`'/`sizing.rs` docs' deferral notes).
4. **Attach/detach interactions**: re-docking a floating structure back into a valid target
   (the region/floating/split targets), and the directional-target semantics for floating
   docks (the `NotSplittable` baseline stands until redefined).
5. Removal/normalization interactions for floating entries (the BFS/region rules already
   cross floating collections — decide the standalone floating lifecycle rules).
6. Tests: extend `tests/domain_foundation_tests/docking_embedding_test.rs` or a new
   `docking_floating_test.rs` (register in `main.rs`); in-crate fixtures continue the
   floating installer routes (`record_floating`, `put_tab_group` precedent).

**Then (preview):** Ph.9 Domain Engine integration (`execute_docking` +
`ExecutedDockingOperation` on `UndoData.replay_into`, the `docking_command_handler.rs`
recipe, `docking.split.resized` event surfaces), Ph.10 persistence/history (region weights,
collapse flags, floating size/positions are the new serialized payload), Ph.11 interaction
contract, Ph.12 acceptance.
