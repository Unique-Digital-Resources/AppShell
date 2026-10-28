//! Integration tests: docking persistence and restoration through the
//! Domain Engine (Phase 10).
//!
//! The docking layout's own serde form is the persisted payload (the
//! established serialization convention — every sibling environment ships
//! derive-only serde, and the host application serializes the cloned
//! model with its chosen container; `ContentRef`, the dock ids, and the
//! id counter ride inside the form). The engine's job is the
//! **trusted-adopt path**: [`DomainEngine::restore_docking`] validates the
//! submitted model with the Phase-3 gate before accepting it — rejection,
//! never repair — records a non-undoable `"docking.restored"` entry,
//! marks the engine saved, and publishes `"docking.restored"`.
//!
//! The docking serde derives are feature-gated, so this file is gated on
//! `serde` (the `serde_roundtrip_test.rs` precedent in the sibling
//! suite). The Phase-10 history-registry area — interleaved canvas +
//! docking replay (no serde dependency) — lives in
//! `docking_history_registry_test.rs`. Public crate API only
//! (agents/testing.md).

use app_shell::app_engine::{MemoryStorageBackend, StorageBackend};
use app_shell::domain_engine::{DomainEngine, DomainError};
use app_shell::domain_foundation::environments::docking::{
    DockId, DockingError, DockingOperation, DockingOutcome, DockLayout, EdgeRegion, EmbedSlot,
    FloatingRect, InsertSource, SplitDirection,
};

// --- Fixtures ---------------------------------------------------------------

fn expect_created(outcome: DockingOutcome) -> DockId {
    match outcome {
        DockingOutcome::Created { dock, .. } => dock,
        other => panic!("expected Created, got {other:?}"),
    }
}

fn expect_inserted(outcome: DockingOutcome) -> (DockId, DockId) {
    match outcome {
        DockingOutcome::InsertedBeside { source, container, .. } => (source, container),
        other => panic!("expected InsertedBeside, got {other:?}"),
    }
}

fn expect_embedded(outcome: DockingOutcome) -> DockId {
    match outcome {
        DockingOutcome::Embedded { source, .. } => source,
        other => panic!("expected Embedded, got {other:?}"),
    }
}

/// Panel root + a second panel beside it in a fresh split wrap.
/// Returns `(root_panel, inserted_panel, wrap_container)`.
fn panel_and_beside(engine: &mut DomainEngine) -> (DockId, DockId, DockId) {
    let p1 = expect_created(
        engine
            .execute_docking(DockingOperation::CreatePanel { content: Some("one".into()) })
            .unwrap(),
    );
    let (p2, s) = expect_inserted(
        engine
            .execute_docking(DockingOperation::InsertBeside {
                source: InsertSource::NewPanel { content: Some("two".into()) },
                target: p1,
                direction: SplitDirection::Right,
            })
            .unwrap(),
    );
    (p1, p2, s)
}

/// The in-crate `structural_equal` comparison, replicated on the public
/// API: docks (id-keyed, order-normalized) plus the root reconstruct
/// exactly. The id counter is deliberately **not** compared — it is
/// monotone by design (never reset; undo of a created dock leaves it
/// incremented, the `Canvas::insert_object` convention).
fn structural_equal(a: &DockLayout, b: &DockLayout) -> bool {
    let snapshot = |layout: &DockLayout| {
        let mut docks: Vec<_> = layout.iter().cloned().collect();
        docks.sort_by_key(|dock| dock.id);
        (layout.root(), docks)
    };
    snapshot(a) == snapshot(b)
}

// --- Restoring a saved layout ----------------------------------------------

/// Session A builds a rich layout (split with non-uniform weights, a
/// temporary auto-promoted host with a set region weight, a floating
/// entry with a set rect) and serializes the cloned model; a fresh
/// engine — the simulated restart — restores the parsed form. Restored
/// state is equivalent, valid, and id-preserving; the adopt is recorded
/// non-undoable and the engine is marked saved; post-restore mutations
/// undo/redo cleanly.
#[test]
fn restore_into_a_fresh_engine_is_equivalent_and_id_preserving() {
    // Session A: build and save.
    let mut engine = DomainEngine::new();
    let p1 = expect_created(
        engine
            .execute_docking(DockingOperation::CreatePanel { content: Some("main".into()) })
            .unwrap(),
    );
    let (_p2, s) = expect_inserted(
        engine
            .execute_docking(DockingOperation::InsertBeside {
                source: InsertSource::NewPanel { content: Some("two".into()) },
                target: p1,
                direction: SplitDirection::Right,
            })
            .unwrap(),
    );
    engine
        .execute_docking(DockingOperation::ResizeSiblings {
            container: s,
            first: 0,
            requested: 1.5,
            first_min: None,
            second_min: None,
        })
        .unwrap();
    // Embedding into the plain panel auto-promotes it to a temporary
    // host; a second embed into the same host uses the floating slot.
    let p3 = expect_embedded(
        engine
            .execute_docking(DockingOperation::Embed {
                source: InsertSource::NewPanel { content: Some("edge".into()) },
                host: p1,
                slot: EmbedSlot::Region(EdgeRegion::Left),
            })
            .unwrap(),
    );
    let _p4 = expect_embedded(
        engine
            .execute_docking(DockingOperation::Embed {
                source: InsertSource::NewPanel { content: Some("widget".into()) },
                host: p1,
                slot: EmbedSlot::Floating { rect: None },
            })
            .unwrap(),
    );
    engine
        .execute_docking(DockingOperation::ResizeRegion {
            host: p1,
            edge: EdgeRegion::Left,
            requested: 2.0,
            min: None,
        })
        .unwrap();
    let rect = FloatingRect::new(0.25, 0.25, 0.5, 0.5).unwrap();
    engine
        .execute_docking(DockingOperation::SetFloatingRect {
            host: p1,
            index: 0,
            rect,
            min_w: None,
            min_h: None,
        })
        .unwrap();

    let saved = engine.docking().clone();
    let json = serde_json::to_string(&saved).unwrap();

    // Simulated restart: a fresh engine knows only the persisted form.
    let mut restored = DomainEngine::new();
    let payload: DockLayout = serde_json::from_str(&json).unwrap();
    restored.restore_docking(payload).unwrap();

    // Id-exact adopt: the whole form, id counter included, rides.
    assert_eq!(restored.docking(), &saved, "restore adopts the form verbatim");
    assert!(restored.docking().validate().is_ok());

    // The adopt is recorded as a non-undoable `docking.restored` entry,
    // marked saved, and published.
    assert!(!restored.is_dirty(), "restored state is marked saved");
    assert_eq!(restored.history().list().len(), 1);
    let (_entry_id, entry_name, entry_undoable) = restored.history().list()[0].clone();
    assert_eq!(entry_name, "docking.restored");
    assert!(!entry_undoable, "the adopt is a non-undoable import");
    assert_eq!(restored.events().len(), 1);
    assert_eq!(restored.events()[0].event_type(), "docking.restored");

    // Persistent-state spot checks across the settled model surfaces.
    assert_eq!(restored.docking().root(), Some(s));
    assert!(matches!(
        restored.docking().get(p1).unwrap().role,
        app_shell::domain_foundation::environments::docking::DockRole::EmbedHost { .. }
    ));
    assert!(matches!(
        restored.docking().placement(p3),
        Some(app_shell::domain_foundation::environments::docking::DockSlot::HostEdge {
            host,
            edge
        }) if host == p1 && edge == EdgeRegion::Left
    ));
    assert!(matches!(
        restored.docking().get(p1).unwrap().role,
        app_shell::domain_foundation::environments::docking::DockRole::EmbedHost { ref regions, .. }
            if regions.weight_on(EdgeRegion::Left) == Some(2.0)
    ));
    let floating = restored.docking().get(p1).unwrap().floating().unwrap();
    assert_eq!(floating[0].rect(), rect, "the floating rect rides verbatim");

    // Post-restore mutations undo/redo cleanly against the adopt.
    let (_late_panel, _late_wrap) = expect_inserted(
        restored
            .execute_docking(DockingOperation::InsertBeside {
                source: InsertSource::NewPanel { content: Some("late".into()) },
                target: p3,
                direction: SplitDirection::Right,
            })
            .unwrap(),
    );
    assert_eq!(restored.undo().unwrap(), 1);
    assert!(
        structural_equal(restored.docking(), &saved),
        "undo reaches exactly the restored adopt (id counter monotone)"
    );
    assert_eq!(restored.redo().unwrap(), 1);
    assert_eq!(restored.docking().root(), Some(s));
}

// --- Rejection on invalid persisted state ----------------------------------

/// Well-formed JSON that violates an invariant is declined wholesale: the
/// first deterministic Phase-3 error surfaces (`UnknownReference`), and
/// the engine's layout, history, and events are untouched (PRD §12.3 —
/// rejection with a meaningful error, never silent repair).
#[test]
fn restore_rejects_invalid_layouts_and_keeps_state_untouched() {
    let mut engine = DomainEngine::new();
    let (p1, _p2, _s) = panel_and_beside(&mut engine);
    let events_before = engine.events().len();
    let history_before = engine.history().position();

    // The model-level Phase-3 garbage fixture (dangling split-child
    // reference), now through the engine gate.
    let json = r#"{
        "docks": {
            "5": { "id": 5, "role": { "Split": { "orientation": "Row",
                    "children": [5, 9], "sizes": [1.0, 1.0] } } }
        },
        "root": 5,
        "next_id": 10
    }"#;
    let payload: DockLayout = serde_json::from_str(json).unwrap();

    let err = engine.restore_docking(payload).unwrap_err();
    assert!(matches!(
        err,
        DomainError::Docking(DockingError::UnknownReference { id }) if id == DockId::from_raw(9)
    ));

    assert_eq!(engine.events().len(), events_before);
    assert_eq!(engine.history().position(), history_before);
    assert!(engine.docking().contains(p1));
    assert_eq!(engine.docking().all_docks().len(), 3);
}

/// Malformed persisted state is rejected in two layers: shapes outside
/// the form fail at the caller-side deserializer (the documented
/// save/restore flow is serialize-with-serde → parse-with-serde →
/// `restore_docking`, the established derive-only serialization
/// convention), and designs inside the form (a root naming no dock —
/// mechanical JSON parses it) fail at the engine's Phase-3 gate with the
/// first deterministic error. The explicit empty layout is always
/// restorable (the valid empty representation, PRD §7.4).
#[test]
fn malformed_persisted_state_is_rejected_in_two_layers() {
    // Layer 1, caller-side: a dock entry that is not a dock cannot parse.
    let json = r#"{ "docks": { "5": "not-a-dock" }, "root": 5, "next_id": 6 }"#;
    assert!(serde_json::from_str::<DockLayout>(json).is_err());

    // Layer 2, engine gate: a root naming no dock parses mechanically
    // but cannot be trusted.
    let json = r#"{ "docks": { }, "root": 3, "next_id": 1 }"#;
    let payload: DockLayout = serde_json::from_str(json).unwrap();

    let mut engine = DomainEngine::new();
    let (p1, _p2, _s) = panel_and_beside(&mut engine);
    let events_before = engine.events().len();
    let err = engine.restore_docking(payload).unwrap_err();
    assert!(matches!(
        err,
        DomainError::Docking(DockingError::UnknownReference { id }) if id == DockId::from_raw(3)
    ));
    assert_eq!(engine.events().len(), events_before, "rejection publishes nothing");
    assert!(engine.docking().contains(p1));

    // The explicit empty layout restores (the valid empty representation).
    engine.restore_docking(DockLayout::new()).unwrap();
    assert!(engine.docking().is_empty());
}

// --- The persistence convention --------------------------------------------

/// The saved form lives in the established app-engine storage
/// conventions — a [`StorageBackend`] key-value entry — with no
/// docking-specific persistence framework (plan completion criterion).
#[test]
fn saved_layout_round_trips_through_a_storage_backend() {
    let mut engine = DomainEngine::new();
    let (p1, _p2, s) = panel_and_beside(&mut engine);
    let json = serde_json::to_string(engine.docking()).unwrap();

    let backend = MemoryStorageBackend::new();
    backend.save("docking.layout", &json).unwrap();
    assert!(backend.exists("docking.layout"));

    let mut restored = DomainEngine::new();
    let payload: DockLayout =
        serde_json::from_str(&backend.load("docking.layout").unwrap().unwrap()).unwrap();
    restored.restore_docking(payload).unwrap();

    assert_eq!(restored.docking().root(), Some(s));
    assert!(restored.docking().contains(p1));
    assert_eq!(restored.docking(), engine.docking());
    assert!(restored.docking().validate().is_ok());
    assert!(!restored.is_dirty());
}
