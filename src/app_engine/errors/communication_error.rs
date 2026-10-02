//! Errors raised by the events/signals communication system.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommunicationError {
    HandlerNotFound { id: u64 },
    SubscriptionNotFound { id: u64 },
    AlreadySubscribed { topic: String },
}

impl fmt::Display for CommunicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommunicationError::HandlerNotFound { id } => {
                write!(f, "Event handler not found: {}", id)
            }
            CommunicationError::SubscriptionNotFound { id } => {
                write!(f, "Signal subscription not found: {}", id)
            }
            CommunicationError::AlreadySubscribed { topic } => {
                write!(f, "Already subscribed to: {}", topic)
            }
        }
    }
}

impl std::error::Error for CommunicationError {}