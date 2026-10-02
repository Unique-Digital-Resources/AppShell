//! Errors raised by the async runtime.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AsyncError {
    RuntimeCreationFailed(String),
    TaskJoinError(String),
    AsyncExecutionFailed(String),
}

impl fmt::Display for AsyncError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AsyncError::RuntimeCreationFailed(msg) => {
                write!(f, "Async runtime creation failed: {}", msg)
            }
            AsyncError::TaskJoinError(msg) => {
                write!(f, "Async task join error: {}", msg)
            }
            AsyncError::AsyncExecutionFailed(msg) => {
                write!(f, "Async execution failed: {}", msg)
            }
        }
    }
}

impl std::error::Error for AsyncError {}

impl From<tokio::task::JoinError> for AsyncError {
    fn from(e: tokio::task::JoinError) -> Self {
        AsyncError::TaskJoinError(e.to_string())
    }
}