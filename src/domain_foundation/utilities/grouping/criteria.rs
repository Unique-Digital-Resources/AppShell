//! Group criteria: developer-defined grouping keys.

use std::collections::HashMap;
use std::hash::Hash;

/// A criterion that assigns a group key to each item.
pub trait GroupCriteria<T> {
    type Key: Eq + Hash;
    fn group_key(&self, item: &T) -> Self::Key;
}

/// Group `items` by a closure that returns the group key.
pub fn group_by<T, K, F>(items: &[T], f: F) -> HashMap<K, Vec<&T>>
where
    K: Eq + Hash,
    F: Fn(&T) -> K,
{
    let mut groups: HashMap<K, Vec<&T>> = HashMap::new();
    for item in items {
        let key = f(item);
        groups.entry(key).or_default().push(item);
    }
    groups
}

/// Like `group_by` but returns a `BTreeMap` for deterministic group order.
pub fn group_sorted<T, K, F>(items: &[T], f: F) -> std::collections::BTreeMap<K, Vec<&T>>
where
    K: Ord,
    F: Fn(&T) -> K,
{
    let mut groups: std::collections::BTreeMap<K, Vec<&T>> = std::collections::BTreeMap::new();
    for item in items {
        let key = f(item);
        groups.entry(key).or_default().push(item);
    }
    groups
}


/// Group `items` by a `GroupCriteria` implementor.
pub fn group<'a, T, G: GroupCriteria<T>>(
    items: &'a [T],
    criteria: &G,
) -> HashMap<G::Key, Vec<&'a T>> {
    group_by(items, |item| criteria.group_key(item))
}