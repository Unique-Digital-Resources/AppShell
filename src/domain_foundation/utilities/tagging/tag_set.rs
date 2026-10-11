//! Tag collections with deterministic iteration order.

use super::tag::Tag;
use std::collections::BTreeSet;

/// A collection of unique tags with deterministic iteration order.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TagSet {
    tags: BTreeSet<Tag>,
}

impl TagSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_tags<I: IntoIterator<Item = Tag>>(tags: I) -> Self {
        Self { tags: tags.into_iter().collect() }
    }

    pub fn add(&mut self, tag: Tag) -> bool {
        self.tags.insert(tag)
    }

    pub fn remove(&mut self, tag: &Tag) -> bool {
        self.tags.remove(tag)
    }

    pub fn contains(&self, tag: &Tag) -> bool {
        self.tags.contains(tag)
    }

    pub fn len(&self) -> usize {
        self.tags.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }

    /// Tags in sorted (deterministic) order.
    pub fn iter(&self) -> impl Iterator<Item = &Tag> {
        self.tags.iter()
    }

    pub fn intersection(&self, other: &TagSet) -> TagSet {
        Self { tags: self.tags.intersection(&other.tags).cloned().collect() }
    }

    pub fn union(&self, other: &TagSet) -> TagSet {
        Self { tags: self.tags.union(&other.tags).cloned().collect() }
    }

    pub fn contains_any(&self, other: &TagSet) -> bool {
        self.tags.iter().any(|t| other.contains(t))
    }

    pub fn contains_all(&self, other: &TagSet) -> bool {
        other.tags.iter().all(|t| self.contains(t))
    }
}

impl<'a> IntoIterator for &'a TagSet {
    type Item = &'a Tag;
    type IntoIter = std::collections::btree_set::Iter<'a, Tag>;

    fn into_iter(self) -> Self::IntoIter {
        self.tags.iter()
    }
}