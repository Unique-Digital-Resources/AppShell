//! Errors raised by the plugin system.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginError {
    NotFound { id: u64 },
    InvalidState {
        id: u64,
        current: String,
        requested: String,
    },
    AlreadyRegistered { id: String },
    LoadFailed { source: String, reason: String },
    InitializeFailed { id: u64, reason: String },
    StartFailed { id: u64, reason: String },
    StopFailed { id: u64, reason: String },
    DisposeFailed { id: u64, reason: String },
    DependencyNotMet { id: String, dependency: String },
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginError::NotFound { id } => write!(f, "Plugin not found: {}", id),
            PluginError::InvalidState {
                id,
                current,
                requested,
            } => {
                write!(
                    f,
                    "Plugin {} invalid state transition: {} -> {}",
                    id, current, requested
                )
            }
            PluginError::AlreadyRegistered { id } => {
                write!(f, "Plugin already registered: {}", id)
            }
            PluginError::LoadFailed { source, reason } => {
                write!(f, "Failed to load plugin '{}': {}", source, reason)
            }
            PluginError::InitializeFailed { id, reason } => {
                write!(f, "Plugin {} failed to initialize: {}", id, reason)
            }
            PluginError::StartFailed { id, reason } => {
                write!(f, "Plugin {} failed to start: {}", id, reason)
            }
            PluginError::StopFailed { id, reason } => {
                write!(f, "Plugin {} failed to stop: {}", id, reason)
            }
            PluginError::DisposeFailed { id, reason } => {
                write!(f, "Plugin {} failed to dispose: {}", id, reason)
            }
            PluginError::DependencyNotMet { id, dependency } => {
                write!(
                    f,
                    "Plugin '{}' requires dependency '{}' which is not registered",
                    id, dependency
                )
            }
        }
    }
}

impl std::error::Error for PluginError {}