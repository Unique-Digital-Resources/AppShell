//! `PluginContext` — the controlled environment through which a plugin
//! accesses the App Engine.
//!
//! This creates an explicit extension boundary: instead of giving a plugin
//! unrestricted access to the entire runtime, the plugin receives a context
//! through which it can interact with engine systems.
//!
//! ✓ plugin identity, engine metadata
//! ✗ direct access to every internal field
//!
//! In Phase 10, the context is minimal. Later phases can extend it to provide
//! controlled access to commands, tasks, events, resources, etc.

//! `PluginContext` — the controlled environment through which a plugin
//! accesses the App Engine.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PluginContext {
    plugin_id: String,
    metadata: HashMap<String, String>,
}

impl PluginContext {
    pub fn new(plugin_id: impl Into<String>) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            metadata: HashMap::new(),
        }
    }

    pub fn plugin_id(&self) -> &str {
        &self.plugin_id
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn with_metadata(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}