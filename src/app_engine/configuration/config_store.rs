//! `ConfigStore` — persistence and retrieval of configuration.
//!
//! Phase 23: now thread-safe using Mutex. All mutating methods take &self.

use std::sync::{Arc, Mutex};

use crate::app_engine::configuration::config::Config;
use crate::app_engine::configuration::config_schema::ConfigSchema;
use crate::app_engine::errors::configuration_error::ConfigurationError;
use crate::app_engine::persistence::memory_backend::MemoryStorageBackend;
use crate::app_engine::persistence::storage_backend::StorageBackend;

/// Callback type for config changes: (key, old_value, new_value).
pub type ConfigChangeCallback =
    Arc<dyn Fn(&str, Option<&str>, Option<&str>) + Send + Sync + 'static>;

pub struct ConfigStore {
    config: Mutex<Config>,
    schema: ConfigSchema,
    backend: Option<Box<dyn StorageBackend>>,
    on_changed: Mutex<Option<ConfigChangeCallback>>,
}

impl ConfigStore {
    pub fn new(schema: ConfigSchema) -> Self {
        let config = schema.defaults();
        Self {
            config: Mutex::new(config),
            schema,
            backend: None,
            on_changed: Mutex::new(None),
        }
    }

    pub fn with_backend(
        schema: ConfigSchema,
        backend: Box<dyn StorageBackend>,
    ) -> Self {
        let defaults = schema.defaults();
        let store = Self {
            config: Mutex::new(defaults),
            schema,
            backend: Some(backend),
            on_changed: Mutex::new(None),
        };
        store.reload_from_backend();
        store
    }

    pub fn with_memory_backend(schema: ConfigSchema) -> Self {
        Self::with_backend(schema, Box::new(MemoryStorageBackend::new()))
    }

    pub fn reload_from_backend(&self) {
        if let Some(backend) = &self.backend {
            let mut config = self.config.lock().unwrap();
            for key in backend.keys().unwrap_or_default() {
                if let Ok(Some(value)) = backend.load(&key) {
                    config.set(key, value);
                }
            }
        }
    }

    pub fn load(&self) -> Config {
        self.config.lock().unwrap().clone()
    }

    pub fn save(&self, config: Config) -> Result<(), ConfigurationError> {
        self.schema.validate(&config)?;

        if let Some(backend) = &self.backend {
            for key in config.keys() {
                if let Some(value) = config.get(key) {
                    backend
                        .save(key, value)
                        .map_err(|e| ConfigurationError::SaveFailed {
                            reason: e.to_string(),
                        })?;
                }
            }
        }

        let old_config = {
            let mut current = self.config.lock().unwrap();
            let old = current.clone();
            *current = config.clone();
            old
        };

        self.notify_all_changes(&old_config, &config);

        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.config.lock().unwrap().get(key).map(|s| s.to_string())
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        self.config.lock().unwrap().get_string(key)
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.config.lock().unwrap().get_bool(key)
    }

    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.config.lock().unwrap().get_i64(key)
    }

    pub fn set(
        &self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<(), ConfigurationError> {
        let key = key.into();
        let value = value.into();

        let mut new_config = self.config.lock().unwrap().clone();
        new_config.set(key.clone(), value.clone());
        self.schema.validate(&new_config)?;

        if let Some(backend) = &self.backend {
            backend
                .save(&key, &value)
                .map_err(|e| ConfigurationError::SaveFailed {
                    reason: e.to_string(),
                })?;
        }

        let old_value = {
            let mut config = self.config.lock().unwrap();
            let old = config.get(&key).map(|s| s.to_string());
            *config = new_config;
            old
        };

        self.notify_change(&key, old_value.as_deref(), Some(&value));

        Ok(())
    }

    pub fn remove(&self, key: &str) -> Option<String> {
        if let Some(backend) = &self.backend {
            let _ = backend.remove(key);
        }

        let old_value = {
            let mut config = self.config.lock().unwrap();
            config.remove(key)
        };

        self.notify_change(key, old_value.as_deref(), None);

        old_value
    }

    pub fn reset(&self) {
        let new_config = self.schema.defaults();
        {
            let mut config = self.config.lock().unwrap();
            *config = new_config;
        }
        self.reload_from_backend();
    }

    pub fn validate(&self, config: &Config) -> Result<(), ConfigurationError> {
        self.schema.validate(config)
    }

    pub fn schema(&self) -> &ConfigSchema {
        &self.schema
    }

    pub fn contains(&self, key: &str) -> bool {
        self.config.lock().unwrap().contains(key)
    }

    pub fn key_count(&self) -> usize {
        self.config.lock().unwrap().len()
    }

    pub fn has_backend(&self) -> bool {
        self.backend.is_some()
    }

    // ----- Phase 19+23: Change notification -----

    pub fn on_changed<F>(&self, f: F)
    where
        F: Fn(&str, Option<&str>, Option<&str>) + Send + Sync + 'static,
    {
        *self.on_changed.lock().unwrap() = Some(Arc::new(f));
    }

    fn notify_change(&self, key: &str, old: Option<&str>, new: Option<&str>) {
        let callback = self.on_changed.lock().unwrap();
        if let Some(callback) = callback.as_ref() {
            callback(key, old, new);
        }
    }

    fn notify_all_changes(&self, old_config: &Config, new_config: &Config) {
        let callback = self.on_changed.lock().unwrap();
        if let Some(callback) = callback.as_ref() {
            for key in new_config.keys() {
                let old_val = old_config.get(key);
                let new_val = new_config.get(key);
                if old_val != new_val {
                    callback(key, old_val, new_val);
                }
            }
            for key in old_config.keys() {
                if !new_config.contains(key) {
                    let old_val = old_config.get(key);
                    callback(key, old_val, None);
                }
            }
        }
    }
}

impl std::fmt::Debug for ConfigStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfigStore")
            .field("keys", &self.config.lock().unwrap().len())
            .field("schema_properties", &self.schema.property_count())
            .field("has_backend", &self.backend.is_some())
            .field("has_change_callback", &self.on_changed.lock().unwrap().is_some())
            .finish()
    }
}