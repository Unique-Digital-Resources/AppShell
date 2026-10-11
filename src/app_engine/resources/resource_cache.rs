//! `ResourceCache` — keeps loaded resources available for reuse.

use std::collections::HashMap;

use crate::app_engine::resources::resource::{Resource, ResourceId};

pub struct ResourceCache {
    entries: HashMap<ResourceId, Resource>,
}

impl ResourceCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn insert(&mut self, resource: Resource) {
        let id = resource.id();
        self.entries.insert(id, resource);
    }

    pub fn get(&self, id: ResourceId) -> Option<&Resource> {
        self.entries.get(&id)
    }

    pub fn get_mut(&mut self, id: ResourceId) -> Option<&mut Resource> {
        self.entries.get_mut(&id)
    }

    pub fn remove(&mut self, id: ResourceId) -> Option<Resource> {
        self.entries.remove(&id)
    }

    pub fn contains(&self, id: ResourceId) -> bool {
        self.entries.contains_key(&id)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn ids(&self) -> Vec<ResourceId> {
        self.entries.keys().copied().collect()
    }
}

impl Default for ResourceCache {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ResourceCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceCache")
            .field("count", &self.entries.len())
            .finish()
    }
}