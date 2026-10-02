//! `ServiceRegistry` — discovery of registered services.

use std::collections::HashMap;

use crate::app_engine::services::service::{ServiceDefinition, ServiceId};

/// Stores service definitions for discovery.
///
/// "What services exist?" — this is separate from "how are they managed?"
/// (which is `ServiceManager`).
pub struct ServiceRegistry {
    definitions: HashMap<String, ServiceDefinition>,
    id_index: HashMap<ServiceId, String>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
            id_index: HashMap::new(),
        }
    }

    pub fn register(
        &mut self,
        service_id: ServiceId,
        definition: ServiceDefinition,
    ) -> Result<(), crate::app_engine::errors::service_error::ServiceError> {
        if self.definitions.contains_key(definition.id()) {
            return Err(
                crate::app_engine::errors::service_error::ServiceError::AlreadyRegistered {
                    id: definition.id().to_string(),
                },
            );
        }
        self.id_index.insert(service_id, definition.id().to_string());
        self.definitions.insert(definition.id().to_string(), definition);
        Ok(())
    }

    pub fn unregister(&mut self, service_id: ServiceId) -> Option<ServiceDefinition> {
        let type_id = self.id_index.remove(&service_id)?;
        self.definitions.remove(&type_id)
    }

    pub fn get_by_id(&self, service_id: ServiceId) -> Option<&ServiceDefinition> {
        let type_id = self.id_index.get(&service_id)?;
        self.definitions.get(type_id)
    }

    pub fn get_by_type(&self, service_type: &str) -> Option<&ServiceDefinition> {
        self.definitions.get(service_type)
    }

    pub fn contains_type(&self, service_type: &str) -> bool {
        self.definitions.contains_key(service_type)
    }

    pub fn list(&self) -> Vec<&ServiceDefinition> {
        self.definitions.values().collect()
    }

    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ServiceRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServiceRegistry")
            .field("count", &self.definitions.len())
            .finish()
    }
}