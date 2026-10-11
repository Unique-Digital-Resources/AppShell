//! Tests for relation definitions, endpoints, and the builder.

use app_shell::domain_foundation::relations::{
    Endpoint, ObjectId, PropertyId, PropertyValue, RelationDefinitionBuilder,
};
use app_shell::domain_foundation::relations::definition::{
    EffectContext, EffectOutcome, PropagationMode, Trigger,
};
use std::sync::Arc;

fn make_endpoints() -> (Endpoint, Endpoint) {
    let src = Endpoint::source(PropertyId::new(ObjectId::new(1), 0));
    let tgt = Endpoint::target(PropertyId::new(ObjectId::new(2), 0));
    (src, tgt)
}

#[test]
fn endpoint_source_and_target_roles_are_distinct() {
    let p = PropertyId::new(ObjectId::new(1), 0);
    let src = Endpoint::source(p);
    let tgt = Endpoint::target(p);
    assert!(src.is_source());
    assert!(!src.is_target());
    assert!(tgt.is_target());
    assert!(!tgt.is_source());
    assert_ne!(src, tgt);
}

#[test]
fn property_id_equality_is_value_based() {
    let a = PropertyId::new(ObjectId::new(1), 0);
    let b = PropertyId::new(ObjectId::new(1), 0);
    let c = PropertyId::new(ObjectId::new(1), 1);
    let d = PropertyId::new(ObjectId::new(2), 0);
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_ne!(a, d);
}

#[test]
fn builder_produces_definition_with_defaults() {
    let (src, tgt) = make_endpoints();
    let def = RelationDefinitionBuilder::new(src, tgt).build();
    assert_eq!(def.source, src);
    assert_eq!(def.target, tgt);
    assert_eq!(def.trigger, Trigger::OnChange);
    assert_eq!(def.propagation, PropagationMode::Direct);
}

#[test]
fn builder_accepts_custom_trigger_and_propagation() {
    let (src, tgt) = make_endpoints();
    let def = RelationDefinitionBuilder::new(src, tgt)
        .trigger(Trigger::OnInvalidation)
        .propagation(PropagationMode::Cascading)
        .build();
    assert_eq!(def.trigger, Trigger::OnInvalidation);
    assert_eq!(def.propagation, PropagationMode::Cascading);
}

#[test]
fn effect_closure_is_called_with_context() {
    let (src, tgt) = make_endpoints();
    let def = RelationDefinitionBuilder::new(src, tgt)
        .effect(Arc::new(|ctx: &EffectContext| {
            // Copy the source value to the target.
            EffectOutcome::Apply(ctx.source_value.clone())
        }))
        .build();

    let source_val = PropertyValue::Int(42);
    let target_val = PropertyValue::Int(0);

    let ctx = EffectContext {
        relation_id: app_shell::domain_foundation::relations::RelationId::from_raw(0),
        source: &src.property,
        target: &tgt.property,
        source_value: &source_val,
        target_value: &target_val,
    };

    let outcome = (def.effect)(&ctx);
    match outcome {
        EffectOutcome::Apply(v) => assert_eq!(v, PropertyValue::Int(42)),
        other => panic!("expected Apply, got {other:?}"),
    }
}

#[test]
fn default_effect_is_no_change() {
    let (src, tgt) = make_endpoints();
    let def = RelationDefinitionBuilder::new(src, tgt).build();

    let source_val = PropertyValue::Int(99);
    let target_val = PropertyValue::Int(0);

    let ctx = EffectContext {
        relation_id: app_shell::domain_foundation::relations::RelationId::from_raw(0),
        source: &src.property,
        target: &tgt.property,
        source_value: &source_val,
        target_value: &target_val,
    };

    let outcome = (def.effect)(&ctx);
    assert_eq!(outcome, EffectOutcome::NoChange);
}

#[test]
fn invalidate_outcome_is_distinct_from_no_change() {
    let (src, tgt) = make_endpoints();
    let def = RelationDefinitionBuilder::new(src, tgt)
        .effect(Arc::new(|_| EffectOutcome::Invalidate))
        .build();

    let source_val = PropertyValue::Int(1);
    let target_val = PropertyValue::Int(2);

    let ctx = EffectContext {
        relation_id: app_shell::domain_foundation::relations::RelationId::from_raw(0),
        source: &src.property,
        target: &tgt.property,
        source_value: &source_val,
        target_value: &target_val,
    };

    let outcome = (def.effect)(&ctx);
    assert_eq!(outcome, EffectOutcome::Invalidate);
    assert_ne!(outcome, EffectOutcome::NoChange);
}