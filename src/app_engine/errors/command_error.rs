//! Errors raised by the command system.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
    UnknownCommand { id: String },
    InvalidArguments(String),
    InvalidState,
    NotAvailable(String),
    ExecutionFailed(String),
    Cancelled,
    PermissionDenied(String),
    NoHandler { id: String },
    AlreadyRegistered { id: String },
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::UnknownCommand { id } => {
                write!(f, "Unknown command: {}", id)
            }
            CommandError::InvalidArguments(msg) => {
                write!(f, "Invalid arguments: {}", msg)
            }
            CommandError::InvalidState => write!(f, "Invalid command state"),
            CommandError::NotAvailable(reason) => {
                write!(f, "Command not available: {}", reason)
            }
            CommandError::ExecutionFailed(msg) => {
                write!(f, "Command execution failed: {}", msg)
            }
            CommandError::Cancelled => write!(f, "Command cancelled"),
            CommandError::PermissionDenied(reason) => {
                write!(f, "Permission denied: {}", reason)
            }
            CommandError::NoHandler { id } => {
                write!(f, "No handler registered for command: {}", id)
            }
            CommandError::AlreadyRegistered { id } => {
                write!(f, "Command already registered: {}", id)
            }
        }
    }
}

impl std::error::Error for CommandError {}