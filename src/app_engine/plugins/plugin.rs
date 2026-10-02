//! `Plugin` — an externally defined extension unit that participates in the
//! App Engine through established contracts.
//!
//! A plugin can provide commands, tasks, services, resources, event handlers,
//! and signal subscriptions. The App Engine provides the mechanisms; the plugin
//! supplies the implementations.

//! `Plugin` — an externally defined extension unit.

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::app_engine::capabilities::engine_ref::EngineRef;
use crate::app_engine::errors::plugin_error::PluginError;
use crate::app_engine::plugins::plugin_context::PluginContext;

static PLUGIN_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Numeric identifier assigned by the PluginManager when a plugin is registered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PluginId(u64);

impl PluginId {
    pub fn new() -> Self {
        Self(PLUGIN_ID_COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Default for PluginId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for PluginId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Plugin({})", self.0)
    }
}

/// Lifecycle states of a plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PluginState {
    Discovered,
    Loaded,
    Initialized,
    Started,
    Stopped,
    Unloaded,
}

impl PluginState {
    pub fn can_initialize(&self) -> bool {
        matches!(self, PluginState::Loaded)
    }

    pub fn can_start(&self) -> bool {
        matches!(self, PluginState::Initialized | PluginState::Stopped)
    }

    pub fn can_stop(&self) -> bool {
        matches!(self, PluginState::Started)
    }

    pub fn can_dispose(&self) -> bool {
        matches!(
            self,
            PluginState::Stopped | PluginState::Loaded | PluginState::Discovered
        )
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, PluginState::Unloaded)
    }

    pub fn is_active(&self) -> bool {
        matches!(self, PluginState::Started)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            PluginState::Discovered => "Discovered",
            PluginState::Loaded => "Loaded",
            PluginState::Initialized => "Initialized",
            PluginState::Started => "Started",
            PluginState::Stopped => "Stopped",
            PluginState::Unloaded => "Unloaded",
        }
    }
}

impl fmt::Display for PluginState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The contract implemented by concrete plugins.
///
/// A plugin participates in the existing lifecycle — it does **not** invent
/// an independent application lifecycle.
pub trait Plugin: Send + Sync {
    /// The semantic ID of this plugin (e.g. "image-plugin").
    fn id(&self) -> &str;

    /// Called when the plugin is initialized.
    fn initialize(&mut self, context: &PluginContext, engine: &EngineRef) -> Result<(), PluginError>;

    /// Called when the plugin starts.
    fn start(&mut self, engine: &EngineRef) -> Result<(), PluginError>;

    /// Called when the plugin stops.
    fn stop(&mut self) -> Result<(), PluginError>;

    /// Called when the plugin is disposed.
    fn dispose(&mut self) -> Result<(), PluginError>;
}

/// Internal wrapper that tracks a plugin instance + its state.
pub(crate) struct PluginEntry {
    pub id: PluginId,
    pub state: PluginState,
    pub manifest: crate::app_engine::plugins::plugin_manifest::PluginManifest,
    pub instance: Box<dyn Plugin>,
}

impl PluginEntry {
    pub fn new(
        manifest: crate::app_engine::plugins::plugin_manifest::PluginManifest,
        instance: Box<dyn Plugin>,
    ) -> Self {
        Self {
            id: PluginId::new(),
            state: PluginState::Loaded,
            manifest,
            instance,
        }
    }
}

impl fmt::Debug for PluginEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PluginEntry")
            .field("id", &self.id)
            .field("state", &self.state)
            .field("manifest", &self.manifest)
            .finish()
    }
}