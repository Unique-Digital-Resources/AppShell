//! Errors raised by the state system.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateError {
    StateNotFound,
    TransitionMismatch { expected: String, actual: String },
    InvalidTransition,
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateError::StateNotFound => write!(f, "State not found in store"),
            StateError::TransitionMismatch { expected, actual } => write!(
                f,
                "State transition mismatch: expected {}, got {}",
                expected, actual
            ),
            StateError::InvalidTransition => write!(f, "Invalid state transition"),
        }
    }
}

impl std::error::Error for StateError {}