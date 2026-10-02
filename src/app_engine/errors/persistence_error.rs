//! Errors raised by the persistence system.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistenceError {
    IoError { reason: String },
    SerializationError { reason: String },
    DeserializationError { reason: String },
    KeyNotFound { key: String },
    BackendNotAvailable,
}

impl fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PersistenceError::IoError { reason } => {
                write!(f, "I/O error: {}", reason)
            }
            PersistenceError::SerializationError { reason } => {
                write!(f, "Serialization error: {}", reason)
            }
            PersistenceError::DeserializationError { reason } => {
                write!(f, "Deserialization error: {}", reason)
            }
            PersistenceError::KeyNotFound { key } => {
                write!(f, "Key not found: {}", key)
            }
            PersistenceError::BackendNotAvailable => {
                write!(f, "No persistence backend available")
            }
        }
    }
}

impl std::error::Error for PersistenceError {}

impl From<std::io::Error> for PersistenceError {
    fn from(e: std::io::Error) -> Self {
        PersistenceError::IoError {
            reason: e.to_string(),
        }
    }
}