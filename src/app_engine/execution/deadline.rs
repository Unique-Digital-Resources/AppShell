//! `Deadline` — a point in time after which an operation should stop.
//!
//! Handlers check `is_expired()` during long loops to implement timeouts.

use std::time::{Duration, Instant};

/// A deadline for an operation.
///
/// If `None`, there is no deadline (the operation can run indefinitely).
#[derive(Debug, Clone)]
pub struct Deadline {
    expiry: Option<Instant>,
}

impl Deadline {
    /// No deadline — the operation can run forever.
    pub fn none() -> Self {
        Self { expiry: None }
    }

    /// Deadline after the given duration from now.
    pub fn after(duration: Duration) -> Self {
        Self {
            expiry: Some(Instant::now() + duration),
        }
    }

    /// Deadline at a specific instant.
    pub fn at(instant: Instant) -> Self {
        Self { expiry: Some(instant) }
    }

    /// Check if the deadline has passed.
    pub fn is_expired(&self) -> bool {
        match self.expiry {
            Some(t) => Instant::now() >= t,
            None => false,
        }
    }

    /// Get the remaining time before the deadline, if any.
    pub fn remaining(&self) -> Option<Duration> {
        let now = Instant::now();
        self.expiry.and_then(|t| {
            if t > now {
                Some(t - now)
            } else {
                None
            }
        })
    }
}

impl Default for Deadline {
    fn default() -> Self {
        Self::none()
    }
}