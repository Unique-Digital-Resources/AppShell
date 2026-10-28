//! Structural validation: the invariants every committed docking layout
//! must satisfy (current_state/plan.md Phase 03, PRD §7.7).
//!
//! [`DockLayout::validate`] is the whole-layout authority for structural
//! invariants. It exists because the creation primitives guarantee
//! validity only for the pieces they build: docks move through explicit
//! unplaced / rootless states (PRD DM-07), and — most importantly —
//! layouts restored from persisted state (Phase 10) are arbitrary and
//! must be checked on import (PRD §5 "Validation system").
//!
//! ## What validate checks, and in which order
//!
//! `validate` reports the **first** violated rule, scanning docks in
//! ascending [`DockId`] order so the reported error is deterministic for
//! a given broken state (PRD §17.2). The check order documents rule
//! precedence:
//!
//! 1. **Root designation** — a layout with docks must designate a root
//!    ([`MissingRoot`](super::error::DockingError::MissingRoot)); the
//!    root pointer must reference an existing dock
//!    ([`UnknownReference`](super::error::DockingError::UnknownReference)).
//! 2. **Reference validity and category parity**, per dock in id order:
//!    every structural reference (split children, tab members, host edge
//!    regions, floating entries) must point at an existing dock, then
//!    each category's steady-state composition (split child/size parity,
//!    tab membership and active index, sizing weights).
//! 3. **Placement multiplicity**, per dock in id order: exactly one
//!    placement — none is an
//!    [`OrphanDock`](super::error::DockingError::OrphanDock), several is
//!    an [`InvalidRoot`](super::error::DockingError::InvalidRoot) when
//!    the root is also owned by a container, otherwise
//!    [`DuplicatePlacement`](super::error::DockingError::DuplicatePlacement).
//! 4. **Acyclicity** — a colour DFS over all structural edges in any
//!    placement category
//!    ([`CycleDetected`](super::error::DockingError::CycleDetected)).
//!
//! A panel's logical collapse preference (PRD §9.7) is deliberately
//! **non-structural** and skipped: it changes no reference, membership,
//! or sizing data, so every collapsed panel is as valid as its expanded
//! self (collapse must never make a dock unreachable).
//!
//! Region sizing (PRD §9.5): an occupied edge region must hold a valid
//! stored weight (finite, strictly positive — the verbatim
//! representation invariant). The stored weight of an **unoccupied**
//! region is inert (weights live with the region lifecycle; imports
//! without weight fields restore at the baseline) and is not checked.
//!
//! Floating state (PRD §9.6): every floating entry must hold a valid
//! stored rect — finite parts, non-empty, within the host's logical
//! bounds (`x + w ≤ 1`, `y + h ≤ 1`, endpoints inclusive). Entries
//! check in collection order after the host's region weights (the
//! `structural_children` edge order), reporting the first violating
//! entry's index deterministically. Imports without rect fields
//! restore at the floating baseline (the `HostRegions` weights
//! precedent), so only explicit garbage trips
//! [`InvalidFloatingState`](super::error::DockingError::InvalidFloatingState).
//!
//! Rules 1–3 together imply every dock is reachable from the root (each
//! dock has exactly one owner, every owner chain is acyclic, and only
//! the root has no owner); tests assert this derived invariant per valid
//! fixture rather than re-implementing it.
//!
//! ## What validate deliberately does *not* do
//!
//! - It is a **steady-state** check. Docks legitimately pass through
//!   unplaced / rootless positions mid-operation (PRD DM-07); operations
//!   (Phase 4) validate their *committed* result, not their internal
//!   build steps. Calling `validate` mid-operation therefore reports the
//!   transient state as invalid — by design.
//! - It is not an operation validator. Candidate-op guards (descendant
//!   cycles, occupied regions, eligibility) arrive with the operations
//!   themselves (Phase 4).
//! - Creation paths are *not* re-validated here: they are
//!   valid-by-construction and their preconditions already reject bad
//!   inputs ([`create_split`](super::model::DockLayout::create_split),
//!   [`create_tab_group`](super::model::DockLayout::create_tab_group)).

use std::collections::HashMap;

use super::dock::{DockId, DockRole};
use super::error::DockingError;
use super::model::DockLayout;
use super::slot::EdgeRegion;

/// DFS colour marking for cycle detection: `Open` nodes sit on the walk
/// stack; a reference back to an `Open` node is a cycle witness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Color {
    Open,
    Done,
}

impl DockLayout {
    /// Validate the whole layout, returning the first violated
    /// invariant (see the module docs for the check order).
    ///
    /// Succeeds for the explicit empty layout, for every layout produced
    /// by creation primitives once placement is complete, and — from
    /// Phase 4 on — for every committed operation result. This is the
    /// gate used to verify imported (restored) state (PRD §7.7, §16).
    pub fn validate(&self) -> Result<(), DockingError> {
        // 1. Root designation.
        match self.root() {
            Some(root) => {
                if !self.contains(root) {
                    return Err(DockingError::UnknownReference { id: root });
                }
            }
            None => {
                if self.len() != 0 {
                    return Err(DockingError::MissingRoot { docks: self.len() });
                }
            }
        }

        // Deterministic report order: ascending dock ids.
        let mut ids = self.all_docks();
        ids.sort();

        // 2. Reference validity and category parity, per dock.
        for id in &ids {
            let dock = self
                .get(*id)
                .expect("ids are read from the layout map itself");
            // (a) Every structural reference must point at an existing
            // dock — split children, tab members, host edge regions in
            // fixed top/bottom/left/right order, then floating entries
            // (the canonical `structural_children` edge set, PRD §7.7).
            for child in dock.structural_children() {
                if !self.contains(child) {
                    return Err(DockingError::UnknownReference { id: child });
                }
            }
            // (b) Steady-state composition per category.
            match &dock.role {
                DockRole::Split { children, sizes, .. } => {
                    if children.len() < 2 {
                        return Err(DockingError::InsufficientChildren {
                            minimum: 2,
                            found: children.len(),
                        });
                    }
                    if !sizes.matches(children.len()) {
                        return Err(DockingError::SizeCountMismatch {
                            sizes: sizes.len(),
                            children: children.len(),
                        });
                    }
                    if let Some((index, _)) = sizes
                        .weights()
                        .iter()
                        .enumerate()
                        .find(|(_, w)| !w.is_finite() || **w <= 0.0)
                    {
                        return Err(DockingError::InvalidSizeState { split: *id, index });
                    }
                }
                DockRole::TabGroup { members, active, .. } => {
                    if members.is_empty() {
                        return Err(DockingError::InsufficientChildren {
                            minimum: 1,
                            found: 0,
                        });
                    }
                    if *active >= members.len() {
                        return Err(DockingError::ActiveIndexOutOfRange {
                            index: *active,
                            member_count: members.len(),
                        });
                    }
                }
                DockRole::EmbedHost { regions, floating, .. } => {
                    // Region sizing (PRD §9.5): an occupied region must
                    // hold a finite, strictly positive stored weight.
                    // The stored value of an unoccupied region is inert
                    // (see the module docs) and skipped.
                    let weights = [
                        (EdgeRegion::Top, regions.weight_on(EdgeRegion::Top)),
                        (EdgeRegion::Bottom, regions.weight_on(EdgeRegion::Bottom)),
                        (EdgeRegion::Left, regions.weight_on(EdgeRegion::Left)),
                        (EdgeRegion::Right, regions.weight_on(EdgeRegion::Right)),
                    ];
                    for (edge, weight) in weights {
                        if let Some(w) = weight {
                            if !w.is_finite() || w <= 0.0 {
                                return Err(DockingError::InvalidRegionState {
                                    host: *id,
                                    edge,
                                });
                            }
                        }
                    }
                    // Floating state (PRD §9.6): every entry's stored
                    // rect must satisfy the rect invariant and the
                    // host-bounds rule (finite parts, non-empty,
                    // `x + w ≤ 1`, `y + h ≤ 1` — endpoints inclusive).
                    // Entries check in collection order: the first
                    // violating index is reported deterministically
                    // (the region rule's scan-order sibling).
                    for (index, entry) in floating.iter().enumerate() {
                        if !entry.rect().is_valid() {
                            return Err(DockingError::InvalidFloatingState { host: *id, index });
                        }
                    }
                }
                DockRole::Panel { .. } => {}
            }
        }

        // 3. Placement multiplicity, per dock.
        for id in &ids {
            let count = self.placement_count(*id);
            if count == 0 {
                return Err(DockingError::OrphanDock { id: *id });
            }
            if count > 1 {
                if Some(*id) == self.root() {
                    // The root must not be owned by a container; its only
                    // correct placement is the Root slot.
                    return Err(DockingError::InvalidRoot { id: *id });
                }
                return Err(DockingError::DuplicatePlacement { id: *id });
            }
        }

        // 4. Acyclicity: colour DFS over every structural edge set.
        let mut color: HashMap<DockId, Color> = HashMap::new();
        for start in &ids {
            if color.contains_key(start) {
                continue;
            }
            if let Some(witness) = walk_for_cycle(self, *start, &mut color) {
                return Err(DockingError::CycleDetected { id: witness });
            }
        }

        Ok(())
    }

    /// `true` when the layout satisfies every structural invariant
    /// ([`validate`](Self::validate) reports no violation).
    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }
}

/// Recursive colour DFS from `id`. Returns the first dock reached while
/// it is still `Open` — the witness of a structural cycle — or `None`
/// when `id`'s reachable subgraph is acyclic and fully explored.
///
/// All placement categories participate: [`Dock`](super::dock::Dock)'s
/// `structural_children` covers split children, tab members, host edge
/// regions (fixed scan order), and floating collections (PRD §7.7).
fn walk_for_cycle(
    layout: &DockLayout,
    id: DockId,
    color: &mut HashMap<DockId, Color>,
) -> Option<DockId> {
    color.insert(id, Color::Open);
    if let Some(dock) = layout.get(id) {
        for child in dock.structural_children() {
            match color.get(&child) {
                Some(Color::Open) => return Some(child),
                Some(Color::Done) => {}
                None => {
                    if let Some(witness) = walk_for_cycle(layout, child, color) {
                        return Some(witness);
                    }
                }
            }
        }
    }
    color.insert(id, Color::Done);
    None
}

#[cfg(test)]
mod validation_tests {
    //! In-crate fixtures use `pub(crate)` mutation (`put_dock`,
    //! `take_dock`, `get_mut`, `assign`, and the test-only
    //! `SplitSizes::from_weights_unchecked`) to assemble invalid layouts
    //! exactly the way imported/restored state can carry them. The
    //! public API alone cannot produce most of these states — creation
    //! preconditions and op-only mutation guard against it — which is
    //! why external validation tests cover the constructible rules
    //! (`MissingRoot`, `OrphanDock`) and this module covers the rest.

    use super::*;

    use crate::domain_foundation::environments::docking::{
        Dock, EdgeRegion, FloatingEntry, FloatingRect, HostRegions, SplitOrientation, SplitSizes,
    };

    fn panel(id: u64) -> Dock {
        Dock::new(
            DockId::from_raw(id),
            DockRole::Panel { content: None, collapsed: false },
        )
    }

    fn split(id: u64, children: Vec<DockId>, sizes: SplitSizes) -> Dock {
        Dock::new(
            DockId::from_raw(id),
            DockRole::Split {
                orientation: SplitOrientation::Row,
                children,
                sizes,
            },
        )
    }

    fn tab_group(id: u64, members: Vec<DockId>, active: usize) -> Dock {
        Dock::new(
            DockId::from_raw(id),
            DockRole::TabGroup { members, active },
        )
    }

    fn embed_host(id: u64) -> Dock {
        Dock::new(
            DockId::from_raw(id),
            DockRole::EmbedHost {
                content: None,
                regions: HostRegions::new(),
                floating: Vec::new(),
                persistent: true,
            },
        )
    }

    /// Wire `host`'s top region and floating list through the
    /// `pub(crate)` dock access. Region occupancy and floating
    /// collection surgery is operation territory (Phases 7–8);
    /// validation fixtures need the direct path to exercise `HostEdge`
    /// / `Floating` invariants (restored-state assembly). Baseline
    /// rects keep the entered entries valid.
    fn host_place(
        layout: &mut DockLayout,
        host: DockId,
        top: Option<DockId>,
        floating: Vec<DockId>,
    ) {
        let dock = layout.get_mut(host).expect("fixture host exists");
        if let DockRole::EmbedHost { regions, floating: list, .. } = &mut dock.role {
            regions.assign(EdgeRegion::Top, top, 1.0);
            *list = floating
                .into_iter()
                .map(|floater| FloatingEntry::new(floater, FloatingRect::default()))
                .collect();
        }
    }

    // --- Valid fixtures (in-crate: full placement categories) ---

    #[test]
    fn valid_rich_fixture_passes() {
        let mut layout = DockLayout::new();
        // Four panels; the first auto-roots so the layout detaches it
        // before building containers over the unplaced docks.
        let a = layout.create_panel(None);
        let b = layout.create_panel(None);
        let c = layout.create_panel(None);
        let d = layout.create_panel(None);
        layout.set_root(None).unwrap();

        let group_a = layout.create_tab_group(vec![a, b], 0).unwrap();
        let group_b = layout.create_tab_group(vec![c, d], 0).unwrap();
        let host = layout.create_embed_host(None);
        let float = layout.create_panel(None);

        // Assemble: the host occupies its top region with `group_a` and
        // keeps `float` floating; the root split places host and
        // group_b. Every dock ends up owned exactly once, across all
        // placement categories.
        host_place(&mut layout, host, Some(group_a), vec![float]);
        let root = DockId::from_raw(900);
        layout.put_dock(split(900, vec![host, group_b], SplitSizes::uniform(2).unwrap()));
        layout.set_root(Some(root)).unwrap();

        assert!(layout.is_valid());
        assert_eq!(layout.validate(), Ok(()));
        // Derived invariant: every dock is reachable from the root
        // (descendants excludes the root itself).
        assert_eq!(layout.descendants(root).len() + 1, layout.len());
    }

    #[test]
    fn empty_layout_is_valid() {
        let layout = DockLayout::new();
        assert!(layout.is_valid());
        assert_eq!(layout.validate(), Ok(()));
    }

    // --- Reference validity ---

    #[test]
    fn missing_reference_in_split_children_rejected() {
        let mut layout = DockLayout::new();
        let _root = layout.create_panel(None); // auto-root, panel
        layout.put_dock(split(
            901,
            vec![DockId::from_raw(0), DockId::from_raw(999)],
            SplitSizes::uniform(2).unwrap(),
        ));
        assert_eq!(
            layout.validate(),
            Err(DockingError::UnknownReference {
                id: DockId::from_raw(999)
            })
        );
    }

    #[test]
    fn missing_reference_in_tab_members_rejected() {
        let mut layout = DockLayout::new();
        let a = layout.create_panel(None); // root
        layout.put_dock(tab_group(902, vec![a, DockId::from_raw(988)], 0));
        assert_eq!(
            layout.validate(),
            Err(DockingError::UnknownReference {
                id: DockId::from_raw(988)
            })
        );
    }

    #[test]
    fn missing_root_reference_rejected() {
        let mut layout = DockLayout::new();
        let a = layout.create_panel(None);
        layout.put_dock(tab_group(909, vec![a], 0));
        // Undo-path surgery: remove the dock that is the root; the root
        // pointer dangles and is reported first (rule 1 precedes the tab
        // group's own dangling member).
        layout.take_dock(a);
        assert_eq!(
            layout.validate(),
            Err(DockingError::UnknownReference { id: a })
        );
    }

    #[test]
    fn dangling_root_without_docks_rejected() {
        let mut layout = DockLayout::new();
        let a = layout.create_panel(None);
        layout.take_dock(a);
        // Zero docks with a root pointer: not the explicit empty layout.
        assert_eq!(
            layout.validate(),
            Err(DockingError::UnknownReference { id: a })
        );
        assert!(!layout.is_valid());
    }

    #[test]
    fn missing_reference_in_host_region_and_floating_rejected() {
        // Fixed scan order: regions (top, bottom, left, right) before
        // floating — the region's missing dock is reported first. The
        // host is auto-rooted and stays rooted; occupants are wired
        // through the `pub(crate)` region access.
        let mut layout = DockLayout::new();
        let host = layout.create_embed_host(None);
        host_place(
            &mut layout,
            host,
            Some(DockId::from_raw(910)),
            vec![DockId::from_raw(911)],
        );
        assert_eq!(
            layout.validate(),
            Err(DockingError::UnknownReference {
                id: DockId::from_raw(910)
            })
        );

        // Floating-only dangling reference.
        let mut layout = DockLayout::new();
        let host = layout.create_embed_host(None);
        host_place(&mut layout, host, None, vec![DockId::from_raw(911)]);
        assert_eq!(
            layout.validate(),
            Err(DockingError::UnknownReference {
                id: DockId::from_raw(911)
            })
        );
    }

    // --- Placement multiplicity ---

    #[test]
    fn orphan_dock_rejected() {
        let mut layout = DockLayout::new();
        let _root = layout.create_panel(None);
        layout.put_dock(panel(950));
        assert_eq!(
            layout.validate(),
            Err(DockingError::OrphanDock {
                id: DockId::from_raw(950)
            })
        );
        assert!(!layout.is_valid());
    }

    #[test]
    fn duplicate_placement_across_containers_rejected() {
        let mut layout = DockLayout::new();
        let a = layout.create_panel(None);
        let b = layout.create_panel(None);
        let c = layout.create_panel(None);
        layout.set_root(None).unwrap();
        let first = layout
            .create_split(SplitOrientation::Row, vec![a, b], SplitSizes::uniform(2).unwrap())
            .unwrap();
        layout.set_root(Some(first)).unwrap();
        // A second container claims `a`, which is already placed. The
        // `pub(crate)` bypass mirrors what restore/import could carry.
        layout.put_dock(split(903, vec![a, c], SplitSizes::uniform(2).unwrap()));
        assert_eq!(
            layout.validate(),
            Err(DockingError::DuplicatePlacement { id: a })
        );
    }

    #[test]
    fn duplicate_membership_within_one_parent_rejected() {
        let mut layout = DockLayout::new();
        let a = layout.create_panel(None);
        let b = layout.create_panel(None);
        // The split lists `a` twice (three entries, three weights). Make
        // the split the root so `a` is an ordinary member and the
        // multiplicity rule — not `InvalidRoot` — is the violated one.
        layout.put_dock(split(
            904,
            vec![a, a, b],
            SplitSizes::uniform(3).unwrap(),
        ));
        layout.set_root(None).unwrap();
        layout.set_root(Some(DockId::from_raw(904))).unwrap();
        assert_eq!(
            layout.validate(),
            Err(DockingError::DuplicatePlacement { id: a })
        );
    }

    #[test]
    fn root_also_placed_rejected() {
        let mut layout = DockLayout::new();
        let a = layout.create_panel(None); // root by auto-rooting
        let b = layout.create_panel(None);
        layout.put_dock(split(905, vec![a, b], SplitSizes::uniform(2).unwrap()));
        // `a` owns the Root slot *and* is a split child: the root must
        // have no owner.
        assert_eq!(layout.validate(), Err(DockingError::InvalidRoot { id: a }));
    }

    #[test]
    fn missing_root_with_docks_rejected() {
        let mut layout = DockLayout::new();
        let _ = layout.create_panel(None);
        let _ = layout.create_panel(None);
        // Public-API transient state: clear the root, docks remain.
        layout.set_root(None).unwrap();
        assert_eq!(
            layout.validate(),
            Err(DockingError::MissingRoot { docks: 2 })
        );
    }

    // --- Category parity ---

    #[test]
    fn split_below_minimum_children_rejected() {
        let mut layout = DockLayout::new();
        let a = layout.create_panel(None); // root
        layout.put_dock(split(906, vec![a], SplitSizes::uniform(1).unwrap()));
        assert_eq!(
            layout.validate(),
            Err(DockingError::InsufficientChildren {
                minimum: 2,
                found: 1
            })
        );
    }

    #[test]
    fn split_size_count_mismatch_rejected() {
        let mut layout = DockLayout::new();
        let a = layout.create_panel(None);
        let b = layout.create_panel(None);
        layout.put_dock(split(907, vec![a, b], SplitSizes::uniform(1).unwrap()));
        assert_eq!(
            layout.validate(),
            Err(DockingError::SizeCountMismatch {
                sizes: 1,
                children: 2
            })
        );
    }

    #[test]
    fn invalid_size_state_rejected() {
        // Not-a-number weight at child index 1.
        let mut layout = DockLayout::new();
        let a = layout.create_panel(None);
        let b = layout.create_panel(None);
        layout.put_dock(split(
            908,
            vec![a, b],
            SplitSizes::from_weights_unchecked(vec![1.0, f64::NAN]),
        ));
        assert_eq!(
            layout.validate(),
            Err(DockingError::InvalidSizeState {
                split: DockId::from_raw(908),
                index: 1
            })
        );

        // Non-positive weight at child index 0.
        let mut layout = DockLayout::new();
        let a = layout.create_panel(None);
        let b = layout.create_panel(None);
        layout.put_dock(split(
            909,
            vec![a, b],
            SplitSizes::from_weights_unchecked(vec![0.0, 1.0]),
        ));
        assert_eq!(
            layout.validate(),
            Err(DockingError::InvalidSizeState {
                split: DockId::from_raw(909),
                index: 0
            })
        );

        // Non-finite (positive infinity) weight.
        let mut layout = DockLayout::new();
        layout.put_dock(split(
            912,
            vec![DockId::from_raw(0), DockId::from_raw(1)],
            SplitSizes::from_weights_unchecked(vec![1.0, f64::INFINITY]),
        ));
        layout.set_root(Some(DockId::from_raw(912))).unwrap();
        // Both referenced docks are missing; reference validity (rule 2a)
        // is reported because it precedes the sizing rule.
        assert_eq!(
            layout.validate(),
            Err(DockingError::UnknownReference {
                id: DockId::from_raw(0)
            })
        );

        // The infinity case with existing docks.
        let mut layout = DockLayout::new();
        let a = layout.create_panel(None);
        let b = layout.create_panel(None);
        layout.put_dock(split(
            910,
            vec![a, b],
            SplitSizes::from_weights_unchecked(vec![1.0, f64::INFINITY]),
        ));
        assert_eq!(
            layout.validate(),
            Err(DockingError::InvalidSizeState {
                split: DockId::from_raw(910),
                index: 1
            })
        );
    }

    #[test]
    fn tab_group_without_members_rejected() {
        let mut layout = DockLayout::new();
        let _root = layout.create_panel(None);
        layout.put_dock(tab_group(913, vec![], 0));
        assert_eq!(
            layout.validate(),
            Err(DockingError::InsufficientChildren {
                minimum: 1,
                found: 0
            })
        );
    }

    #[test]
    fn tab_active_out_of_range_rejected() {
        let mut layout = DockLayout::new();
        let a = layout.create_panel(None); // root
        layout.put_dock(tab_group(914, vec![a], 5));
        // `a`'s dual role (root + member) would be rule 3; the active
        // index rule (2b) wins on scan order.
        assert_eq!(
            layout.validate(),
            Err(DockingError::ActiveIndexOutOfRange {
                index: 5,
                member_count: 1
            })
        );
    }

    // --- Cycles ---

    #[test]
    fn split_listed_as_its_own_child_rejected() {
        let mut layout = DockLayout::new();
        let x = layout.create_panel(None);
        let root = layout.create_panel(None);
        layout.set_root(None).unwrap();
        layout.set_root(Some(root)).unwrap();
        // Container 904 lists itself plus `x`; every placement is unique
        // (904 owns itself once), so only the cycle check can flag it.
        layout.put_dock(split(
            904,
            vec![DockId::from_raw(904), x],
            SplitSizes::uniform(2).unwrap(),
        ));
        assert_eq!(
            layout.validate(),
            Err(DockingError::CycleDetected {
                id: DockId::from_raw(904)
            })
        );
    }

    #[test]
    fn mutual_cycle_between_two_splits_rejected() {
        let mut layout = DockLayout::new();
        let x = layout.create_panel(None);
        let y = layout.create_panel(None);
        let root = layout.create_panel(None);
        layout.set_root(None).unwrap();
        layout.set_root(Some(root)).unwrap();
        let s1 = DockId::from_raw(915);
        let s2 = DockId::from_raw(916);
        layout.put_dock(split(915, vec![s2, x], SplitSizes::uniform(2).unwrap()));
        layout.put_dock(split(916, vec![s1, y], SplitSizes::uniform(2).unwrap()));
        // All placements unique (915 ← 916, 916 ← 915, x ← 915, y ← 916);
        // only rule 4 can flag this state.
        assert_eq!(layout.validate(), Err(DockingError::CycleDetected { id: s1 }));
    }

    #[test]
    fn cycle_through_host_region_rejected() {
        let mut layout = DockLayout::new();
        let root = layout.create_panel(None); // panel root, untouched
        layout.set_root(None).unwrap();
        layout.put_dock(embed_host(917));
        layout.put_dock(split(
            918,
            vec![DockId::from_raw(917), DockId::from_raw(919)],
            SplitSizes::uniform(2).unwrap(),
        ));
        layout.put_dock(panel(919));
        layout.set_root(Some(root)).unwrap();
        host_place(
            &mut layout,
            DockId::from_raw(917),
            Some(DockId::from_raw(918)),
            vec![],
        );
        // 917 (host) → region → 918 (split) → child → 917. An edge
        // region counts as containment for cycles (PRD §10.4).
        assert_eq!(
            layout.validate(),
            Err(DockingError::CycleDetected {
                id: DockId::from_raw(917)
            })
        );
    }

    // --- Deterministic rule precedence ---

    #[test]
    fn references_are_reported_before_placements_and_cycles() {
        let mut layout = DockLayout::new();
        let a = layout.create_panel(None); // root
        layout.put_dock(panel(950)); // orphan (rule 3)
        layout.put_dock(tab_group(951, vec![a, DockId::from_raw(999)], 0)); // dangling member (rule 2)
        layout.put_dock(split(
            952,
            vec![DockId::from_raw(952), DockId::from_raw(952)],
            SplitSizes::uniform(2).unwrap(), // self-cycle (rule 4)
        ));
        // Reference validity outranks multiplicity and acyclicity.
        assert_eq!(
            layout.validate(),
            Err(DockingError::UnknownReference {
                id: DockId::from_raw(999)
            })
        );
    }

    #[test]
    fn placements_are_reported_before_cycles() {
        let mut layout = DockLayout::new();
        let _root = layout.create_panel(None);
        layout.put_dock(panel(950)); // orphan (rule 3)
        layout.put_dock(split(
            960,
            vec![DockId::from_raw(960), DockId::from_raw(961)],
            SplitSizes::uniform(2).unwrap(), // self-cycle (rule 4)
        ));
        layout.put_dock(panel(961)); // 961's only placement: 960's child
        // The orphan (lowest offending id) is reported before the cycle.
        assert_eq!(
            layout.validate(),
            Err(DockingError::OrphanDock {
                id: DockId::from_raw(950)
            })
        );
    }

    // --- Validation is a pure check ---

    #[test]
    fn validate_does_not_mutate_the_layout() {
        let mut layout = DockLayout::new();
        let a = layout.create_panel(None);
        let b = layout.create_panel(None);
        layout.set_root(None).unwrap();
        let row = layout
            .create_split(SplitOrientation::Row, vec![a, b], SplitSizes::uniform(2).unwrap())
            .unwrap();
        layout.set_root(Some(row)).unwrap();
        let before = layout.clone();
        let _ = layout.validate();
        let _ = layout.validate();
        assert_eq!(layout, before);
        assert!(layout.is_valid());
    }

    /// Imported (restored) layouts are the reason `validate` exists: a
    /// structurally well-formed payload can still violate every
    /// invariant the creation primitives enforce. These fixtures feed
    /// raw JSON through the serde path so validation is proven to work
    /// on deserialized state (PRD §7.7, §16; the fixtures are reachable
    /// without any fixture surgery).
    #[cfg(all(test, feature = "serde"))]
    mod serde_import {
        use super::*;

        #[test]
        fn imported_invalid_size_state_rejected() {
            let json = r#"{
                "docks": {
                    "1": { "id": 1, "role": { "Split": { "orientation": "Row",
                            "children": [1, 2], "sizes": [1.0, -2.5] } } },
                    "2": { "id": 2, "role": { "Panel": { "content": null } } }
                },
                "root": 1,
                "next_id": 3
            }"#;
            let layout: DockLayout = serde_json::from_str(json).unwrap();
            assert_eq!(
                layout.validate(),
                Err(DockingError::InvalidSizeState {
                    split: DockId::from_raw(1),
                    index: 1
                })
            );
        }

        #[test]
        fn imported_dangling_reference_rejected() {
            let json = r#"{
                "docks": {
                    "1": { "id": 1, "role": { "Split": { "orientation": "Row",
                            "children": [1, 7], "sizes": [1.0, 1.0] } } }
                },
                "root": 1,
                "next_id": 2
            }"#;
            let layout: DockLayout = serde_json::from_str(json).unwrap();
            assert_eq!(
                layout.validate(),
                Err(DockingError::UnknownReference { id: DockId::from_raw(7) })
            );
        }

        #[test]
        fn imported_orphan_rejected() {
            let json = r#"{
                "docks": {
                    "1": { "id": 1, "role": { "Panel": { "content": null } } },
                    "2": { "id": 2, "role": { "Panel": { "content": null } } }
                },
                "root": 1,
                "next_id": 3
            }"#;
            let layout: DockLayout = serde_json::from_str(json).unwrap();
            assert_eq!(
                layout.validate(),
                Err(DockingError::OrphanDock { id: DockId::from_raw(2) })
            );
        }

        #[test]
        fn imported_valid_layout_accepted() {
            let json = r#"{
                "docks": {
                    "1": { "id": 1, "role": { "Split": { "orientation": "Row",
                            "children": [2, 3], "sizes": [0.7, 0.3] } } },
                    "2": { "id": 2, "role": { "Panel": { "content": "editor" } } },
                    "3": { "id": 3, "role": { "Panel": { "content": null } } }
                },
                "root": 1,
                "next_id": 4
            }"#;
            let layout: DockLayout = serde_json::from_str(json).unwrap();
            assert_eq!(layout.validate(), Ok(()));
            assert!(layout.is_valid());
        }

        #[test]
        fn imported_valid_host_region_sizing_accepted() {
            // PRD §9.5: an occupied region's weight satisfying the
            // sizing invariant plus a floating entry import cleanly.
            // The entry's absent rect fields restore at the floating
            // baseline (the struct-level `serde(default)` — the
            // `HostRegions` weights precedent).
            let json = r#"{
                "docks": {
                    "1": { "id": 1, "role": { "EmbedHost": { "content": "host",
                            "regions": { "top": 2, "top_weight": 2.5,
                                        "bottom": null, "bottom_weight": 1.0,
                                        "left": null, "left_weight": 1.0,
                                        "right": null, "right_weight": 1.0 },
                            "floating": [ { "dock": 3 } ], "persistent": true } } },
                    "2": { "id": 2, "role": { "Panel": { "content": "p" } } },
                    "3": { "id": 3, "role": { "Panel": { "content": "q" } } }
                },
                "root": 1,
                "next_id": 4
            }"#;
            let layout: DockLayout = serde_json::from_str(json).unwrap();
            assert_eq!(layout.validate(), Ok(()));
            assert!(layout.is_valid());
            let regions = layout
                .get(DockId::from_raw(1))
                .unwrap()
                .host_regions()
                .unwrap();
            assert_eq!(regions.weight_on(EdgeRegion::Top), Some(2.5));
            let floating = layout.get(DockId::from_raw(1)).unwrap().floating().unwrap();
            assert_eq!(floating.len(), 1);
            assert_eq!(floating[0].dock, DockId::from_raw(3));
            assert_eq!(floating[0].rect(), FloatingRect::default()); // the baseline
        }

        #[test]
        fn imported_invalid_region_state_rejected() {
            // PRD §9.5: an occupied region's stored weight must satisfy
            // the sizing invariant, and the first violating region is
            // reported in the fixed top/bottom/left/right scan order
            // (the mirror of `imported_invalid_size_state_rejected`).
            // The struct-level `serde(default)` on `HostRegions`
            // restores absent fields from the struct `Default`
            // (baseline weights), so only explicit garbage imports.
            let json = r#"{
                "docks": {
                    "1": { "id": 1, "role": { "EmbedHost": { "content": "host",
                            "regions": { "top": 2, "top_weight": 1.0,
                                        "bottom": null, "bottom_weight": 1.0,
                                        "left": 3, "left_weight": -3.5,
                                        "right": null, "right_weight": 1.0 },
                            "floating": [], "persistent": true } } },
                    "2": { "id": 2, "role": { "Panel": { "content": "p" } } },
                    "3": { "id": 3, "role": { "Panel": { "content": "q" } } }
                },
                "root": 1,
                "next_id": 4
            }"#;
            let layout: DockLayout = serde_json::from_str(json).unwrap();
            assert_eq!(
                layout.validate(),
                Err(DockingError::InvalidRegionState {
                    host: DockId::from_raw(1),
                    edge: EdgeRegion::Left
                })
            );
        }

        #[test]
        fn imported_garbage_floating_rect_rejected() {
            // PRD §9.6: an entry's stored rect must satisfy the rect
            // invariant and the host-bounds rule (here: `x + w > 1` —
            // outside the host's logical bounds). Entries check in
            // collection order, so the first violating index is named.
            let json = r#"{
                "docks": {
                    "1": { "id": 1, "role": { "EmbedHost": { "content": "host",
                            "regions": { "top": null, "top_weight": 1.0,
                                        "bottom": null, "bottom_weight": 1.0,
                                        "left": null, "left_weight": 1.0,
                                        "right": null, "right_weight": 1.0 },
                            "floating": [
                                { "dock": 2,
                                  "rect": { "x": 0.6, "y": 0.1, "w": 0.5, "h": 0.3 } },
                                { "dock": 3,
                                  "rect": { "x": -0.2, "y": 0.1, "w": 0.2, "h": 0.2 } }
                            ], "persistent": true } } },
                    "2": { "id": 2, "role": { "Panel": { "content": "p" } } },
                    "3": { "id": 3, "role": { "Panel": { "content": "q" } } }
                },
                "root": 1,
                "next_id": 4
            }"#;
            let layout: DockLayout = serde_json::from_str(json).unwrap();
            assert_eq!(
                layout.validate(),
                Err(DockingError::InvalidFloatingState {
                    host: DockId::from_raw(1),
                    index: 0
                })
            );
        }
    }
}
