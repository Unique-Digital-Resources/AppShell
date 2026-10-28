//! `ServiceManager` — the main service lifecycle coordinator.
//!
//! Phase 24: now with panic isolation. Service lifecycle methods
//! are wrapped in `catch_unwind` to prevent panics from crashing the engine.

use std::collections::HashMap;

use crate::app_engine::capabilities::engine_ref::EngineRef;
use crate::app_engine::errors::service_error::ServiceError;
use crate::app_engine::services::service::{
    Service, ServiceDefinition, ServiceEntry, ServiceId, ServiceState,
};
use crate::app_engine::services::service_registry::ServiceRegistry;

pub struct ServiceManager {
    entries: HashMap<ServiceId, ServiceEntry>,
    type_index: HashMap<String, ServiceId>,
    registry: ServiceRegistry,
}

impl ServiceManager {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            type_index: HashMap::new(),
            registry: ServiceRegistry::new(),
        }
    }

    // ----- Registration -----

    pub fn register(
        &mut self,
        definition: ServiceDefinition,
        instance: Box<dyn Service>,
    ) -> Result<ServiceId, ServiceError> {
        if self.type_index.contains_key(definition.id()) {
            return Err(ServiceError::AlreadyRegistered {
                id: definition.id().to_string(),
            });
        }

        let entry = ServiceEntry::new(definition.clone(), instance);
        let id = entry.id;
        self.registry.register(id, definition)?;
        self.type_index
            .insert(entry.definition.id().to_string(), id);
        self.entries.insert(id, entry);
        Ok(id)
    }

    pub fn unregister(&mut self, id: ServiceId) -> Option<ServiceDefinition> {
        let entry = self.entries.remove(&id)?;
        self.type_index.remove(entry.definition.id());
        self.registry.unregister(id)
    }

    // ----- Queries -----

    pub fn get(&self, id: ServiceId) -> Option<&(dyn Service + '_)> {
        self.entries.get(&id).map(|e| e.instance.as_ref())
    }

    pub fn get_definition(&self, id: ServiceId) -> Option<&ServiceDefinition> {
        self.entries.get(&id).map(|e| &e.definition)
    }

    pub fn get_definition_by_type(
        &self,
        service_type: &str,
    ) -> Option<&ServiceDefinition> {
        self.type_index
            .get(service_type)
            .and_then(|id| self.entries.get(id).map(|e| &e.definition))
    }

    pub fn get_id_by_type(&self, service_type: &str) -> Option<ServiceId> {
        self.type_index.get(service_type).copied()
    }

    pub fn contains(&self, id: ServiceId) -> bool {
        self.entries.contains_key(&id)
    }

    pub fn contains_type(&self, service_type: &str) -> bool {
        self.type_index.contains_key(service_type)
    }

    pub fn state(&self, id: ServiceId) -> Option<ServiceState> {
        self.entries.get(&id).map(|e| e.state)
    }

    pub fn list(&self) -> Vec<&ServiceDefinition> {
        self.entries.values().map(|e| &e.definition).collect()
    }

    pub fn list_ids(&self) -> Vec<ServiceId> {
        self.entries.keys().copied().collect()
    }

    pub fn count(&self) -> usize {
        self.entries.len()
    }

    pub fn registry(&self) -> &ServiceRegistry {
        &self.registry
    }

    // ----- Lifecycle (with panic isolation) -----

    /// Initialize a single service. Panics are caught.
    pub fn initialize(
        &mut self,
        id: ServiceId,
        engine: &EngineRef,
    ) -> Result<(), ServiceError> {
        let entry = self
            .entries
            .get_mut(&id)
            .ok_or(ServiceError::NotFound { id: id.value() })?;

        if !entry.state.can_initialize() {
            return Err(ServiceError::InvalidState {
                id: id.value(),
                current: entry.state.as_str().to_string(),
                requested: "Initialized".to_string(),
            });
        }

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            entry.instance.initialize(engine)
        }));

        match result {
            Ok(Ok(())) => {
                entry.state = ServiceState::Initialized;
                Ok(())
            }
            Ok(Err(e)) => Err(ServiceError::InitializeFailed {
                id: id.value(),
                reason: e.to_string(),
            }),
            Err(panic_payload) => {
                let reason = extract_panic_message(&panic_payload);
                Err(ServiceError::InitializeFailed {
                    id: id.value(),
                    reason: format!("service panicked: {}", reason),
                })
            }
        }
    }

    /// Start a single service. Panics are caught.
    pub fn start(
        &mut self,
        id: ServiceId,
        engine: &EngineRef,
    ) -> Result<(), ServiceError> {
        let entry = self
            .entries
            .get_mut(&id)
            .ok_or(ServiceError::NotFound { id: id.value() })?;

        if !entry.state.can_start() {
            return Err(ServiceError::InvalidState {
                id: id.value(),
                current: entry.state.as_str().to_string(),
                requested: "Running".to_string(),
            });
        }

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            entry.instance.start(engine)
        }));

        match result {
            Ok(Ok(())) => {
                entry.state = ServiceState::Running;
                Ok(())
            }
            Ok(Err(e)) => Err(ServiceError::StartFailed {
                id: id.value(),
                reason: e.to_string(),
            }),
            Err(panic_payload) => {
                let reason = extract_panic_message(&panic_payload);
                Err(ServiceError::StartFailed {
                    id: id.value(),
                    reason: format!("service panicked: {}", reason),
                })
            }
        }
    }

    /// Stop a single service. Panics are caught.
    pub fn stop(&mut self, id: ServiceId) -> Result<(), ServiceError> {
        let entry = self
            .entries
            .get_mut(&id)
            .ok_or(ServiceError::NotFound { id: id.value() })?;

        if !entry.state.can_stop() {
            return Err(ServiceError::InvalidState {
                id: id.value(),
                current: entry.state.as_str().to_string(),
                requested: "Stopped".to_string(),
            });
        }

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            entry.instance.stop()
        }));

        match result {
            Ok(Ok(())) => {
                entry.state = ServiceState::Stopped;
                Ok(())
            }
            Ok(Err(e)) => Err(ServiceError::StopFailed {
                id: id.value(),
                reason: e.to_string(),
            }),
            Err(panic_payload) => {
                let reason = extract_panic_message(&panic_payload);
                Err(ServiceError::StopFailed {
                    id: id.value(),
                    reason: format!("service panicked: {}", reason),
                })
            }
        }
    }

    /// Dispose a single service. Panics are caught.
    pub fn dispose(&mut self, id: ServiceId) -> Result<(), ServiceError> {
        let entry = self
            .entries
            .get_mut(&id)
            .ok_or(ServiceError::NotFound { id: id.value() })?;

        if !entry.state.can_dispose() {
            return Err(ServiceError::InvalidState {
                id: id.value(),
                current: entry.state.as_str().to_string(),
                requested: "Disposed".to_string(),
            });
        }

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            entry.instance.dispose()
        }));

        match result {
            Ok(Ok(())) => {
                entry.state = ServiceState::Disposed;
                Ok(())
            }
            Ok(Err(e)) => Err(ServiceError::DisposeFailed {
                id: id.value(),
                reason: e.to_string(),
            }),
            Err(panic_payload) => {
                let reason = extract_panic_message(&panic_payload);
                Err(ServiceError::DisposeFailed {
                    id: id.value(),
                    reason: format!("service panicked: {}", reason),
                })
            }
        }
    }

    // ----- Bulk lifecycle -----

    pub fn initialize_all(&mut self, engine: &EngineRef) -> Result<(), ServiceError> {
        let ids: Vec<ServiceId> = self.list_ids();
        for id in ids {
            if let Some(entry) = self.entries.get(&id) {
                if entry.state.can_initialize() {
                    self.initialize(id, engine)?;
                }
            }
        }
        Ok(())
    }

    pub fn start_all(&mut self, engine: &EngineRef) -> Result<(), ServiceError> {
        let ids: Vec<ServiceId> = self.list_ids();
        for id in ids {
            if let Some(entry) = self.entries.get(&id) {
                if entry.state.can_start() {
                    self.start(id, engine)?;
                }
            }
        }
        Ok(())
    }

    pub fn stop_all(&mut self) -> Result<(), ServiceError> {
        let mut ids: Vec<ServiceId> = self.list_ids();
        ids.reverse();
        for id in ids {
            if let Some(entry) = self.entries.get(&id) {
                if entry.state.can_stop() {
                    self.stop(id)?;
                }
            }
        }
        Ok(())
    }

    pub fn dispose_all(&mut self) -> Result<(), ServiceError> {
        let mut ids: Vec<ServiceId> = self.list_ids();
        ids.reverse();
        for id in ids {
            if let Some(entry) = self.entries.get(&id) {
                if entry.state.can_dispose() {
                    self.dispose(id)?;
                }
            }
        }
        Ok(())
    }

    /// Apply a closure to a mutable service instance.
    pub fn with_service_mut<F, R>(&mut self, id: ServiceId, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn Service) -> R,
    {
        let entry = self.entries.get_mut(&id)?;
        Some(f(entry.instance.as_mut()))
    }
}

impl Default for ServiceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ServiceManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServiceManager")
            .field("services", &self.entries.len())
            .field("registry", &self.registry)
            .finish()
    }
}

/// Extract a human-readable message from a panic payload.
fn extract_panic_message(payload: &Box<dyn std::any::Any + Send>) -> String {
    payload
        .downcast_ref::<String>()
        .map(|s| s.clone())
        .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown panic".to_string())
}