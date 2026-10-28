//! Integration tests: the docking interaction contract (Phase 11,
//! PRD §10–§11).
//!
//! These pin the *handoff* a drag-and-drop frontend relies on, from
//! domain-level inputs only (the environment never needs coordinates):
//!
//! - the region + mode → operation mapping (§10.2–§10.3);
//! - the eligibility discovery questions (§10.4) and the settled
//!   rejection payloads — a rejection is the whole consequence (§10.5,
//!   §8.2) and never reaches the persisted form (§11.1);
//! - gesture commit boundaries / cancellation rollback through frames
//!   and transaction spans (§10.5, §11.4);
//! - the full lifecycle walkthrough in stage order (§10.1).
//!
//! Per-operation behavior detail lives in the operation/environment
//! suites (`docking_operations_test.rs`, `docking_floating_test.rs`,
//! …); this file pins the contract-level shapes those compose into.
//! Public crate API only (agents/testing.md); known-absent ids use the
//! `DockId::from_raw` fixture precedent of `docking_operations_test.rs`.

use app_shell::domain_engine::{DomainEngine, DomainError};
use app_shell::domain_foundation::environments::docking::{
    DockId, DockKind, DockLayout, DockSlot, DockingError, DockingOperation, DockingOutcome,
    EdgeRegion, EmbedSlot, FloatingRect, InsertSource, SplitDirection,
};

// --- Fixtures ----------------------------------------------------------

fn expect_created(outcome: Result<DockingOutcome, DomainError>) -> DockId {
    match outcome {
        Ok(DockingOutcome::Created { dock, is_root, .. }) => {
            assert!(is_root, "creation on an empty layout auto-roots");
            dock
        }
        other => panic!("expected Created, got {other:?}"),
    }
}

fn expect_inserted(outcome: Result<DockingOutcome, DomainError>) -> (DockId, DockId) {
    match outcome {
        Ok(DockingOutcome::InsertedBeside { source, container, .. }) => (source, container),
        other => panic!("expected InsertedBeside, got {other:?}"),
    }
}

fn expect_tab_joined(outcome: Result<DockingOutcome, DomainError>) -> (DockId, DockId) {
    match outcome {
        Ok(DockingOutcome::TabJoined { source, group, .. }) => (source, group),
        other => panic!("expected TabJoined, got {other:?}"),
    }
}

fn expect_embedded(
    outcome: Result<DockingOutcome, DomainError>,
) -> (DockId, DockId, EmbedSlot, bool) {
    match outcome {
        Ok(DockingOutcome::Embedded { source, host, slot, promoted, .. }) => {
            (source, host, slot, promoted)
        }
        other => panic!("expected Embedded, got {other:?}"),
    }
}

/// Submit an incompatible intent; assert the settled rejection payload
/// and that the rejection is the *whole* consequence — the layout is
/// untouched exactly as it was and nothing is recorded or persisted
/// (PRD §10.5, §8.2; §11.1).
fn rejection(
    engine: &mut DomainEngine,
    operation: DockingOperation,
    case: &'static str,
    expected: DockingError,
    layout_before: &DockLayout,
) {
    match engine.execute_docking(operation) {
        Err(DomainError::Docking(err)) => assert_eq!(err, expected, "{case}: rejection payload"),
        other => panic!("{case}: expected a rejection, got {other:?}"),
    }
    assert_eq!(engine.docking(), layout_before, "{case}: layout untouched");
}

// --- §10.3: region + mode → operation ----------------------------------

#[test]
fn split_mode_edge_intents_map_to_split_directions() {
    let intents = [
        (SplitDirection::Top, false),
        (SplitDirection::Bottom, true),
        (SplitDirection::Left, false),
        (SplitDirection::Right, true),
    ];
    for (direction, target_first) in intents {
        let mut engine = DomainEngine::new();
        let target = expect_created(engine.execute_docking(DockingOperation::CreatePanel {
            content: Some("target".into()),
        }));

        let (source, container) = expect_inserted(
            engine.execute_docking(DockingOperation::InsertBeside {
                source: InsertSource::NewPanel { content: Some("drop".into()) },
                target,
                direction,
            }),
        );

        // The mode + edge intent pair selects a split along the
        // intent's orientation, with child order following the intent.
        let dock = engine.docking().get(container).expect("container exists");
        assert_eq!(dock.kind(), DockKind::Split);
        assert_eq!(
            dock.orientation(),
            Some(direction.orientation()),
            "{direction:?}: orientation"
        );
        let source_index = if direction.inserts_after() { 1 } else { 0 };
        assert_eq!(
            engine.docking().placement(source),
            Some(DockSlot::SplitChild { parent: container, index: source_index }),
            "{direction:?}: source placement"
        );
        let children = dock.children().expect("split children");
        assert_eq!(children.len(), 2, "{direction:?}: pair membership");
        let expected = if target_first { vec![target, source] } else { vec![source, target] };
        assert_eq!(children, &expected, "{direction:?}: child order follows the edge intent");
    }
}

#[test]
fn tab_mode_center_intent_joins_or_creates_a_group() {
    let mut engine = DomainEngine::new();
    let target = expect_created(engine.execute_docking(DockingOperation::CreatePanel {
        content: Some("target".into()),
    }));

    // Center drop on an ungrouped panel: a new group wraps the pair and
    // inherits the target's slot; the joined source becomes active.
    let (source, group) = expect_tab_joined(engine.execute_docking(DockingOperation::TabJoin {
        source: InsertSource::NewPanel { content: Some("drop".into()) },
        target,
    }));
    assert_eq!(
        engine.docking().get(group).expect("group exists").kind(),
        DockKind::TabGroup
    );
    assert_eq!(
        engine.docking().placement(source),
        Some(DockSlot::TabMember { parent: group, index: 1 })
    );
    assert_eq!(
        engine.docking().get(group).and_then(|d| d.active()),
        Some(1),
        "the joined member is active"
    );

    // Center drop on an already-grouped target: joins the *existing*
    // group as its last member, which becomes active (no second group).
    let (third, same_group) =
        expect_tab_joined(engine.execute_docking(DockingOperation::TabJoin {
            source: InsertSource::NewPanel { content: Some("third".into()) },
            target,
        }));
    assert_eq!(same_group, group, "an existing group is reused");
    assert_eq!(
        engine.docking().placement(third),
        Some(DockSlot::TabMember { parent: group, index: 2 })
    );
    assert_eq!(engine.docking().get(group).and_then(|d| d.active()), Some(2));

    let members = engine
        .docking()
        .get(group)
        .expect("group exists")
        .structural_children();
    assert_eq!(members, vec![target, source, third], "membership in tab order");
}

#[test]
fn embed_mode_edge_intents_take_the_host_regions() {
    for edge in [EdgeRegion::Top, EdgeRegion::Bottom, EdgeRegion::Left, EdgeRegion::Right] {
        let mut engine = DomainEngine::new();
        let host = expect_created(engine.execute_docking(DockingOperation::CreateEmbedHost {
            content: Some("host".into()),
        }));
        let (source, _) = expect_inserted(
            engine.execute_docking(DockingOperation::InsertBeside {
                source: InsertSource::NewPanel { content: Some("panel".into()) },
                target: host,
                direction: SplitDirection::Right,
            }),
        );

        // The eligibility discovery reads agree with the commit (§10.4):
        // a host target, a free region, no host-inside-source cycle.
        let dock = engine.docking().get(host).expect("host exists");
        assert_eq!(dock.kind(), DockKind::EmbedHost);
        assert!(dock.host_regions().expect("host regions").on(edge).is_none());
        assert!(!engine.docking().is_descendant(host, source), "no cycle");

        let (src, _host, slot, promoted) =
            expect_embedded(engine.execute_docking(DockingOperation::Embed {
                source: InsertSource::Existing(source),
                host,
                slot: EmbedSlot::Region(edge),
            }));
        assert_eq!(src, source, "{edge:?}: identity preserved");
        assert_eq!(slot, EmbedSlot::Region(edge));
        assert!(!promoted, "{edge:?}: an explicit host is never promoted");
        assert_eq!(
            engine.docking().placement(source),
            Some(DockSlot::HostEdge { host, edge }),
            "{edge:?}: the intent landed in the named region"
        );
        let weight = engine
            .docking()
            .get(host)
            .and_then(|d| d.host_regions())
            .and_then(|regions| regions.weight_on(edge))
            .expect("the occupied region carries its slot weight");
        assert!(
            (weight - 1.0).abs() < f64::EPSILON,
            "{edge:?}: freshly occupied regions start at the §9.5 baseline"
        );
    }
}

#[test]
fn embed_mode_floating_drop_carries_the_drop_rect() {
    let mut engine = DomainEngine::new();
    let host = expect_created(engine.execute_docking(DockingOperation::CreateEmbedHost {
        content: Some("host".into()),
    }));
    let (first, _) = expect_inserted(
        engine.execute_docking(DockingOperation::InsertBeside {
            source: InsertSource::NewPanel { content: Some("float me".into()) },
            target: host,
            direction: SplitDirection::Right,
        }),
    );

    // A body drop supplies the drop interaction's logical rect (§9.6):
    // it lands as the entry's initial placement, subtree intact.
    let drop_rect = FloatingRect::new(0.25, 0.1, 0.4, 0.6).expect("valid drop rect");
    let (source, _, slot, _) =
        expect_embedded(engine.execute_docking(DockingOperation::Embed {
            source: InsertSource::Existing(first),
            host,
            slot: EmbedSlot::Floating { rect: Some(drop_rect) },
        }));
    assert_eq!(source, first, "the floater's root keeps its identity");
    assert_eq!(slot, EmbedSlot::Floating { rect: Some(drop_rect) });
    assert_eq!(
        engine.docking().placement(first),
        Some(DockSlot::Floating { host, index: 0 })
    );
    let entries = engine.docking().get(host).expect("host").floating().expect("entries");
    assert_eq!(entries[0].dock, first);
    assert_eq!(entries[0].rect(), drop_rect);

    // A body drop without an initial rect starts at the floating
    // baseline (§9.6 — the region-weight embedding baseline's
    // floating counterpart).
    let (second, _, slot, _) = expect_embedded(engine.execute_docking(DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("float me too".into()) },
        host,
        slot: EmbedSlot::Floating { rect: None },
    }));
    assert_eq!(slot, EmbedSlot::Floating { rect: None });
    assert_eq!(engine.docking().placement(second), Some(DockSlot::Floating { host, index: 1 }));
    let entries = engine.docking().get(host).expect("host").floating().expect("entries");
    assert_eq!(entries[1].dock, second);
    assert_eq!(entries[1].rect(), FloatingRect::BASELINE, "baseline initial rect");
}

// --- §10.4: settled rejection payloads ---------------------------------

#[test]
fn ineligible_intents_reject_with_settled_payloads() {
    let mut engine = DomainEngine::new();
    let a = expect_created(engine.execute_docking(DockingOperation::CreatePanel {
        content: Some("a".into()),
    }));
    let (b, _) = expect_inserted(engine.execute_docking(DockingOperation::InsertBeside {
        source: InsertSource::NewPanel { content: Some("b".into()) },
        target: a,
        direction: SplitDirection::Right,
    }));
    let (c, split) = expect_inserted(engine.execute_docking(DockingOperation::InsertBeside {
        source: InsertSource::NewPanel { content: Some("c".into()) },
        target: b,
        direction: SplitDirection::Right,
    }));
    let (tab_member, tab) =
        expect_tab_joined(engine.execute_docking(DockingOperation::TabJoin {
            source: InsertSource::NewPanel { content: Some("tab member".into()) },
            target: a,
        }));
    let host = match engine.execute_docking(DockingOperation::PromoteHost { dock: c }) {
        Ok(DockingOutcome::HostPromoted { dock, promoted, .. }) => {
            assert!(promoted, "an explicit promotion persists the host");
            dock
        }
        other => panic!("expected HostPromoted, got {other:?}"),
    };
    let (occupant, _, _, _) = expect_embedded(engine.execute_docking(DockingOperation::Embed {
        source: InsertSource::Existing(tab_member),
        host,
        slot: EmbedSlot::Region(EdgeRegion::Top),
    }));
    assert_eq!(occupant, tab_member, "an unoccupied region takes the source subtree");
    let (floater, _, _, _) = expect_embedded(engine.execute_docking(DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("floater".into()) },
        host,
        slot: EmbedSlot::Floating { rect: None },
    }));
    let (doomed, _) = expect_inserted(engine.execute_docking(DockingOperation::InsertBeside {
        source: InsertSource::NewPanel { content: Some("doomed".into()) },
        target: host,
        direction: SplitDirection::Right,
    }));
    engine
        .execute_docking(DockingOperation::Remove { dock: doomed })
        .expect("a removed panel's id is no longer resolvable");

    // Everything after this point is rejected-only: one snapshot covers
    // every case, since a rejection records nothing (§10.5).
    let before = engine.docking().clone();
    let events_before = engine.events().len();
    let history_before = engine.history().position();

    // Embed into a split target — structural containers have no work
    // area to host or promote (v1).
    rejection(
        &mut engine,
        DockingOperation::Embed {
            source: InsertSource::Existing(b),
            host: split,
            slot: EmbedSlot::Region(EdgeRegion::Left),
        },
        "embed into a split",
        DockingError::IneligibleDock { id: split },
        &before,
    );
    // Embed into a tab-group target — the same settle.
    rejection(
        &mut engine,
        DockingOperation::Embed {
            source: InsertSource::Existing(b),
            host: tab,
            slot: EmbedSlot::Region(EdgeRegion::Right),
        },
        "embed into a tab group",
        DockingError::IneligibleDock { id: tab },
        &before,
    );
    // Embed onto an occupied edge region — never replaces an occupant.
    rejection(
        &mut engine,
        DockingOperation::Embed {
            source: InsertSource::NewPanel { content: Some("two-on-top".into()) },
            host,
            slot: EmbedSlot::Region(EdgeRegion::Top),
        },
        "embed onto an occupied region",
        DockingError::RegionOccupied { host, edge: EdgeRegion::Top },
        &before,
    );
    // Directional-insert beside a floated dock — the floating layer has
    // no directional layout surface (the settled §8.6 baseline).
    rejection(
        &mut engine,
        DockingOperation::InsertBeside {
            source: InsertSource::NewPanel { content: Some("beside floater".into()) },
            target: floater,
            direction: SplitDirection::Right,
        },
        "insert beside a floater",
        DockingError::NotSplittable { id: floater },
        &before,
    );
    // Self-targeting: a dock is its own descendant (§10.4: the
    // environment MUST reject invalid self-targeting).
    rejection(
        &mut engine,
        DockingOperation::InsertBeside {
            source: InsertSource::Existing(host),
            target: host,
            direction: SplitDirection::Right,
        },
        "drop onto itself",
        DockingError::CycleDetected { id: host },
        &before,
    );
    // Making the source its own descendant: embedding a subtree into a
    // host that lives inside it names that host.
    rejection(
        &mut engine,
        DockingOperation::Embed {
            source: InsertSource::Existing(split),
            host,
            slot: EmbedSlot::Region(EdgeRegion::Bottom),
        },
        "host inside the dragged subtree",
        DockingError::CycleDetected { id: host },
        &before,
    );
    // An unknown source id names itself (the removed panel's id).
    rejection(
        &mut engine,
        DockingOperation::InsertBeside {
            source: InsertSource::Existing(doomed),
            target: a,
            direction: SplitDirection::Left,
        },
        "unknown source id",
        DockingError::DockNotFound { id: doomed },
        &before,
    );

    assert_eq!(
        engine.events().len(),
        events_before,
        "rejections are recorded nowhere (§10.5)"
    );
    assert_eq!(engine.history().position(), history_before);
}

#[cfg(feature = "serde")]
#[test]
fn rejected_intent_leaves_the_persisted_form_untouched() {
    // The persisted payload is the committed model (§11.1, §12.1): a
    // rejected request — and any preview state a frontend builds around
    // it — never reaches it (the §11.4 explicit commit boundary).
    let mut engine = DomainEngine::new();
    let a = expect_created(engine.execute_docking(DockingOperation::CreatePanel {
        content: Some("a".into()),
    }));
    let (b, _) = expect_inserted(engine.execute_docking(DockingOperation::InsertBeside {
        source: InsertSource::NewPanel { content: Some("b".into()) },
        target: a,
        direction: SplitDirection::Right,
    }));
    let host = match engine.execute_docking(DockingOperation::PromoteHost { dock: b }) {
        Ok(DockingOutcome::HostPromoted { dock, promoted, .. }) => {
            assert!(promoted, "an explicit promotion persists the host");
            dock
        }
        other => panic!("expected HostPromoted, got {other:?}"),
    };
    let (floater, _, _, _) = expect_embedded(engine.execute_docking(DockingOperation::Embed {
        source: InsertSource::Existing(a),
        host,
        slot: EmbedSlot::Floating { rect: None },
    }));

    let payload_before = serde_json::to_vec(engine.docking()).expect("serialize");
    let before = engine.docking().clone();
    rejection(
        &mut engine,
        DockingOperation::InsertBeside {
            source: InsertSource::NewPanel { content: Some("beside floater".into()) },
            target: floater,
            direction: SplitDirection::Right,
        },
        "insert beside a floater",
        DockingError::NotSplittable { id: floater },
        &before,
    );
    let payload_after = serde_json::to_vec(engine.docking()).expect("serialize");
    assert_eq!(payload_before, payload_after, "the persisted form never sees previews");
}

// --- §10.5 / §11.4: gesture commit boundaries ---------------------------

#[test]
fn committed_gesture_frames_rollback_frame_by_frame() {
    let mut engine = DomainEngine::new();
    let host = expect_created(engine.execute_docking(DockingOperation::CreateEmbedHost {
        content: Some("host".into()),
    }));
    let (_, _, _, _) = expect_embedded(engine.execute_docking(DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("float me".into()) },
        host,
        slot: EmbedSlot::Floating { rect: None },
    }));
    let initial = engine.docking().clone();

    // Two committed frames (the absolute-request gesture contract:
    // intermediate measurements never enter the model).
    let frame1 = FloatingRect::new(0.1, 0.0, 0.4, 0.5).expect("valid");
    let frame2 = FloatingRect::new(0.2, 0.2, 0.5, 0.6).expect("valid");

    match engine.execute_docking(DockingOperation::SetFloatingRect {
        host,
        index: 0,
        rect: frame1,
        min_w: None,
        min_h: None,
    }) {
        Ok(DockingOutcome::FloatingRectSet { previous, .. }) => assert_eq!(
            previous,
            FloatingRect::BASELINE,
            "frame 1 moved from the baseline"
        ),
        other => panic!("expected FloatingRectSet, got {other:?}"),
    }
    match engine.execute_docking(DockingOperation::SetFloatingRect {
        host,
        index: 0,
        rect: frame2,
        min_w: None,
        min_h: None,
    }) {
        Ok(DockingOutcome::FloatingRectSet { previous, .. }) => {
            assert_eq!(previous, frame1, "frame 2 moved from frame 1")
        }
        other => panic!("expected FloatingRectSet, got {other:?}"),
    }
    let float = engine.docking().get(host).expect("host").floating().expect("entries");
    assert_eq!(float[0].rect(), frame2);

    // The gesture is cancelled: without a span wrapper the frontend
    // walks the frames back one undo call at a time (§10.5).
    assert_eq!(engine.undo().expect("undo frame 2"), 1);
    let float = engine.docking().get(host).expect("host").floating().expect("entries");
    assert_eq!(float[0].rect(), frame1, "one frame back");
    assert_eq!(engine.undo().expect("undo frame 1"), 1);
    assert_eq!(engine.docking(), &initial, "back to the pre-gesture layout");
}

#[test]
fn gesture_span_transactions_revert_or_undo_as_one_unit() {
    let mut engine = DomainEngine::new();
    let host = expect_created(engine.execute_docking(DockingOperation::CreateEmbedHost {
        content: Some("host".into()),
    }));
    let (_, _, _, _) = expect_embedded(engine.execute_docking(DockingOperation::Embed {
        source: InsertSource::NewPanel { content: Some("float me".into()) },
        host,
        slot: EmbedSlot::Floating { rect: None },
    }));
    let pre_gesture = engine.docking().clone();
    let pre_position = engine.history().position();

    // Cancelled gesture: a transaction span reverts whole, in one call.
    let frame1 = FloatingRect::new(0.1, 0.0, 0.4, 0.5).expect("valid");
    let frame2 = FloatingRect::new(0.2, 0.2, 0.5, 0.6).expect("valid");
    engine.begin_transaction();
    for rect in [frame1, frame2] {
        engine
            .execute_docking(DockingOperation::SetFloatingRect {
                host,
                index: 0,
                rect,
                min_w: None,
                min_h: None,
            })
            .expect("frame commit inside the span");
    }
    assert_eq!(
        engine.docking().get(host).expect("host").floating().expect("entries")[0].rect(),
        frame2,
        "the gesture's frames are visible while it lasts"
    );
    assert!(engine.rollback_transaction().is_some(), "the span rolls back");
    assert_eq!(engine.docking(), &pre_gesture, "the whole span reverted");
    assert_eq!(engine.history().position(), pre_position, "span entries removed");
    assert!(!engine.can_redo(), "a rolled-back span is not redoable");

    // Committed gesture: the span undoes and redoes as one unit.
    engine.begin_transaction();
    for rect in [frame1, frame2] {
        engine
            .execute_docking(DockingOperation::SetFloatingRect {
                host,
                index: 0,
                rect,
                min_w: None,
                min_h: None,
            })
            .expect("frame commit inside the span");
    }
    engine.commit_transaction();
    let committed = engine.docking().clone();
    engine.undo().expect("one call rewinds the committed span");
    assert_eq!(engine.docking(), &pre_gesture, "committed span undoes whole");
    engine.redo().expect("one call replays the committed span");
    assert_eq!(engine.docking(), &committed, "committed span redoes whole");
}

// --- §10.1: the full lifecycle, intent-only -----------------------------

#[test]
fn frontend_drag_lifecycle_walkthrough() {
    let mut engine = DomainEngine::new();

    // Stage 1 — the source is identified through public reads
    // (content → id); no coordinate is involved anywhere.
    let editor = expect_created(engine.execute_docking(DockingOperation::CreatePanel {
        content: Some("editor".into()),
    }));
    let (inspector, _) = expect_inserted(engine.execute_docking(DockingOperation::InsertBeside {
        source: InsertSource::NewPanel { content: Some("inspector".into()) },
        target: editor,
        direction: SplitDirection::Right,
    }));
    let found = engine
        .docking()
        .iter()
        .find(|d| d.content().map(|c| c.as_str()) == Some("inspector"))
        .expect("content lookup")
        .id();
    assert_eq!(found, inspector, "stage 1 identifies the dragged element");

    // Stage 2 — "start drag" is frontend state; the model observes
    // nothing.

    // Stage 3 — eligibility discovery for the staged drag (tab mode,
    // center region over `editor`): panel target, panel source, no
    // cycle. The reads are advisory; the commit re-validates.
    assert_eq!(
        engine.docking().get(inspector).expect("source exists").kind(),
        DockKind::Panel
    );
    assert_eq!(
        engine.docking().get(editor).expect("target exists").kind(),
        DockKind::Panel
    );
    assert!(!engine.docking().is_descendant(editor, inspector), "no cycle");

    // Stages 4–6 — target + center region + tab mode → the request;
    // one confirmed drop is one submitted value.
    let (source, group) = expect_tab_joined(engine.execute_docking(DockingOperation::TabJoin {
        source: InsertSource::Existing(inspector),
        target: editor,
    }));
    assert_eq!(source, inspector, "stage 6: identity carried through");

    // Stage 7 — the outcome realizes the ids/slots to display.
    assert_eq!(
        engine.docking().placement(inspector),
        Some(DockSlot::TabMember { parent: group, index: 1 })
    );
    assert_eq!(engine.docking().get(group).and_then(|d| d.active()), Some(1));

    // Stage 8 — transient drag state cleared (frontend posture);
    // nothing domain-side exists to clear.

    // A second drag picks a mode whose mode+region pair does not fit
    // the target: the embed mode over a tab group. The rejection names
    // the violation, the layout is untouched, and the frontend clears
    // state regardless — no drag residue anywhere.
    let (_chat, _) = expect_inserted(engine.execute_docking(DockingOperation::InsertBeside {
        source: InsertSource::NewPanel { content: Some("chat".into()) },
        target: editor,
        direction: SplitDirection::Right,
    }));
    let before = engine.docking().clone();
    let events_before = engine.events().len();
    rejection(
        &mut engine,
        DockingOperation::Embed {
            source: InsertSource::Existing(found),
            host: group,
            slot: EmbedSlot::Region(EdgeRegion::Left),
        },
        "second drag: incompatible mode",
        DockingError::IneligibleDock { id: group },
        &before,
    );
    assert_eq!(
        engine.events().len(),
        events_before,
        "the failed drag left exactly the pre-drag record"
    );
}
