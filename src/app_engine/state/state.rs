//! Generic state abstraction.
//!
//! `State` is a description of condition — it is **not** the operation that
//! changes the condition.
//!
//! Phase 2 keeps this minimal: a `StateId` for lookup, and generic type
//! parameters on `StateStore`/`StateManager`/`StateTransition` for the
//! concrete state value (e.g. `LifecycleState`).

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StateId(u64);

impl StateId {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl From<u64> for StateId {
    fn from(v: u64) -> Self {
        Self::new(v)
    }
}