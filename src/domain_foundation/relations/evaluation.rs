//! Evaluation engine: given a changed source property, find applicable
//! relations, call their effect closures, and return the outcomes.

use super::definition::{EffectContext, EffectOutcome, PropagationMode, PropertyValue};
use super::endpoint::PropertyId;
use super::error::RelationError;
use super::registry::RelationRegistry;
use super::relation::RelationId;
use std::collections::{HashMap, HashSet};

/// Holds the current property values needed during evaluation.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct ValueStore {
    values: HashMap<PropertyId, PropertyValue>,
}

impl ValueStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, property: PropertyId) -> &PropertyValue {
        self.values
            .get(&property)
            .unwrap_or(&PropertyValue::None)
    }

    pub fn set(&mut self, property: PropertyId, value: PropertyValue) {
        self.values.insert(property, value);
    }

    pub fn contains(&self, property: PropertyId) -> bool {
        self.values.contains_key(&property)
    }

    pub fn apply_outcome(&mut self, property: PropertyId, outcome: &EffectOutcome) {
        match outcome {
            EffectOutcome::Apply(v) => {
                self.values.insert(property, v.clone());
            }
            EffectOutcome::Invalidate => {}
            EffectOutcome::NoChange => {}
        }
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

}

// --- Gap 12: PropertyValueSource / Sink traits ---

/// A trait that lets a Domain Engine's state serve as the value source
/// for relation evaluation, avoiding manual sync to a separate ValueStore.
///
/// The Domain Engine implements this for its environment state and passes
/// it to `EvaluationContext::with_source`.
pub trait PropertyValueSource {
    fn get_property_value(&self, property: PropertyId) -> PropertyValue;
}

/// A trait that lets a Domain Engine receive property value updates
/// from relation evaluation.
pub trait PropertyValueSink {
    fn set_property_value(&mut self, property: PropertyId, value: PropertyValue);
}

/// A ValueStore adapter that reads from a `PropertyValueSource`.
/// This lets the evaluation engine read from the Domain Engine's
/// actual state instead of a separate snapshot.
pub struct SourcedValueStore<S: PropertyValueSource> {
    source: S,
    overrides: HashMap<PropertyId, PropertyValue>,
}

impl<S: PropertyValueSource> SourcedValueStore<S> {
    pub fn new(source: S) -> Self {
        Self {
            source,
            overrides: HashMap::new(),
        }
    }

    pub fn get(&self, property: PropertyId) -> PropertyValue {
        self.overrides
            .get(&property)
            .cloned()
            .unwrap_or_else(|| self.source.get_property_value(property))
    }

    pub fn set(&mut self, property: PropertyId, value: PropertyValue) {
        self.overrides.insert(property, value);
    }

    pub fn apply_outcome(&mut self, property: PropertyId, outcome: &EffectOutcome) {
        match outcome {
            EffectOutcome::Apply(v) => {
                self.overrides.insert(property, v.clone());
            }
            EffectOutcome::Invalidate => {}
            EffectOutcome::NoChange => {}
        }
    }
}

// --- Evaluation engine ---

pub struct EvaluationEngine<'a> {
    registry: &'a RelationRegistry,
    values: &'a mut ValueStore,
}

impl<'a> EvaluationEngine<'a> {
    pub fn new(registry: &'a RelationRegistry, values: &'a mut ValueStore) -> Self {
        Self { registry, values }
    }

    pub fn relations_with_source(
        &self,
        source: PropertyId,
    ) -> impl Iterator<Item = RelationId> + '_ {
        self.registry.relations_with_source(source)
    }

    pub fn evaluate_one(
        &mut self,
        relation_id: RelationId,
        source: PropertyId,
    ) -> Result<(PropertyId, EffectOutcome, PropagationMode), RelationError> {
        let def = self
            .registry
            .get_definition(relation_id)
            .ok_or(RelationError::RelationNotFound { id: relation_id })?;

        if def.source.property != source {
            return Err(RelationError::SourceMismatch {
                relation_id,
                expected: def.source.property,
                actual: source,
            });
        }

        let source_value = self.values.get(source).clone();
        let target_value = self.values.get(def.target.property).clone();

        let ctx = EffectContext {
            relation_id,
            source: &def.source.property,
            target: &def.target.property,
            source_value: &source_value,
            target_value: &target_value,
        };

        let outcome = (def.effect)(&ctx);
        self.values.apply_outcome(def.target.property, &outcome);

        Ok((def.target.property, outcome, def.propagation))
    }
}

// --- Gap 13: Batch propagation ---

/// External handle for triggering evaluation.
pub struct EvaluationContext<'a> {
    pub registry: &'a RelationRegistry,
    pub values: ValueStore,
    pending_changes: Vec<PropertyId>,
    in_batch: bool,
}

impl<'a> EvaluationContext<'a> {
    pub fn new(registry: &'a RelationRegistry) -> Self {
        Self {
            registry,
            values: ValueStore::new(),
            pending_changes: Vec::new(),
            in_batch: false,
        }
    }

    pub fn set_value(&mut self, property: PropertyId, value: PropertyValue) {
        self.values.set(property, value);
        if self.in_batch {
            self.pending_changes.push(property);
        }
    }

    pub fn get_value(&self, property: PropertyId) -> &PropertyValue {
        self.values.get(property)
    }

    /// Propagate a single property change immediately.
    pub fn propagate_from(
        &mut self,
        source: PropertyId,
    ) -> Result<Vec<super::propagation::PropagationStep>, RelationError> {
        let mut engine = EvaluationEngine::new(self.registry, &mut self.values);
        let mut propagator = super::propagation::Propagator::new(&mut engine);
        propagator.propagate_from(source)
    }

    // --- Batch API ---

    /// Begin a batch. Subsequent `set_value` calls accumulate changes
    /// without propagating. Call `commit_batch` to propagate all at once.
    pub fn begin_batch(&mut self) {
        self.in_batch = true;
        self.pending_changes.clear();
    }

    /// Commit the batch: propagate all accumulated changes, deduplicated.
    /// Returns all propagation steps in order.
    pub fn commit_batch(&mut self) -> Result<Vec<super::propagation::PropagationStep>, RelationError> {
        self.in_batch = false;
        let changes = std::mem::take(&mut self.pending_changes);

        let mut seen = HashSet::new();
        let mut all_steps = Vec::new();
        for source in changes {
            if seen.insert(source) {
                let steps = self.propagate_from(source)?;
                all_steps.extend(steps);
            }
        }
        Ok(all_steps)
    }

    /// Rollback the batch: discard all accumulated changes.
    pub fn rollback_batch(&mut self) {
        self.in_batch = false;
        self.pending_changes.clear();
    }

    pub fn is_in_batch(&self) -> bool {
        self.in_batch
    }
}