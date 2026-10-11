//! Tests for the evaluation engine: single-relation evaluation, source
//! mismatch detection, and value store management.

use app_shell::domain_foundation::relations::{
    Endpoint, ObjectId, PropertyId, PropertyValue, RelationDefinitionBuilder,
    RelationRegistry,
};
use app_shell::domain_foundation::relations::definition::{
    EffectContext, EffectOutcome, PropagationMode,
};
use app_shell::domain_foundation::relations::evaluation::{EvaluationContext, ValueStore};
use std::sync::Arc;

#[test]
fn evaluate_one_applies_effect_to_target() {
    let mut registry = RelationRegistry::new();
    let src = Endpoint::source(PropertyId::new(ObjectId::new(1), 0));
    let tgt = Endpoint::target(PropertyId::new(ObjectId::new(2), 0));

    // Effect: copy source value to target.
    let def = RelationDefinitionBuilder::new(src, tgt)
        .effect(Arc::new(|ctx: &EffectContext| {
            EffectOutcome::Apply(ctx.source_value.clone())
        }))
        .build();
    let _id = registry.register(def);

    let mut ctx = EvaluationContext::new(&registry);
    ctx.set_value(src.property, PropertyValue::Int(42));
    ctx.set_value(tgt.property, PropertyValue::Int(0));

    let steps = ctx.propagate_from(src.property).unwrap();
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].target, tgt.property);
    match &steps[0].outcome {
        EffectOutcome::Apply(v) => assert_eq!(*v, PropertyValue::Int(42)),
        other => panic!("expected Apply, got {other:?}"),
    }

    // Value store should now hold the updated value.
    assert_eq!(ctx.get_value(tgt.property), &PropertyValue::Int(42));
}

#[test]
fn evaluate_one_no_change_does_not_modify_target() {
    let mut registry = RelationRegistry::new();
    let src = Endpoint::source(PropertyId::new(ObjectId::new(1), 0));
    let tgt = Endpoint::target(PropertyId::new(ObjectId::new(2), 0));

    let def = RelationDefinitionBuilder::new(src, tgt)
        .effect(Arc::new(|_| EffectOutcome::NoChange))
        .build();
    registry.register(def);

    let mut ctx = EvaluationContext::new(&registry);
    ctx.set_value(src.property, PropertyValue::Int(42));
    ctx.set_value(tgt.property, PropertyValue::Int(99));

    let steps = ctx.propagate_from(src.property).unwrap();
    assert!(steps.is_empty());
    assert_eq!(ctx.get_value(tgt.property), &PropertyValue::Int(99));
}

#[test]
fn evaluate_one_invalidate_marks_target_without_changing_value() {
    let mut registry = RelationRegistry::new();
    let src = Endpoint::source(PropertyId::new(ObjectId::new(1), 0));
    let tgt = Endpoint::target(PropertyId::new(ObjectId::new(2), 0));

    let def = RelationDefinitionBuilder::new(src, tgt)
        .effect(Arc::new(|_| EffectOutcome::Invalidate))
        .build();
    registry.register(def);

    let mut ctx = EvaluationContext::new(&registry);
    ctx.set_value(src.property, PropertyValue::Int(1));
    ctx.set_value(tgt.property, PropertyValue::Int(99));

    let steps = ctx.propagate_from(src.property).unwrap();
    // Invalidate produces a step (the propagator records it), but the
    // value store is not modified.
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].outcome, EffectOutcome::Invalidate);
    assert_eq!(ctx.get_value(tgt.property), &PropertyValue::Int(99));
}

#[test]
fn value_store_get_missing_returns_none() {
    let store = ValueStore::new();
    let p = PropertyId::new(ObjectId::new(1), 0);
    assert_eq!(store.get(p), &PropertyValue::None);
}

#[test]
fn value_store_set_and_get() {
    let mut store = ValueStore::new();
    let p = PropertyId::new(ObjectId::new(1), 0);
    store.set(p, PropertyValue::Text("hello".to_string()));
    assert_eq!(store.get(p), &PropertyValue::Text("hello".to_string()));
    assert!(store.contains(p));
}

#[test]
fn value_store_apply_outcome_updates_value() {
    let mut store = ValueStore::new();
    let p = PropertyId::new(ObjectId::new(1), 0);
    store.set(p, PropertyValue::Int(0));
    store.apply_outcome(p, &EffectOutcome::Apply(PropertyValue::Int(55)));
    assert_eq!(store.get(p), &PropertyValue::Int(55));
}

#[test]
fn value_store_apply_no_change_does_not_modify() {
    let mut store = ValueStore::new();
    let p = PropertyId::new(ObjectId::new(1), 0);
    store.set(p, PropertyValue::Int(7));
    store.apply_outcome(p, &EffectOutcome::NoChange);
    assert_eq!(store.get(p), &PropertyValue::Int(7));
}

#[test]
fn multiple_relations_same_source_all_evaluate() {
    let mut registry = RelationRegistry::new();
    let src = Endpoint::source(PropertyId::new(ObjectId::new(1), 0));

    let tgt_a = Endpoint::target(PropertyId::new(ObjectId::new(2), 0));
    let tgt_b = Endpoint::target(PropertyId::new(ObjectId::new(3), 0));

    registry.register(
        RelationDefinitionBuilder::new(src, tgt_a)
            .effect(Arc::new(|ctx: &EffectContext| {
                EffectOutcome::Apply(ctx.source_value.clone())
            }))
            .build(),
    );
    registry.register(
        RelationDefinitionBuilder::new(src, tgt_b)
            .effect(Arc::new(|ctx: &EffectContext| {
                EffectOutcome::Apply(PropertyValue::Int(
                    if let PropertyValue::Int(v) = ctx.source_value {
                        *v * 10
                    } else {
                        0
                    },
                ))
            }))
            .build(),
    );

    let mut ctx = EvaluationContext::new(&registry);
    ctx.set_value(src.property, PropertyValue::Int(5));
    ctx.set_value(tgt_a.property, PropertyValue::Int(0));
    ctx.set_value(tgt_b.property, PropertyValue::Int(0));

    let steps = ctx.propagate_from(src.property).unwrap();
    assert_eq!(steps.len(), 2);

    assert_eq!(ctx.get_value(tgt_a.property), &PropertyValue::Int(5));
    assert_eq!(ctx.get_value(tgt_b.property), &PropertyValue::Int(50));
}

#[test]
fn value_store_management() {
    let mut store = ValueStore::new();
    let p = PropertyId::new(ObjectId::new(1), 0);
    
    assert!(store.is_empty());
    assert_eq!(store.len(), 0);
    
    store.set(p, PropertyValue::Int(42));
    assert!(!store.is_empty());
    assert_eq!(store.len(), 1);
    
    store.clear();
    assert!(store.is_empty());
    assert_eq!(store.len(), 0);
}