//! Errors raised by the service system.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceError {
    NotFound { id: u64 },
    InvalidState {
        id: u64,
        current: String,
        requested: String,
    },
    AlreadyRegistered { id: String },
    StartFailed { id: u64, reason: String },
    StopFailed { id: u64, reason: String },
    InitializeFailed { id: u64, reason: String },
    DisposeFailed { id: u64, reason: String },
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceError::NotFound { id } => write!(f, "Service not found: {}", id),
            ServiceError::InvalidState {
                id,
                current,
                requested,
            } => {
                write!(
                    f,
                    "Service {} invalid state transition: {} -> {}",
                    id, current, requested
                )
            }
            ServiceError::AlreadyRegistered { id } => {
                write!(f, "Service already registered: {}", id)
            }
            ServiceError::StartFailed { id, reason } => {
                write!(f, "Service {} failed to start: {}", id, reason)
            }
            ServiceError::StopFailed { id, reason } => {
                write!(f, "Service {} failed to stop: {}", id, reason)
            }
            ServiceError::InitializeFailed { id, reason } => {
                write!(f, "Service {} failed to initialize: {}", id, reason)
            }
            ServiceError::DisposeFailed { id, reason } => {
                write!(f, "Service {} failed to dispose: {}", id, reason)
            }
        }
    }
}

impl std::error::Error for ServiceError {}