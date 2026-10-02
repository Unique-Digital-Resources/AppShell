//! `ResourceManager` — the main resource lifecycle coordinator.
//!
//! Combines `Resource`, `ResourceHandle`, `ResourceLoader`, and `ResourceCache`.
//!
//! ```text
//! request(type, source)
//!        ↓
//! ResourceManager
//!        │
//!        ├── check source_index (dedup)
//!        ├── check cache
//!        ├── resolve loader
//!        └── load
//!              ↓
//!          Resource
//!              ↓
//!           Handle
//! ```

//! `ResourceManager` — the main resource lifecycle coordinator.
//!
//! Phase 16: now supports RAII handles (auto-release on drop) and
//! dependency tracking (load dependencies first, prevent unloading
//! resources that have dependents).

//! `ResourceManager` — the main resource lifecycle coordinator.
//!
//! Phase 25: now supports RAII handles that auto-release on drop.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock, Weak};

use crate::app_engine::errors::resource_error::ResourceError;
use crate::app_engine::resources::resource::{
    Resource, ResourceId, ResourceKey,
};
use crate::app_engine::resources::resource_cache::ResourceCache;
use crate::app_engine::resources::resource_dependency::ResourceDependencyGraph;
use crate::app_engine::resources::resource_handle::ResourceHandle;
use crate::app_engine::resources::resource_loader::ResourceLoader;

pub struct ResourceManager {
    cache: Mutex<ResourceCache>,
    source_index: Mutex<HashMap<ResourceKey, ResourceId>>,
    loaders: RwLock<HashMap<String, Box<dyn ResourceLoader>>>,
    ref_counts: Mutex<HashMap<ResourceId, usize>>,
    dependencies: Mutex<ResourceDependencyGraph>,
    /// Weak reference to self, set by AppRuntime after construction.
    /// Used to create RAII handles that auto-release on drop.
    self_ref: Mutex<Option<Weak<ResourceManager>>>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(ResourceCache::new()),
            source_index: Mutex::new(HashMap::new()),
            loaders: RwLock::new(HashMap::new()),
            ref_counts: Mutex::new(HashMap::new()),
            dependencies: Mutex::new(ResourceDependencyGraph::new()),
            self_ref: Mutex::new(None),
        }
    }

    /// Set the weak self-reference. Called by AppRuntime after wrapping in Arc.
    pub fn set_self_ref(&self, weak: Weak<ResourceManager>) {
        *self.self_ref.lock().unwrap() = Some(weak);
    }

    /// Create an RAII handle that auto-releases on drop.
    fn create_raii_handle(&self, id: ResourceId) -> ResourceHandle {
        let weak = self.self_ref.lock().unwrap().clone();
        ResourceHandle::with_release_callback(id, move |resource_id| {
            if let Some(ref weak) = weak {
                if let Some(mgr) = weak.upgrade() {
                    let _ = mgr.release_by_id(resource_id);
                }
            }
        })
    }

    // ----- Loader registration -----

    pub fn register_loader(&self, loader: Box<dyn ResourceLoader>) {
        let type_name = loader.resource_type().to_string();
        self.loaders.write().unwrap().insert(type_name, loader);
    }

    pub fn has_loader(&self, resource_type: &str) -> bool {
        self.loaders.read().unwrap().contains_key(resource_type)
    }

    pub fn loader_count(&self) -> usize {
        self.loaders.read().unwrap().len()
    }

    // ----- Dependency registration -----

    pub fn register_dependency(
        &self,
        resource_key: &str,
        dependency_key: &str,
    ) {
        self.dependencies.lock().unwrap().register_dependency(resource_key, dependency_key);
    }

    pub fn dependencies_of(&self, resource_key: &str) -> Vec<String> {
        self.dependencies.lock().unwrap().dependencies_of(resource_key)
    }

    pub fn has_dependents(&self, resource_key: &str) -> bool {
        self.dependencies.lock().unwrap().has_dependents(resource_key)
    }

    // ----- Loading -----

    pub fn load(
        &self,
        resource_type: &str,
        source: &str,
    ) -> Result<ResourceHandle, ResourceError> {
        let key = ResourceKey::new(resource_type, source);
        let key_str = format!("{}:{}", resource_type, source);

        // Check if already loaded/cached
        {
            let source_index = self.source_index.lock().unwrap();
            if let Some(&id) = source_index.get(&key) {
                let cache = self.cache.lock().unwrap();
                if cache.contains(id) {
                    drop(cache);
                    drop(source_index);
                    *self.ref_counts.lock().unwrap().entry(id).or_insert(0) += 1;
                    return Ok(self.create_raii_handle(id));
                }
            }
        }

        // Load dependencies first
        let deps = {
            let deps_graph = self.dependencies.lock().unwrap();
            deps_graph.dependencies_of(&key_str)
        };
        for dep_key in &deps {
            if let Some((dep_type, dep_source)) = dep_key.split_once(':') {
                self.load(dep_type, dep_source)?;
            }
        }

        // Find the loader
        let loaders_guard = self.loaders.read().unwrap();
        let loader = loaders_guard.get(resource_type)
            .ok_or_else(|| ResourceError::LoaderNotFound {
                resource_type: resource_type.to_string(),
            })?;

        // Load the data
        let data = loader.load(source)?;

        // Create the resource
        let mut resource = Resource::new(resource_type, source);
        resource.set_data_box(data);

        let id = resource.id();
        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(resource);
        }
        {
            let mut source_index = self.source_index.lock().unwrap();
            source_index.insert(key, id);
        }
        *self.ref_counts.lock().unwrap().entry(id).or_insert(0) += 1;

        Ok(self.create_raii_handle(id))
    }

    // ----- Access -----

    #[allow(dead_code)]
    pub fn get(&self, _handle: &ResourceHandle) -> Option<&Resource> {
        unimplemented!("Use with_resource / exists instead")
    }

    /// Access a resource within the lock (closure pattern).
    pub fn with_resource<F, R>(&self, id: ResourceId, f: F) -> Option<R>
    where
        F: FnOnce(&Resource) -> R,
    {
        let cache = self.cache.lock().unwrap();
        cache.get(id).map(|r| f(r))
    }

    pub fn exists(&self, id: ResourceId) -> bool {
        self.cache.lock().unwrap().contains(id)
    }

    pub fn resource_count(&self) -> usize {
        self.cache.lock().unwrap().len()
    }

    pub fn active_count(&self, id: ResourceId) -> usize {
        self.ref_counts.lock().unwrap().get(&id).copied().unwrap_or(0)
    }

    pub fn is_in_use(&self, id: ResourceId) -> bool {
        self.active_count(id) > 0
    }

    // ----- Acquire / Release -----

    pub fn acquire(&self, handle: &ResourceHandle) -> Result<(), ResourceError> {
        let id = handle.resource_id();
        if !self.cache.lock().unwrap().contains(id) {
            return Err(ResourceError::InvalidHandle { id: id.value() });
        }
        *self.ref_counts.lock().unwrap().entry(id).or_insert(0) += 1;
        Ok(())
    }

    pub fn release(&self, handle: ResourceHandle) -> Result<(), ResourceError> {
        let id = handle.resource_id();
        let result = self.release_by_id(id);
        // Prevent the Drop callback from firing (which would double-release)
        std::mem::forget(handle);
        result
    }

    pub fn release_by_id(&self, id: ResourceId) -> Result<(), ResourceError> {
        let mut counts = self.ref_counts.lock().unwrap();
        let count = counts
            .get_mut(&id)
            .ok_or(ResourceError::InvalidHandle { id: id.value() })?;
        *count = count.saturating_sub(1);
        Ok(())
    }

    // ----- Unload / Reload -----

    pub fn unload(&self, id: ResourceId) -> Result<(), ResourceError> {
        if self.is_in_use(id) {
            return Err(ResourceError::InUse { id: id.value() });
        }

        let source_str = self.cache.lock().unwrap().get(id)
            .map(|r| format!("{}:{}", r.resource_type(), r.source()));

        if let Some(source_str) = source_str {
            if self.dependencies.lock().unwrap().has_dependents(&source_str) {
                return Err(ResourceError::InUse { id: id.value() });
            }
        }

        self.cache.lock().unwrap().remove(id);
        self.ref_counts.lock().unwrap().remove(&id);
        Ok(())
    }

    pub fn reload(&self, id: ResourceId) -> Result<(), ResourceError> {
        let (resource_type, source) = {
            let cache = self.cache.lock().unwrap();
            let resource = cache.get(id)
                .ok_or(ResourceError::NotFound { id: id.value() })?;
            (resource.resource_type().to_string(), resource.source().to_string())
        };

        let loaders_guard = self.loaders.read().unwrap();
        let loader = loaders_guard.get(&resource_type)
            .ok_or_else(|| ResourceError::LoaderNotFound {
                resource_type: resource_type.clone(),
            })?;

        let data = loader.load(&source).map_err(|e| ResourceError::ReloadFailed {
            id: id.value(),
            reason: e.to_string(),
        })?;

        let mut cache = self.cache.lock().unwrap();
        let resource = cache.get_mut(id)
            .ok_or(ResourceError::NotFound { id: id.value() })?;
        resource.set_data_box(data);

        Ok(())
    }

    // ----- Cache access -----

    pub fn cache(&self) -> std::sync::MutexGuard<'_, ResourceCache> {
        self.cache.lock().unwrap()
    }

    pub fn dependency_graph(&self) -> std::sync::MutexGuard<'_, ResourceDependencyGraph> {
        self.dependencies.lock().unwrap()
    }

    /// Unload all non-in-use resources. Returns count of unloaded resources.
    pub fn unload_all(&self) -> usize {
        let cache_ids: Vec<ResourceId> = self.cache.lock().unwrap().ids();
        let mut count = 0;
        for id in cache_ids {
            if !self.is_in_use(id) {
                self.cache.lock().unwrap().remove(id);
                self.ref_counts.lock().unwrap().remove(&id);
                count += 1;
            }
        }
        count
    }

}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ResourceManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceManager")
            .field("cached_resources", &self.cache.lock().unwrap().len())
            .field("loaders", &self.loaders.read().unwrap().len())
            .field("source_index", &self.source_index.lock().unwrap().len())
            .field("has_self_ref", &self.self_ref.lock().unwrap().is_some())
            .finish()
    }
}