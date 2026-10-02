//! `PluginRegistry` — discovery of registered plugins.

use std::collections::HashMap;

use crate::app_engine::plugins::plugin_manifest::PluginManifest;

/// Stores plugin manifests for discovery.
pub struct PluginRegistry {
    manifests: HashMap<String, PluginManifest>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            manifests: HashMap::new(),
        }
    }

    pub fn register(
        &mut self,
        manifest: PluginManifest,
    ) -> Result<(), crate::app_engine::errors::plugin_error::PluginError> {
        if self.manifests.contains_key(manifest.id()) {
            return Err(
                crate::app_engine::errors::plugin_error::PluginError::AlreadyRegistered {
                    id: manifest.id().to_string(),
                },
            );
        }
        self.manifests
            .insert(manifest.id().to_string(), manifest);
        Ok(())
    }

    pub fn unregister(&mut self, id: &str) -> Option<PluginManifest> {
        self.manifests.remove(id)
    }

    pub fn get(&self, id: &str) -> Option<&PluginManifest> {
        self.manifests.get(id)
    }

    pub fn contains(&self, id: &str) -> bool {
        self.manifests.contains_key(id)
    }

    pub fn list(&self) -> Vec<&PluginManifest> {
        self.manifests.values().collect()
    }

    pub fn len(&self) -> usize {
        self.manifests.len()
    }

    pub fn is_empty(&self) -> bool {
        self.manifests.is_empty()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for PluginRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginRegistry")
            .field("count", &self.manifests.len())
            .finish()
    }
}