//! In-memory state store.
//!
//! Phase 2 only needs an in-memory implementation. Later phases can add
//! `FileStateStore` or `PersistentStateStore` behind the same API.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::app_engine::state::state::StateId;

pub struct StateStore<S> {
    states: Mutex<HashMap<StateId, S>>,
}

impl<S: Clone> StateStore<S> {
    pub fn new() -> Self {
        Self {
            states: Mutex::new(HashMap::new()),
        }
    }

    pub fn read(&self, id: StateId) -> Option<S> {
        self.states.lock().unwrap().get(&id).cloned()
    }

    pub fn write(&self, id: StateId, state: S) {
        self.states.lock().unwrap().insert(id, state);
    }

    pub fn remove(&self, id: StateId) -> Option<S> {
        self.states.lock().unwrap().remove(&id)
    }

    pub fn contains(&self, id: StateId) -> bool {
        self.states.lock().unwrap().contains_key(&id)
    }

    pub fn clear(&self) {
        self.states.lock().unwrap().clear();
    }

    pub fn len(&self) -> usize {
        self.states.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.states.lock().unwrap().is_empty()
    }
}

impl<S: Clone> Default for StateStore<S> {
    fn default() -> Self {
        Self::new()
    }
}