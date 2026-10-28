//! Embedding and host lifecycle tests for the Docking environment
//! (current_state/plan.md Phase 07, PRD §8.8–§8.9, §9.5, §8.11).
//!
//! Coverage: the `Embed` lifecycle (edge regions and floating appends,
//! the occupied-region rejection, the cycle rule), the host lifecycle
//! (explicit *persistent* promotion vs. the temporary auto-promotion
//! and its automatic demotion through normalization), `ResizeRegion`
//! (exactly one stored weight; carried minimum bounds, rejection
//! policy), and `Remove` of embedded structures (a host removal takes
//! its whole arrangement — the BFS crossings cover host regions and
//! floating entries, PRD §7.5).
//!
//! All fixtures go through the public operation API. Undo *replay*
//! (the `pub(crate)` reverse seam) and imported-garbage region state
//! (the `InvalidRegionState` gate) are covered in-crate —
//! `operations.rs` / `validation.rs` respectively (agents/testing.md).

use app_shell::domain_foundation::environments::docking::{
    DockId, DockKind, DockLayout, DockSlot, DockingError, DockingOperation, DockingOutcome,
    EmbedSlot, EdgeRegion, FloatingRect, FormerCollection, InsertSource, SplitDirection,
    SplitOrientation, SplitSizes,
};

/// A fresh persistent embed host as the layout root, created through
/// the public operation (auto-rooted from the empty layout).
fn host_root() -> (DockLayout, DockId) {
    let mut layout = DockLayout::new();
    let outcome = DockingOperation::CreateEmbedHost { content: Some("host".into()) }
        .apply(&mut layout)
        .unwrap();
    let DockingOutcome::Created { dock: host, is_root: true, .. } = &outcome else {
        panic!("expected a Created (auto-rooted host)");
    };
    let host = *host;
    assert!(layout.is_valid());
    (layout, host)
}

/// Two-panel column root: `a` (child 0, id 0) and `b` (child 1, id 1),
/// the split id 2.
fn column_root() -> (DockLayout, DockId, DockId, DockId) {
    let mut layout = DockLayout::new();
    let a = layout.create_panel(Some("a".into()));
    let b = layout.create_panel(Some("b".into()));
    layout.set_root(None).unwrap();
    let column = layout
        .create_split(
            SplitOrientation::Column,
            vec![a, b],
            SplitSizes::uniform(2).unwrap(),
        )
        .unwrap();
    layout.set_root(Some(column)).unwrap();
    assert!(layout.is_valid());
    (layout, a, b, column)
}

// --- Embed: edge regions and the floating collection (PRD §8.8) ---

#[test]
fn embed_places_sources_into_regions_and_floating_with_baselines() {
    let (mut layout, host) = host_root();
    let first = DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("p1".into()) },
        host,
        slot: EmbedSlot::Region(EdgeRegion::Top),
    }
    .apply(&mut layout)
    .unwrap();
    let DockingOutcome::Embedded { source: p1, host: h, slot, promoted, .. } = &first else {
        panic!("expected Embedded");
    };
    assert_eq!(*h, host);
    assert_eq!(*slot, EmbedSlot::Region(EdgeRegion::Top));
    assert!(!*promoted); // the target was already a host
    let p1 = *p1;

    let second = DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("p2".into()) },
        host,
        slot: EmbedSlot::Floating { rect: None },
    }
    .apply(&mut layout)
    .unwrap();
    let DockingOutcome::Embedded { source: p2, slot: float_slot, .. } = &second else {
        panic!("expected Embedded");
    };
    assert_eq!(*float_slot, EmbedSlot::Floating { rect: None });
    let p2 = *p2;
    assert!(layout.is_valid());

    // Placements land exactly where requested; a freshly occupied
    // region starts at the §9.5 embedding weight 1.0, a rect-less
    // floating entry at the §9.6 floating baseline.
    assert_eq!(
        layout.placement(p1),
        Some(DockSlot::HostEdge { host, edge: EdgeRegion::Top })
    );
    assert_eq!(layout.placement(p2), Some(DockSlot::Floating { host, index: 0 }));
    let regions = layout.get(host).unwrap().host_regions().unwrap();
    assert_eq!(regions.on(EdgeRegion::Top), Some(p1));
    assert_eq!(regions.weight_on(EdgeRegion::Top), Some(1.0));
    let floating = layout.get(host).unwrap().floating().unwrap();
    assert_eq!(floating.len(), 1);
    assert_eq!(floating[0].dock, p2);
    assert_eq!(floating[0].rect(), FloatingRect::default()); // the baseline
}

#[test]
fn floating_appends_last_and_rejects_directional_insertion() {
    let (mut layout, host) = host_root();
    let first = DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("f1".into()) },
        host,
        slot: EmbedSlot::Floating { rect: None },
    }
    .apply(&mut layout)
    .unwrap();
    let second = DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("f2".into()) },
        host,
        slot: EmbedSlot::Floating { rect: Some(FloatingRect::new(0.2, 0.15, 0.5, 0.6).unwrap()) },
    }
    .apply(&mut layout)
    .unwrap();
    let f1 = match first {
        DockingOutcome::Embedded { source, .. } => source,
        _ => unreachable!("matched elsewhere"),
    };
    let f2 = match second {
        DockingOutcome::Embedded { source, .. } => source,
        _ => unreachable!("matched elsewhere"),
    };
    // Appended last in insertion order (the collection order is the
    // placement order; visual stacking is frontend-derived, PRD §11.3).
    let floating = layout.get(host).unwrap().floating().unwrap();
    assert_eq!(floating.len(), 2);
    assert_eq!(floating[0].dock, f1);
    assert_eq!(floating[1].dock, f2);
    assert_eq!(floating[1].rect(), FloatingRect::new(0.2, 0.15, 0.5, 0.6).unwrap());
    assert_eq!(layout.placement(f2), Some(DockSlot::Floating { host, index: 1 }));

    // Floating docks have no directional layout surface in v1.
    let before = layout.clone();
    let err = DockingOperation::InsertBeside {
        source: InsertSource::NewPanel { content: None },
        target: f1,
        direction: SplitDirection::Left,
    }
    .apply(&mut layout)
    .unwrap_err();
    assert_eq!(err, DockingError::NotSplittable { id: f1 });
    assert_eq!(layout, before);
    assert!(layout.is_valid());
}

#[test]
fn occupied_region_is_rejected_and_layout_untouched() {
    let (mut layout, host) = host_root();
    let _ = DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("p1".into()) },
        host,
        slot: EmbedSlot::Region(EdgeRegion::Top),
    }
    .apply(&mut layout)
    .unwrap();
    let between = layout.clone();

    // PRD §8.8 baseline: an occupied region rejects the embed — the
    // existing occupant is never silently discarded (no v1 replacement
    // policy). Meanwhile other edges accept their own embeds.
    let err = DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("intruder".into()) },
        host,
        slot: EmbedSlot::Region(EdgeRegion::Top),
    }
    .apply(&mut layout)
    .unwrap_err();
    assert_eq!(err, DockingError::RegionOccupied { host, edge: EdgeRegion::Top });
    assert_eq!(layout, between);

    let _ = DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("p2".into()) },
        host,
        slot: EmbedSlot::Region(EdgeRegion::Bottom),
    }
    .apply(&mut layout)
    .unwrap();
    assert!(layout.is_valid());
}

#[test]
fn embed_into_a_split_or_tab_group_is_ineligible_and_a_cycle_rejects() {
    // Splits and tab groups hold no work area: they can neither host
    // nor auto-promote (PRD §8.9).
    let (mut layout, a, _b, column) = column_root();
    let before = layout.clone();
    let err = DockingOperation::Embed {
        source: InsertSource::Existing(a),
        host: column,
        slot: EmbedSlot::Floating { rect: None },
    }
    .apply(&mut layout)
    .unwrap_err();
    assert_eq!(err, DockingError::IneligibleDock { id: column });

    // The host inside the source's subtree is the cycle: embedding the
    // wrapping split into one of its own children would make the
    // source its own descendant (PRD §10.4).
    let err = DockingOperation::Embed {
        source: InsertSource::Existing(column),
        host: a,
        slot: EmbedSlot::Region(EdgeRegion::Left),
    }
    .apply(&mut layout)
    .unwrap_err();
    assert_eq!(err, DockingError::CycleDetected { id: a });
    assert_eq!(layout, before);
    assert!(layout.is_valid());
}

#[test]
fn embed_re_docks_a_source_within_its_own_host() {
    // A source *inside* the host's subtree re-embeds into another slot
    // of the same host (allowed — the cycle is only the host inside
    // the source). A region occupant undocks to floating.
    let (mut layout, host) = host_root();
    let embed = DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("p".into()) },
        host,
        slot: EmbedSlot::Region(EdgeRegion::Right),
    }
    .apply(&mut layout)
    .unwrap();
    let source = match embed {
        DockingOutcome::Embedded { source, .. } => source,
        _ => unreachable!("matched Embedded above"),
    };

    let _ = DockingOperation::Embed {
        source: InsertSource::Existing(source),
        host,
        slot: EmbedSlot::Floating { rect: None },
    }
    .apply(&mut layout)
    .unwrap();
    assert!(layout.is_valid());
    // The source moved slots inside the same host; the vacated region
    // cleared (the occupant and its slot weight reset together — the
    // §9.5 lifecycle).
    let regions = layout.get(host).unwrap().host_regions().unwrap();
    assert_eq!(regions.on(EdgeRegion::Right), None);
    assert_eq!(regions.on(EdgeRegion::Top), None);
    let floating = layout.get(host).unwrap().floating().unwrap();
    assert_eq!(floating.len(), 1);
    assert_eq!(floating[0].dock, source);
    assert_eq!(layout.placement(source), Some(DockSlot::Floating { host, index: 0 }));
}

// --- Host promotion and demotion (PRD §8.9) ---

#[test]
fn embed_auto_promotion_is_temporary_and_demotes_through_removal() {
    // A panel promoted solely to support one embed keeps identity,
    // content, and placement, and demotes to an expanded panel when
    // its last embedded dock leaves (the PRD §8.9 baseline matrix).
    let mut layout = DockLayout::new();
    let p = layout.create_panel(Some("work area".into())); // root
    assert_eq!(layout.get(p).unwrap().kind(), DockKind::Panel);

    let embed = DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("tool".into()) },
        host: p,
        slot: EmbedSlot::Region(EdgeRegion::Top),
    }
    .apply(&mut layout)
    .unwrap();
    let DockingOutcome::Embedded { source, promoted: true, .. } = &embed else {
        panic!("expected an auto-promoted Embedded");
    };
    assert_eq!(layout.get(p).unwrap().kind(), DockKind::EmbedHost);
    assert_eq!(layout.get(p).unwrap().persistent(), Some(false));
    assert_eq!(layout.get(p).unwrap().content(), Some(&"work area".to_string()));
    assert_eq!(layout.root(), Some(p));
    assert!(layout.is_valid());

    let _ = DockingOperation::Remove { dock: *source }.apply(&mut layout).unwrap();
    assert!(layout.is_valid());
    // The same dock is a panel again — expanded, same content, same
    // root placement.
    assert_eq!(layout.root(), Some(p));
    assert_eq!(layout.get(p).unwrap().kind(), DockKind::Panel);
    assert_eq!(layout.get(p).unwrap().content(), Some(&"work area".to_string()));
    assert_eq!(layout.get(p).unwrap().collapsed(), Some(false));
    assert_eq!(layout.get(p).unwrap().persistent(), None);

    // Fixpoint: normalization finds nothing else to do.
    let settled = layout.clone();
    layout.normalize().unwrap();
    assert_eq!(layout, settled);
}

#[test]
fn explicit_promotion_is_persistent_and_survives_emptiness() {
    let mut layout = DockLayout::new();
    let p = layout.create_panel(Some("work area".into())); // root

    let promote = DockingOperation::PromoteHost { dock: p }.apply(&mut layout).unwrap();
    let DockingOutcome::HostPromoted { dock, promoted: true, .. } = &promote else {
        panic!("expected HostPromoted");
    };
    assert_eq!(*dock, p);
    assert_eq!(layout.get(p).unwrap().persistent(), Some(true));
    // The panel collapse preference does not carry over (v1: hosts are
    // not a collapsible category; the dock lands expanded).
    assert_eq!(layout.get(p).unwrap().collapsed(), None);
    assert_eq!(layout.get(p).unwrap().content(), Some(&"work area".to_string()));
    assert!(layout.is_valid());

    // Embed, then empty the host again: it stays a host (an empty
    // persistent host is a valid steady state, PRD §8.8).
    let embed = DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("dock".into()) },
        host: p,
        slot: EmbedSlot::Floating { rect: None },
    }
    .apply(&mut layout)
    .unwrap();
    let source = match embed {
        DockingOutcome::Embedded { source, .. } => source,
        _ => unreachable!("matched Embedded above"),
    };
    let _ = DockingOperation::Remove { dock: source }.apply(&mut layout).unwrap();

    assert!(layout.is_valid());
    assert_eq!(layout.get(p).unwrap().kind(), DockKind::EmbedHost);
    assert_eq!(layout.get(p).unwrap().persistent(), Some(true));
    let settled = layout.clone();
    layout.normalize().unwrap();
    assert_eq!(layout, settled); // no demotion, no other change
}

#[test]
fn promoting_a_nested_panel_keeps_its_slot_verbatim() {
    // Promotion and demotion are in-place role changes: the host keeps
    // its exact slot and the parent split keeps children, order, and
    // weights verbatim (PRD §8.9).
    let (mut layout, x, y, column) = column_root();
    let embed = DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("tag".into()) },
        host: x,
        slot: EmbedSlot::Region(EdgeRegion::Left),
    }
    .apply(&mut layout)
    .unwrap();
    let source = match embed {
        DockingOutcome::Embedded { source, .. } => source,
        _ => unreachable!("matched Embedded above"),
    };
    assert_eq!(
        layout.placement(x),
        Some(DockSlot::SplitChild { parent: column, index: 0 })
    );
    assert_eq!(
        layout.get(column).unwrap().sizes().map(|s| s.weights().to_vec()),
        Some(vec![1.0, 1.0])
    );
    assert!(layout.is_valid());

    let _ = DockingOperation::Remove { dock: source }.apply(&mut layout).unwrap();
    assert!(layout.is_valid());
    assert_eq!(layout.get(x).unwrap().kind(), DockKind::Panel);
    assert_eq!(
        layout.placement(x),
        Some(DockSlot::SplitChild { parent: column, index: 0 })
    );
    assert_eq!(
        layout.get(column).unwrap().sizes().map(|s| s.weights().to_vec()),
        Some(vec![1.0, 1.0])
    );
    assert_eq!(layout.get(y).unwrap().kind(), DockKind::Panel);
}

// --- Region resizing (PRD §9.5) ---

#[test]
fn resize_region_rewrites_one_weight_and_reports_the_previous() {
    let (mut layout, host) = host_root();
    let first = DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("p1".into()) },
        host,
        slot: EmbedSlot::Region(EdgeRegion::Top),
    }
    .apply(&mut layout)
    .unwrap();
    let second = DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("p2".into()) },
        host,
        slot: EmbedSlot::Region(EdgeRegion::Bottom),
    }
    .apply(&mut layout)
    .unwrap();
    let p1 = match first {
        DockingOutcome::Embedded { source, .. } => source,
        _ => unreachable!("matched Embedded above"),
    };
    let p2 = match second {
        DockingOutcome::Embedded { source, .. } => source,
        _ => unreachable!("matched Embedded above"),
    };

    let resize = DockingOperation::ResizeRegion {
        host,
        edge: EdgeRegion::Top,
        requested: 3.25,
        min: None,
    }
    .apply(&mut layout)
    .unwrap();
    let DockingOutcome::RegionResized { host: h, edge, previous, inverse } = &resize else {
        panic!("expected RegionResized");
    };
    assert_eq!(*h, host);
    assert_eq!(*edge, EdgeRegion::Top);
    assert_eq!(*previous, 1.0); // the embedding baseline it replaced
    // Occupants untouched: exactly one stored weight changed and the
    // sibling region's preference is verbatim (the §9.5 coherence
    // rule — resizing never invalidates the host's other placements).
    let regions = layout.get(host).unwrap().host_regions().unwrap();
    assert_eq!(regions.on(EdgeRegion::Top), Some(p1));
    assert_eq!(regions.weight_on(EdgeRegion::Top), Some(3.25));
    assert_eq!(regions.on(EdgeRegion::Bottom), Some(p2));
    assert_eq!(regions.weight_on(EdgeRegion::Bottom), Some(1.0));
    assert!(layout.is_valid());

    // The inverse record restores occupant and weight together (the
    // Phase-9 seam replays it in-crate).
    assert_eq!(
        inverse.containers.first(),
        Some(&FormerCollection::Region {
            host,
            edge: EdgeRegion::Top,
            previous: Some(p1),
            weight: Some(1.0),
        })
    );
}

#[test]
fn resize_region_requires_occupancy_and_enforces_bounds_exclusively() {
    let (mut layout, host) = host_root();
    let embed = DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("p".into()) },
        host,
        slot: EmbedSlot::Region(EdgeRegion::Right),
    }
    .apply(&mut layout)
    .unwrap();
    let source = match embed {
        DockingOutcome::Embedded { source, .. } => source,
        _ => unreachable!("matched Embedded above"),
    };
    let valid = layout.clone();

    // Unoccupied regions have no boundary — and no preference — to
    // resize (weights live with the region lifecycle).
    let err = DockingOperation::ResizeRegion {
        host,
        edge: EdgeRegion::Top,
        requested: 2.0,
        min: None,
    }
    .apply(&mut layout)
    .unwrap_err();
    assert_eq!(err, DockingError::RegionUnoccupied { host, edge: EdgeRegion::Top });
    assert_eq!(layout, valid);

    // The sizing invariant (finite, strictly positive) is the implicit
    // bound; a carried minimum binds inclusively. Rejection, never
    // clamping (PRD §9.3 policy).
    for (requested, min) in [(0.0, None), (f64::NAN, None), (f64::INFINITY, None), (0.5, Some(1.0))] {
        let err = DockingOperation::ResizeRegion {
            host,
            edge: EdgeRegion::Right,
            requested,
            min,
        }
        .apply(&mut layout)
        .unwrap_err();
        assert_eq!(err, DockingError::RegionBoundViolation { host, edge: EdgeRegion::Right });
        assert_eq!(layout, valid);
    }

    // The bound endpoint itself honors, and the occupant stays.
    let _ = DockingOperation::ResizeRegion {
        host,
        edge: EdgeRegion::Right,
        requested: 1.0,
        min: Some(1.0),
    }
    .apply(&mut layout)
    .unwrap();
    assert!(layout.is_valid());
    let regions = layout.get(host).unwrap().host_regions().unwrap();
    assert_eq!(regions.on(EdgeRegion::Right), Some(source));
    assert_eq!(regions.weight_on(EdgeRegion::Right), Some(1.0));

    // The host argument names an embed host.
    let (mut split_layout, a, _b, column) = column_root();
    let err = DockingOperation::ResizeRegion {
        host: column,
        edge: EdgeRegion::Top,
        requested: 2.0,
        min: None,
    }
    .apply(&mut split_layout)
    .unwrap_err();
    assert_eq!(err, DockingError::NotAnEmbedHost { id: column });
    assert!(split_layout.contains(a));
}

// --- Removal of embedded structures (PRD §8.11) ---

#[test]
fn removing_a_host_takes_its_whole_arrangement() {
    // `Remove` of a host removes the dock, every occupied-region
    // subtree, and every floating entry (the removed subtree's BFS
    // crossings include host edges — PRD §7.5 edge set, §8.11
    // semantics).
    let (mut layout, host) = host_root();
    let region = DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("p1".into()) },
        host,
        slot: EmbedSlot::Region(EdgeRegion::Top),
    }
    .apply(&mut layout)
    .unwrap();
    let p1 = match region {
        DockingOutcome::Embedded { source, .. } => source,
        _ => unreachable!("matched Embedded above"),
    };
    let floating = DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("p2".into()) },
        host,
        slot: EmbedSlot::Floating { rect: None },
    }
    .apply(&mut layout)
    .unwrap();
    let p2 = match floating {
        DockingOutcome::Embedded { source, .. } => source,
        _ => unreachable!("matched Embedded above"),
    };
    // A tab group around the region occupant: the group inherits the
    // region slot and embeds with it.
    let join = DockingOperation::TabJoin {
        source: InsertSource::NewPanel { content: Some("p3".into()) },
        target: p1,
    }
    .apply(&mut layout)
    .unwrap();
    let DockingOutcome::TabJoined { source: p3, group, .. } = &join else {
        panic!("expected TabJoined");
    };
    let p3 = *p3;
    let group = *group;
    assert!(layout.is_valid());

    let outcome = DockingOperation::Remove { dock: host }.apply(&mut layout).unwrap();
    let DockingOutcome::Removed { docks, inverse } = &outcome else {
        panic!("expected Removed");
    };
    // BFS order: the host; then the region occupant (the tab group —
    // edge scan order) and the floating entry; then the group's
    // members.
    assert_eq!(
        docks.iter().map(|dock| dock.id()).collect::<Vec<_>>(),
        vec![host, group, p2, p1, p3]
    );
    assert!(layout.is_valid());
    assert_eq!(layout.len(), 0);
    assert_eq!(layout.root(), None);
    // The payload records carry the whole arrangement (the Phase-9
    // seam replays them in-crate).
    assert_eq!(inverse.removed.len(), 5);
}

#[test]
fn removing_an_occupant_keeps_the_host_and_clears_one_slot() {
    // `Remove` of an occupant detaches it from its slot alone: the
    // host's other placements and region preferences are untouched
    // (PRD §8.11, §9.5), and a created persistent host stays a host.
    let (mut layout, host) = host_root();
    let region = DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("p1".into()) },
        host,
        slot: EmbedSlot::Region(EdgeRegion::Top),
    }
    .apply(&mut layout)
    .unwrap();
    let bottom = DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("p2".into()) },
        host,
        slot: EmbedSlot::Region(EdgeRegion::Bottom),
    }
    .apply(&mut layout)
    .unwrap();
    let float = DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("f1".into()) },
        host,
        slot: EmbedSlot::Floating { rect: None },
    }
    .apply(&mut layout)
    .unwrap();
    let p1 = match region {
        DockingOutcome::Embedded { source, .. } => source,
        _ => unreachable!("matched Embedded above"),
    };
    let p2 = match bottom {
        DockingOutcome::Embedded { source, .. } => source,
        _ => unreachable!("matched Embedded above"),
    };
    let f1 = match float {
        DockingOutcome::Embedded { source, .. } => source,
        _ => unreachable!("matched Embedded above"),
    };

    let outcome = DockingOperation::Remove { dock: p2 }.apply(&mut layout).unwrap();
    let DockingOutcome::Removed { inverse, .. } = &outcome else {
        panic!("expected Removed");
    };
    assert!(layout.is_valid());
    assert_eq!(layout.get(host).unwrap().kind(), DockKind::EmbedHost);
    assert_eq!(layout.get(host).unwrap().persistent(), Some(true));
    // The bottom slot cleared (the cleared region resumes the embedding
    // baseline); the other placements verbatim.
    let regions = layout.get(host).unwrap().host_regions().unwrap();
    assert_eq!(regions.on(EdgeRegion::Top), Some(p1));
    assert_eq!(regions.weight_on(EdgeRegion::Top), Some(1.0));
    assert_eq!(regions.on(EdgeRegion::Bottom), None);
    let floating = layout.get(host).unwrap().floating().unwrap();
    assert_eq!(floating.len(), 1);
    assert_eq!(floating[0].dock, f1);

    // The inverse holds exactly one region record + the removed payload.
    assert_eq!(inverse.removed.len(), 1);
    assert_eq!(
        inverse.containers.first(),
        Some(&FormerCollection::Region {
            host,
            edge: EdgeRegion::Bottom,
            previous: Some(p2),
            weight: Some(1.0),
        })
    );
}
