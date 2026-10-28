//! Floating docking structures for the Docking environment
//! (current_state/plan.md Phase 08, PRD §7.5–§7.6, §8.11, §9.6, §11.3).
//!
//! Coverage: the floating layer of a host (`EmbedSlot::Floating` —
//! explicit entries appended last, each carrying its persistent
//! [`FloatingRect`]), structure preservation (a floated subtree never
//! flattens, including the §8.11 "float from tab group" member detach
//! and whole-group floats), re-docking a floater through a valid target
//! (`Embed` region / `InsertBeside` / `TabJoin`) while the floating
//! target direction stays `NotSplittable`, `SetFloatingRect` (exactly
//! one entry, carried minimums, rejection policy, inclusive bound
//! endpoints), and the removal/normalization side (entry clearing with
//! the slot, collection shifts, subtree BFS removal, and the temporary
//! host demotion of a last-float removal).
//!
//! All fixtures go through the public operation API. Undo *replay*
//! (the `pub(crate)` reverse seam) and invalid-rect construction /
//! imported-garbage floating state is covered in-crate —
//! `operations.rs` / `validation.rs` respectively (agents/testing.md).

use app_shell::domain_foundation::environments::docking::{
    DockId, DockKind, DockLayout, DockSlot, DockingError, DockingOperation, DockingOutcome,
    EmbedSlot, EdgeRegion, FloatingRect, FormerCollection, InsertSource, SplitDirection,
    SplitOrientation,
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

/// Float a new panel onto `host`'s floating collection through the
/// public `Embed` operation, returning the floated dock's id.
fn float_new(
    layout: &mut DockLayout,
    host: DockId,
    content: &str,
    rect: Option<FloatingRect>,
) -> DockId {
    let outcome = DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some(content.into()) },
        host,
        slot: EmbedSlot::Floating { rect },
    }
    .apply(layout)
    .unwrap();
    match outcome {
        DockingOutcome::Embedded { source, .. } => source,
        _ => unreachable!("matched Embedded above"),
    }
}

// --- The floating layer (PRD §7.5–§7.6, §9.6) ---

#[test]
fn floated_entries_carry_explicit_rects_and_place_by_index() {
    let (mut layout, host) = host_root();
    let f1 = float_new(
        &mut layout,
        host,
        "f1",
        Some(FloatingRect::new(0.125, 0.25, 0.375, 0.5).unwrap()),
    );
    // A rect-less append starts at the floating baseline (the
    // `EmbedSlot::Floating`'s `None`; PRD §9.6 initialization policy).
    let f2 = float_new(&mut layout, host, "f2", None);
    let f3 = float_new(&mut layout, host, "f3", Some(FloatingRect::new(0.5, 0.5, 0.5, 0.5).unwrap()));
    assert!(layout.is_valid());

    // One entry per floated root, appended last in placement order (the
    // collection order is placement order; visual stacking is
    // frontend-derived, PRD §11.3).
    let floating = layout.get(host).unwrap().floating().unwrap();
    assert_eq!(floating.len(), 3);
    assert_eq!(floating[0].dock, f1);
    assert_eq!(floating[0].rect(), FloatingRect::new(0.125, 0.25, 0.375, 0.5).unwrap());
    assert_eq!(floating[1].dock, f2);
    assert_eq!(floating[1].rect(), FloatingRect::default()); // the baseline
    assert_eq!(floating[2].dock, f3);
    assert_eq!(floating[2].rect(), FloatingRect::new(0.5, 0.5, 0.5, 0.5).unwrap());

    // Every entry places its subtree root at `DockSlot::Floating`
    // through the layout's derived placement scan — explicit and valid.
    assert_eq!(layout.placement(f1), Some(DockSlot::Floating { host, index: 0 }));
    assert_eq!(layout.placement(f2), Some(DockSlot::Floating { host, index: 1 }));
    assert_eq!(layout.placement(f3), Some(DockSlot::Floating { host, index: 2 }));
}

#[test]
fn floating_a_subtree_preserves_its_structure_verbatim() {
    // PRD §7.6 (§9.6 SHOULD): floating never flattens — the source
    // detaches with its subtree intact and the entry names its root.
    let (mut layout, host) = host_root();
    let embed = DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("p1".into()) },
        host,
        slot: EmbedSlot::Region(EdgeRegion::Top),
    }
    .apply(&mut layout)
    .unwrap();
    let p1 = match embed {
        DockingOutcome::Embedded { source, .. } => source,
        _ => unreachable!("matched Embedded above"),
    };
    let insert = DockingOperation::InsertBeside {
        source: InsertSource::NewPanel { content: Some("p2".into()) },
        target: p1,
        direction: SplitDirection::Top,
    }
    .apply(&mut layout)
    .unwrap();
    let DockingOutcome::InsertedBeside { source: p2, container: column, .. } = &insert else {
        panic!("expected InsertedBeside");
    };
    let p2 = *p2;
    let column = *column;
    let children_before = layout.get(column).unwrap().children().unwrap().to_vec();
    let sizes_before = layout.get(column).unwrap().sizes().unwrap().weights().to_vec();

    let float = DockingOperation::Embed {
        source: InsertSource::Existing(column),
        host,
        slot: EmbedSlot::Floating {
            rect: Some(FloatingRect::new(0.25, 0.25, 0.5, 0.5).unwrap()),
        },
    }
    .apply(&mut layout)
    .unwrap();
    let DockingOutcome::Embedded { source: floated, slot, promoted: false, .. } = &float else {
        panic!("expected Embedded");
    };
    assert_eq!(*floated, column);
    assert_eq!(
        *slot,
        EmbedSlot::Floating { rect: Some(FloatingRect::new(0.25, 0.25, 0.5, 0.5).unwrap()) }
    );
    assert!(layout.is_valid());

    // The entry names the split root; the split keeps orientation,
    // children, order, and sizing verbatim beneath its floating slot.
    let floating = layout.get(host).unwrap().floating().unwrap();
    assert_eq!(floating.len(), 1);
    assert_eq!(floating[0].dock, column);
    assert_eq!(
        floating[0].rect(),
        FloatingRect::new(0.25, 0.25, 0.5, 0.5).unwrap()
    );
    assert_eq!(layout.placement(column), Some(DockSlot::Floating { host, index: 0 }));
    assert_eq!(layout.get(column).unwrap().orientation(), Some(SplitOrientation::Column));
    assert_eq!(layout.get(column).unwrap().children().unwrap(), &children_before[..]);
    assert_eq!(layout.get(column).unwrap().sizes().unwrap().weights(), &sizes_before[..]);
    assert_eq!(children_before, vec![p2, p1]);
    assert_eq!(layout.placement(p2), Some(DockSlot::SplitChild { parent: column, index: 0 }));
    assert_eq!(layout.placement(p1), Some(DockSlot::SplitChild { parent: column, index: 1 }));

    // The vacated region cleared with its occupant (the §9.5 slot
    // lifecycle; the host imposes no child minimum, PRD §8.8).
    let regions = layout.get(host).unwrap().host_regions().unwrap();
    assert_eq!(regions.on(EdgeRegion::Top), None);
}

#[test]
fn floating_a_tab_member_keeps_the_group_and_selection_rules() {
    // §8.11: "float from tab group" is the member detach — the group
    // keeps every remaining member and the active selection follows the
    // established member-removal rules (PRD §8.7).
    let (mut layout, host) = host_root();
    let embed = DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("m1".into()) },
        host,
        slot: EmbedSlot::Region(EdgeRegion::Top),
    }
    .apply(&mut layout)
    .unwrap();
    let m1 = match embed {
        DockingOutcome::Embedded { source, .. } => source,
        _ => unreachable!("matched Embedded above"),
    };
    let join = DockingOperation::TabJoin {
        source: InsertSource::NewPanel { content: Some("m2".into()) },
        target: m1,
    }
    .apply(&mut layout)
    .unwrap();
    let DockingOutcome::TabJoined { source: m2, group, .. } = &join else {
        panic!("expected TabJoined");
    };
    let m2 = *m2;
    let group = *group;
    assert_eq!(layout.get(group).unwrap().active(), Some(1)); // the joined member

    // Float the *active* member out.
    let _ = DockingOperation::Embed {
        source: InsertSource::Existing(m2),
        host,
        slot: EmbedSlot::Floating { rect: None },
    }
    .apply(&mut layout)
    .unwrap();
    assert!(layout.is_valid());
    // The group survives (the ≥1-member rule) and the selection stays
    // at the same position, clamped to the last member.
    assert_eq!(layout.get(group).unwrap().active(), Some(0));
    assert_eq!(layout.placement(m1), Some(DockSlot::TabMember { parent: group, index: 0 }));
    assert_eq!(layout.placement(group), Some(DockSlot::HostEdge { host, edge: EdgeRegion::Top }));
    let floating = layout.get(host).unwrap().floating().unwrap();
    assert_eq!(floating.len(), 1);
    assert_eq!(floating[0].dock, m2);
    assert_eq!(floating[0].rect(), FloatingRect::default()); // the baseline
}

#[test]
fn floating_a_whole_tab_group_preserves_members_and_redocks_through_tab_join() {
    let (mut layout, host) = host_root();
    let embed = DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("m1".into()) },
        host,
        slot: EmbedSlot::Region(EdgeRegion::Top),
    }
    .apply(&mut layout)
    .unwrap();
    let m1 = match embed {
        DockingOutcome::Embedded { source, .. } => source,
        _ => unreachable!("matched Embedded above"),
    };
    let join = DockingOperation::TabJoin {
        source: InsertSource::NewPanel { content: Some("m2".into()) },
        target: m1,
    }
    .apply(&mut layout)
    .unwrap();
    let DockingOutcome::TabJoined { source: m2, group, .. } = &join else {
        panic!("expected TabJoined");
    };
    let m2 = *m2;
    let group = *group;

    // Float the group itself: the subtree detaches intact (members and
    // selection verbatim) and the entry names the group as its root.
    let rect = FloatingRect::new(0.25, 0.25, 0.5, 0.5).unwrap();
    let _ = DockingOperation::Embed {
        source: InsertSource::Existing(group),
        host,
        slot: EmbedSlot::Floating { rect: Some(rect) },
    }
    .apply(&mut layout)
    .unwrap();
    assert!(layout.is_valid());
    assert_eq!(layout.placement(group), Some(DockSlot::Floating { host, index: 0 }));
    assert_eq!(layout.placement(m1), Some(DockSlot::TabMember { parent: group, index: 0 }));
    assert_eq!(layout.placement(m2), Some(DockSlot::TabMember { parent: group, index: 1 }));
    assert_eq!(layout.get(group).unwrap().active(), Some(1));
    assert_eq!(layout.get(host).unwrap().host_regions().unwrap().on(EdgeRegion::Top), None);
    let floating = layout.get(host).unwrap().floating().unwrap();
    assert_eq!(floating.len(), 1);
    assert_eq!(floating[0].dock, group);
    assert_eq!(floating[0].rect(), rect);

    // Re-dock the member through `TabJoin`: it detaches from the
    // floating collection (the group's entry shifts up one index — its
    // rect follows the entry, not the index) and rejoins the group.
    let _ = DockingOperation::TabJoin {
        source: InsertSource::Existing(m2),
        target: m1,
    }
    .apply(&mut layout)
    .unwrap();
    assert!(layout.is_valid());
    assert_eq!(layout.get(group).unwrap().active(), Some(1)); // the rejoined member
    assert_eq!(layout.placement(m2), Some(DockSlot::TabMember { parent: group, index: 1 }));
    let floating = layout.get(host).unwrap().floating().unwrap();
    assert_eq!(floating.len(), 1);
    assert_eq!(floating[0].dock, group, "the group's entry shifted to index 0");
    assert_eq!(floating[0].rect(), rect, "the entry's rect follows the entry");
    assert_eq!(layout.placement(group), Some(DockSlot::Floating { host, index: 0 }));
}

// --- Re-docking a floater onto a valid target (PRD §8.11, §9.6) ---

#[test]
fn redocking_a_floater_through_embed_region_shifts_the_collection() {
    let (mut layout, host) = host_root();
    let r1 = FloatingRect::new(0.125, 0.125, 0.25, 0.25).unwrap();
    let r2 = FloatingRect::new(0.5, 0.5, 0.5, 0.5).unwrap();
    let f1 = float_new(&mut layout, host, "f1", Some(r1));
    let f2 = float_new(&mut layout, host, "f2", Some(r2));

    // Re-dock the first floater into the host's top region.
    let outcome = DockingOperation::Embed {
        source: InsertSource::Existing(f1),
        host,
        slot: EmbedSlot::Region(EdgeRegion::Top),
    }
    .apply(&mut layout)
    .unwrap();
    let DockingOutcome::Embedded { source, slot, promoted: false, .. } = &outcome else {
        panic!("expected Embedded");
    };
    assert_eq!(*source, f1);
    assert_eq!(*slot, EmbedSlot::Region(EdgeRegion::Top));
    assert!(layout.is_valid());

    // The region attachment rebuilt at the embedding baseline weight
    // (the §9.5 initialization policy).
    let regions = layout.get(host).unwrap().host_regions().unwrap();
    assert_eq!(regions.on(EdgeRegion::Top), Some(f1));
    assert_eq!(regions.weight_on(EdgeRegion::Top), Some(1.0));
    assert_eq!(layout.placement(f1), Some(DockSlot::HostEdge { host, edge: EdgeRegion::Top }));

    // The vacated entry is gone; the surviving entry shifted up one
    // index *with* its persistent rect (rects live with the entry, not
    // with an index number).
    let floating = layout.get(host).unwrap().floating().unwrap();
    assert_eq!(floating.len(), 1);
    assert_eq!(floating[0].dock, f2);
    assert_eq!(floating[0].rect(), r2);
    assert_eq!(layout.placement(f2), Some(DockSlot::Floating { host, index: 0 }));
}

#[test]
fn redocking_a_floater_through_insert_beside_retires_its_entry() {
    let (mut layout, host) = host_root();
    let embed = DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("p1".into()) },
        host,
        slot: EmbedSlot::Region(EdgeRegion::Top),
    }
    .apply(&mut layout)
    .unwrap();
    let p1 = match embed {
        DockingOutcome::Embedded { source, .. } => source,
        _ => unreachable!("matched Embedded above"),
    };
    let rect = FloatingRect::new(0.25, 0.25, 0.5, 0.5).unwrap();
    let f = float_new(&mut layout, host, "f", Some(rect));

    // `InsertBeside` wraps the region occupant: the fresh row inherits
    // the region's whole slot and the floater becomes a split child.
    let outcome = DockingOperation::InsertBeside {
        source: InsertSource::Existing(f),
        target: p1,
        direction: SplitDirection::Right,
    }
    .apply(&mut layout)
    .unwrap();
    let DockingOutcome::InsertedBeside { source: placed, container: row, .. } = &outcome else {
        panic!("expected InsertedBeside");
    };
    assert_eq!(*placed, f);
    assert!(layout.is_valid());

    assert_eq!(layout.placement(*row), Some(DockSlot::HostEdge { host, edge: EdgeRegion::Top }));
    let regions = layout.get(host).unwrap().host_regions().unwrap();
    assert_eq!(regions.on(EdgeRegion::Top), Some(*row));
    assert_eq!(regions.weight_on(EdgeRegion::Top), Some(1.0)); // the slot's weight stayed
    assert_eq!(
        layout.get(*row).unwrap().children().unwrap(),
        &[p1, f][..]
    );
    assert_eq!(layout.placement(p1), Some(DockSlot::SplitChild { parent: *row, index: 0 }));
    assert_eq!(layout.placement(f), Some(DockSlot::SplitChild { parent: *row, index: 1 }));

    // The entry retired with the cleared slot: the rect carries into no
    // split state (split sizing is `SplitSizes`-based) and the next
    // float starts from the baseline or an explicit rect.
    let floating = layout.get(host).unwrap().floating().unwrap();
    assert!(
        floating.is_empty(),
        "the floater's entry went with the cleared slot"
    );
}

#[test]
fn floating_docks_stay_rejected_as_directional_targets() {
    // The `NotSplittable` baseline stands in Phase 8: floating docks
    // have no directional layout surface (PRD §11.3 stacking is
    // frontend-derived and v1 defines no floating-rearrangement op).
    let (mut layout, host) = host_root();
    let embed = DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("m1".into()) },
        host,
        slot: EmbedSlot::Region(EdgeRegion::Top),
    }
    .apply(&mut layout)
    .unwrap();
    let m1 = match embed {
        DockingOutcome::Embedded { source, .. } => source,
        _ => unreachable!("matched Embedded above"),
    };
    let join = DockingOperation::TabJoin {
        source: InsertSource::NewPanel { content: Some("m2".into()) },
        target: m1,
    }
    .apply(&mut layout)
    .unwrap();
    let DockingOutcome::TabJoined { group, .. } = &join else {
        panic!("expected TabJoined");
    };
    let group = *group;
    let _ = DockingOperation::Embed {
        source: InsertSource::Existing(group),
        host,
        slot: EmbedSlot::Floating { rect: None },
    }
    .apply(&mut layout)
    .unwrap();
    let before = layout.clone();

    // Toward the floated root itself…
    let err = DockingOperation::InsertBeside {
        source: InsertSource::NewPanel { content: None },
        target: group,
        direction: SplitDirection::Right,
    }
    .apply(&mut layout)
    .unwrap_err();
    assert_eq!(err, DockingError::NotSplittable { id: group });
    assert_eq!(layout, before);

    // …and toward a dock inside the floated group: the resolution walks
    // up the tab-member chain to the floated group and rejects there.
    let err = DockingOperation::InsertBeside {
        source: InsertSource::NewPanel { content: None },
        target: m1,
        direction: SplitDirection::Left,
    }
    .apply(&mut layout)
    .unwrap_err();
    assert_eq!(err, DockingError::NotSplittable { id: group });
    assert_eq!(layout, before);
    assert!(layout.is_valid());
}

#[test]
fn tab_join_wrapping_a_floater_inherits_the_entry_rect() {
    // The wrap in a floating slot inherits the entry: the group replaces
    // the floated panel as the entry's root and the slot's persistent
    // rect carries forward verbatim (the §9.5 slot-attachment decision,
    // extended to floating in §9.6).
    let (mut layout, host) = host_root();
    let rect = FloatingRect::new(0.25, 0.125, 0.5, 0.625).unwrap();
    let f1 = float_new(&mut layout, host, "f1", Some(rect));

    let outcome = DockingOperation::TabJoin {
        source: InsertSource::NewPanel { content: Some("p2".into()) },
        target: f1,
    }
    .apply(&mut layout)
    .unwrap();
    let DockingOutcome::TabJoined { source: p2, group, .. } = &outcome else {
        panic!("expected TabJoined");
    };
    let p2 = *p2;
    let group = *group;
    assert!(layout.is_valid());

    let floating = layout.get(host).unwrap().floating().unwrap();
    assert_eq!(floating.len(), 1);
    assert_eq!(floating[0].dock, group);
    assert_eq!(floating[0].rect(), rect, "the entry's rect was inherited by the wrap");
    assert_eq!(layout.placement(group), Some(DockSlot::Floating { host, index: 0 }));
    assert_eq!(layout.placement(f1), Some(DockSlot::TabMember { parent: group, index: 0 }));
    assert_eq!(layout.placement(p2), Some(DockSlot::TabMember { parent: group, index: 1 }));
    assert_eq!(layout.get(group).unwrap().active(), Some(1)); // the joined member
}

// --- SetFloatingRect (PRD §9.6) ---

#[test]
fn set_floating_rect_rewrites_one_entry_and_reports_the_previous() {
    let (mut layout, host) = host_root();
    let embed = DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("p".into()) },
        host,
        slot: EmbedSlot::Region(EdgeRegion::Top),
    }
    .apply(&mut layout)
    .unwrap();
    let p = match embed {
        DockingOutcome::Embedded { source, .. } => source,
        _ => unreachable!("matched Embedded above"),
    };
    let f1 = float_new(&mut layout, host, "f1", None);
    let f2 = float_new(&mut layout, host, "f2", None);
    let baseline = FloatingRect::default();

    let requested = FloatingRect::new(0.25, 0.125, 0.625, 0.875).unwrap();
   let outcome = DockingOperation::SetFloatingRect {
        host,
        index: 1,
        rect: requested,
        min_w: None,
        min_h: None,
    }
    .apply(&mut layout)
    .unwrap();
    let DockingOutcome::FloatingRectSet { host: h, index, previous, inverse } = &outcome else {
        panic!("expected FloatingRectSet");
    };
    assert_eq!(*h, host);
    assert_eq!(*index, 1);
    assert_eq!(*previous, baseline);
    assert!(layout.is_valid());

    // Exactly the named entry changed: the sibling entry, the region
    // occupant, and every placement stay verbatim (the non-structural
    // `ResizeRegion` write shape).
    let floating = layout.get(host).unwrap().floating().unwrap();
    assert_eq!(floating[0].dock, f1);
    assert_eq!(floating[0].rect(), baseline);
    assert_eq!(floating[1].dock, f2);
    assert_eq!(floating[1].rect(), requested);
    assert_eq!(layout.placement(f1), Some(DockSlot::Floating { host, index: 0 }));
    assert_eq!(layout.placement(f2), Some(DockSlot::Floating { host, index: 1 }));
    let regions = layout.get(host).unwrap().host_regions().unwrap();
    assert_eq!(regions.on(EdgeRegion::Top), Some(p));

    // The inverse owns the whole former collection — the entries (dock
    // *and* rect) restore together on undo (the Phase-9 seam replays
    // them in-crate).
    match inverse.containers.first() {
        Some(FormerCollection::Floating { host: h, entries }) => {
            assert_eq!(*h, host);
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].dock, f1);
            assert_eq!(entries[0].rect(), baseline);
            assert_eq!(entries[1].dock, f2);
            assert_eq!(entries[1].rect(), baseline);
        }
        other => panic!("expected the floating collection record, got {other:?}"),
    }

    // A repeated set is a plain rewrite: the previous value now reports
    // what the last set installed.
    let outcome = DockingOperation::SetFloatingRect {
        host,
        index: 1,
        rect: requested,
        min_w: None,
        min_h: None,
    }
    .apply(&mut layout)
    .unwrap();
    let DockingOutcome::FloatingRectSet { previous, .. } = &outcome else {
        panic!("expected FloatingRectSet");
    };
    assert_eq!(*previous, requested);
}

#[test]
fn set_floating_rect_enforces_carried_minimums_exclusively() {
    let (mut layout, host) = host_root();
    let rect = FloatingRect::new(0.25, 0.25, 0.5, 0.5).unwrap();
    let _ = float_new(&mut layout, host, "f1", Some(rect));
    let valid = layout.clone();

    // PRD §9.3 policy: rejection — a conflicting request is refused,
    // never clamped; a NaN or infinite minimum is unsatisfiable like
    // any other bound.
    for (min_w, min_h) in [
        (Some(0.625), None),
        (None, Some(1.0)),
        (Some(f64::NAN), None),
        (None, Some(f64::INFINITY)),
        (Some(0.5), Some(0.625)),
    ] {
        let err = DockingOperation::SetFloatingRect {
            host,
            index: 0,
            rect,
            min_w,
            min_h,
        }
        .apply(&mut layout)
        .unwrap_err();
        assert_eq!(err, DockingError::FloatingBoundViolation { host, index: 0 });
        assert_eq!(layout, valid); // untouched, deterministically
    }

    // The bound endpoint itself is honored inclusively (§9.3).
    let outcome = DockingOperation::SetFloatingRect {
        host,
        index: 0,
        rect,
        min_w: Some(0.5),
        min_h: Some(0.5),
    }
    .apply(&mut layout)
    .unwrap();
    let DockingOutcome::FloatingRectSet { previous, .. } = &outcome else {
        panic!("expected FloatingRectSet");
    };
    assert_eq!(*previous, rect);
    assert!(layout.is_valid());
}

#[test]
fn set_floating_rect_validates_the_entry_index_and_host_argument() {
    let (mut layout, host) = host_root();
    let f1 = float_new(&mut layout, host, "f1", None);
    let valid = layout.clone();

    // The index names a floating entry of the host's collection.
    let err = DockingOperation::SetFloatingRect {
        host,
        index: 1,
        rect: FloatingRect::new(0.25, 0.25, 0.5, 0.5).unwrap(),
        min_w: None,
        min_h: None,
    }
    .apply(&mut layout)
    .unwrap_err();
    assert_eq!(err, DockingError::FloatingIndexOutOfRange { index: 1, entry_count: 1 });
    assert_eq!(layout, valid);

    // The host argument names an embed host (the first check — a panel
    // rejects even with a valid in-range argument).
    let embed = DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("p".into()) },
        host,
        slot: EmbedSlot::Region(EdgeRegion::Top),
    }
    .apply(&mut layout)
    .unwrap();
    let panel = match embed {
        DockingOutcome::Embedded { source, .. } => source,
        _ => unreachable!("matched Embedded above"),
    };
    let embedded = layout.clone();
    let err = DockingOperation::SetFloatingRect {
        host: panel,
        index: 0,
        rect: FloatingRect::new(0.25, 0.25, 0.5, 0.5).unwrap(),
        min_w: None,
        min_h: None,
    }
    .apply(&mut layout)
    .unwrap_err();
    assert_eq!(err, DockingError::NotAnEmbedHost { id: panel });
    assert_eq!(layout, embedded);

    // An unknown host id rejects first of all (checked before both the
    // category and the index).
    let _ = DockingOperation::Remove { dock: f1 }.apply(&mut layout).unwrap();
    let err = DockingOperation::SetFloatingRect {
        host: f1, // removed: dangling
        index: 0,
        rect: FloatingRect::new(0.25, 0.25, 0.5, 0.5).unwrap(),
        min_w: None,
        min_h: None,
    }
    .apply(&mut layout)
    .unwrap_err();
    assert_eq!(err, DockingError::DockNotFound { id: f1 });
    assert!(layout.is_valid());
}

#[test]
fn floating_rect_bounds_are_inclusive() {
    let (mut layout, host) = host_root();
    // A full-body rect is valid (x + w = y + h = 1.0; PRD §9.6 SHOULD —
    // the entry stays within its host's logical bounds, endpoints in).
    let f1 = float_new(
        &mut layout,
        host,
        "f1",
        Some(FloatingRect::new(0.0, 0.0, 1.0, 1.0).unwrap()),
    );
    let floating = layout.get(host).unwrap().floating().unwrap();
    let stored = floating[0].rect();
    assert_eq!((stored.x(), stored.y(), stored.w(), stored.h()), (0.0, 0.0, 1.0, 1.0));
    assert!(layout.is_valid());

    // A rect stacked in the far corner touches both bounds inclusively.
    let outcome = DockingOperation::SetFloatingRect {
        host,
        index: 0,
        rect: FloatingRect::new(0.25, 0.25, 0.75, 0.75).unwrap(),
        min_w: None,
        min_h: None,
    }
    .apply(&mut layout)
    .unwrap();
    let DockingOutcome::FloatingRectSet { previous, .. } = &outcome else {
        panic!("expected FloatingRectSet");
    };
    assert_eq!(*previous, FloatingRect::new(0.0, 0.0, 1.0, 1.0).unwrap());
    assert!(layout.is_valid());
    let floating = layout.get(host).unwrap().floating().unwrap();
    assert_eq!(floating[0].rect(), FloatingRect::new(0.25, 0.25, 0.75, 0.75).unwrap());
    assert_eq!(layout.placement(f1), Some(DockSlot::Floating { host, index: 0 }));
}

// --- Removal and normalization interactions (PRD §8.11, §8.12, §9.6) ---

#[test]
fn removing_a_floater_clears_its_entry_and_shifts_the_collection() {
    let (mut layout, host) = host_root();
    let r1 = FloatingRect::new(0.125, 0.125, 0.25, 0.25).unwrap();
    let r2 = FloatingRect::new(0.5, 0.5, 0.25, 0.25).unwrap();
    let r3 = FloatingRect::new(0.75, 0.75, 0.125, 0.125).unwrap();
    let f1 = float_new(&mut layout, host, "f1", Some(r1));
    let _f2 = float_new(&mut layout, host, "f2", Some(r2));
    let f3 = float_new(&mut layout, host, "f3", Some(r3));

    let outcome = DockingOperation::Remove { dock: _f2 }.apply(&mut layout).unwrap();
    let DockingOutcome::Removed { docks, inverse } = &outcome else {
        panic!("expected Removed");
    };
    assert_eq!(docks.len(), 1);
    assert!(layout.is_valid());

    // The collection keeps relative order; the removed index's entry is
    // deleted with its rect and the survivors shift by index only, each
    // carrying its persistent rect (the member-removal rule, the §9.6
    // transfer table's cleared stage).
    let floating = layout.get(host).unwrap().floating().unwrap();
    assert_eq!(floating.len(), 2);
    assert_eq!(floating[0].dock, f1);
    assert_eq!(floating[0].rect(), r1);
    assert_eq!(floating[1].dock, f3);
    assert_eq!(floating[1].rect(), r3);
    assert_eq!(layout.placement(f1), Some(DockSlot::Floating { host, index: 0 }));
    assert_eq!(layout.placement(f3), Some(DockSlot::Floating { host, index: 1 }));

    match inverse.containers.first() {
        Some(FormerCollection::Floating { host: h, entries }) => {
            assert_eq!(*h, host);
            assert_eq!(entries.len(), 3);
            assert_eq!(entries[1].dock, _f2);
            assert_eq!(entries[1].rect(), r2);
        }
        other => panic!("expected the floating collection record, got {other:?}"),
    }
}

#[test]
fn removing_a_floated_subtree_takes_its_descendants_in_bfs_order() {
    // A removal of the entry's root removes the whole floated subtree —
    // the BFS sweep crosses split edges, tab-member edges, and the
    // floating edge exactly like any other placement category (PRD
    // §7.5, §8.11).
    let (mut layout, host) = host_root();
    let embed = DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("m1".into()) },
        host,
        slot: EmbedSlot::Region(EdgeRegion::Top),
    }
    .apply(&mut layout)
    .unwrap();
    let m1 = match embed {
        DockingOutcome::Embedded { source, .. } => source,
        _ => unreachable!("matched Embedded above"),
    };
    let join = DockingOperation::TabJoin {
        source: InsertSource::NewPanel { content: Some("m2".into()) },
        target: m1,
    }
    .apply(&mut layout)
    .unwrap();
    let DockingOutcome::TabJoined { source: m2, group, .. } = &join else {
        panic!("expected TabJoined");
    };
    let m2 = *m2;
    let group = *group;
    let insert = DockingOperation::InsertBeside {
        source: InsertSource::NewPanel { content: Some("s".into()) },
        target: group,
        direction: SplitDirection::Right,
    }
    .apply(&mut layout)
    .unwrap();
    let DockingOutcome::InsertedBeside { source: s, container: row, .. } = &insert else {
        panic!("expected InsertedBeside");
    };
    let s = *s;
    let row = *row;
    let _ = DockingOperation::Embed {
        source: InsertSource::Existing(row),
        host,
        slot: EmbedSlot::Floating { rect: None },
    }
    .apply(&mut layout)
    .unwrap();
    assert!(layout.is_valid());

    let outcome = DockingOperation::Remove { dock: row }.apply(&mut layout).unwrap();
    let DockingOutcome::Removed { docks, .. } = &outcome else {
        panic!("expected Removed");
    };
    // BFS: the row, then its children in order (the tab group, the
    // panel), then the group's members.
    assert_eq!(
        docks.iter().map(|dock| dock.id()).collect::<Vec<_>>(),
        vec![row, group, s, m1, m2]
    );

    assert!(layout.is_valid());
    let floating = layout.get(host).unwrap().floating().unwrap();
    assert!(floating.is_empty());
    // The persistent host stands (a valid steady state, PRD §8.8).
    assert_eq!(layout.get(host).unwrap().kind(), DockKind::EmbedHost);
    assert_eq!(layout.get(host).unwrap().persistent(), Some(true));
    assert_eq!(layout.root(), Some(host));
    assert_eq!(layout.len(), 1);
}

#[test]
fn removing_the_last_floater_demotes_the_temporary_host() {
    // §8.9: the auto-promotion exists to support one embedded dock; a
    // floating entry fully occupies the host the same way a region
    // occupant does, and the last float's removal runs the demotion.
    let mut layout = DockLayout::new();
    let p = layout.create_panel(Some("work area".into())); // auto-rooted
    let float = DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("dock".into()) },
        host: p,
        slot: EmbedSlot::Floating {
            rect: Some(FloatingRect::new(0.25, 0.25, 0.5, 0.5).unwrap()),
        },
    }
    .apply(&mut layout)
    .unwrap();
    let DockingOutcome::Embedded { source: f, promoted: true, .. } = &float else {
        panic!("expected an auto-promoted Embedded");
    };
    assert_eq!(layout.get(p).unwrap().kind(), DockKind::EmbedHost);
    assert_eq!(layout.get(p).unwrap().persistent(), Some(false));
    let floating = layout.get(p).unwrap().floating().unwrap();
    assert_eq!(floating.len(), 1);
    assert_eq!(floating[0].rect(), FloatingRect::new(0.25, 0.25, 0.5, 0.5).unwrap());
    assert!(layout.is_valid());

    let _ = DockingOperation::Remove { dock: *f }.apply(&mut layout).unwrap();
    assert!(layout.is_valid());
    // The same dock is a panel again — expanded, same content, same
    // root placement (the in-place demotion, PRD §8.9).
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
