//! Tests for the relation registry: registration, lookup, unregistration.

use app_shell::domain_foundation::relations::{
    Endpoint, ObjectId, PropertyId, RelationDefinition, RelationDefinitionBuilder, RelationId,
    RelationRegistry,
};

use app_shell::domain_foundation::relations::definition::{
    EffectOutcome, PropagationMode, Trigger,
};
use std::sync::Arc;

fn make_definition(
    src_obj: u64,
    src_prop: u32,
    tgt_obj: u64,
    tgt_prop: u32,
) -> RelationDefinition {
    let src = Endpoint::source(PropertyId::new(ObjectId::new(src_obj), src_prop));
    let tgt = Endpoint::target(PropertyId::new(ObjectId::new(tgt_obj), tgt_prop));
    RelationDefinitionBuilder::new(src, tgt)
        .trigger(Trigger::OnChange)
        .propagation(PropagationMode::Direct)
        .effect(Arc::new(|_| EffectOutcome::NoChange))
        .build()
}

#[test]
fn new_registry_is_empty() {
    let registry = RelationRegistry::new();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
}

#[test]
fn register_returns_unique_ids() {
    let mut registry = RelationRegistry::new();
    let a = registry.register(make_definition(1, 0, 2, 0));
    let b = registry.register(make_definition(1, 1, 2, 1));
    let c = registry.register(make_definition(3, 0, 4, 0));
    assert_ne!(a, b);
    assert_ne!(b, c);
    assert_ne!(a, c);
    assert_eq!(registry.len(), 3);
}

#[test]
fn get_definition_returns_registered_definition() {
    let mut registry = RelationRegistry::new();
    let id = registry.register(make_definition(1, 0, 2, 0));
    let def = registry.get_definition(id).expect("definition must exist");
    assert_eq!(def.source.property, PropertyId::new(ObjectId::new(1), 0));
    assert_eq!(def.target.property, PropertyId::new(ObjectId::new(2), 0));
}

#[test]
fn get_definition_for_unknown_id_returns_none() {
    let registry = RelationRegistry::new();
    let unknown = app_shell::domain_foundation::relations::RelationId::from_raw(999);
    assert!(registry.get_definition(unknown).is_none());
}

#[test]
fn relations_with_source_returns_matching_ids() {
    let mut registry = RelationRegistry::new();
    let src_prop = PropertyId::new(ObjectId::new(1), 0);

    let a = registry.register(make_definition(1, 0, 2, 0));
    let b = registry.register(make_definition(1, 0, 3, 0)); // same source
    let _c = registry.register(make_definition(1, 1, 4, 0)); // different source

    let found: Vec<_> = registry.relations_with_source(src_prop).collect();
    assert_eq!(found.len(), 2);
    assert!(found.contains(&a));
    assert!(found.contains(&b));
}

#[test]
fn relations_with_target_returns_matching_ids() {
    let mut registry = RelationRegistry::new();
    let tgt_prop = PropertyId::new(ObjectId::new(2), 0);

    let a = registry.register(make_definition(1, 0, 2, 0));
    let b = registry.register(make_definition(3, 0, 2, 0)); // same target
    let _c = registry.register(make_definition(4, 0, 5, 0)); // different target

    let found: Vec<_> = registry.relations_with_target(tgt_prop).collect();
    assert_eq!(found.len(), 2);
    assert!(found.contains(&a));
    assert!(found.contains(&b));
}

#[test]
fn unregister_removes_definition_and_graph_entries() {
    let mut registry = RelationRegistry::new();
    let id = registry.register(make_definition(1, 0, 2, 0));
    assert_eq!(registry.len(), 1);

    let removed = registry.unregister(id);
    assert!(removed.is_some());
    assert_eq!(registry.len(), 0);
    assert!(registry.get_definition(id).is_none());

    let src_prop = PropertyId::new(ObjectId::new(1), 0);
    let found: Vec<_> = registry.relations_with_source(src_prop).collect();
    assert!(found.is_empty());
}

#[test]
fn stats_reflect_registered_relations() {
    let mut registry = RelationRegistry::new();
    registry.register(make_definition(1, 0, 2, 0));
    registry.register(make_definition(1, 1, 2, 1));
    registry.register(make_definition(3, 0, 4, 0));

    let stats = registry.stats();
    assert_eq!(stats.total_relations, 3);
    assert_eq!(stats.unique_source_properties, 3); // (1,0), (1,1), (3,0)
    assert_eq!(stats.unique_target_properties, 3); // (2,0), (2,1), (4,0)
}

#[test]
fn relations_iterator_visits_all() {
    let mut registry = RelationRegistry::new();
    let _a = registry.register(make_definition(1, 0, 2, 0));
    let _b = registry.register(make_definition(3, 0, 4, 0));
    let _c = registry.register(make_definition(5, 0, 6, 0));

    let count = registry.relations().count();
    assert_eq!(count, 3);
}

#[test]
fn registry_contains_and_clear() {
    let mut registry = RelationRegistry::new();
    let def = RelationDefinitionBuilder::new(
        Endpoint::source(PropertyId::new(ObjectId::new(1), 0)),
        Endpoint::target(PropertyId::new(ObjectId::new(2), 0)),
    ).build();
    let id = registry.register(def);
    
    assert!(registry.contains(id));
    assert!(!registry.contains(RelationId::from_raw(999)));
    
    registry.clear();
    assert_eq!(registry.len(), 0);
    assert!(!registry.contains(id));
}
#[test]
fn relations_between_specific_properties() {
    let mut registry = RelationRegistry::new();

    let src1 = Endpoint::source(PropertyId::new(ObjectId::new(1), 0));
    let tgt1 = Endpoint::target(PropertyId::new(ObjectId::new(2), 0));
    let src2 = Endpoint::source(PropertyId::new(ObjectId::new(1), 0));
    let tgt2 = Endpoint::target(PropertyId::new(ObjectId::new(3), 0));

    // A→B and A→C share source but have different targets.
    registry.register(
        RelationDefinitionBuilder::new(src1, tgt1).build(),
    );
    registry.register(
        RelationDefinitionBuilder::new(src2, tgt2).build(),
    );

    let prop_a = PropertyId::new(ObjectId::new(1), 0);
    let prop_b = PropertyId::new(ObjectId::new(2), 0);
    let prop_c = PropertyId::new(ObjectId::new(3), 0);

    // A→B: 1 relation.
    let between_ab: Vec<_> = registry.relations_between(prop_a, prop_b).collect();
    assert_eq!(between_ab.len(), 1);

    // A→C: 1 relation.
    let between_ac: Vec<_> = registry.relations_between(prop_a, prop_c).collect();
    assert_eq!(between_ac.len(), 1);

    // B→A: 0 relations (reverse direction).
    let between_ba: Vec<_> = registry.relations_between(prop_b, prop_a).collect();
    assert!(between_ba.is_empty());
}