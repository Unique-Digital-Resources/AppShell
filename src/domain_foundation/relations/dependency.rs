//! Dependency representation.
//!
//! A dependency records that one relation's target is another relation's
//! source, forming a dependency chain. The evaluation engine uses this to
//! determine cascade order without re-scanning the entire registry.

use super::endpoint::PropertyId;
use super::relation::RelationId;
use std::collections::{HashMap, HashSet};

/// Records which relations depend on a given property (i.e., have that
/// property as their source) and which relations produce a given property
/// (i.e., have that property as their target).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Default, Clone)]
pub struct DependencyGraph {
    /// Property → relations whose source is this property.
    source_of: HashMap<PropertyId, HashSet<RelationId>>,
    /// Property → relations whose target is this property.
    target_of: HashMap<PropertyId, HashSet<RelationId>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record that `relation` has `source` as its source endpoint.
    pub fn register_source(&mut self, source: PropertyId, relation: RelationId) {
        self.source_of.entry(source).or_default().insert(relation);
    }

    /// Record that `relation` has `target` as its target endpoint.
    pub fn register_target(&mut self, target: PropertyId, relation: RelationId) {
        self.target_of.entry(target).or_default().insert(relation);
    }

    /// Remove all entries for `relation`.
    pub fn remove(&mut self, relation: RelationId) {
        for set in self.source_of.values_mut() {
            set.remove(&relation);
        }
        for set in self.target_of.values_mut() {
            set.remove(&relation);
        }
    }

    /// All relations whose source is `property`.
    pub fn relations_with_source(&self, property: PropertyId) -> impl Iterator<Item = RelationId> + '_ {
        self.source_of
            .get(&property)
            .map(|s| s.iter().copied())
            .into_iter()
            .flatten()
    }

    /// All relations whose target is `property`.
    pub fn relations_with_target(&self, property: PropertyId) -> impl Iterator<Item = RelationId> + '_ {
        self.target_of
            .get(&property)
            .map(|s| s.iter().copied())
            .into_iter()
            .flatten()
    }

    /// Whether any relation has `property` as its source.
    pub fn has_sources_for(&self, property: PropertyId) -> bool {
        self.source_of
            .get(&property)
            .map_or(false, |s| !s.is_empty())
    }

    /// Whether any relation has `property` as its target.
    pub fn has_targets_for(&self, property: PropertyId) -> bool {
        self.target_of
            .get(&property)
            .map_or(false, |s| !s.is_empty())
    }
}