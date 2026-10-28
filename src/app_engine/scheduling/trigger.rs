//! `Trigger` — defines conditions that cause a scheduled job to become executable.
//!
//! A `Schedule` is primarily time-based. A `Trigger` is primarily
//! condition/event-based.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Trigger {
    /// No trigger condition — just use the schedule.
    Manual,
    /// Triggered when an event with the given name occurs.
    OnEvent(String),
    /// Triggered when a signal with the given name is emitted.
    OnSignal(String),
    /// Triggered when a condition with the given name becomes true.
    OnCondition(String),
}

impl Trigger {
    pub fn is_manual(&self) -> bool {
        matches!(self, Trigger::Manual)
    }

    pub fn name(&self) -> &str {
        match self {
            Trigger::Manual => "manual",
            Trigger::OnEvent(n) => n,
            Trigger::OnSignal(n) => n,
            Trigger::OnCondition(n) => n,
        }
    }
}

impl fmt::Display for Trigger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Trigger::Manual => write!(f, "Manual"),
            Trigger::OnEvent(n) => write!(f, "OnEvent({})", n),
            Trigger::OnSignal(n) => write!(f, "OnSignal({})", n),
            Trigger::OnCondition(n) => write!(f, "OnCondition({})", n),
        }
    }
}