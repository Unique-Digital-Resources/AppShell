//! `DynamicPluginLoader` — loads plugins from shared libraries (`.so`/`.dll`).
//!
//! This is a contract stub. Actual dynamic loading requires the `libloading`
//! crate and an `unsafe` FFI call to an exported `create_plugin()` function.
//! The application adds the `libloading` dependency and implements the real
//! loading logic.
//!
//! The contract for dynamic plugins:
//! 1. The shared library exports a function `create_plugin() -> Box<dyn Plugin>`
//! 2. The shared library exports `create_manifest() -> PluginManifest`
//! 3. The loader calls both, verifies the manifest, and returns them

use std::path::PathBuf;

use crate::app_engine::errors::plugin_error::PluginError;
use crate::app_engine::plugins::plugin::Plugin;
use crate::app_engine::plugins::plugin_manifest::PluginManifest;
use crate::app_engine::plugins::plugin_loader::PluginLoader;

/// A loader that finds plugins from shared libraries.
///
/// This stub demonstrates the contract. To implement real loading:
/// 1. Add `libloading` to Cargo.toml
/// 2. Use `unsafe` to call the exported `create_plugin()` symbol
/// 3. Keep the `Library` alive (leak it or store it in the manager)
pub struct DynamicPluginLoader {
    search_paths: Vec<PathBuf>,
}

impl DynamicPluginLoader {
    pub fn new() -> Self {
        Self {
            search_paths: Vec::new(),
        }
    }

    pub fn with_search_paths(paths: Vec<PathBuf>) -> Self {
        Self { search_paths: paths }
    }

    pub fn add_search_path(&mut self, path: impl Into<PathBuf>) {
        self.search_paths.push(path.into());
    }

    /// Check if a file path looks like a shared library.
    pub fn is_shared_library(source: &str) -> bool {
        source.ends_with(".so") || source.ends_with(".dll") || source.ends_with(".dylib")
    }

    pub fn search_paths_len(&self) -> usize {
        self.search_paths.len()
    }

    /// Resolve a source string to a full path by checking search paths.
    fn resolve_path(&self, source: &str) -> Option<PathBuf> {
        // If it's an absolute path, use it directly
        let path = std::path::Path::new(source);
        if path.is_absolute() && path.exists() {
            return Some(path.to_path_buf());
        }

        // Check each search path
        for base in &self.search_paths {
            let full = base.join(source);
            if full.exists() {
                return Some(full);
            }
        }

        None
    }
}

impl Default for DynamicPluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginLoader for DynamicPluginLoader {
    fn can_load(&self, source: &str) -> bool {
        Self::is_shared_library(source) && self.resolve_path(source).is_some()
    }

    fn load(&self, source: &str) -> Result<(PluginManifest, Box<dyn Plugin>), PluginError> {
        let path = self.resolve_path(source).ok_or_else(|| PluginError::LoadFailed {
            source: source.to_string(),
            reason: "shared library not found in search paths".to_string(),
        })?;

        // Real implementation would use libloading:
        // ```
        // let lib = unsafe { libloading::Library::new(&path) }
        //     .map_err(|e| PluginError::LoadFailed {
        //         source: source.to_string(),
        //         reason: e.to_string(),
        //     })?;
        // let create_plugin = unsafe {
        //     lib.get::<unsafe fn() -> Box<dyn Plugin>>(b"create_plugin")
        // }.map_err(|e| PluginError::LoadFailed { ... })?;
        // let plugin = unsafe { create_plugin() };
        // let manifest = plugin.manifest().clone();
        // std::mem::forget(lib); // Keep library alive (leak)
        // Ok((manifest, plugin))
        // ```

        Err(PluginError::LoadFailed {
            source: source.to_string(),
            reason: format!(
                "Dynamic loading requires the 'libloading' crate. Found library at {:?} but cannot load.",
                path
            ),
        })
    }
}

impl std::fmt::Debug for DynamicPluginLoader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynamicPluginLoader")
            .field("search_paths", &self.search_paths)
            .finish()
    }
}