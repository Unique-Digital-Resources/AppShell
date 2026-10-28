//! `StorageBackend` — trait for persistence backends.
//!
//! Provides a generic key-value storage interface that can be backed by
//! files, databases, or in-memory maps.

use crate::app_engine::errors::persistence_error::PersistenceError;

/// A persistence backend that stores key-value pairs.
pub trait StorageBackend: Send + Sync {
    /// Load a value by key. Returns `None` if the key doesn't exist.
    fn load(&self, key: &str) -> Result<Option<String>, PersistenceError>;

    /// Save a key-value pair.
    fn save(&self, key: &str, value: &str) -> Result<(), PersistenceError>;

    /// Remove a key-value pair.
    fn remove(&self, key: &str) -> Result<(), PersistenceError>;

    /// Check if a key exists.
    fn exists(&self, key: &str) -> bool;

    /// List all keys.
    fn keys(&self) -> Result<Vec<String>, PersistenceError>;
}