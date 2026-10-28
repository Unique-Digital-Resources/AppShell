//! `MemoryStorageBackend` — in-memory storage (preserves current behavior).
//!
//! Useful for tests and applications that don't need persistence.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::app_engine::errors::persistence_error::PersistenceError;
use crate::app_engine::persistence::storage_backend::StorageBackend;

pub struct MemoryStorageBackend {
    data: Mutex<HashMap<String, String>>,
}

impl MemoryStorageBackend {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(HashMap::new()),
        }
    }

    pub fn with_data(data: HashMap<String, String>) -> Self {
        Self {
            data: Mutex::new(data),
        }
    }
}

impl Default for MemoryStorageBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageBackend for MemoryStorageBackend {
    fn load(&self, key: &str) -> Result<Option<String>, PersistenceError> {
        Ok(self.data.lock().unwrap().get(key).cloned())
    }

    fn save(&self, key: &str, value: &str) -> Result<(), PersistenceError> {
        self.data.lock().unwrap().insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn remove(&self, key: &str) -> Result<(), PersistenceError> {
        self.data.lock().unwrap().remove(key);
        Ok(())
    }

    fn exists(&self, key: &str) -> bool {
        self.data.lock().unwrap().contains_key(key)
    }

    fn keys(&self) -> Result<Vec<String>, PersistenceError> {
        Ok(self.data.lock().unwrap().keys().cloned().collect())
    }
}

impl std::fmt::Debug for MemoryStorageBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryStorageBackend")
            .field("count", &self.data.lock().unwrap().len())
            .finish()
    }
}