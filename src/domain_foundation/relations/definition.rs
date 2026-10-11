//! Relation definitions: declarative descriptions of what a relation
//! connects and when it fires, plus the developer-supplied effect.

use super::endpoint::{Endpoint, PropertyId};
use super::relation::RelationId;
use std::sync::Arc;

/// What change activates a relation.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trigger {
    OnChange,
    OnInitialSet,
    OnInvalidation,
}

/// The direction in which a relation's effect propagates after firing.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropagationMode {
    Direct,
    Cascading,
}

/// The developer-supplied effect function.
pub type EffectFn = Arc<dyn Fn(&EffectContext) -> EffectOutcome + Send + Sync>;

/// Read-only context passed to an [`EffectFn`] during evaluation.
pub struct EffectContext<'a> {
    pub relation_id: RelationId,
    pub source: &'a PropertyId,
    pub target: &'a PropertyId,
    pub source_value: &'a PropertyValue,
    pub target_value: &'a PropertyValue,
}

/// The outcome of evaluating an effect.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum EffectOutcome {
    Apply(PropertyValue),
    NoChange,
    Invalidate,
}

/// A property value used by the relation system.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    None,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    Bytes(Vec<u8>),
}

impl Default for PropertyValue {
    fn default() -> Self {
        PropertyValue::None
    }
}

impl PropertyValue {
    pub fn is_none(&self) -> bool {
        matches!(self, PropertyValue::None)
    }
}


/// Default effect function used when deserializing a RelationDefinition.
/// Since closures can't be serialized, the effect is always deserialized
/// as a no-op.
#[cfg(feature = "serde")]
fn default_effect() -> EffectFn {
    Arc::new(|_| EffectOutcome::NoChange)
}


/// A declarative relation definition.
///
/// `priority` controls evaluation order when multiple relations share
/// the same source property. Higher priority fires first.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone)]
pub struct RelationDefinition {
    pub source: Endpoint,
    pub target: Endpoint,
    pub trigger: Trigger,
    pub propagation: PropagationMode,
    pub priority: i32,
    #[cfg_attr(feature = "serde", serde(skip, default = "default_effect"))]
    pub effect: EffectFn,
}

impl std::fmt::Debug for RelationDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RelationDefinition")
            .field("source", &self.source)
            .field("target", &self.target)
            .field("trigger", &self.trigger)
            .field("propagation", &self.propagation)
            .field("priority", &self.priority)
            .field("effect", &"<closure>")
            .finish()
    }
}

impl PartialEq for RelationDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
            && self.target == other.target
            && self.trigger == other.trigger
            && self.propagation == other.propagation
            && self.priority == other.priority
    }
}

/// Builder for [`RelationDefinition`].
pub struct RelationDefinitionBuilder {
    source: Endpoint,
    target: Endpoint,
    trigger: Trigger,
    propagation: PropagationMode,
    priority: i32,
    effect: Option<EffectFn>,
}

impl RelationDefinitionBuilder {
    pub fn new(source: Endpoint, target: Endpoint) -> Self {
        Self {
            source,
            target,
            trigger: Trigger::OnChange,
            propagation: PropagationMode::Direct,
            priority: 0,
            effect: None,
        }
    }

    pub fn trigger(mut self, trigger: Trigger) -> Self {
        self.trigger = trigger;
        self
    }

    pub fn propagation(mut self, mode: PropagationMode) -> Self {
        self.propagation = mode;
        self
    }

    /// Set the priority. Higher priority fires first when multiple
    /// relations share the same source.
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn effect(mut self, effect: EffectFn) -> Self {
        self.effect = Some(effect);
        self
    }

    pub fn build(self) -> RelationDefinition {
        RelationDefinition {
            source: self.source,
            target: self.target,
            trigger: self.trigger,
            propagation: self.propagation,
            priority: self.priority,
            effect: self.effect.unwrap_or_else(|| Arc::new(|_| EffectOutcome::NoChange)),
        }
    }
}