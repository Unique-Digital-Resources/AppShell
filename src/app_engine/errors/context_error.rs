//! Errors raised by the context system.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextError {
    NoApplicationContext,
    SessionNotFound,
    ExecutionNotFound,
}

impl fmt::Display for ContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContextError::NoApplicationContext => write!(f, "No application context exists"),
            ContextError::SessionNotFound => write!(f, "Session not found"),
            ContextError::ExecutionNotFound => write!(f, "Execution not found"),
        }
    }
}

impl std::error::Error for ContextError {}