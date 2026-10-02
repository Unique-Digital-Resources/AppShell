//! `FileStorageBackend` — persists key-value pairs to a directory.
//!
//! Each key is stored as a separate file. Values are written as plain text.
//! This is intentionally simple — for structured data, use a database backend.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::app_engine::errors::persistence_error::PersistenceError;
use crate::app_engine::persistence::storage_backend::StorageBackend;

pub struct FileStorageBackend {
    base_dir: PathBuf,
    /// In-memory cache for fast reads; files are the source of truth.
    cache: Mutex<HashMap<String, String>>,
}

impl FileStorageBackend {
    /// Create a file-based storage backend rooted at `base_dir`.
    /// The directory is created if it doesn't exist.
    pub fn new(base_dir: impl AsRef<Path>) -> Result<Self, PersistenceError> {
        let base_dir = base_dir.as_ref().to_path_buf();
        fs::create_dir_all(&base_dir)?;

        // Load existing files into cache
        let mut cache = HashMap::new();
        if let Ok(entries) = fs::read_dir(&base_dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        let key = entry.file_name().to_string_lossy().to_string();
                        if let Ok(value) = fs::read_to_string(entry.path()) {
                            cache.insert(key, value);
                        }
                    }
                }
            }
        }

        Ok(Self {
            base_dir,
            cache: Mutex::new(cache),
        })
    }

    /// Get the file path for a key.
    fn file_path(&self, key: &str) -> PathBuf {
        // Sanitize key to prevent directory traversal
        let safe_key = key.replace('/', "_").replace('\\', "_");
        self.base_dir.join(safe_key)
    }
}

impl StorageBackend for FileStorageBackend {
    fn load(&self, key: &str) -> Result<Option<String>, PersistenceError> {
        // Check cache first
        if let Some(value) = self.cache.lock().unwrap().get(key) {
            return Ok(Some(value.clone()));
        }

        // Fall back to file
        let path = self.file_path(key);
        if path.exists() {
            let value = fs::read_to_string(&path)?;
            self.cache.lock().unwrap().insert(key.to_string(), value.clone());
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn save(&self, key: &str, value: &str) -> Result<(), PersistenceError> {
        let path = self.file_path(key);

        // Write to file
        fs::write(&path, value)?;

        // Update cache
        self.cache.lock().unwrap().insert(key.to_string(), value.to_string());

        Ok(())
    }

    fn remove(&self, key: &str) -> Result<(), PersistenceError> {
        let path = self.file_path(key);

        // Remove file if it exists
        if path.exists() {
            fs::remove_file(&path)?;
        }

        // Remove from cache
        self.cache.lock().unwrap().remove(key);

        Ok(())
    }

    fn exists(&self, key: &str) -> bool {
        if self.cache.lock().unwrap().contains_key(key) {
            return true;
        }
        self.file_path(key).exists()
    }

    fn keys(&self) -> Result<Vec<String>, PersistenceError> {
        Ok(self.cache.lock().unwrap().keys().cloned().collect())
    }
}

impl std::fmt::Debug for FileStorageBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileStorageBackend")
            .field("base_dir", &self.base_dir)
            .field("cached_count", &self.cache.lock().unwrap().len())
            .finish()
    }
}