//! Errors raised by the task system.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskError {
    NotFound { id: u64 },
    UnknownDefinition { id: String },
    InvalidState {
        id: u64,
        current: String,
        requested: String,
    },
    NotPausable { id: u64, current: String },
    NotCancellable { id: u64, current: String },
    AlreadyRunning { id: u64 },
    AlreadyCompleted { id: u64 },
    ExecutionFailed { id: u64, reason: String },
    Cancelled { id: u64 },
    InvalidDefinition { id: String },
    AlreadyRegistered { id: String },
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskError::NotFound { id } => write!(f, "Task not found: {}", id),
            TaskError::UnknownDefinition { id } => {
                write!(f, "Unknown task definition: {}", id)
            }
            TaskError::InvalidState {
                id,
                current,
                requested,
            } => {
                write!(
                    f,
                    "Task {} invalid state transition: {} -> {}",
                    id, current, requested
                )
            }
            TaskError::NotPausable { id, current } => {
                write!(f, "Task {} is not pausable (current: {})", id, current)
            }
            TaskError::NotCancellable { id, current } => {
                write!(
                    f,
                    "Task {} is not cancellable (current: {})",
                    id, current
                )
            }
            TaskError::AlreadyRunning { id } => write!(f, "Task {} is already running", id),
            TaskError::AlreadyCompleted { id } => {
                write!(f, "Task {} is already completed", id)
            }
            TaskError::ExecutionFailed { id, reason } => {
                write!(f, "Task {} execution failed: {}", id, reason)
            }
            TaskError::Cancelled { id } => write!(f, "Task {} was cancelled", id),
            TaskError::InvalidDefinition { id } => {
                write!(f, "Invalid task definition: {}", id)
            }
            TaskError::AlreadyRegistered { id } => {
                write!(f, "Task definition already registered: {}", id)
            }
        }
    }
}

impl std::error::Error for TaskError {}