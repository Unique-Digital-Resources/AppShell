//! Errors raised by the scheduling system.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchedulingError {
    JobNotFound { id: u64 },
    JobAlreadyCancelled { id: u64 },
    JobAlreadyCompleted { id: u64 },
    InvalidState {
        id: u64,
        current: String,
        requested: String,
    },
}

impl fmt::Display for SchedulingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SchedulingError::JobNotFound { id } => write!(f, "Job not found: {}", id),
            SchedulingError::JobAlreadyCancelled { id } => {
                write!(f, "Job {} is already cancelled", id)
            }
            SchedulingError::JobAlreadyCompleted { id } => {
                write!(f, "Job {} is already completed", id)
            }
            SchedulingError::InvalidState {
                id,
                current,
                requested,
            } => {
                write!(
                    f,
                    "Job {} invalid state: {} -> {}",
                    id, current, requested
                )
            }
        }
    }
}

impl std::error::Error for SchedulingError {}