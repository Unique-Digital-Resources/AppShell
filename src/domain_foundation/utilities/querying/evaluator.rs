//! Query evaluator: apply queries to collections.

use super::query::Query;

/// Evaluate a query against a slice of items, returning matching references.
pub fn evaluate<'a, T: 'static>(query: &Query<T>, items: &'a [T]) -> Vec<&'a T> {
    items
        .iter()
        .filter(|item| query.matches(item))
        .collect()
}