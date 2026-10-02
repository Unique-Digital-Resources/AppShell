//! Errors raised by the resource system.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceError {
    NotFound { id: u64 },
    LoaderNotFound { resource_type: String },
    LoadFailed { source: String, reason: String },
    InvalidResource { id: u64 },
    AlreadyLoaded { id: u64 },
    AlreadyReleased { id: u64 },
    UnloadFailed { id: u64, reason: String },
    InvalidHandle { id: u64 },
    NoData { id: u64 },
    ReloadFailed { id: u64, reason: String },
    InUse { id: u64 },
}

impl fmt::Display for ResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceError::NotFound { id } => write!(f, "Resource not found: {}", id),
            ResourceError::LoaderNotFound { resource_type } => {
                write!(f, "No loader for resource type: {}", resource_type)
            }
            ResourceError::LoadFailed { source, reason } => {
                write!(f, "Failed to load resource '{}': {}", source, reason)
            }
            ResourceError::InvalidResource { id } => {
                write!(f, "Invalid resource: {}", id)
            }
            ResourceError::AlreadyLoaded { id } => {
                write!(f, "Resource {} is already loaded", id)
            }
            ResourceError::AlreadyReleased { id } => {
                write!(f, "Resource {} is already released", id)
            }
            ResourceError::UnloadFailed { id, reason } => {
                write!(f, "Failed to unload resource {}: {}", id, reason)
            }
            ResourceError::InvalidHandle { id } => {
                write!(f, "Invalid resource handle: {}", id)
            }
            ResourceError::NoData { id } => {
                write!(f, "Resource {} has no data", id)
            }
            ResourceError::ReloadFailed { id, reason } => {
                write!(f, "Failed to reload resource {}: {}", id, reason)
            }
            ResourceError::InUse { id } => {
                write!(f, "Resource {} is in use and cannot be unloaded", id)
            }
        }
    }
}

impl std::error::Error for ResourceError {}