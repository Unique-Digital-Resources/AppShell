//! `PluginManifest` — describes a plugin without loading its implementation.

use std::collections::HashMap;
use std::fmt;

use crate::app_engine::metadata::version::Version;
use crate::app_engine::plugins::plugin_permissions::PluginPermissions;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginManifest {
    id: String,
    name: String,
    version: String,
    description: String,
    author: String,
    dependencies: Vec<String>,
    capabilities: Vec<String>,
    permissions: PluginPermissions,
    engine_api_version: Version,
    metadata: HashMap<String, String>,
}

impl PluginManifest {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: "0.1.0".to_string(),
            description: String::new(),
            author: String::new(),
            dependencies: Vec::new(),
            capabilities: Vec::new(),
            permissions: PluginPermissions::FULL,
            engine_api_version: Version::new(1, 0, 0),
            metadata: HashMap::new(),
        }
    }

    pub fn id(&self) -> &str { &self.id }
    pub fn name(&self) -> &str { &self.name }
    pub fn version(&self) -> &str { &self.version }
    pub fn description(&self) -> &str { &self.description }
    pub fn author(&self) -> &str { &self.author }
    pub fn dependencies(&self) -> &[String] { &self.dependencies }
    pub fn capabilities(&self) -> &[String] { &self.capabilities }
    pub fn permissions(&self) -> PluginPermissions { self.permissions }
    pub fn engine_api_version(&self) -> &Version { &self.engine_api_version }
    pub fn metadata(&self) -> &HashMap<String, String> { &self.metadata }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = author.into();
        self
    }

    pub fn with_dependency(mut self, dependency: impl Into<String>) -> Self {
        self.dependencies.push(dependency.into());
        self
    }

    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    pub fn with_permissions(mut self, permissions: PluginPermissions) -> Self {
        self.permissions = permissions;
        self
    }

    pub fn with_engine_api_version(mut self, version: Version) -> Self {
        self.engine_api_version = version;
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Check if this plugin is compatible with the given engine API version.
    pub fn is_compatible_with(&self, engine_version: &Version) -> bool {
        self.engine_api_version.is_compatible_with(engine_version)
    }
}

impl fmt::Display for PluginManifest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} v{} ({})", self.id, self.version, self.name)
    }
}