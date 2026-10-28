//! Structural normalization: the deterministic cleanup pass that keeps
//! committed docking layouts free of redundant structure
//! (current_state/plan.md Phase 05, PRD §8.12).
//!
//! Normalization resolves containers whose membership fell below their
//! steady-state minimum — the redundant wrappers detached members leave
//! behind — so the layout never accumulates them. It is the shared pass
//! of the canonical operation pipeline
//!
//! ```text
//! mutate → normalize → validate → commit
//! ```
//!
//! (see [`DockingOperation::apply`]), and [`DockLayout::normalize`] is
//! the whole-layout maintenance entry (imported / restored state).
//!
//! [`DockingOperation::apply`]:
//!     super::operations::DockingOperation::apply
//!
//! ## The rules (PRD §8.12, concretized)
//!
//! A container is *below minimum* when its membership cannot stand as a
//! steady state:
//!
//! | Container  | Minimum   | Below minimum resolves as |
//! |---|---|---|
//! | Split | ≥ 2 children | collapses: its survivor (first child) inherits the split's slot and the split is removed; a zero-child split (defensive, imported state) has no survivor |
//! | Tab group | ≥ 1 member | a zero-member group is removed; nothing inherits |
//! | Panel | — | a leaf; nothing to resolve |
//! | Embed host | — | imposes no child minimum (PRD §8.8): empty edge regions / floating lists are valid steady state; a **temporary** host (PRD §8.9) whose regions *and* floating collection are both empty **demotes** back to its former category — an in-place role change that keeps identity, content, and placement |
//!
//! - A tab group with one or more members is **preserved**: tab groups
//!   have meaningful independent semantics (PRD §8.12 rule 3) and their
//!   steady-state minimum is one member, unlike the split's two.
//!   Removing it would silently discard a valid tab group (plan
//!   criterion); one-member groups are satisfied by re-rooting or
//!   refilling, never deleted.
//! - The **host demotion rule** is the automatic side of PRD §8.9:
//!   panels promoted to host status solely to support embedding
//!   (`persistent: false`) return to their prior ordinary category
//!   (v1: a panel, expanded — the panel-only collapse preference is not
//!   carried across the boundary) once nothing is embedded in them.
//!   Explicitly configured persistent hosts (`persistent: true`, created
//!   via `CreateEmbedHost`) are never demoted — an empty persistent
//!   host is a valid steady state (PRD §8.8). Identity and content
//!   survive the demotion; the rule is an in-place role change, so no
//!   parent collection is rewritten and the chain ends there. Demotion,
//!   like every rule here, is recorded in the cascade's `undo` and
//!   skipped for unplaced docks (left for the Phase-3 gate — the same
//!   no-slot-guessing rule as the below-minimum containers).
//! - **Floating entries** (PRD §9.6, Phase 8) are ordinary placed
//!   members of the host's floating collection: a survivor replacing a
//!   collapsed dock in a floated slot copies into the entry and keeps
//!   the entry's logical rect verbatim (the slot-attached §9.6 state);
//!   a zero-survivor dock leaves the collection with its entry (rects
//!   live with the slot lifecycle). A below-minimum container **inside
//!   a floated subtree** resolves like any other — the cascade walks
//!   the structural edges regardless of the anchored placement. Removing
//!   a floater may thereby empty (and demote) the temporary host it
//!   anchored through the same cascade.
//! - The designated root participates in the same rules: a below-minimum
//!   root is replaced by its survivor, so the designation never dangles
//!   while docks remain, and the last removal yields the explicit empty
//!   layout (PRD §7.4 — the root is the current top-level dock, Ph.2).
//! - An **unplaced** below-minimum container is invalid state the
//!   Phase-3 gate rejects; normalization does not guess a slot for it
//!   and leaves it for [`validate`](super::validation).
//! - Replacements never reorder: a survivor copies into the exact slot
//!   reference it inherits, and member removals shift only by the
//!   removed index, so the **relative order of surviving children is
//!   preserved** (plan criterion).
//!
//! ## Sizing redistribution policy (PRD §9.2 — Phases 4–6)
//!
//! - Collapse: the parent keeps the slot's weight — the survivor
//!   inherits the collapsed container's area.
//! - Zero-survivor removal: the removed index's weight is deleted
//!   (`prune_weight`) so the relative shares of the survivors stay
//!   verbatim.
//! - Insertion and fresh-container baselines (target-weight copy,
//!   uniform distribution) live with the insertion flows
//!   ([`operations`]). The splitter gesture rewrites the adjacent pair
//!   through `ResizeSiblings` with pair conservation: the operation can
//!   never create a below-minimum container (it changes no membership),
//!   and it never rescales the untouched children.
//! - A collapsed panel (PRD §9.7) is an **ordinary member** to every
//!   rule here: its preference changes no placement or sizing data, it
//!   counts toward its container's minimum, and no normalization step
//!   adds, removes, or rewrites the flag. Stored weights are verbatim
//!   (the representation decision in [`sizing`]).
//!
//! ## Determinism and idempotence (plan criteria, PRD §17.2)
//!
//! - Resolution is deterministic: the whole-layout pass picks
//!   below-minimum containers and empty temporary hosts in ascending
//!   [`DockId`] order and walks each collapse chain rootward; the same
//!   layout always normalizes to the same result.
//! - The pass runs to a **fixpoint** — it repeats until no below-minimum
//!   container or empty temporary host remains — so repeated `normalize`
//!   calls leave an already normalized layout unchanged (plan
//!   criterion).
//! - Normalization never manufactures a rule violation: every rewrite
//!   keeps counts, weights, and active selections consistent, and the
//!   committed result must still pass
//!   [`validate`](super::validation) (the operation gate). Each step
//!   checks before it mutates; a mid-pass error (invalid imported
//!   sizing) leaves the already-completed steps applied, the failing
//!   step's state untouched, and is surfaced to the caller rather than
//!   silently repaired.
//!
//! [`operations`]: super::operations
//! [`sizing`]: super::sizing

use super::dock::{Dock, DockId, DockRole};
use super::error::DockingError;
use super::model::DockLayout;
use super::operations::{record_demotion, record_floating, record_region, record_split, record_tab, UndoData};
use super::sizing::SplitSizes;
use super::slot::DockSlot;

impl DockLayout {
    /// Normalize the whole layout: resolve every below-minimum container
    /// until none remains (the fixpoint); see the module docs for the
    /// rules, policy, and guarantees.
    ///
    /// A whole-layout maintenance pass — e.g. for imported /
    /// restored state (Phase 10) or direct repair of a validated-but-
    /// unnormalized layout. It records no inverse data: undo coverage
    /// belongs to operations, which run the same rules through
    /// `collapse_cascade` while recording their inverse payloads.
    pub fn normalize(&mut self) -> Result<(), DockingError> {
        loop {
            let Some(id) = self.lowest_to_resolve() else {
                return Ok(());
            };
            // The selector only returns placed below-minimum containers,
            // so this first step acts; the reported chain continues
            // rootward until it dies out. The inverse data is scratch:
            // the pass is not undoable (see above).
            let scratch = &mut UndoData::default();
            let mut next = resolve_below_minimum(self, scratch, id)?;
            while let Some(current) = next {
                next = resolve_below_minimum(self, scratch, current)?;
            }
        }
    }

    /// The lowest-id placed dock needing resolution — a below-minimum
    /// container or an empty temporary host (the PRD §8.9 demotion
    /// rule) — or `None` at the fixpoint. Unplaced ones are skipped
    /// (rules above) so the pass cannot loop on state it declines to
    /// repair. The scan is the whole-layout price of this entry point;
    /// the operation path (`collapse_cascade`) touches only the
    /// affected chain.
    fn lowest_to_resolve(&self) -> Option<DockId> {
        let mut ids = self.all_docks();
        ids.sort();
        ids.into_iter()
            .find(|&id| {
                self.get(id).is_some_and(needs_resolution) && self.placement(id).is_some()
            })
    }
}

/// Collapse the affected container chain rootward until every visited
/// container meets its minimum — the shared operation cleanup pass'
/// (PRD §8.12).
///
/// Operations detach members and then call this on the vacated owner;
/// each resolution rewrites one parent collection (recorded in `undo`
/// before the rewrite) and deletes the collapsed container (recorded in
/// `undo.removed`). Returns Ok(()) when nothing along the chain is
/// below minimum.
pub(crate) fn collapse_cascade(
    layout: &mut DockLayout,
    undo: &mut UndoData,
    start: DockId,
) -> Result<(), DockingError> {
    let mut next = Some(start);
    while let Some(id) = next {
        next = resolve_below_minimum(layout, undo, id)?;
    }
    Ok(())
}

/// Whether the dock's membership needs the pass to act on it: a
/// below-minimum container (the redundant-wrapper rules) or an empty
/// temporary host (the PRD §8.9 demotion rule — see the module rules
/// table).
fn needs_resolution(dock: &Dock) -> bool {
    match &dock.role {
        DockRole::Split { children, .. } => children.len() < 2,
        DockRole::TabGroup { members, .. } => members.is_empty(),
        // A temporary host with no embedded or floating docks remaining
        // demotes to its former category (PRD §8.9); persistent hosts
        // and hosts with any occupant are valid steady state.
        DockRole::EmbedHost { regions, floating, persistent, .. } => {
            !persistent && regions.is_empty() && floating.is_empty()
        }
        // Leaves: nothing to resolve.
        DockRole::Panel { .. } => false,
    }
}

/// Resolve one container if it sits below its minimum. Returns the next
/// dock the driver should examine, or `None` when there is nothing to
/// resolve here (not below minimum / missing / unplaced / the chain
/// dies out).
///
/// Result of a resolution: the parent collection rewrite, the slot
/// reference replacement (survivor) or member removal (zero survivor),
/// the active-tab adjustment, and the container's deletion.
fn resolve_below_minimum(
    layout: &mut DockLayout,
    undo: &mut UndoData,
    id: DockId,
) -> Result<Option<DockId>, DockingError> {
    // The dock may already have been deleted by an earlier chain step.
    let Some(dock) = layout.get(id) else {
        return Ok(None);
    };
    // Host demotion (PRD §8.9): an *empty temporary* host returns to
    // its former category — an in-place role change that keeps
    // identity, content, and placement, so no parent collection is
    // rewritten and the chain ends here. Persistent hosts and hosts
    // with any occupant are valid steady state (PRD §8.8).
    let empty_temporary_host = match &dock.role {
        DockRole::EmbedHost { regions, floating, persistent, .. } => {
            !*persistent && regions.is_empty() && floating.is_empty()
        }
        _ => false,
    };
    if empty_temporary_host {
        record_demotion(undo, id);
        if let Some(host) = layout.get_mut(id) {
            if let DockRole::EmbedHost { content, .. } = &host.role {
                let content = content.clone();
                host.role = DockRole::Panel { content, collapsed: false };
            }
        }
        return Ok(None);
    }
    let (below_min, survivor) = match &dock.role {
        DockRole::Split { children, .. } => (children.len() < 2, children.first().copied()),
        DockRole::TabGroup { members, .. } => (members.is_empty(), None),
        DockRole::Panel { .. } | DockRole::EmbedHost { .. } => (false, None),
    };
    if !below_min {
        return Ok(None);
    }
    // The container's own placement decides which parent collection is
    // rewritten; that parent is recorded *before* the rewrite (its
    // snapshot is the pre-operation state of one collection).
    let next = match layout.placement(id) {
        // Unplaced: invalid state the Phase-3 gate rejects; do not guess
        // a slot for it.
        None => None,
        Some(DockSlot::Root) => {
            // The survivor becomes the designated root; a zero-survivor
            // root yields the explicit empty layout (PRD §7.4).
            layout.set_root_unchecked(survivor);
            survivor
        }
        Some(DockSlot::SplitChild { parent, index }) => {
            record_split(layout, undo, parent);
            if let Some(dock) = layout.get_mut(parent) {
                if let DockRole::Split { children, sizes, .. } = &mut dock.role {
                    match survivor {
                        Some(child) => {
                            // The parent keeps the slot's weight: the
                            // survivor inherits the area (PRD §9.2).
                            children[index] = child;
                        }
                        None => {
                            // Check before mutating: the pruned set may
                            // be invalid for imported garbage sizing.
                            let pruned = prune_weight(sizes, index, parent)?;
                            children.remove(index);
                            *sizes = pruned;
                        }
                    }
                }
            }
            Some(parent)
        }
        Some(DockSlot::TabMember { parent, index }) => {
            record_tab(layout, undo, parent);
            if let Some(dock) = layout.get_mut(parent) {
                if let DockRole::TabGroup { members, active, .. } = &mut dock.role {
                    match survivor {
                        Some(child) => members[index] = child,
                        None => {
                            members.remove(index);
                            if index < *active {
                                *active -= 1;
                            } else if index == *active {
                                // The active member left: keep the
                                // selection at the same position,
                                // clamped to the last member (PRD §8.7).
                                *active = (*active).min(members.len().saturating_sub(1));
                            }
                        }
                    }
                }
            }
            Some(parent)
        }
        Some(DockSlot::HostEdge { host, edge }) => {
            record_region(layout, undo, host, edge);
            // The survivor inherits the region's whole slot: occupant
            // *and* the slot's sizing weight (the §9.5 slot
            // inheritance; the slot's weight lifecycle is the region's
            // preference, untouched by who occupies it).
            let weight = layout
                .get(host)
                .and_then(|dock| dock.host_regions())
                .map(|regions| regions.slot_weight(edge))
                .expect("host-edge slot names an existing host");
            if let Some(dock) = layout.get_mut(host) {
                if let DockRole::EmbedHost { regions, .. } = &mut dock.role {
                    regions.assign(edge, survivor, weight);
                }
            }
            // Hosts impose no child minimum (PRD §8.8).
            None
        }
        Some(DockSlot::Floating { host, index }) => {
            record_floating(layout, undo, host);
            if let Some(dock) = layout.get_mut(host) {
                if let DockRole::EmbedHost { floating, .. } = &mut dock.role {
                    match survivor {
                        Some(child) => {
                            // The survivor copies into the exact
                            // reference it inherits; the entry (and
                            // its logical rect — the slot's state
                            // carried forward, the §9.6 slot
                            // inheritance) stays put.
                            floating[index].dock = child;
                        }
                        None => {
                            // Zero survivor: the entry goes with the
                            // cleared slot (rects live with the slot
                            // lifecycle, PRD §9.6).
                            floating.remove(index);
                        }
                    }
                }
            }
            None
        }
    };
    // The collapsed container's payload is deleted and recorded (the
    // inverse stores it to rebuild the structure on undo).
    if let Some(destroyed) = layout.take_dock(id) {
        undo.removed.push(destroyed);
    }
    Ok(next)
}

/// Delete the weight at `index`, keeping the remaining relative weights
/// verbatim: weights are relative, so the removed child's share flows to
/// the surviving siblings automatically (PRD §9.2 — the removal-side
/// redistribution policy, see `normalization`). Fails without mutating
/// when a *remaining* stored weight is invalid — reachable only for
/// imported (restored) state; the error names the offending index.
pub(crate) fn prune_weight(
    sizes: &SplitSizes,
    index: usize,
    split: DockId,
) -> Result<SplitSizes, DockingError> {
    let remaining: Vec<f64> = sizes
        .weights()
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != index)
        .map(|(_, w)| *w)
        .collect();
    SplitSizes::from_weights(remaining).ok_or_else(|| {
        let offender = sizes
            .weights()
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != index)
            .find(|(_, w)| !w.is_finite() || **w <= 0.0)
            .map(|(i, _)| i)
            .unwrap_or(index);
        DockingError::InvalidSizeState { split, index: offender }
    })
}

#[cfg(test)]
mod normalization_tests {
    //! In-crate fixtures for the rule core: below-minimum containers are
    //! unreachable through the public creation preconditions and committed
    //! operations, so the fixtures assemble them the way imported /
    //! restored state can carry them — `pub(crate)` installers
    //! (`put_split`, `put_tab_group`), region assignment on hosts, and
    //! the test-only `SplitSizes::from_weights_unchecked` for garbage
    //! sizing (the sanctioned route, see `agents/testing.md`).

    use super::*;

    use crate::domain_foundation::environments::docking::{
        DockKind, DockingOperation, DockingOutcome, EmbedSlot, EdgeRegion, FloatingEntry,
        FloatingRect, InsertSource, SplitOrientation,
    };

    /// Start from the explicit empty layout: the first `create_panel`
    /// auto-roots, so the fixture removes it again (the id counter is
    /// not reset — the established convention).
    fn empty_layout() -> DockLayout {
        let mut layout = DockLayout::new();
        let first = layout.create_panel(None);
        DockingOperation::Remove { dock: first }.apply(&mut layout).unwrap();
        layout
    }

    /// Assign a top-region occupant on the host (import-style surgery;
    /// public operations create region placement — this fixture route
    /// still exercises direct assignment for below-minimum subtrees).
    fn host_assign_top(layout: &mut DockLayout, host: DockId, occupant: Option<DockId>) {
        let dock = layout.get_mut(host).expect("fixture host exists");
        if let DockRole::EmbedHost { regions, .. } = &mut dock.role {
            regions.assign(EdgeRegion::Top, occupant, 1.0);
        }
    }

    /// Two nested below-minimum splits ending at one panel: `inner`
    /// (one child) inside `outer` (one child) as the root.
    /// Returns (layout, inner id, inner's surviving panel id).
    fn nested_below_min_chain() -> (DockLayout, DockId, DockId) {
        let mut layout = empty_layout();
        let a = layout.create_panel(None);
        layout.set_root(None).unwrap(); // the panel auto-rooted: clear for the fixture root
        let inner =
            layout.put_split(SplitOrientation::Column, vec![a], SplitSizes::uniform(1).unwrap());
        let outer =
            layout.put_split(SplitOrientation::Row, vec![inner], SplitSizes::uniform(1).unwrap());
        layout.set_root(Some(outer)).unwrap();
        (layout, inner, a)
    }

    /// Two *independent* below-minimum one-child splits (`s1`, `s2`)
    /// under a shared two-child root: one rootward chain does not cover
    /// both, so only the fixpoint pass resolves them in one call.
    /// Returns (layout, s1, s2, s1's survivor, s2's survivor).
    #[allow(clippy::type_complexity)]
    fn two_independent_below_min() -> (DockLayout, DockId, DockId, DockId, DockId) {
        let mut layout = empty_layout();
        let a = layout.create_panel(None);
        let b = layout.create_panel(None);
        layout.set_root(None).unwrap(); // the first panel auto-rooted: clear for the fixture root
        let s1 = layout.put_split(SplitOrientation::Row, vec![a], SplitSizes::uniform(1).unwrap());
        let s2 = layout.put_split(SplitOrientation::Column, vec![b], SplitSizes::uniform(1).unwrap());
        let root = layout
            .create_split(SplitOrientation::Row, vec![s1, s2], SplitSizes::uniform(2).unwrap())
            .unwrap();
        layout.set_root(Some(root)).unwrap();
        (layout, s1, s2, a, b)
    }

    // --- Collapse rules ---

    #[test]
    fn below_min_root_split_collapses_to_survivor_child() {
        let mut layout = empty_layout();
        let p = layout.create_panel(None);
        layout.set_root(None).unwrap(); // the panel auto-rooted: clear for the fixture root
        let s = layout.put_split(SplitOrientation::Row, vec![p], SplitSizes::uniform(1).unwrap());
        layout.set_root(Some(s)).unwrap();
        assert!(!layout.is_valid()); // below-min state: rejected by design

        layout.normalize().unwrap();
        // The survivor inherits the Root slot; the split is deleted and
        // the layout ends at the minimal valid form.
        assert!(layout.is_valid());
        assert_eq!(layout.root(), Some(p));
        assert_eq!(layout.placement(p), Some(DockSlot::Root));
        assert!(!layout.contains(s));
        assert_eq!(layout.len(), 1);
    }

    #[test]
    fn one_child_split_survivor_replaces_reference_in_parent_split() {
        let mut layout = empty_layout();
        let a = layout.create_panel(None);
        let b = layout.create_panel(None);
        layout.set_root(None).unwrap(); // the first panel auto-rooted: clear for the fixture root
        let child =
            layout.put_split(SplitOrientation::Row, vec![a], SplitSizes::uniform(1).unwrap());
        let parent = layout
            .create_split(
                SplitOrientation::Column,
                vec![child, b],
                SplitSizes::from_weights(vec![0.7, 0.3]).unwrap(),
            )
            .unwrap();
        layout.set_root(Some(parent)).unwrap();
        assert!(!layout.is_valid());

        layout.normalize().unwrap();
        assert!(layout.is_valid());
        assert_eq!(layout.root(), Some(parent));
        assert!(!layout.contains(child));
        // The survivor copies into the exact reference it inherits —
        // order preserved — and the parent keeps the slot's weight.
        assert_eq!(layout.placement(a), Some(DockSlot::SplitChild { parent, index: 0 }));
        assert_eq!(
            layout.get(parent).unwrap().sizes().map(|s| s.weights().to_vec()),
            Some(vec![0.7, 0.3])
        );
    }

    #[test]
    fn nested_below_min_chain_reaches_fixpoint_through_root() {
        let (mut layout, _inner, a) = nested_below_min_chain();
        assert!(!layout.is_valid());

        layout.normalize().unwrap();
        // Both links resolve: inner collapses into outer, then the
        // (now one-child) root collapses to the panel.
        assert!(layout.is_valid());
        assert_eq!(layout.len(), 1);
        assert_eq!(layout.root(), Some(a));
        assert_eq!(layout.placement(a), Some(DockSlot::Root));
    }

    // --- Zero-survivor removals ---

    #[test]
    fn zero_member_tab_group_removed_with_parent_cascade() {
        let mut layout = empty_layout();
        let g = layout.put_tab_group(vec![], 0);
        let x = layout.create_panel(None);
        let root = layout
            .create_split(SplitOrientation::Row, vec![g, x], SplitSizes::uniform(2).unwrap())
            .unwrap();
        layout.set_root(Some(root)).unwrap();
        assert!(!layout.is_valid());

        layout.normalize().unwrap();
        // g had no members: nothing inherits. The parent split loses
        // the reference (weight pruned) and cascades — one child left —
        // so only x remains, rooted.
        assert!(layout.is_valid());
        assert!(!layout.contains(g) && !layout.contains(root));
        assert_eq!(layout.root(), Some(x));
        assert_eq!(layout.placement(x), Some(DockSlot::Root));
        assert_eq!(layout.len(), 1);
    }

    #[test]
    fn zero_child_split_in_tab_slot_removed_and_active_adjusted() {
        let mut layout = empty_layout();
        let zs = layout.put_split(SplitOrientation::Row, vec![], SplitSizes::default());
        let m = layout.create_panel(None);
        let group = layout.put_tab_group(vec![zs, m], 1);
        layout.set_root(Some(group)).unwrap();
        assert!(!layout.is_valid());

        layout.normalize().unwrap();
        // zs had no children: nothing inherits. The group survives (one
        // member left) and the active index shifts down from 1 to 0.
        assert!(layout.is_valid());
        assert!(!layout.contains(zs));
        assert!(layout.contains(group));
        assert_eq!(layout.get(group).unwrap().active(), Some(0));
        assert_eq!(layout.placement(m), Some(DockSlot::TabMember { parent: group, index: 0 }));
    }

    #[test]
    fn zero_survivor_in_host_region_clears_it() {
        let mut layout = DockLayout::new();
        let host = layout.create_embed_host(None); // auto-rooted
        let zs = layout.put_split(SplitOrientation::Column, vec![], SplitSizes::default());
        host_assign_top(&mut layout, host, Some(zs));
        assert!(!layout.is_valid());

        layout.normalize().unwrap();
        assert!(layout.is_valid());
        assert!(!layout.contains(zs));
        assert_eq!(
            layout.get(host).unwrap().host_regions().unwrap().on(EdgeRegion::Top),
            None
        );
    }

    #[test]
    fn one_child_split_survivor_inherits_host_region() {
        let mut layout = DockLayout::new();
        let host = layout.create_embed_host(None); // auto-rooted
        let p = layout.create_panel(None);
        let os = layout.put_split(SplitOrientation::Column, vec![p], SplitSizes::uniform(1).unwrap());
        host_assign_top(&mut layout, host, Some(os));
        assert!(!layout.is_valid());

        layout.normalize().unwrap();
        assert!(layout.is_valid());
        assert!(!layout.contains(os));
        assert_eq!(
            layout.get(host).unwrap().host_regions().unwrap().on(EdgeRegion::Top),
            Some(p)
        );
        assert_eq!(layout.placement(p), Some(DockSlot::HostEdge { host, edge: EdgeRegion::Top }));
    }

    // --- Host demotion (PRD §8.9; the temporary-host rule) ---

    #[test]
    fn empty_temporary_host_demotes_back_to_its_panel_through_the_cascade() {
        // An auto-promoted host is a temporary promotion (persistent:
        // false) — removing its last embedded dock empties it and the
        // cascade demotes it back to the panel: identity, content, and
        // root placement preserved (PRD §8.9 baseline).
        let mut layout = DockLayout::new();
        let p = layout.create_panel(Some("work area".into())); // root, expanded

        let DockingOutcome::Embedded { source, promoted, .. } =
            DockingOperation::Embed {
                source: InsertSource::NewPanel { content: None },
                host: p,
                slot: EmbedSlot::Region(EdgeRegion::Top),
            }
            .apply(&mut layout)
            .unwrap()
        else {
            panic!("expected Embedded");
        };
        assert!(promoted);
        assert_eq!(layout.get(p).unwrap().persistent(), Some(false));
        assert!(layout.is_valid());

        DockingOperation::Remove { dock: source }.apply(&mut layout).unwrap();

        // Demotion: the same dock is an (expanded) panel again.
        assert!(layout.is_valid());
        let docks = layout.get(p).unwrap();
        assert_eq!(docks.kind(), DockKind::Panel);
        assert_eq!(docks.content(), Some(&"work area".to_string()));
        assert_eq!(docks.collapsed(), Some(false));
        assert_eq!(layout.root(), Some(p));
        // Fixpoint: nothing left for the pass to resolve.
        layout.normalize().unwrap();
        assert_eq!(layout.get(p).unwrap().kind(), DockKind::Panel);
    }

    #[test]
    fn explicit_promotion_is_a_persistent_host() {
        // The explicit PromoteHost operation is an intentionally
        // persistent host (PRD §8.9): it stays a host when its last
        // embedded dock leaves, exactly like a created host.
        let mut layout = DockLayout::new();
        let p = layout.create_panel(Some("work area".into()));
        let outcome = DockingOperation::PromoteHost { dock: p }.apply(&mut layout).unwrap();
        assert!(matches!(outcome, DockingOutcome::HostPromoted { promoted: true, .. }));
        assert_eq!(layout.get(p).unwrap().persistent(), Some(true));

        let DockingOutcome::Embedded { source, .. } =
            DockingOperation::Embed {
                source: InsertSource::NewPanel { content: None },
                host: p,
                slot: EmbedSlot::Region(EdgeRegion::Top),
            }
            .apply(&mut layout)
            .unwrap()
        else {
            panic!("expected Embedded");
        };
        DockingOperation::Remove { dock: source }.apply(&mut layout).unwrap();

        assert!(layout.is_valid());
        assert_eq!(layout.get(p).unwrap().kind(), DockKind::EmbedHost);
        assert_eq!(layout.get(p).unwrap().persistent(), Some(true));
        layout.normalize().unwrap(); // fixpoint: no demotion, no change
        assert_eq!(layout.get(p).unwrap().kind(), DockKind::EmbedHost);
    }

    #[test]
    fn persistent_host_is_never_demoted_when_emptied() {
        // Created hosts are persistent (PRD §8.9): an empty persistent
        // host is a valid steady state (PRD §8.8) and no rule removes it.
        let mut layout = DockLayout::new();
        let host = layout.create_embed_host(Some("host".into())); // auto-rooted, persistent
        assert_eq!(layout.get(host).unwrap().persistent(), Some(true));
        let DockingOutcome::Embedded { source, .. } =
            DockingOperation::Embed {
                source: InsertSource::NewPanel { content: None },
                host,
                slot: EmbedSlot::Floating { rect: None },
            }
            .apply(&mut layout)
            .unwrap()
        else {
            panic!("expected Embedded");
        };
        DockingOperation::Remove { dock: source }.apply(&mut layout).unwrap();

        assert!(layout.is_valid());
        assert_eq!(layout.get(host).unwrap().kind(), DockKind::EmbedHost);
        assert_eq!(layout.get(host).unwrap().persistent(), Some(true));
        layout.normalize().unwrap(); // fixpoint: no demotion, no change
        assert_eq!(layout.get(host).unwrap().kind(), DockKind::EmbedHost);
    }

    #[test]
    fn floated_split_survivor_inherits_floating_slot_and_rect() {
        // A below-minimum split can float: the survivor copies into the
        // inherited `DockSlot::Floating` reference and the entry — with
        // its persistent rect — stays put (the §9.6 slot inheritance,
        // mirroring the §9.5 region rule).
        let mut layout = empty_layout();
        let host = layout.create_embed_host(Some("host".into())); // auto-rooted, persistent
        let a = layout.create_panel(None);
        let split =
            layout.put_split(SplitOrientation::Row, vec![a], SplitSizes::uniform(1).unwrap());
        // Import-style surgery: the below-minimum split enters the
        // layout through the host's floating collection.
        let entry_rect = FloatingRect { x: 0.25, y: 0.25, w: 0.5, h: 0.5 };
        {
            let dock = layout.get_mut(host).expect("host exists");
            if let DockRole::EmbedHost { floating, .. } = &mut dock.role {
                floating.push(FloatingEntry::new(split, entry_rect));
            }
        }
        assert!(!layout.is_valid()); // below-min state: rejected by design

        layout.normalize().unwrap();
        // The survivor inherits the floated split's slot verbatim; the
        // entry keeps its persistent rect.
        assert!(layout.is_valid());
        assert!(!layout.contains(split));
        assert_eq!(layout.placement(a), Some(DockSlot::Floating { host, index: 0 }));
        let floating = layout.get(host).unwrap().floating().unwrap();
        assert_eq!(floating.len(), 1);
        assert_eq!(floating[0].dock, a);
        assert_eq!(floating[0].rect(), entry_rect);

        // Fixpoint: normalization finds nothing else to do.
        let settled = layout.clone();
        layout.normalize().unwrap();
        assert_eq!(layout, settled);
    }

    #[test]
    fn demotion_is_an_in_place_role_change_parent_kept_verbatim() {
        // Unlike a split collapse, demotion rewrites no parent
        // collection: the host stays in its exact slot with the exact
        // slot weight.
        let mut layout = empty_layout();
        let p = layout.create_panel(None);
        let other = layout.create_panel(None);
        // Installed with the `put_split` route: `p` is still the root
        // pane here, and the public creation preconditions reject placed
        // children — the fixture clears the designation right after.
        let split_slot = layout
            .put_split(SplitOrientation::Row, vec![p, other], SplitSizes::from_weights(vec![0.7, 0.3]).unwrap());
        layout.set_root(None).unwrap(); // clear the auto-rooted designation first
        layout.set_root(Some(split_slot)).unwrap();

        // The auto-promotion makes p a temporary host in the split slot.
        let DockingOutcome::Embedded { source, .. } =
            DockingOperation::Embed {
                source: InsertSource::NewPanel { content: None },
                host: p,
                slot: EmbedSlot::Region(EdgeRegion::Left),
            }
            .apply(&mut layout)
            .unwrap()
        else {
            panic!("expected Embedded");
        };
        assert_eq!(layout.get(p).unwrap().persistent(), Some(false));

        // Imported-state surgery: the occupant vanishes and the region
        // clears without any operation.
        let dock = layout.get_mut(p).expect("host exists");
        if let DockRole::EmbedHost { regions, .. } = &mut dock.role {
            regions.assign(EdgeRegion::Left, None, 1.0);
        }
        assert!(layout.take_dock(source).is_some());
        assert!(layout.is_valid());

        layout.normalize().unwrap();
        // p is a panel again — expanded, in its exact split slot — and
        // the parent split is untouched (weights, order, child count
        // verbatim).
        assert!(layout.is_valid());
        assert_eq!(layout.get(p).unwrap().kind(), DockKind::Panel);
        assert_eq!(layout.get(p).unwrap().collapsed(), Some(false));
        assert_eq!(
            layout.placement(p),
            Some(DockSlot::SplitChild { parent: split_slot, index: 0 })
        );
        assert_eq!(
            layout.get(split_slot).unwrap().sizes().map(|s| s.weights().to_vec()),
            Some(vec![0.7, 0.3])
        );
        assert_eq!(layout.root(), Some(split_slot));
    }

    #[test]
    fn whole_layout_pass_demotes_emptied_temporary_hosts_and_is_idempotent() {
        let mut layout = DockLayout::new();
        let p = layout.create_panel(None);
        // The Embed auto-promotion is a *temporary* promotion (PRD
        // §8.9): p becomes a persistent:false host around the embedded
        // source.
        let DockingOutcome::Embedded { source, .. } =
            DockingOperation::Embed {
                source: InsertSource::NewPanel { content: None },
                host: p,
                slot: EmbedSlot::Region(EdgeRegion::Top),
            }
            .apply(&mut layout)
            .unwrap()
        else {
            panic!("expected Embedded");
        };
        // Imported-state surgery: the occupant vanishes and the region
        // is cleared without any operation (the way a restored layout
        // could carry an emptied temporary host — structurally valid,
        // but unnormalized).
        let dock = layout.get_mut(p).expect("host exists");
        if let DockRole::EmbedHost { regions, .. } = &mut dock.role {
            regions.assign(EdgeRegion::Top, None, 1.0);
        }
        assert!(layout.take_dock(source).is_some());
        assert!(layout.is_valid());
        let before = layout.clone();

        layout.normalize().unwrap();
        assert!(layout.is_valid());
        let once = layout.clone();
        assert_eq!(layout.get(p).unwrap().kind(), DockKind::Panel);
        layout.normalize().unwrap();
        assert_eq!(layout, once); // idempotent — and the removal solves nothing else
        assert_ne!(layout, before);
    }

    // --- Failure behavior and pass guarantees ---

    #[test]
    fn prune_failure_propagates_without_rewriting_the_parent() {
        let mut layout = empty_layout();
        let zs = layout.put_split(SplitOrientation::Row, vec![], SplitSizes::default());
        let x = layout.create_panel(None);
        let parent = layout.put_split(
            SplitOrientation::Column,
            vec![zs, x],
            // Imported garbage sizing: a non-finite weight where pruning
            // zs's index would leave only the invalid weight.
            SplitSizes::from_weights_unchecked(vec![1.0, f64::NAN]),
        );
        layout.set_root(Some(parent)).unwrap();
        let before = layout.clone();
        assert!(!layout.is_valid());

        let err = layout.normalize().unwrap_err();
        // The offender is named in the parent's weight order.
        let DockingError::InvalidSizeState { split, index } = err else {
            panic!("expected InvalidSizeState");
        };
        assert_eq!(split, parent);
        assert_eq!(index, 1);
        // The failing step checked before it mutated: the layout is
        // exactly as it arrived and a repeated call reports the same
        // error (deterministic, not a one-shot corruption). The compare
        // goes through Debug — `f64::NAN != NAN` under `PartialEq`.
        assert_eq!(format!("{layout:?}"), format!("{before:?}"));
        assert!(!layout.is_valid());
        let again = layout.normalize().unwrap_err();
        assert_eq!(again, DockingError::InvalidSizeState { split: parent, index: 1 });
    }

    #[test]
    fn full_pass_resolves_multiple_independent_below_min_containers() {
        let (mut layout, s1, s2, a, b) = two_independent_below_min();
        assert!(!layout.is_valid());

        layout.normalize().unwrap();
        assert!(layout.is_valid());
        assert!(!layout.contains(s1) && !layout.contains(s2));
        let root = layout.root().unwrap();
        // Relative order of the survivors is preserved.
        assert_eq!(layout.descendants(root), vec![a, b]);
        assert_eq!(layout.root(), Some(root));
    }

    #[test]
    fn unplaced_below_min_container_left_for_validation() {
        let mut layout = DockLayout::new();
        let root_panel = layout.create_panel(None); // valid root
        let q = layout.create_panel(None); // unplaced (populated layout)
        let s = layout.put_split(SplitOrientation::Row, vec![q], SplitSizes::uniform(1).unwrap());
        let before = layout.clone();
        assert!(!layout.is_valid()); // `s` is below minimum

        // Normalization resolves no slot it cannot see: `s` stays for
        // the Phase-3 gate to reject, everything else is untouched.
        layout.normalize().unwrap();
        assert_eq!(layout, before);
        assert!(layout.contains(s) && layout.contains(q));
        assert_eq!(layout.root(), Some(root_panel));
    }

    #[test]
    fn repeated_normalization_is_idempotent() {
        let (mut layout, _, _, _, _) = two_independent_below_min();
        layout.normalize().unwrap();
        assert!(layout.is_valid());
        let once = layout.clone();
        layout.normalize().unwrap();
        layout.normalize().unwrap();
        assert_eq!(layout, once);
        // Also idempotent on already-committed (operation-produced) state.
        layout.normalize().unwrap();
        assert_eq!(layout, once);
    }

    #[test]
    fn chain_driver_and_full_pass_agree() {
        // The operation path (a rootward chain from one container) and
        // the whole-layout pass land on the same layout when the chain
        // covers every below-minimum container — the case every
        // operation candidate is in.
        let (mut full, _inner, _a) = nested_below_min_chain();
        full.normalize().unwrap();
        let (mut chained, inner, a) = nested_below_min_chain();
        collapse_cascade(&mut chained, &mut UndoData::default(), inner).unwrap();
        assert_eq!(full, chained);
        assert!(full.is_valid());
        assert_eq!(full.root(), Some(a));
    }

    #[test]
    fn collapsed_member_is_an_ordinary_member_to_normalization() {
        // A collapsed panel is presentation logic only (PRD §9.7): the
        // flag changes no placement or sizing data, so the split holding
        // it resolves exactly as if the dock were expanded, and the
        // cascade leaves the flag untouched.
        let mut layout = empty_layout();
        let a = layout.create_panel(None);
        let outcome = DockingOperation::SetDockCollapsed { dock: a, collapsed: true }
            .apply(&mut layout)
            .unwrap();
        assert!(matches!(outcome, DockingOutcome::DockCollapsedSet { .. }));

        // Now make the collapsed dock the sole child of a one-child
        // split root: below minimum, resolved like any other member.
        layout.set_root(None).unwrap();
        let s = layout.put_split(SplitOrientation::Row, vec![a], SplitSizes::uniform(1).unwrap());
        layout.set_root(Some(s)).unwrap();
        assert!(!layout.is_valid());

        layout.normalize().unwrap();
        assert!(layout.is_valid());
        assert_eq!(layout.root(), Some(a));
        assert_eq!(layout.placement(a), Some(DockSlot::Root));
        // The preference survived the collapse rewrite untouched.
        assert_eq!(layout.get(a).unwrap().collapsed(), Some(true));
    }
}

