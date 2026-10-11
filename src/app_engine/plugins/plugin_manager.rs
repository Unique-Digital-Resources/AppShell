//! `PluginManager` — the main plugin lifecycle coordinator.
//!
//! Phase 24: now with panic isolation. Plugin lifecycle methods
//! are wrapped in `catch_unwind` to prevent panics from crashing the engine.

use std::collections::HashMap;

use crate::app_engine::capabilities::engine_ref::EngineRef;
use crate::app_engine::errors::plugin_error::PluginError;
use crate::app_engine::plugins::plugin::{
    Plugin, PluginEntry, PluginId, PluginState,
};
use crate::app_engine::plugins::plugin_context::PluginContext;
use crate::app_engine::plugins::plugin_loader::PluginLoader;
use crate::app_engine::plugins::plugin_manifest::PluginManifest;
use crate::app_engine::plugins::plugin_permissions::{
    PluginPermissions, REGISTER_COMMANDS, REGISTER_TASKS, LOAD_RESOURCES,
    REGISTER_SERVICES, PUBLISH_EVENTS, WRITE_CONFIG,
};
use crate::app_engine::plugins::plugin_registry::PluginRegistry;

pub struct PluginManager {
    entries: HashMap<PluginId, PluginEntry>,
    type_index: HashMap<String, PluginId>,
    registry: PluginRegistry,
    loaders: Vec<Box<dyn PluginLoader>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            type_index: HashMap::new(),
            registry: PluginRegistry::new(),
            loaders: Vec::new(),
        }
    }

    // ----- Loader registration -----

    pub fn register_loader(&mut self, loader: Box<dyn PluginLoader>) {
        self.loaders.push(loader);
    }

    pub fn loader_count(&self) -> usize {
        self.loaders.len()
    }

    // ----- Plugin registration -----

    pub fn register(
        &mut self,
        manifest: PluginManifest,
        instance: Box<dyn Plugin>,
    ) -> Result<PluginId, PluginError> {
        if self.type_index.contains_key(manifest.id()) {
            return Err(PluginError::AlreadyRegistered {
                id: manifest.id().to_string(),
            });
        }

        let entry = PluginEntry::new(manifest.clone(), instance);
        let id = entry.id;
        self.registry.register(manifest)?;
        self.type_index
            .insert(entry.manifest.id().to_string(), id);
        self.entries.insert(id, entry);
        Ok(id)
    }

    pub fn unregister(&mut self, id: PluginId) -> Option<PluginManifest> {
        let entry = self.entries.remove(&id)?;
        self.type_index.remove(entry.manifest.id());
        self.registry.unregister(entry.manifest.id())
    }

    // ----- Loading via loaders -----

    pub fn load(&mut self, source: &str) -> Result<PluginId, PluginError> {
        for loader in &self.loaders {
            if loader.can_load(source) {
                let (manifest, instance) = loader.load(source)?;
                return self.register(manifest, instance);
            }
        }
        Err(PluginError::LoadFailed {
            source: source.to_string(),
            reason: "no suitable loader found".to_string(),
        })
    }

    // ----- Permission queries -----

    pub fn permissions(&self, id: PluginId) -> Option<PluginPermissions> {
        self.entries.get(&id).map(|e| e.manifest.permissions())
    }

    pub fn has_permission(&self, id: PluginId, permission: u32) -> bool {
        self.entries
            .get(&id)
            .map(|e| e.manifest.permissions().has(permission))
            .unwrap_or(false)
    }

    pub fn can_register_commands(&self, id: PluginId) -> bool {
        self.has_permission(id, REGISTER_COMMANDS)
    }

    pub fn can_register_tasks(&self, id: PluginId) -> bool {
        self.has_permission(id, REGISTER_TASKS)
    }

    pub fn can_load_resources(&self, id: PluginId) -> bool {
        self.has_permission(id, LOAD_RESOURCES)
    }

    pub fn can_register_services(&self, id: PluginId) -> bool {
        self.has_permission(id, REGISTER_SERVICES)
    }

    pub fn can_publish_events(&self, id: PluginId) -> bool {
        self.has_permission(id, PUBLISH_EVENTS)
    }

    /// Check if a plugin can write config.
    pub fn can_write_config(&self, id: PluginId) -> bool {
        self.has_permission(id, WRITE_CONFIG)
    }

    // ----- Queries -----

    pub fn get(&self, id: PluginId) -> Option<&(dyn Plugin + '_)> {
        self.entries.get(&id).map(|e| e.instance.as_ref())
    }

    pub fn get_manifest(&self, id: PluginId) -> Option<&PluginManifest> {
        self.entries.get(&id).map(|e| &e.manifest)
    }

    pub fn get_manifest_by_id(&self, plugin_id: &str) -> Option<&PluginManifest> {
        self.type_index
            .get(plugin_id)
            .and_then(|id| self.entries.get(id).map(|e| &e.manifest))
    }

    pub fn get_id_by_type(&self, plugin_id: &str) -> Option<PluginId> {
        self.type_index.get(plugin_id).copied()
    }

    pub fn contains(&self, id: PluginId) -> bool {
        self.entries.contains_key(&id)
    }

    pub fn contains_type(&self, plugin_id: &str) -> bool {
        self.type_index.contains_key(plugin_id)
    }

    pub fn state(&self, id: PluginId) -> Option<PluginState> {
        self.entries.get(&id).map(|e| e.state)
    }

    pub fn list(&self) -> Vec<&PluginManifest> {
        self.entries.values().map(|e| &e.manifest).collect()
    }

    pub fn list_ids(&self) -> Vec<PluginId> {
        self.entries.keys().copied().collect()
    }

    pub fn count(&self) -> usize {
        self.entries.len()
    }

    pub fn registry(&self) -> &PluginRegistry {
        &self.registry
    }

    // ----- Dependency validation -----

    pub fn validate_dependencies(&self, id: PluginId) -> Result<(), PluginError> {
        let entry = self
            .entries
            .get(&id)
            .ok_or(PluginError::NotFound { id: id.value() })?;
        for dep in entry.manifest.dependencies() {
            if !self.type_index.contains_key(dep) {
                return Err(PluginError::DependencyNotMet {
                    id: entry.manifest.id().to_string(),
                    dependency: dep.clone(),
                });
            }
        }
        Ok(())
    }

    pub fn validate_all_dependencies(&self) -> Result<(), PluginError> {
        let ids: Vec<PluginId> = self.list_ids();
        for id in ids {
            self.validate_dependencies(id)?;
        }
        Ok(())
    }

    // ----- Lifecycle (with panic isolation) -----

    /// Initialize a plugin. Panics are caught.
    pub fn initialize(
        &mut self,
        id: PluginId,
        engine: &EngineRef,
    ) -> Result<(), PluginError> {
        let entry = self
            .entries
            .get_mut(&id)
            .ok_or(PluginError::NotFound { id: id.value() })?;

        if !entry.state.can_initialize() {
            return Err(PluginError::InvalidState {
                id: id.value(),
                current: entry.state.as_str().to_string(),
                requested: "Initialized".to_string(),
            });
        }

        let plugin_id = entry.manifest.id().to_string();
        let context = PluginContext::new(plugin_id);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            entry.instance.initialize(&context, engine)
        }));

        match result {
            Ok(Ok(())) => {
                entry.state = PluginState::Initialized;
                Ok(())
            }
            Ok(Err(e)) => Err(PluginError::InitializeFailed {
                id: id.value(),
                reason: e.to_string(),
            }),
            Err(panic_payload) => {
                let reason = extract_panic_message(&panic_payload);
                Err(PluginError::InitializeFailed {
                    id: id.value(),
                    reason: format!("plugin panicked: {}", reason),
                })
            }
        }
    }

    /// Start a plugin. Panics are caught.
    pub fn start(
        &mut self,
        id: PluginId,
        engine: &EngineRef,
    ) -> Result<(), PluginError> {
        let entry = self
            .entries
            .get_mut(&id)
            .ok_or(PluginError::NotFound { id: id.value() })?;

        if !entry.state.can_start() {
            return Err(PluginError::InvalidState {
                id: id.value(),
                current: entry.state.as_str().to_string(),
                requested: "Started".to_string(),
            });
        }

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            entry.instance.start(engine)
        }));

        match result {
            Ok(Ok(())) => {
                entry.state = PluginState::Started;
                Ok(())
            }
            Ok(Err(e)) => Err(PluginError::StartFailed {
                id: id.value(),
                reason: e.to_string(),
            }),
            Err(panic_payload) => {
                let reason = extract_panic_message(&panic_payload);
                Err(PluginError::StartFailed {
                    id: id.value(),
                    reason: format!("plugin panicked: {}", reason),
                })
            }
        }
    }

    /// Stop a plugin. Panics are caught.
    pub fn stop(&mut self, id: PluginId) -> Result<(), PluginError> {
        let entry = self
            .entries
            .get_mut(&id)
            .ok_or(PluginError::NotFound { id: id.value() })?;

        if !entry.state.can_stop() {
            return Err(PluginError::InvalidState {
                id: id.value(),
                current: entry.state.as_str().to_string(),
                requested: "Stopped".to_string(),
            });
        }

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            entry.instance.stop()
        }));

        match result {
            Ok(Ok(())) => {
                entry.state = PluginState::Stopped;
                Ok(())
            }
            Ok(Err(e)) => Err(PluginError::StopFailed {
                id: id.value(),
                reason: e.to_string(),
            }),
            Err(panic_payload) => {
                let reason = extract_panic_message(&panic_payload);
                Err(PluginError::StopFailed {
                    id: id.value(),
                    reason: format!("plugin panicked: {}", reason),
                })
            }
        }
    }

    /// Dispose a plugin. Panics are caught.
    pub fn dispose(&mut self, id: PluginId) -> Result<(), PluginError> {
        let entry = self
            .entries
            .get_mut(&id)
            .ok_or(PluginError::NotFound { id: id.value() })?;

        if !entry.state.can_dispose() {
            return Err(PluginError::InvalidState {
                id: id.value(),
                current: entry.state.as_str().to_string(),
                requested: "Unloaded".to_string(),
            });
        }

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            entry.instance.dispose()
        }));

        match result {
            Ok(Ok(())) => {
                entry.state = PluginState::Unloaded;
                Ok(())
            }
            Ok(Err(e)) => Err(PluginError::DisposeFailed {
                id: id.value(),
                reason: e.to_string(),
            }),
            Err(panic_payload) => {
                let reason = extract_panic_message(&panic_payload);
                Err(PluginError::DisposeFailed {
                    id: id.value(),
                    reason: format!("plugin panicked: {}", reason),
                })
            }
        }
    }

    // ----- Bulk lifecycle -----

    pub fn initialize_all(&mut self, engine: &EngineRef) -> Result<(), PluginError> {
        self.validate_all_dependencies()?;
        let ids: Vec<PluginId> = self.list_ids();
        for id in ids {
            if let Some(entry) = self.entries.get(&id) {
                if entry.state.can_initialize() {
                    self.initialize(id, engine)?;
                }
            }
        }
        Ok(())
    }

    pub fn start_all(&mut self, engine: &EngineRef) -> Result<(), PluginError> {
        let ids: Vec<PluginId> = self.list_ids();
        for id in ids {
            if let Some(entry) = self.entries.get(&id) {
                if entry.state.can_start() {
                    self.start(id, engine)?;
                }
            }
        }
        Ok(())
    }

    pub fn stop_all(&mut self) -> Result<(), PluginError> {
        let mut ids: Vec<PluginId> = self.list_ids();
        ids.reverse();
        for id in ids {
            if let Some(entry) = self.entries.get(&id) {
                if entry.state.can_stop() {
                    self.stop(id)?;
                }
            }
        }
        Ok(())
    }

    pub fn dispose_all(&mut self) -> Result<(), PluginError> {
        let mut ids: Vec<PluginId> = self.list_ids();
        ids.reverse();
        for id in ids {
            if let Some(entry) = self.entries.get(&id) {
                if entry.state.can_dispose() {
                    self.dispose(id)?;
                }
            }
        }
        Ok(())
    }

    /// Apply a closure to a mutable plugin instance.
    pub fn with_plugin_mut<F, R>(&mut self, id: PluginId, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn Plugin) -> R,
    {
        let entry = self.entries.get_mut(&id)?;
        Some(f(entry.instance.as_mut()))
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for PluginManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginManager")
            .field("plugins", &self.entries.len())
            .field("loaders", &self.loaders.len())
            .field("registry", &self.registry)
            .finish()
    }
}

/// Extract a human-readable message from a panic payload.
fn extract_panic_message(payload: &Box<dyn std::any::Any + Send>) -> String {
    payload
        .downcast_ref::<String>()
        .map(|s| s.clone())
        .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown panic".to_string())
}