//! Integration tests: docking operation **sequences** through the Domain
//! Engine (Phase 12 — verification and release readiness, PRD §18.6–§18.7,
//! AC-14/AC-18).
//!
//! The Phase-4–11 suites pin the individual operations; this file composes
//! them into the sequence-level acceptance shapes the plan names:
//!
//! 1. **split → move → tab group → remove** — the plan's named sequence,
//!    end-to-end through `execute_docking`, with undo walking the whole
//!    sequence back to empty and redo recovering the same ids (§18.6:
//!    nested structures + sizing + normalization, not only simple moves).
//! 2. **host promotion/demotion round-trips** — the explicit persistent
//!    promotion's undo round-trip and the temporary auto-promotion's
//!    automatic demotion when its occupant leaves (PRD §8.9).
//! 3. **floating and re-docking of nested structures** — a nested subtree
//!    (split containing a tab group) floats as one entry, re-docks
//!    through another host's region, and its sizing/membership ride the
//!    sequence verbatim (PRD §8.11, §9.6; §18.2/§18.4).
//! 4. **sequence-level rejection atomicity** — invalid requests inside a
//!    rich sequence reject with the settled payload and change nothing
//!    (AC-11, §18.3 across categories).
//!
//! Per-operation behavior detail lives in the operation/environment
//! suites; per-op undo is pinned in `docking_undo_redo_test.rs`. Public
//! crate API only (agents/testing.md); absent ids use the
//! `DockId::from_raw` fixture precedent.

use app_shell::domain_engine::{DomainEngine, DomainError};
use app_shell::domain_foundation::environments::docking::{
    DockId, DockLayout, DockRole, DockSlot, DockingError, DockingOperation, DockingOutcome,
    EdgeRegion, EmbedSlot, FloatingRect, InsertSource, SplitDirection,
};

// --- Fixtures (the sibling suites' shapes, kept local) ----------------------

fn expect_created(outcome: Result<DockingOutcome, DomainError>) -> DockId {
    match outcome {
        Ok(DockingOutcome::Created { dock, is_root, .. }) => {
            assert!(is_root, "first dock on an empty layout auto-roots");
            dock
        }
        other => panic!("expected Created, got {other:?}"),
    }
}

fn expect_inserted(outcome: Result<DockingOutcome, DomainError>) -> (DockId, DockId, DockId) {
    match outcome {
        Ok(DockingOutcome::InsertedBeside { source, target, container, .. }) => {
            (source, target, container)
        }
        other => panic!("expected InsertedBeside, got {other:?}"),
    }
}

fn expect_tab_joined(outcome: Result<DockingOutcome, DomainError>) -> (DockId, DockId, DockId) {
    match outcome {
        Ok(DockingOutcome::TabJoined { source, target, group, .. }) => (source, target, group),
        other => panic!("expected TabJoined, got {other:?}"),
    }
}

fn expect_removed(outcome: Result<DockingOutcome, DomainError>) -> Vec<DockId> {
    match outcome {
        Ok(DockingOutcome::Removed { docks, .. }) => docks.into_iter().map(|d| d.id).collect(),
        other => panic!("expected Removed, got {other:?}"),
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

fn expect_promoted(outcome: Result<DockingOutcome, DomainError>) -> (DockId, bool) {
    match outcome {
        Ok(DockingOutcome::HostPromoted { dock, promoted, .. }) => (dock, promoted),
        other => panic!("expected HostPromoted, got {other:?}"),
    }
}

/// Submit an invalid request inside a sequence; assert the settled
/// rejection payload and that the rejection is the whole consequence —
/// the layout is untouched exactly as it was (AC-11, PRD §10.5).
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

fn new_panel(content: &str) -> InsertSource {
    InsertSource::NewPanel { content: Some(content.into()) }
}

fn absent(id: u64) -> DockId {
    DockId::from_raw(id)
}

/// The owner of `dock`'s current placement, resolved through the public
/// read API (placement is derived by scan — no parent back-pointers,
/// PRD §7.6).
fn owner_of(engine: &DomainEngine, dock: DockId) -> DockId {
    engine
        .docking()
        .placement(dock)
        .and_then(|slot| slot.owner())
        .expect("a placed dock has an owner")
}

/// Compare two layouts up to the monotone id counter (the Phase-4
/// `structural_equal` shape — dock payloads + root; replay is
/// counter-monotone, so `PartialEq` need not hold after an undo).
fn structural_equal(a: &DockLayout, b: &DockLayout) -> bool {
    let snapshot = |layout: &DockLayout| {
        let mut docks: Vec<_> = layout.iter().cloned().collect();
        docks.sort_by_key(|dock| dock.id);
        (layout.root(), docks)
    };
    snapshot(a) == snapshot(b)
}

// --- 1. split → move → tab group → remove (plan Phase 12) -------------------

/// The plan's named sequence, driven end-to-end through the engine:
/// root panel → split-insert → move the inserted panel across the split
/// (identity preserved) → a fresh panel tab-joins the editor (joined
/// member active) → tab reorder + active selection + a sizing frame →
/// remove the tab group with its cascade. Every step leaves a valid
/// layout; undo walks the whole sequence back to the empty layout; redo
/// recovers the same ids (AC-14, §18.6).
#[test]
fn split_move_tab_group_remove_sequence_round_trips_history() {
    let mut engine = DomainEngine::new();

    // (a) Create the root panel.
    let editor = expect_created(engine.execute_docking(DockingOperation::CreatePanel {
        content: Some("editor".into()),
    }));
    assert_eq!(engine.docking().root(), Some(editor));

    // (b) Split: a second panel beside it in a fresh row wrap.
    let (inspector, _target, row) =
        expect_inserted(engine.execute_docking(DockingOperation::InsertBeside {
            source: new_panel("inspector"),
            target: editor,
            direction: SplitDirection::Right,
        }));
    assert_eq!(engine.docking().root(), Some(row), "the wrap inherited the root slot");
    assert_eq!(
        engine.docking().placement(editor),
        Some(DockSlot::SplitChild { parent: row, index: 0 })
    );
    assert_eq!(
        engine.docking().placement(inspector),
        Some(DockSlot::SplitChild { parent: row, index: 1 })
    );

    // (c) Move: the *existing* panel re-places across the split — same
    // id, same content (PRD DM-03, AC-03/AC-08). The detach and insert
    // cancel out inside one operation: the row survives.
    let (moved, moved_target, move_container) =
        expect_inserted(engine.execute_docking(DockingOperation::InsertBeside {
            source: InsertSource::Existing(inspector),
            target: editor,
            direction: SplitDirection::Left,
        }));
    assert_eq!(moved, inspector, "moves keep identity (PRD DM-03)");
    assert_eq!(moved_target, editor);
    assert_eq!(move_container, row, "same-row move reuses the container");
    assert_eq!(
        engine.docking().placement(inspector),
        Some(DockSlot::SplitChild { parent: row, index: 0 }),
        "the moved panel now sits left of the editor"
    );
    assert_eq!(
        engine.docking().get(inspector).unwrap().content().map(|c| c.as_str()),
        Some("inspector"),
        "content rides with identity"
    );

    // (d) Tab group: a fresh panel tab-joins the editor, wrapping it in
    // a new group that inherits the editor's row slot; the joined
    // member becomes active (PRD §8.7 baseline).
    let (notes, _tab_target, group) =
        expect_tab_joined(engine.execute_docking(DockingOperation::TabJoin {
            source: new_panel("notes"),
            target: editor,
        }));
    assert_eq!(owner_of(&engine, editor), group, "the group wrapped the editor's slot");
    assert_eq!(
        engine.docking().placement(notes),
        Some(DockSlot::TabMember { parent: group, index: 1 }),
        "the joined dock appended last"
    );
    let tab_dock = engine.docking().get(group).unwrap();
    let active = tab_dock.active().unwrap();
    assert_eq!(
        tab_dock.structural_children()[active],
        notes,
        "the joined member became active (PRD §8.7 baseline)"
    );

    // (e) Tab order + selection updates and a sizing frame within the
    // same sequence (§18.4: sizing stays coherent across moves).
    engine
        .execute_docking(DockingOperation::ReorderTab { group, from: 0, to: 1 })
        .expect("reorder the editor after the joined member");
    engine
        .execute_docking(DockingOperation::SetActiveTab { group, index: 1 })
        .unwrap();
    engine
        .execute_docking(DockingOperation::ResizeSiblings {
            container: row,
            first: 0,
            requested: 1.5,
            first_min: None,
            second_min: None,
        })
        .unwrap();
    let weights: Vec<f64> = engine
        .docking()
        .get(row)
        .unwrap()
        .sizes()
        .unwrap()
        .weights()
        .to_vec();
    assert_eq!(weights[0], 1.5, "the pair rewrite landed verbatim (PRD §9.2)");

    assert!(engine.docking().is_valid(), "the sequence leaves a valid layout");

    // (f) Remove the tab group: the whole subtree goes (PRD §8.11 (b)),
    // and the emptied split collapses onto its survivor (§8.12).
    let removed = expect_removed(engine.execute_docking(DockingOperation::Remove { dock: group }));
    assert_eq!(removed[0], group, "BFS: the removed dock first");
    assert!(removed.contains(&editor) && removed.contains(&notes));
    assert!(!engine.docking().contains(editor) && !engine.docking().contains(notes));
    assert!(!engine.docking().contains(row), "the emptied split was cascade-removed");
    assert!(engine.docking().is_valid());
    assert_eq!(
        engine.docking().root(),
        Some(inspector),
        "the survivor inherited the root slot"
    );

    // (g) Undo walks the whole sequence back to the empty layout; redo
    // recovers the same ids (§18.6; AC-14).
    let final_layout = engine.docking().clone();
    assert_eq!(engine.history().position(), 8, "eight operations recorded");
    for _ in 0..8 {
        engine.undo().unwrap();
    }
    assert!(engine.docking().is_empty(), "full undo reaches the empty layout");
    assert!(engine.docking().is_valid());
    for _ in 0..8 {
        engine.redo().unwrap();
    }
    assert!(
        structural_equal(engine.docking(), &final_layout),
        "redo recovers the same ids and layout"
    );
    assert_eq!(engine.docking().root(), Some(inspector));
    assert!(!engine.docking().contains(editor), "the tab group stays removed after redo");
}

// --- 2. Host promotion / demotion round-trips (PRD §8.9) --------------------

/// Explicit **persistent** promotion round-trip: a promoted host stays a
/// host even when emptied, and undo restores the panel payload exactly
/// in the same slot (AC-06, §18.2).
#[test]
fn explicit_persistent_promotion_undo_round_trip_keeps_identity_and_slot() {
    let mut engine = DomainEngine::new();
    let p1 = expect_created(engine.execute_docking(DockingOperation::CreatePanel {
        content: Some("one".into()),
    }));
    let (p2, _target, row) =
        expect_inserted(engine.execute_docking(DockingOperation::InsertBeside {
            source: new_panel("two"),
            target: p1,
            direction: SplitDirection::Right,
        }));

    let (promoted, was_promoted) =
        expect_promoted(engine.execute_docking(DockingOperation::PromoteHost { dock: p2 }));
    assert_eq!(promoted, p2, "promotion preserves identity (PRD §8.9)");
    assert!(was_promoted);
    assert_eq!(engine.docking().root(), Some(row), "placement untouched by the role change");
    assert!(
        matches!(
            engine.docking().get(p2).unwrap().role,
            DockRole::EmbedHost { persistent: true, .. }
        ),
        "the explicit promotion is *persistent* (stays a host when emptied)"
    );
    // An empty persistent host is a valid steady state (PRD §8.8).
    assert!(engine.docking().is_valid());

    // Undo restores the pre-promotion panel role in the same slot.
    engine.undo().unwrap();
    assert!(
        matches!(engine.docking().get(p2).unwrap().role, DockRole::Panel { .. }),
        "undo restores the pre-promotion panel role"
    );
    assert_eq!(
        engine.docking().placement(p2),
        Some(DockSlot::SplitChild { parent: row, index: 1 }),
        "the slot survived the role round-trip verbatim"
    );
    assert!(engine.docking().is_valid());
    engine.redo().unwrap();
    assert!(
        matches!(
            engine.docking().get(p2).unwrap().role,
            DockRole::EmbedHost { persistent: true, .. }
        ),
        "redo re-promotes to a *persistent* host"
    );
}

/// The temporary route: embedding into a panel auto-promotes it (a host
/// created solely to support that embed), and removing the last occupant
/// demotes it back to a panel — identity, content, and placement
/// verbatim, through the shared normalization pass (PRD §8.9 baseline).
#[test]
fn temporary_auto_promotion_demotes_when_its_last_occupant_leaves() {
    let mut engine = DomainEngine::new();
    let p1 = expect_created(engine.execute_docking(DockingOperation::CreatePanel {
        content: Some("work area".into()),
    }));

    // Embedding into a panel target promotes it — a *temporary* host.
    let (embedded, host, slot, promoted) =
        expect_embedded(engine.execute_docking(DockingOperation::Embed {
            source: new_panel("left dock"),
            host: p1,
            slot: EmbedSlot::Region(EdgeRegion::Left),
        }));
    assert_eq!(host, p1, "the panel became the host in place");
    assert!(embedded != p1, "the embedded dock is a fresh panel");
    assert_eq!(slot, EmbedSlot::Region(EdgeRegion::Left));
    assert!(promoted, "a panel target is auto-promoted by this operation");
    assert!(
        matches!(
            engine.docking().get(p1).unwrap().role,
            DockRole::EmbedHost { persistent: false, .. }
        ),
        "the auto-promotion is temporary"
    );
    assert!(
        matches!(engine.docking().get(embedded).unwrap().role, DockRole::Panel { .. }),
        "the embedded dock stays a panel (§8.9 category parity)"
    );
    assert_eq!(
        engine.docking().get(p1).unwrap().content().map(|c| c.as_str()),
        Some("work area"),
        "the host retains its underlying work area (PRD §8.8)"
    );
    assert_eq!(engine.docking().root(), Some(p1));

    // Remove the occupant: the emptied temporary host demotes back to a
    // panel (the §8.9 baseline rule) — same id, panel role again, same
    // content.
    expect_removed(engine.execute_docking(DockingOperation::Remove { dock: embedded }));
    let p1_after = engine.docking().get(p1).unwrap();
    assert!(
        matches!(p1_after.role, DockRole::Panel { .. })
            && p1_after.content().map(|c| c.as_str()) == Some("work area"),
        "demotion restores the panel role in place with its content"
    );
    assert!(engine.docking().is_valid());
    assert_eq!(
        engine.docking().root(),
        Some(p1),
        "identity/placement untouched by the round-trip"
    );

    // Undoing the removal restores the (temporary) host role together
    // with its occupant — the demotion's inverse is the promotion
    // (§8.9); redo re-runs the round-trip.
    engine.undo().unwrap();
    assert!(
        matches!(
            engine.docking().get(p1).unwrap().role,
            DockRole::EmbedHost { persistent: false, .. }
        ),
        "undo re-promotes the temporary host"
    );
    assert!(engine.docking().contains(embedded), "undo restores the occupant");
    engine.redo().unwrap();
    assert!(
        matches!(engine.docking().get(p1).unwrap().role, DockRole::Panel { .. }),
        "redo re-demotes through the shared pass"
    );
}

// --- 3. Floating and re-docking a nested structure (PRD §8.11, §9.6) --------

/// A nested subtree — a split whose child is a tab group — floats as
/// **one entry** with its internal structure verbatim (never flattened),
/// then re-docks through a second host's edge region. Sizing and
/// membership ride the whole sequence (AC-07, §18.2, §18.4 "preservation
/// of sizing preferences after movement").
#[test]
fn floating_and_redocking_a_nested_structure_preserves_it_verbatim() {
    let mut engine = DomainEngine::new();

    // Nested build through the engine: root row [shell, inspector,
    // inner[tabs[editor, notes], output]] — the inner split carries
    // non-uniform weights that must survive the float (PRD §9.2).
    let shell = expect_created(engine.execute_docking(DockingOperation::CreatePanel {
        content: Some("shell".into()),
    }));
    let (editor, _t1, _row) =
        expect_inserted(engine.execute_docking(DockingOperation::InsertBeside {
            source: new_panel("editor"),
            target: shell,
            direction: SplitDirection::Right,
        }));
    let (inspector, _t2, _row) =
        expect_inserted(engine.execute_docking(DockingOperation::InsertBeside {
            source: new_panel("inspector"),
            target: editor,
            direction: SplitDirection::Left,
        }));
    // An incompatible direction wraps the editor into a fresh inner
    // column that inherits the editor's row slot (§8.6).
    let (_output, _t3, inner) =
        expect_inserted(engine.execute_docking(DockingOperation::InsertBeside {
            source: new_panel("output"),
            target: editor,
            direction: SplitDirection::Bottom,
        }));
    // Tab-join a fresh panel into the editor: the group wraps the
    // editor inside the inner split (joined member active).
    let (_notes, _t4, group) =
        expect_tab_joined(engine.execute_docking(DockingOperation::TabJoin {
            source: new_panel("notes"),
            target: editor,
        }));
    // Non-uniform weights on the inner split, set *after* the wrap so
    // they describe the floated arrangement itself (§9.2 verbatim).
    engine
        .execute_docking(DockingOperation::ResizeSiblings {
            container: inner,
            first: 0,
            requested: 1.5,
            first_min: None,
            second_min: None,
        })
        .unwrap();
    let inner_sizes = |engine: &DomainEngine, id: DockId| {
        engine.docking().get(id).unwrap().sizes().unwrap().weights().to_vec()
    };
    assert_eq!(inner_sizes(&engine, inner), vec![1.5, 0.5]);
    assert_eq!(owner_of(&engine, group), inner, "the group lives inside the inner split");
    assert!(engine.docking().is_valid());

    // Make the anchor an explicit *persistent* host first: promotion
    // preserves identity, content, and placement (PRD §8.9).
    let (promoted_dock, was_promoted) =
        expect_promoted(engine.execute_docking(DockingOperation::PromoteHost { dock: shell }));
    assert_eq!(promoted_dock, shell);
    assert!(was_promoted);
    assert!(
        matches!(
            engine.docking().get(shell).unwrap().role,
            DockRole::EmbedHost { persistent: true, .. }
        ),
        "shell is a persistent host now"
    );

    // Float the whole nested subtree as one entry with an explicit
    // rect (§9.6): source kind unrestricted (§8.11).
    let rect = FloatingRect::new(0.1, 0.1, 0.8, 0.8).unwrap();
    let (floated, float_host, slot, promoted) =
        expect_embedded(engine.execute_docking(DockingOperation::Embed {
            source: InsertSource::Existing(inner),
            host: shell,
            slot: EmbedSlot::Floating { rect: Some(rect) },
        }));
    assert_eq!(floated, inner, "float keeps the subtree root's identity");
    assert_eq!(float_host, shell);
    assert_eq!(slot, EmbedSlot::Floating { rect: Some(rect) });
    assert!(!promoted, "no promotion happened in this test");
    let floating = engine.docking().get(shell).unwrap().floating().unwrap();
    assert_eq!(floating.len(), 1, "one entry for the floated subtree");
    assert_eq!(floating[0].rect(), rect, "the entry carries the drop rect (§9.6)");
    assert_eq!(owner_of(&engine, group), inner, "the tab group rode with the subtree");
    assert_eq!(inner_sizes(&engine, inner), vec![1.5, 0.5], "weights ride verbatim (§9.2)");
    assert!(engine.docking().is_valid());

    // Re-dock through a second host's edge region: the vacated entry
    // retires with the slot, the subtree lands intact.
    let (host_b, _tb, _row2) =
        expect_inserted(engine.execute_docking(DockingOperation::InsertBeside {
            source: new_panel("second shell"),
            target: inspector,
            direction: SplitDirection::Right,
        }));
    let (b_dock, b_was_promoted) =
        expect_promoted(engine.execute_docking(DockingOperation::PromoteHost { dock: host_b }));
    assert_eq!(b_dock, host_b);
    assert!(b_was_promoted, "the explicit promotion persists host B");
    let (redocked, redock_host, redock_slot, promoted_b) =
        expect_embedded(engine.execute_docking(DockingOperation::Embed {
            source: InsertSource::Existing(inner),
            host: host_b,
            slot: EmbedSlot::Region(EdgeRegion::Top),
        }));
    assert_eq!(redocked, inner);
    assert_eq!(redock_host, host_b);
    assert_eq!(redock_slot, EmbedSlot::Region(EdgeRegion::Top));
    assert!(!promoted_b, "host B was already promoted explicitly");
    assert!(
        engine.docking().get(shell).unwrap().floating().unwrap().is_empty(),
        "the vacated entry went with the cleared slot"
    );
    assert!(
        matches!(
            engine.docking().placement(inner),
            Some(DockSlot::HostEdge { host, edge: EdgeRegion::Top }) if host == host_b
        ),
        "the subtree re-docked under host B's top edge"
    );
    assert_eq!(owner_of(&engine, group), inner, "the nested group re-docked with its parent");
    assert_eq!(inner_sizes(&engine, inner), vec![1.5, 0.5], "weights survive re-docking (§18.4)");
    assert!(engine.docking().is_valid());

    // A later floater in host B starts at the baseline: entry positions
    // are per-entry persistent state (§9.6).
    let (floater2, host2, slot2, _) =
        expect_embedded(engine.execute_docking(DockingOperation::Embed {
            source: new_panel("palette"),
            host: host_b,
            slot: EmbedSlot::Floating { rect: None },
        }));
    assert_eq!(host2, host_b);
    assert_eq!(slot2, EmbedSlot::Floating { rect: None });
    let entries = engine.docking().get(host_b).unwrap().floating().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].rect(), FloatingRect::default(), "None lands the baseline rect");
    assert!(floater2 != inner, "the re-docked subtree is not a floating entry anymore");

    // Undo the sequence piecewise — the re-dock, the promotion, the
    // insert, the second float — each lands on the recorded pre-state
    // (§18.6: nested structures, not only simple moves).
    engine.undo().unwrap();
    assert!(
        engine.docking().get(host_b).unwrap().floating().unwrap().is_empty(),
        "undo unwinds the second float"
    );
    assert!(!engine.docking().contains(floater2), "the created floater left with its undo");
    engine.undo().unwrap(); // the re-dock: inner re-floats under shell
    assert!(
        matches!(
            engine.docking().placement(inner),
            Some(DockSlot::Floating { host, index: 0 }) if host == shell
        ),
        "undo restores the float slot with the entry"
    );
    assert_eq!(
        engine.docking().get(shell).unwrap().floating().unwrap()[0].rect(),
        rect,
        "the entry's rect rides the undo verbatim"
    );
    engine.undo().unwrap(); // the promotion: B back to a panel
    assert!(
        matches!(engine.docking().get(host_b).unwrap().role, DockRole::Panel { .. }),
        "undo restores B's panel role"
    );
    engine.undo().unwrap(); // the insert: B leaves
    assert!(!engine.docking().contains(host_b));
    // The floated subtree is still intact with its sizing state.
    assert_eq!(inner_sizes(&engine, inner), vec![1.5, 0.5], "weights rode every replay (§18.4)");
    assert_eq!(owner_of(&engine, group), inner);
    assert!(engine.docking().is_valid());
    // Redo replays the whole float → promote → re-dock → float sequence.
    for _ in 0..4 {
        engine.redo().unwrap();
    }
    assert!(
        matches!(
            engine.docking().placement(inner),
            Some(DockSlot::HostEdge { host, edge: EdgeRegion::Top }) if host == host_b
        ),
        "redo re-docks the subtree under host B"
    );
    assert!(engine.docking().is_valid());
}

// --- 4. Sequence-level rejection atomicity (AC-11, §18.3) -------------------

/// Invalid requests submitted inside a rich composed sequence reject with
/// the settled typed payloads and change nothing — one snapshot covers
/// every case, since a rejection records nothing (PRD §10.5, §8.2).
#[test]
fn invalid_requests_inside_a_sequence_reject_without_touching_anything() {
    let mut engine = DomainEngine::new();

    // Build: row [p1, p2(host)]; p2 hosts a top occupant and a floater;
    // p1 is wrapped in a tab group with a joined member.
    let p1 = expect_created(engine.execute_docking(DockingOperation::CreatePanel {
        content: Some("one".into()),
    }));
    let (p2, _target, row) =
        expect_inserted(engine.execute_docking(DockingOperation::InsertBeside {
            source: new_panel("two"),
            target: p1,
            direction: SplitDirection::Right,
        }));
    let (p2_promoted, p2_was_promoted) =
        expect_promoted(engine.execute_docking(DockingOperation::PromoteHost { dock: p2 }));
    assert_eq!(p2_promoted, p2);
    assert!(p2_was_promoted);
    let (_top, _host, _top_slot, _) = expect_embedded(
        engine.execute_docking(DockingOperation::Embed {
            source: new_panel("top dock"),
            host: p2,
            slot: EmbedSlot::Region(EdgeRegion::Top),
        }),
    );
    let (floater, _fhost, _fslot, _) = expect_embedded(
        engine.execute_docking(DockingOperation::Embed {
            source: new_panel("floater"),
            host: p2,
            slot: EmbedSlot::Floating { rect: None },
        }),
    );
    let (_notes, _tab_target, group) =
        expect_tab_joined(engine.execute_docking(DockingOperation::TabJoin {
            source: new_panel("notes"),
            target: p1,
        }));
    let before = engine.docking().clone();
    let events_before = engine.events().len();
    let history_before = engine.history().position();

    // Self-targeting (§10.4): the source would become its own
    // descendant. The target must not be a tab member — insert-beside
    // resolves those targets up the member chain to the group — so
    // self-target the split-child host instead.
    rejection(
        &mut engine,
        DockingOperation::InsertBeside {
            source: InsertSource::Existing(p2),
            target: p2,
            direction: SplitDirection::Left,
        },
        "self-targeting insert",
        DockingError::CycleDetected { id: p2 },
        &before,
    );
    // Embedding into a split target — structural containers have no
    // work area to host or promote (v1).
    rejection(
        &mut engine,
        DockingOperation::Embed {
            source: new_panel("x"),
            host: row,
            slot: EmbedSlot::Region(EdgeRegion::Left),
        },
        "embed into a split",
        DockingError::IneligibleDock { id: row },
        &before,
    );
    // Occupied edge region — the settled reject-by-default policy.
    rejection(
        &mut engine,
        DockingOperation::Embed {
            source: new_panel("y"),
            host: p2,
            slot: EmbedSlot::Region(EdgeRegion::Top),
        },
        "occupied region",
        DockingError::RegionOccupied { host: p2, edge: EdgeRegion::Top },
        &before,
    );
    // A floated dock has no directional layout surface (§8.6 baseline).
    rejection(
        &mut engine,
        DockingOperation::InsertBeside {
            source: new_panel("z"),
            target: floater,
            direction: SplitDirection::Right,
        },
        "floater as directional target",
        DockingError::NotSplittable { id: floater },
        &before,
    );
    // Sizing bound / invariant violation — rejection, never clamping.
    rejection(
        &mut engine,
        DockingOperation::ResizeSiblings {
            container: row,
            first: 0,
            requested: 0.0,
            first_min: None,
            second_min: None,
        },
        "zero-size resize",
        DockingError::SizeBoundViolation { split: row, index: 0 },
        &before,
    );
    // Floating entry index out of range.
    rejection(
        &mut engine,
        DockingOperation::SetFloatingRect {
            host: p2,
            index: 9,
            rect: FloatingRect::default(),
            min_w: None,
            min_h: None,
        },
        "floating index out of range",
        DockingError::FloatingIndexOutOfRange { index: 9, entry_count: 1 },
        &before,
    );
    // Reorder index out of range.
    rejection(
        &mut engine,
        DockingOperation::ReorderTab { group, from: 5, to: 0 },
        "reorder index out of range",
        DockingError::TabIndexOutOfRange { index: 5, member_count: 2 },
        &before,
    );
    // Unknown id.
    rejection(
        &mut engine,
        DockingOperation::Remove { dock: absent(999) },
        "unknown id",
        DockingError::DockNotFound { id: absent(999) },
        &before,
    );

    // A rejection is the whole consequence: no event, no history entry.
    assert_eq!(engine.events().len(), events_before, "rejections record no events (§10.5)");
    assert_eq!(engine.history().position(), history_before);
    assert!(engine.docking().is_valid());

    // The sequence continues cleanly after the rejections: the next
    // valid request commits normally on top of the untouched layout.
    let (late, _lt, _late_group) =
        expect_tab_joined(engine.execute_docking(DockingOperation::TabJoin {
            source: new_panel("late"),
            target: p1,
        }));
    assert_eq!(engine.history().position(), history_before + 1);
    assert_eq!(
        engine.docking().placement(late),
        Some(DockSlot::TabMember { parent: group, index: 2 }),
        "the late member joined the same group"
    );
    assert!(engine.docking().is_valid());
}
