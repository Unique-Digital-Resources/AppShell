//! Sort criteria: developer-defined ordering.

use std::cmp::Ordering;

/// A criterion that determines the ordering of two items.
pub trait SortCriteria<T> {
    fn compare(&self, a: &T, b: &T) -> Ordering;
}

/// Closure-based sort criteria.
pub struct FnSortCriteria<F> {
    f: F,
}

impl<T, F: Fn(&T, &T) -> Ordering> SortCriteria<T> for FnSortCriteria<F> {
    fn compare(&self, a: &T, b: &T) -> Ordering {
        (self.f)(a, b)
    }
}

/// Create a sort criteria from a closure.
pub fn by<T, F: Fn(&T, &T) -> Ordering>(f: F) -> FnSortCriteria<F> {
    FnSortCriteria { f }
}

/// Sort `items` in-place using `criteria`.
pub fn sort<T, C: SortCriteria<T>>(items: &mut [T], criteria: &C) {
    items.sort_by(|a, b| criteria.compare(a, b));
}

/// Return a new sorted `Vec` (does not mutate the input).
pub fn sorted<'a, T, C: SortCriteria<T>>(items: &'a [T], criteria: &C) -> Vec<&'a T> {
    let mut refs: Vec<&T> = items.iter().collect();
    refs.sort_by(|a, b| criteria.compare(a, b));
    refs
}