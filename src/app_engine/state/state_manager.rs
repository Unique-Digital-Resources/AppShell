//! Coordinates state operations.
//!
//! ```text
//!       request transition
//!              ↓
//!         StateManager
//!              ↓
//!   StateStore ──→ current state
//!              ↓
//!          validate
//!              ↓
//!       StateTransition
//!              ↓
//!   StateStore ──→ new state
//! ```

use crate::app_engine::errors::state_error::StateError;
use crate::app_engine::state::state::StateId;
use crate::app_engine::state::state_store::StateStore;
use crate::app_engine::state::state_transition::StateTransition;

pub struct StateManager<S> {
    store: StateStore<S>,
}

impl<S: Clone + PartialEq + std::fmt::Debug> StateManager<S> {
    pub fn new() -> Self {
        Self {
            store: StateStore::new(),
        }
    }

    pub fn current(&self, id: StateId) -> Option<S> {
        self.store.read(id)
    }

    pub fn set(&self, id: StateId, state: S) {
        self.store.write(id, state);
    }

    pub fn remove(&self, id: StateId) -> Option<S> {
        self.store.remove(id)
    }

    pub fn contains(&self, id: StateId) -> bool {
        self.store.contains(id)
    }

    /// Apply a transition. Validates that the current state matches `from`.
    pub fn apply_transition(
        &self,
        id: StateId,
        transition: &StateTransition<S>,
    ) -> Result<(), StateError> {
        let current = self.store.read(id).ok_or(StateError::StateNotFound)?;
        if current != *transition.from() {
            return Err(StateError::TransitionMismatch {
                expected: format!("{:?}", transition.from()),
                actual: format!("{:?}", current),
            });
        }
        self.store.write(id, transition.to().clone());
        Ok(())
    }

    /// Convenience: apply a transition constructed from raw `from`/`to` values.
    pub fn transition(&self, id: StateId, from: S, to: S) -> Result<(), StateError> {
        self.apply_transition(id, &StateTransition::new(from, to))
    }

    pub fn store(&self) -> &StateStore<S> {
        &self.store
    }
}

impl<S: Clone + PartialEq + std::fmt::Debug> Default for StateManager<S> {
    fn default() -> Self {
        Self::new()
    }
}