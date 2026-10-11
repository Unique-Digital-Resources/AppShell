//! `Preferences` — user-facing configuration.
//!
//! Phase 23: now thread-safe using Mutex.

use std::collections::HashMap;
use std::sync::Mutex;
use std::fmt;

pub struct Preferences {
    values: Mutex<HashMap<String, String>>,
}

impl Preferences {
    pub fn new() -> Self {
        Self {
            values: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.values.lock().unwrap().get(key).map(|s| s.to_string())
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        self.values.lock().unwrap().get(key).cloned()
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.values.lock().unwrap().get(key).and_then(|s| match s.as_str() {
            "true" | "1" | "yes" => Some(true),
            "false" | "0" | "no" => Some(false),
            _ => None,
        })
    }

    pub fn set(&self, key: impl Into<String>, value: impl Into<String>) {
        self.values.lock().unwrap().insert(key.into(), value.into());
    }

    pub fn remove(&self, key: &str) -> Option<String> {
        self.values.lock().unwrap().remove(key)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.values.lock().unwrap().contains_key(key)
    }

    pub fn keys(&self) -> Vec<String> {
        self.values.lock().unwrap().keys().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.values.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.lock().unwrap().is_empty()
    }

    pub fn clear(&self) {
        self.values.lock().unwrap().clear();
    }
}

impl Default for Preferences {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Preferences {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Preferences({} keys)", self.values.lock().unwrap().len())
    }
}