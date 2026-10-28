//! `PermissionCheckedEngineRef` — wraps `EngineRef` with permission checks.
//!
//! Provides `try_*` methods that check `PluginPermissions` before allowing
//! access to restricted operations. Read-only access is always allowed.
//!
//! The wrapper is available for domain plugins that want to respect
//! permissions. The `Plugin` trait still receives `&EngineRef` directly
//! for backward compatibility — this wrapper is an optional utility.

use crate::app_engine::capabilities::engine_ref::EngineRef;
use crate::app_engine::configuration::config_store::ConfigStore;
use crate::app_engine::configuration::preferences::Preferences;
use crate::app_engine::errors::plugin_security_error::PluginSecurityError;
use crate::app_engine::events::event_bus::EventBus;
use crate::app_engine::history::history_store::HistoryStore;
use crate::app_engine::lifecycle::lifecycle_state::LifecycleState;
use crate::app_engine::plugins::plugin_permissions::PluginPermissions;
use crate::app_engine::resources::resource_manager::ResourceManager;
use crate::app_engine::scheduling::scheduler::Scheduler;
use crate::app_engine::state::state_manager::StateManager;
use crate::app_engine::commands::command_executor::CommandExecutor;
use crate::app_engine::signals::signal_bus::SignalBus;

/// Wraps `EngineRef` with plugin permission checks.
///
/// Provides `try_*` methods for restricted operations (returns `Result`).
/// Read-only access methods are always allowed.
pub struct PermissionCheckedEngineRef<'a> {
    engine: &'a EngineRef<'a>,
    permissions: PluginPermissions,
    plugin_id: String,
}

impl<'a> PermissionCheckedEngineRef<'a> {
    pub fn new(
        engine: &'a EngineRef<'a>,
        permissions: PluginPermissions,
        plugin_id: &str,
    ) -> Self {
        Self {
            engine,
            permissions,
            plugin_id: plugin_id.to_string(),
        }
    }

    pub fn plugin_id(&self) -> &str {
        &self.plugin_id
    }

    pub fn permissions(&self) -> PluginPermissions {
        self.permissions
    }

    // ----- Read-only access (always allowed) -----

    pub fn config_store(&self) -> &ConfigStore {
        self.engine.config_store
    }

    pub fn preferences(&self) -> &Preferences {
        self.engine.preferences
    }

    pub fn state_manager(&self) -> &StateManager<LifecycleState> {
        self.engine.state_manager
    }

    pub fn history_store(&self) -> &HistoryStore {
        self.engine.history_store
    }

    pub fn scheduler(&self) -> &Scheduler {
        self.engine.scheduler
    }

    pub fn event_bus(&self) -> &EventBus {
        self.engine.event_bus
    }

    pub fn signal_bus(&self) -> &SignalBus {
        self.engine.signal_bus
    }

    pub fn command_executor(&self) -> &CommandExecutor {
        self.engine.command_executor
    }

    pub fn resource_manager(&self) -> &ResourceManager {
        self.engine.resource_manager
    }

    // ----- Permission-checked write access -----

    /// Write to config. Requires WRITE_CONFIG permission.
    pub fn try_write_config(&self) -> Result<&ConfigStore, PluginSecurityError> {
        if self.permissions.can_write_config() {
            Ok(self.engine.config_store)
        } else {
            Err(PluginSecurityError::PermissionDenied {
                plugin_id: self.plugin_id.clone(),
                permission: "WRITE_CONFIG".to_string(),
            })
        }
    }

    /// Load resources. Requires LOAD_RESOURCES permission.
    pub fn try_load_resource(&self) -> Result<&ResourceManager, PluginSecurityError> {
        if self.permissions.can_load_resources() {
            Ok(self.engine.resource_manager)
        } else {
            Err(PluginSecurityError::PermissionDenied {
                plugin_id: self.plugin_id.clone(),
                permission: "LOAD_RESOURCES".to_string(),
            })
        }
    }

    /// Publish events. Requires PUBLISH_EVENTS permission.
    pub fn try_publish_event(&self) -> Result<&EventBus, PluginSecurityError> {
        if self.permissions.can_publish_events() {
            Ok(self.engine.event_bus)
        } else {
            Err(PluginSecurityError::PermissionDenied {
                plugin_id: self.plugin_id.clone(),
                permission: "PUBLISH_EVENTS".to_string(),
            })
        }
    }

    /// Subscribe to events. Requires SUBSCRIBE_EVENTS permission.
    pub fn try_subscribe_events(&self) -> Result<&EventBus, PluginSecurityError> {
        if self.permissions.can_subscribe_events() {
            Ok(self.engine.event_bus)
        } else {
            Err(PluginSecurityError::PermissionDenied {
                plugin_id: self.plugin_id.clone(),
                permission: "SUBSCRIBE_EVENTS".to_string(),
            })
        }
    }

    /// Register commands. Requires REGISTER_COMMANDS permission.
    pub fn try_register_commands(&self) -> Result<&CommandExecutor, PluginSecurityError> {
        if self.permissions.can_register_commands() {
            Ok(self.engine.command_executor)
        } else {
            Err(PluginSecurityError::PermissionDenied {
                plugin_id: self.plugin_id.clone(),
                permission: "REGISTER_COMMANDS".to_string(),
            })
        }
    }

    /// Register tasks. Requires REGISTER_TASKS permission.
    pub fn try_register_tasks(&self) -> Result<&CommandExecutor, PluginSecurityError> {
        if self.permissions.can_register_tasks() {
            Ok(self.engine.command_executor)
        } else {
            Err(PluginSecurityError::PermissionDenied {
                plugin_id: self.plugin_id.clone(),
                permission: "REGISTER_TASKS".to_string(),
            })
        }
    }

    /// Register services. Requires REGISTER_SERVICES permission.
    pub fn try_register_services(&self) -> Result<&CommandExecutor, PluginSecurityError> {
        if self.permissions.can_register_services() {
            Ok(self.engine.command_executor)
        } else {
            Err(PluginSecurityError::PermissionDenied {
                plugin_id: self.plugin_id.clone(),
                permission: "REGISTER_SERVICES".to_string(),
            })
        }
    }
}

impl<'a> std::fmt::Debug for PermissionCheckedEngineRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PermissionCheckedEngineRef")
            .field("plugin_id", &self.plugin_id)
            .field("permissions", &self.permissions)
            .finish()
    }
}