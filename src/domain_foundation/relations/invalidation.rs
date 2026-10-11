//! Invalidation tracking.
//!
//! When a property is "invalidated" (marked dirty), relations with
//! `Trigger::OnInvalidation` may fire. The invalidation tracker records
//! which properties are currently dirty so the evaluation engine can
//! query them during propagation.

use super::endpoint::PropertyId;
use std::collections::HashSet;

/// Tracks which properties are currently invalidated (dirty).
#[derive(Debug, Default, Clone)]
pub struct InvalidationTracker {
    dirty: HashSet<PropertyId>,
}

impl InvalidationTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark `property` as dirty.
    pub fn invalidate(&mut self, property: PropertyId) {
        self.dirty.insert(property);
    }

    /// Clear the dirty flag for `property`.
    pub fn clear(&mut self, property: PropertyId) {
        self.dirty.remove(&property);
    }

    /// Clear all dirty flags.
    pub fn clear_all(&mut self) {
        self.dirty.clear();
    }

    /// Whether `property` is currently dirty.
    pub fn is_dirty(&self, property: PropertyId) -> bool {
        self.dirty.contains(&property)
    }

    /// All currently-dirty properties.
    pub fn dirty_properties(&self) -> impl Iterator<Item = PropertyId> + '_ {
        self.dirty.iter().copied()
    }

    /// Number of dirty properties.
    pub fn dirty_count(&self) -> usize {
        self.dirty.len()
    }

    /// Take all dirty properties, clearing the tracker.
    pub fn drain(&mut self) -> HashSet<PropertyId> {
        std::mem::take(&mut self.dirty)
    }
}