//! Errors raised by the plugin security system.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginSecurityError {
    PermissionDenied {
        plugin_id: String,
        permission: String,
    },
    InsufficientPermissions {
        plugin_id: String,
        required: u32,
        actual: u32,
    },
}

impl fmt::Display for PluginSecurityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginSecurityError::PermissionDenied {
                plugin_id,
                permission,
            } => {
                write!(
                    f,
                    "Plugin '{}' lacks permission: {}",
                    plugin_id, permission
                )
            }
            PluginSecurityError::InsufficientPermissions {
                plugin_id,
                required,
                actual,
            } => {
                write!(
                    f,
                    "Plugin '{}' has insufficient permissions (has {:b}, needs {:b})",
                    plugin_id, actual, required
                )
            }
        }
    }
}

impl std::error::Error for PluginSecurityError {}