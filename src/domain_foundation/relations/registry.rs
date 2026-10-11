//! Relation registry: stores definitions, issues ids, and provides
//! lookup by source property for the evaluation engine.

use super::definition::RelationDefinition;
use super::dependency::DependencyGraph;
use super::endpoint::PropertyId;
use super::relation::{Relation, RelationId};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Central store for registered relation definitions.
///
/// The registry issues [`RelationId`]s, maintains a
/// [`DependencyGraph`] for fast source/target lookup, and provides
/// the definitions the [`EvaluationEngine`](super::evaluation::EvaluationEngine)
/// needs during propagation.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Default)]
pub struct RelationRegistry {
    next_id: AtomicU64,
    definitions: HashMap<RelationId, RelationDefinition>,
    /// Quick lookup: source property → relation ids.
    by_source: HashMap<PropertyId, Vec<RelationId>>,
    /// Quick lookup: target property → relation ids.
    by_target: HashMap<PropertyId, Vec<RelationId>>,
    graph: DependencyGraph,
}

/// Lightweight statistics about the registry's contents.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default)]
pub struct RelationRegistryStats {
    pub total_relations: usize,
    pub unique_source_properties: usize,
    pub unique_target_properties: usize,
}

impl RelationRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new relation definition. Returns the assigned id.
		pub fn register(&mut self, definition: RelationDefinition) -> RelationId {
				let id = RelationId::from_raw(self.next_id.fetch_add(1, Ordering::Relaxed));
				let source_prop = definition.source.property;
				let target_prop = definition.target.property;

				self.graph.register_source(source_prop, id);
				self.graph.register_target(target_prop, id);

				// Insert into by_source, sorted by priority (descending).
				let vec = self.by_source.entry(source_prop).or_default();
				let pos = vec.partition_point(|&rid| {
				    self.definitions
				        .get(&rid)
				        .map(|d| d.priority >= definition.priority)
				        .unwrap_or(false)
				});
				vec.insert(pos, id);

				self.by_target.entry(target_prop).or_default().push(id);
				self.definitions.insert(id, definition);

				id
		}

    /// Remove a relation by id. Returns the removed definition if it existed.
    pub fn unregister(&mut self, id: RelationId) -> Option<RelationDefinition> {
        let def = self.definitions.remove(&id)?;
        self.graph.remove(id);
        if let Some(vec) = self.by_source.get_mut(&def.source.property) {
            vec.retain(|rid| *rid != id);
        }
        if let Some(vec) = self.by_target.get_mut(&def.target.property) {
            vec.retain(|rid| *rid != id);
        }
        Some(def)
    }

    /// Get a reference to a definition by id.
    pub fn get_definition(&self, id: RelationId) -> Option<&RelationDefinition> {
        self.definitions.get(&id)
    }

    /// All relation ids whose source endpoint matches `property`.
    pub fn relations_with_source(
        &self,
        property: PropertyId,
    ) -> impl Iterator<Item = RelationId> + '_ {
        self.by_source
            .get(&property)
            .map(|v| v.iter().copied())
            .into_iter()
            .flatten()
    }

    /// All relation ids whose target endpoint matches `property`.
    pub fn relations_with_target(
        &self,
        property: PropertyId,
    ) -> impl Iterator<Item = RelationId> + '_ {
        self.by_target
            .get(&property)
            .map(|v| v.iter().copied())
            .into_iter()
            .flatten()
    }

    /// A lightweight view of all registered relations (without the effect
    /// closures).
    pub fn relations(&self) -> impl Iterator<Item = Relation> + '_ {
        self.definitions.iter().map(|(id, def)| Relation::new(*id, def.source, def.target))
    }

    /// Borrow the dependency graph. Used by the evaluation/propagation
    /// engine for cascade traversal.
    pub fn dependency_graph(&self) -> &DependencyGraph {
        &self.graph
    }

    /// Number of registered relations.
    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    /// Summary statistics.
    pub fn stats(&self) -> RelationRegistryStats {
        RelationRegistryStats {
            total_relations: self.definitions.len(),
            unique_source_properties: self.by_source.len(),
            unique_target_properties: self.by_target.len(),
        }
    }

    /// Whether a relation with the given id is registered.
    pub fn contains(&self, id: RelationId) -> bool {
        self.definitions.contains_key(&id)
    }

    /// Remove all registered relations. Id counter is not reset.
    pub fn clear(&mut self) {
				self.definitions.clear();
				self.by_source.clear();
				self.by_target.clear();
				// Reset the dependency graph
				self.graph = DependencyGraph::default();
		}

    /// All relation ids from `source_prop` to `target_prop`.
    pub fn relations_between(
        &self,
        source_prop: PropertyId,
        target_prop: PropertyId,
    ) -> impl Iterator<Item = RelationId> + '_ {
        self.by_source
            .get(&source_prop)
            .into_iter()
            .flatten()
            .copied()
            .filter(move |rid| {
                self.definitions
                    .get(rid)
                    .map(|d| d.target.property == target_prop)
                    .unwrap_or(false)
            })
    }


}