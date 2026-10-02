//! `RetryPolicy` — defines how failed tasks are retried.
//!
//! Retry creates new job occurrences in the scheduler, rather than
//! re-running the same task instance.

use std::time::Duration;

/// Policy for retrying failed tasks.
#[derive(Debug, Clone, PartialEq)]
pub enum RetryPolicy {
    /// No retry — the task runs once.
    None,

    /// Retry with a fixed delay between attempts.
    Fixed {
        max_attempts: u32,
        delay: Duration,
    },

    /// Retry with exponential backoff.
    Exponential {
        max_attempts: u32,
        base_delay: Duration,
        factor: f64,
    },
}

impl RetryPolicy {
    /// Default: no retry.
    pub fn none() -> Self {
        RetryPolicy::None
    }

    /// Fixed retry with the given max attempts and delay.
    pub fn fixed(max_attempts: u32, delay: Duration) -> Self {
        RetryPolicy::Fixed {
            max_attempts,
            delay,
        }
    }

    /// Exponential backoff retry.
    pub fn exponential(max_attempts: u32, base_delay: Duration, factor: f64) -> Self {
        RetryPolicy::Exponential {
            max_attempts,
            base_delay,
            factor,
        }
    }

    /// Whether this policy enables retry at all.
    pub fn should_retry(&self) -> bool {
        !matches!(self, RetryPolicy::None)
    }

    /// Get the delay before the next attempt (0-based attempt index).
    pub fn delay_for_attempt(&self, attempt: u32) -> Option<Duration> {
        match self {
            RetryPolicy::None => None,
            RetryPolicy::Fixed {
                max_attempts,
                delay,
            } => {
                if attempt < *max_attempts {
                    Some(*delay)
                } else {
                    None
                }
            }
            RetryPolicy::Exponential {
                max_attempts,
                base_delay,
                factor,
            } => {
                if attempt < *max_attempts {
                    let multiplier = factor.powi(attempt as i32);
                    let delay_ms = (base_delay.as_millis() as f64 * multiplier) as u64;
                    Some(Duration::from_millis(delay_ms))
                } else {
                    None
                }
            }
        }
    }

    /// Get the max number of attempts (including the first).
    pub fn max_attempts(&self) -> u32 {
        match self {
            RetryPolicy::None => 1,
            RetryPolicy::Fixed { max_attempts, .. } => max_attempts + 1,
            RetryPolicy::Exponential { max_attempts, .. } => max_attempts + 1,
        }
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        RetryPolicy::None
    }
}

impl std::fmt::Display for RetryPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RetryPolicy::None => write!(f, "NoRetry"),
            RetryPolicy::Fixed {
                max_attempts,
                delay,
            } => {
                write!(f, "Fixed(max={}, delay={:?})", max_attempts, delay)
            }
            RetryPolicy::Exponential {
                max_attempts,
                base_delay,
                factor,
            } => {
                write!(
                    f,
                    "Exponential(max={}, base={:?}, factor={:.1})",
                    max_attempts, base_delay, factor
                )
            }
        }
    }
}