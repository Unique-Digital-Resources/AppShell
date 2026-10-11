//! Filter: apply criteria to collections.

use super::criteria::FilterCriteria;

/// Apply `criteria` to `items`, returning references to matching items.
pub fn filter<'a, T, C: FilterCriteria<T>>(
    items: &'a [T],
    criteria: &C,
) -> Vec<&'a T> {
    items
        .iter()
        .filter(|item| criteria.matches(item))
        .collect()
}

/// Count matching items without collecting.
pub fn count<T, C: FilterCriteria<T>>(items: &[T], criteria: &C) -> usize {
    items.iter().filter(|item| criteria.matches(item)).count()
}

/// Whether any item matches.
pub fn any<T, C: FilterCriteria<T>>(items: &[T], criteria: &C) -> bool {
    items.iter().any(|item| criteria.matches(item))
}

/// Whether all items match.
pub fn all<T, C: FilterCriteria<T>>(items: &[T], criteria: &C) -> bool {
    items.iter().all(|item| criteria.matches(item))
}