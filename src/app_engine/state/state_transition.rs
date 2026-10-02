//! Represents a change from one state to another.
//!
//! ```text
//! previous state
//!       │
//!       ▼
//!  transition
//!       │
//!       ▼
//!  next state
//! ```
//!
//! A `State` is the current condition; a `StateTransition` is the change
//! between conditions.

#[derive(Debug, Clone, PartialEq)]
pub struct StateTransition<S> {
    from: S,
    to: S,
}

impl<S> StateTransition<S> {
    pub fn new(from: S, to: S) -> Self {
        Self { from, to }
    }

    pub fn from(&self) -> &S {
        &self.from
    }

    pub fn to(&self) -> &S {
        &self.to
    }
}

impl<S: PartialEq> StateTransition<S> {
    /// A no-op transition (from == to).
    pub fn is_no_op(&self) -> bool {
        self.from == self.to
    }
}