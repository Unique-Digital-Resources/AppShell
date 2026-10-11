//! Query: a named, reusable filter.

use super::super::filtering::FilterCriteria;

/// A named query combining a label with filter criteria.
pub struct Query<T: 'static> {
    name: String,
    criteria: Box<dyn FilterCriteria<T>>,
}

impl<T: 'static> Query<T> {
    /// Create a query from any `FilterCriteria` implementor.
    pub fn new<C: FilterCriteria<T> + 'static>(name: String, criteria: C) -> Self {
        Self { name, criteria: Box::new(criteria) }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn matches(&self, item: &T) -> bool {
        self.criteria.matches(item)
    }
}