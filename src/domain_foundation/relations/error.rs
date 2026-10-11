//! Relation-specific errors.

use super::endpoint::PropertyId;
use super::relation::RelationId;
use std::fmt;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationError {
    /// No relation exists with the given id.
    RelationNotFound { id: RelationId },
    /// The property passed as the source does not match the relation's
    /// declared source endpoint.
    SourceMismatch {
        relation_id: RelationId,
        expected: PropertyId,
        actual: PropertyId,
    },
    /// Cascading propagation exceeded the maximum depth, indicating a
    /// cycle or an excessively deep chain.
    MaxPropagationDepthReached {
        depth: usize,
        property: PropertyId,
    },
}

impl fmt::Display for RelationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RelationError::RelationNotFound { id } => {
                write!(f, "relation {id} not found in registry")
            }
            RelationError::SourceMismatch {
                relation_id,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "relation {relation_id}: source mismatch (expected {expected}, got {actual})"
                )
            }
            RelationError::MaxPropagationDepthReached { depth, property } => {
                write!(
                    f,
                    "max propagation depth ({depth}) reached at {property}; possible cycle"
                )
            }
        }
    }
}

impl std::error::Error for RelationError {}