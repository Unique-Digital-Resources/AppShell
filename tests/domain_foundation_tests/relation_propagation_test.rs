//! Tests for cascading propagation: direct vs cascading modes, cycle
//! protection, multi-hop chains, and batch propagation.

use app_shell::domain_foundation::relations::{
    Endpoint, ObjectId, PropertyId, PropertyValue, RelationDefinitionBuilder,
    RelationRegistry,
};
use app_shell::domain_foundation::relations::definition::{
    EffectContext, EffectOutcome, PropagationMode,
};
use app_shell::domain_foundation::relations::evaluation::EvaluationContext;
use std::sync::Arc;

fn prop(obj: u64, p: u32) -> PropertyId {
    PropertyId::new(ObjectId::new(obj), p)
}

#[test]
fn direct_propagation_does_not_cascade() {
    let mut registry = RelationRegistry::new();

    // A.p0 → B.p0 (Direct)
    let src = Endpoint::source(prop(1, 0));
    let tgt = Endpoint::target(prop(2, 0));
    registry.register(
        RelationDefinitionBuilder::new(src, tgt)
            .propagation(PropagationMode::Direct)
            .effect(Arc::new(|ctx: &EffectContext| {
                EffectOutcome::Apply(ctx.source_value.clone())
            }))
            .build(),
    );

    // B.p0 → C.p0 (would cascade if propagation were Cascading)
    let src2 = Endpoint::source(prop(2, 0));
    let tgt2 = Endpoint::target(prop(3, 0));
    registry.register(
        RelationDefinitionBuilder::new(src2, tgt2)
            .propagation(PropagationMode::Direct)
            .effect(Arc::new(|ctx: &EffectContext| {
                EffectOutcome::Apply(ctx.source_value.clone())
            }))
            .build(),
    );

    let mut ctx = EvaluationContext::new(&registry);
    ctx.set_value(prop(1, 0), PropertyValue::Int(10));
    ctx.set_value(prop(2, 0), PropertyValue::Int(0));
    ctx.set_value(prop(3, 0), PropertyValue::Int(0));

    let steps = ctx.propagate_from(prop(1, 0)).unwrap();

    // Only the first relation fires; the second is not re-evaluated
    // because the first relation's propagation mode is Direct.
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].target, prop(2, 0));
    assert_eq!(ctx.get_value(prop(2, 0)), &PropertyValue::Int(10));
    assert_eq!(ctx.get_value(prop(3, 0)), &PropertyValue::Int(0));
}

#[test]
fn cascading_propagation_chains_through_targets() {
    let mut registry = RelationRegistry::new();

    // A.p0 → B.p0 (Cascading)
    let src = Endpoint::source(prop(1, 0));
    let tgt = Endpoint::target(prop(2, 0));
    registry.register(
        RelationDefinitionBuilder::new(src, tgt)
            .propagation(PropagationMode::Cascading)
            .effect(Arc::new(|ctx: &EffectContext| {
                EffectOutcome::Apply(ctx.source_value.clone())
            }))
            .build(),
    );

    // B.p0 → C.p0 (Cascading)
    let src2 = Endpoint::source(prop(2, 0));
    let tgt2 = Endpoint::target(prop(3, 0));
    registry.register(
        RelationDefinitionBuilder::new(src2, tgt2)
            .propagation(PropagationMode::Cascading)
            .effect(Arc::new(|ctx: &EffectContext| {
                EffectOutcome::Apply(PropertyValue::Int(
                    if let PropertyValue::Int(v) = ctx.source_value {
                        *v + 1
                    } else {
                        0
                    },
                ))
            }))
            .build(),
    );

    let mut ctx = EvaluationContext::new(&registry);
    ctx.set_value(prop(1, 0), PropertyValue::Int(10));
    ctx.set_value(prop(2, 0), PropertyValue::Int(0));
    ctx.set_value(prop(3, 0), PropertyValue::Int(0));

    let steps = ctx.propagate_from(prop(1, 0)).unwrap();

    // Both relations fire: A→B, then B→C.
    assert_eq!(steps.len(), 2);
    assert_eq!(steps[0].target, prop(2, 0));
    assert_eq!(steps[1].target, prop(3, 0));
    assert_eq!(ctx.get_value(prop(2, 0)), &PropertyValue::Int(10));
    assert_eq!(ctx.get_value(prop(3, 0)), &PropertyValue::Int(11));
}

#[test]
fn three_hop_cascading_chain() {
    let mut registry = RelationRegistry::new();

    // A.p0 → B.p0 → C.p0 → D.p0, all Cascading, each adds 1.
    for (src_obj, tgt_obj) in [(1, 2), (2, 3), (3, 4)] {
        let src = Endpoint::source(prop(src_obj, 0));
        let tgt = Endpoint::target(prop(tgt_obj, 0));
        registry.register(
            RelationDefinitionBuilder::new(src, tgt)
                .propagation(PropagationMode::Cascading)
                .effect(Arc::new(|ctx: &EffectContext| {
                    EffectOutcome::Apply(PropertyValue::Int(
                        if let PropertyValue::Int(v) = ctx.source_value {
                            *v + 1
                        } else {
                            0
                        },
                    ))
                }))
                .build(),
        );
    }

    let mut ctx = EvaluationContext::new(&registry);
    ctx.set_value(prop(1, 0), PropertyValue::Int(100));
    ctx.set_value(prop(2, 0), PropertyValue::Int(0));
    ctx.set_value(prop(3, 0), PropertyValue::Int(0));
    ctx.set_value(prop(4, 0), PropertyValue::Int(0));

    let steps = ctx.propagate_from(prop(1, 0)).unwrap();

    assert_eq!(steps.len(), 3);
    assert_eq!(ctx.get_value(prop(2, 0)), &PropertyValue::Int(101));
    assert_eq!(ctx.get_value(prop(3, 0)), &PropertyValue::Int(102));
    assert_eq!(ctx.get_value(prop(4, 0)), &PropertyValue::Int(103));
}

#[test]
fn cycle_protection_prevents_infinite_loop() {
    let mut registry = RelationRegistry::new();

    // A.p0 → B.p0 → A.p0 (cycle!)
    let src1 = Endpoint::source(prop(1, 0));
    let tgt1 = Endpoint::target(prop(2, 0));
    registry.register(
        RelationDefinitionBuilder::new(src1, tgt1)
            .propagation(PropagationMode::Cascading)
            .effect(Arc::new(|ctx: &EffectContext| {
                EffectOutcome::Apply(ctx.source_value.clone())
            }))
            .build(),
    );

    let src2 = Endpoint::source(prop(2, 0));
    let tgt2 = Endpoint::target(prop(1, 0));
    registry.register(
        RelationDefinitionBuilder::new(src2, tgt2)
            .propagation(PropagationMode::Cascading)
            .effect(Arc::new(|ctx: &EffectContext| {
                EffectOutcome::Apply(ctx.source_value.clone())
            }))
            .build(),
    );

    let mut ctx = EvaluationContext::new(&registry);
    ctx.set_value(prop(1, 0), PropertyValue::Int(7));
    ctx.set_value(prop(2, 0), PropertyValue::Int(0));

    // Should not hang. Each relation fires once; the cycle is broken by
    // the propagator's visited set.
    let steps = ctx.propagate_from(prop(1, 0)).unwrap();

    // Both relations fire once each before the cycle is detected.
    assert_eq!(steps.len(), 2);
}

#[test]
fn mixed_direct_and_cascading_in_same_graph() {
    let mut registry = RelationRegistry::new();

    // A.p0 → B.p0 (Cascading)
    // B.p0 → C.p0 (Direct)
    // C.p0 → D.p0 (Cascading)
    let configs = [
        (1u64, 0u32, 2u64, 0u32, PropagationMode::Cascading),
        (2, 0, 3, 0, PropagationMode::Direct),
        (3, 0, 4, 0, PropagationMode::Cascading),
    ];

    for (src_obj, src_p, tgt_obj, tgt_p, mode) in configs {
        let src = Endpoint::source(prop(src_obj, src_p));
        let tgt = Endpoint::target(prop(tgt_obj, tgt_p));
        registry.register(
            RelationDefinitionBuilder::new(src, tgt)
                .propagation(mode)
                .effect(Arc::new(|ctx: &EffectContext| {
                    EffectOutcome::Apply(ctx.source_value.clone())
                }))
                .build(),
        );
    }

    let mut ctx = EvaluationContext::new(&registry);
    ctx.set_value(prop(1, 0), PropertyValue::Int(50));
    for obj in [2, 3, 4] {
        ctx.set_value(prop(obj, 0), PropertyValue::Int(0));
    }

    let steps = ctx.propagate_from(prop(1, 0)).unwrap();

    // A→B fires (Cascading), B→C fires (Direct, no cascade from C),
    // C→D does NOT fire because B→C was Direct.
    assert_eq!(steps.len(), 2);
    assert_eq!(steps[0].target, prop(2, 0));
    assert_eq!(steps[1].target, prop(3, 0));
    assert_eq!(ctx.get_value(prop(2, 0)), &PropertyValue::Int(50));
    assert_eq!(ctx.get_value(prop(3, 0)), &PropertyValue::Int(50));
    assert_eq!(ctx.get_value(prop(4, 0)), &PropertyValue::Int(0));
}

#[test]
fn invalidate_outcome_does_not_change_value_but_cascades() {
    let mut registry = RelationRegistry::new();

    // A.p0 → B.p0 (Cascading, Invalidate)
    let src = Endpoint::source(prop(1, 0));
    let tgt = Endpoint::target(prop(2, 0));
    registry.register(
        RelationDefinitionBuilder::new(src, tgt)
            .propagation(PropagationMode::Cascading)
            .effect(Arc::new(|_| EffectOutcome::Invalidate))
            .build(),
    );

    // B.p0 → C.p0 (Cascading, would fire if B.p0 changes or is invalidated)
    let src2 = Endpoint::source(prop(2, 0));
    let tgt2 = Endpoint::target(prop(3, 0));
    registry.register(
        RelationDefinitionBuilder::new(src2, tgt2)
            .propagation(PropagationMode::Cascading)
            .effect(Arc::new(|ctx: &EffectContext| {
                EffectOutcome::Apply(ctx.source_value.clone())
            }))
            .build(),
    );

    let mut ctx = EvaluationContext::new(&registry);
    ctx.set_value(prop(1, 0), PropertyValue::Int(1));
    ctx.set_value(prop(2, 0), PropertyValue::Int(99));
    ctx.set_value(prop(3, 0), PropertyValue::Int(0));

    let steps = ctx.propagate_from(prop(1, 0)).unwrap();

    // First relation invalidates B.p0; second relation cascades from
    // B.p0 and applies its value to C.p0.
    assert_eq!(steps.len(), 2);
    assert_eq!(steps[0].outcome, EffectOutcome::Invalidate);
    assert_eq!(ctx.get_value(prop(2, 0)), &PropertyValue::Int(99)); // unchanged
    assert_eq!(ctx.get_value(prop(3, 0)), &PropertyValue::Int(99)); // copied from B
}

// --- E3: Batch propagation tests ---

#[test]
fn batch_propagation_accumulates_and_commits() {
    let mut registry = RelationRegistry::new();

    // A.p0 → B.p0 (copy source value)
    let src1 = Endpoint::source(prop(1, 0));
    let tgt1 = Endpoint::target(prop(2, 0));
    registry.register(
        RelationDefinitionBuilder::new(src1, tgt1)
            .propagation(PropagationMode::Direct)
            .effect(Arc::new(|ctx: &EffectContext| {
                EffectOutcome::Apply(ctx.source_value.clone())
            }))
            .build(),
    );

    // A.p1 → C.p0 (copy source value)
    let src2 = Endpoint::source(prop(1, 1));
    let tgt2 = Endpoint::target(prop(3, 0));
    registry.register(
        RelationDefinitionBuilder::new(src2, tgt2)
            .propagation(PropagationMode::Direct)
            .effect(Arc::new(|ctx: &EffectContext| {
                EffectOutcome::Apply(ctx.source_value.clone())
            }))
            .build(),
    );

    let mut ctx = EvaluationContext::new(&registry);

    // Begin batch, set two values, commit — both should propagate.
    ctx.begin_batch();
    ctx.set_value(prop(1, 0), PropertyValue::Int(42));
    ctx.set_value(prop(1, 1), PropertyValue::Text("hello".into()));
    assert!(ctx.is_in_batch());

    let steps = ctx.commit_batch().unwrap();
    assert!(!ctx.is_in_batch());
    assert_eq!(steps.len(), 2); // Two propagation chains.

    // Verify results.
    assert_eq!(
        ctx.get_value(prop(2, 0)),
        &PropertyValue::Int(42),
    );
    assert_eq!(
        ctx.get_value(prop(3, 0)),
        &PropertyValue::Text("hello".into()),
    );
}

#[test]
fn batch_rollback_discards_changes() {
    let mut registry = RelationRegistry::new();

    let src = Endpoint::source(prop(1, 0));
    let tgt = Endpoint::target(prop(2, 0));
    registry.register(
        RelationDefinitionBuilder::new(src, tgt)
            .propagation(PropagationMode::Direct)
            .effect(Arc::new(|ctx: &EffectContext| {
                EffectOutcome::Apply(ctx.source_value.clone())
            }))
            .build(),
    );

    let mut ctx = EvaluationContext::new(&registry);
    ctx.begin_batch();
    ctx.set_value(prop(1, 0), PropertyValue::Int(99));
    assert!(ctx.is_in_batch());

    ctx.rollback_batch();
    assert!(!ctx.is_in_batch());

    // Target should not have been updated.
    assert_eq!(
        ctx.get_value(prop(2, 0)),
        &PropertyValue::None,
    );
}