//! Errors raised by the configuration system.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigurationError {
    ValidationFailed { key: String, reason: String },
    NotFound { key: String },
    InvalidType { key: String, expected: String, actual: String },
    LoadFailed { reason: String },
    SaveFailed { reason: String },
}

impl fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigurationError::ValidationFailed { key, reason } => {
                write!(f, "Configuration validation failed for '{}': {}", key, reason)
            }
            ConfigurationError::NotFound { key } => {
                write!(f, "Configuration key not found: {}", key)
            }
            ConfigurationError::InvalidType {
                key,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Configuration '{}' has invalid type: expected {}, got {}",
                    key, expected, actual
                )
            }
            ConfigurationError::LoadFailed { reason } => {
                write!(f, "Configuration load failed: {}", reason)
            }
            ConfigurationError::SaveFailed { reason } => {
                write!(f, "Configuration save failed: {}", reason)
            }
        }
    }
}

impl std::error::Error for ConfigurationError {}