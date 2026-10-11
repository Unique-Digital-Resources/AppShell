//! `PluginLoader` — defines how a plugin implementation is obtained.
//!
//! The loader is responsible for **acquisition**, not global plugin management.
//!
//! Possible future implementations: static Rust plugins, dynamic libraries,
//! WebAssembly, scripts, package-based plugins. Phase 10 keeps the contract
//! independent of the packaging mechanism.

use crate::app_engine::errors::plugin_error::PluginError;
use crate::app_engine::plugins::plugin::Plugin;
use crate::app_engine::plugins::plugin_manifest::PluginManifest;

/// Implemented by anything that can load plugins.
pub trait PluginLoader: Send + Sync {
    /// Whether this loader can handle the given source string.
    fn can_load(&self, source: &str) -> bool;

    /// Load a plugin from the given source. Returns a manifest + instance.
    fn load(&self, source: &str) -> Result<(PluginManifest, Box<dyn Plugin>), PluginError>;
}