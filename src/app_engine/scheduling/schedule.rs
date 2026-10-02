//! `Schedule` — describes **when** a job should become eligible for execution.

use std::fmt;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
pub enum Schedule {
    /// Execute as soon as possible.
    Immediate,
    /// Execute after the specified delay from job creation.
    Delayed(Duration),
    /// Execute at or after the specified instant.
    AtTime(Instant),
    /// Execute repeatedly at the given interval.
    Interval(Duration),
}

impl Schedule {
    /// Check if the time-based portion of this schedule is eligible.
    pub fn is_eligible(&self, created_at: Instant, now: Instant) -> bool {
        match self {
            Schedule::Immediate => true,
            Schedule::Delayed(d) => now >= created_at + *d,
            Schedule::AtTime(t) => now >= *t,
            Schedule::Interval(d) => now >= created_at + *d,
        }
    }

    /// For recurring schedules, compute the next occurrence after `last`.
    pub fn next_occurrence(&self, last: Instant) -> Option<Instant> {
        match self {
            Schedule::Interval(d) => Some(last + *d),
            _ => None,
        }
    }

    pub fn is_recurring(&self) -> bool {
        matches!(self, Schedule::Interval(_))
    }

    pub fn is_immediate(&self) -> bool {
        matches!(self, Schedule::Immediate)
    }
}

impl fmt::Display for Schedule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Schedule::Immediate => write!(f, "Immediate"),
            Schedule::Delayed(d) => write!(f, "Delayed({:?})", d),
            Schedule::AtTime(_) => write!(f, "AtTime"),
            Schedule::Interval(d) => write!(f, "Interval({:?})", d),
        }
    }
}