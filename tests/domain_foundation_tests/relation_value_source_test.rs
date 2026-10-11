//! Test PropertyValueSource and PropertyValueSink traits.

use app_shell::domain_foundation::relations::definition::{EffectContext, EffectOutcome};
use app_shell::domain_foundation::relations::evaluation::{
    EvaluationContext, PropertyValueSink, PropertyValueSource,
};
use app_shell::domain_foundation::relations::{
    Endpoint, ObjectId, PropagationMode, PropertyId, PropertyValue,
    RelationDefinitionBuilder, RelationRegistry, Trigger,
};
use std::collections::HashMap;
use std::sync::Arc;

/// A simple Domain Engine state that implements PropertyValueSource/Sink.
struct DomainState {
    values: HashMap<PropertyId, PropertyValue>,
}

impl PropertyValueSource for DomainState {
    fn get_property_value(&self, property: PropertyId) -> PropertyValue {
        self.values
            .get(&property)
            .cloned()
            .unwrap_or(PropertyValue::None)
    }
}

impl PropertyValueSink for DomainState {
    fn set_property_value(&mut self, property: PropertyId, value: PropertyValue) {
        self.values.insert(property, value);
    }
}

#[test]
fn sourced_value_store_reads_from_domain_state() {
    let mut state = DomainState { values: HashMap::new() };
    let prop = PropertyId::new(ObjectId::new(1), 0);
    state.set_property_value(prop, PropertyValue::Int(42));

    // The SourcedValueStore should read from the DomainState.
    use app_shell::domain_foundation::relations::evaluation::SourcedValueStore;
    let store = SourcedValueStore::new(state);
    assert_eq!(store.get(prop), PropertyValue::Int(42));
}

#[test]
fn sourced_value_store_overrides_take_precedence() {
    let mut state = DomainState { values: HashMap::new() };
    let prop = PropertyId::new(ObjectId::new(1), 0);
    state.set_property_value(prop, PropertyValue::Int(42));

    use app_shell::domain_foundation::relations::evaluation::SourcedValueStore;
    let mut store = SourcedValueStore::new(state);

    // Override the value.
    store.set(prop, PropertyValue::Int(99));
    assert_eq!(store.get(prop), PropertyValue::Int(99));
}

#[test]
fn property_value_sink_writes_back_to_domain_state() {
    let mut state = DomainState { values: HashMap::new() };
    let prop = PropertyId::new(ObjectId::new(1), 0);

    state.set_property_value(prop, PropertyValue::Bool(true));
    assert_eq!(state.get_property_value(prop), PropertyValue::Bool(true));
}