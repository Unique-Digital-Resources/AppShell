//! Errors raised by the serialization system.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SerializationError {
    NoSerializer { type_name: String },
    SerializationFailed { type_name: String, reason: String },
    DeserializationFailed { type_name: String, reason: String },
    TypeMismatch { expected: String, actual: String },
}

impl fmt::Display for SerializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SerializationError::NoSerializer { type_name } => {
                write!(f, "No serializer registered for type: {}", type_name)
            }
            SerializationError::SerializationFailed { type_name, reason } => {
                write!(f, "Failed to serialize {}: {}", type_name, reason)
            }
            SerializationError::DeserializationFailed { type_name, reason } => {
                write!(f, "Failed to deserialize {}: {}", type_name, reason)
            }
            SerializationError::TypeMismatch { expected, actual } => {
                write!(f, "Type mismatch: expected {}, got {}", expected, actual)
            }
        }
    }
}

impl std::error::Error for SerializationError {}