//! Propagation: how a single property change fans out through the
//! relation graph.
//!
//! The propagator gets relation ids from the [`EvaluationEngine`], which
//! delegates to the [`RelationRegistry`]'s priority-sorted list. This
//! ensures higher-priority relations fire first.

use super::definition::{EffectOutcome, PropagationMode};
use super::endpoint::PropertyId;
use super::error::RelationError;
use super::evaluation::EvaluationEngine;
use super::relation::RelationId;
use std::collections::HashSet;

/// A propagation request: starting from a changed source property,
/// evaluate all applicable relations and cascade if needed.
pub struct Propagator<'a> {
    engine: &'a mut EvaluationEngine<'a>,
    visited: HashSet<RelationId>,
    max_depth: usize,
}

impl<'a> Propagator<'a> {
    /// Create a propagator that draws relation ids from the evaluation
    /// engine (which delegates to the registry's priority-sorted list).
    pub fn new(engine: &'a mut EvaluationEngine<'a>) -> Self {
        Self {
            engine,
            visited: HashSet::new(),
            max_depth: 256,
        }
    }

    /// Set the maximum cascade depth to prevent infinite loops in cyclic
    /// relation graphs.
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Propagate from a changed source property.
    pub fn propagate_from(
        &mut self,
        source: PropertyId,
    ) -> Result<Vec<PropagationStep>, RelationError> {
        let mut applied = Vec::new();
        self.propagate_recursive(source, &mut applied, 0)?;
        Ok(applied)
    }

    fn propagate_recursive(
        &mut self,
        source: PropertyId,
        applied: &mut Vec<PropagationStep>,
        depth: usize,
    ) -> Result<(), RelationError> {
        if depth >= self.max_depth {
            return Err(RelationError::MaxPropagationDepthReached {
                depth,
                property: source,
            });
        }

        // Get relation ids from the engine (priority-sorted, highest first).
        let relation_ids: Vec<RelationId> = self.engine.relations_with_source(source).collect();

        for rel_id in relation_ids {
            if !self.visited.insert(rel_id) {
                continue;
            }

            let (target, outcome, propagation_mode) = self.engine.evaluate_one(rel_id, source)?;

            match &outcome {
                EffectOutcome::NoChange => {}
                EffectOutcome::Invalidate => {
                    applied.push(PropagationStep {
                        relation_id: rel_id,
                        target,
                        outcome: outcome.clone(),
                    });
                    if propagation_mode == PropagationMode::Cascading {
                        self.propagate_recursive(target, applied, depth + 1)?;
                    }
                }
                EffectOutcome::Apply(_) => {
                    applied.push(PropagationStep {
                        relation_id: rel_id,
                        target,
                        outcome: outcome.clone(),
                    });
                    if propagation_mode == PropagationMode::Cascading {
                        self.propagate_recursive(target, applied, depth + 1)?;
                    }
                }
            }
        }

        Ok(())
    }
}

/// One step in a propagation chain.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct PropagationStep {
    pub relation_id: RelationId,
    pub target: PropertyId,
    pub outcome: EffectOutcome,
}