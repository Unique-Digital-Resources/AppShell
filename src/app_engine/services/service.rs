//! `Service` — a long-lived capability/process within the App Engine.
//!
//! Unlike a Task (finite work instance), a Service starts with the application,
//! remains available while the engine runs, and stops on shutdown.
//!
//! A service participates in the existing lifecycle system rather than
//! inventing its own. The `ServiceManager` coordinates startup/shutdown.

//! `Service` — a long-lived capability/process within the App Engine.

use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::app_engine::capabilities::engine_ref::EngineRef;
use crate::app_engine::errors::service_error::ServiceError;

static SERVICE_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ServiceId(u64);

impl ServiceId {
    pub fn new() -> Self {
        Self(SERVICE_ID_COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Default for ServiceId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ServiceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Service({})", self.0)
    }
}

/// Lifecycle states of a service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceState {
    Registered,
    Initialized,
    Running,
    Stopped,
    Disposed,
}

impl ServiceState {
    pub fn can_initialize(&self) -> bool {
        matches!(self, ServiceState::Registered)
    }

    pub fn can_start(&self) -> bool {
        matches!(self, ServiceState::Initialized | ServiceState::Stopped)
    }

    pub fn can_stop(&self) -> bool {
        matches!(self, ServiceState::Running)
    }

    pub fn can_dispose(&self) -> bool {
        matches!(self, ServiceState::Stopped | ServiceState::Registered)
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, ServiceState::Disposed)
    }

    pub fn is_active(&self) -> bool {
        matches!(self, ServiceState::Running)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ServiceState::Registered => "Registered",
            ServiceState::Initialized => "Initialized",
            ServiceState::Running => "Running",
            ServiceState::Stopped => "Stopped",
            ServiceState::Disposed => "Disposed",
        }
    }
}

impl fmt::Display for ServiceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Metadata describing a service type (analogous to `TaskDefinition`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceDefinition {
    id: String,
    name: String,
    description: String,
    metadata: HashMap<String, String>,
}

impl ServiceDefinition {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            metadata: HashMap::new(),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn with_metadata(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// The contract implemented by concrete services.
///
/// Domain or application code provides implementations. The `ServiceManager`
/// calls these methods during application lifecycle transitions.
pub trait Service: Send + Sync {
    /// A stable string identifier for this service type (e.g. "autosave").
    fn service_type(&self) -> &str;

    /// Called when the service is initialized (before start).
    fn initialize(&mut self, engine: &EngineRef) -> Result<(), ServiceError>;

    /// Called when the service starts.
    fn start(&mut self, engine: &EngineRef) -> Result<(), ServiceError>;

    /// Called when the service stops.
    fn stop(&mut self) -> Result<(), ServiceError>;

    /// Called when the service is disposed.
    fn dispose(&mut self) -> Result<(), ServiceError>;
}

/// Internal wrapper that tracks a service instance + its state.
pub(crate) struct ServiceEntry {
    pub id: ServiceId,
    pub state: ServiceState,
    pub definition: ServiceDefinition,
    pub instance: Box<dyn Service>,
}

impl ServiceEntry {
    pub fn new(definition: ServiceDefinition, instance: Box<dyn Service>) -> Self {
        Self {
            id: ServiceId::new(),
            state: ServiceState::Registered,
            definition,
            instance,
        }
    }
}

impl fmt::Debug for ServiceEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServiceEntry")
            .field("id", &self.id)
            .field("state", &self.state)
            .field("definition", &self.definition)
            .finish()
    }
}