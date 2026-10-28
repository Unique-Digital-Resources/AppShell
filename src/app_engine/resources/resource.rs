//! `Resource` — a managed piece of data whose lifetime matters to the application.
//!
//! The App Engine manages resource lifecycle; the Domain Engine defines what
//! the resource means.

use std::any::Any;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

static RESOURCE_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceId(u64);

impl ResourceId {
    pub fn new() -> Self {
        Self(RESOURCE_ID_COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Default for ResourceId {
    fn default() -> Self {
        Self::new()
    }
}

/// Lifecycle states of a resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceState {
    Unloaded,
    Loading,
    Loaded,
    Failed,
    Released,
}

impl ResourceState {
    pub fn is_loaded(&self) -> bool {
        matches!(self, ResourceState::Loaded)
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self,
            ResourceState::Loading | ResourceState::Loaded
        )
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            ResourceState::Unloaded | ResourceState::Released
        )
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ResourceState::Unloaded => "Unloaded",
            ResourceState::Loading => "Loading",
            ResourceState::Loaded => "Loaded",
            ResourceState::Failed => "Failed",
            ResourceState::Released => "Released",
        }
    }
}

impl std::fmt::Display for ResourceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A key for deduplicating resources by (type, source).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceKey {
    resource_type: String,
    source: String,
}

impl ResourceKey {
    pub fn new(
        resource_type: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            resource_type: resource_type.into(),
            source: source.into(),
        }
    }

    pub fn resource_type(&self) -> &str {
        &self.resource_type
    }

    pub fn source(&self) -> &str {
        &self.source
    }
}

/// A generic resource with type-erased data.
///
/// The App Engine provides the structure; the Domain Engine supplies the
/// actual resource data and meaning.
pub struct Resource {
    id: ResourceId,
    resource_type: String,
    source: String,
    state: ResourceState,
    data: Option<Box<dyn Any + Send>>,
    metadata: HashMap<String, String>,
    loaded_at: Instant,
}

impl Resource {
    pub fn new(
        resource_type: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            id: ResourceId::new(),
            resource_type: resource_type.into(),
            source: source.into(),
            state: ResourceState::Unloaded,
            data: None,
            metadata: HashMap::new(),
            loaded_at: Instant::now(),
        }
    }

    pub fn with_data<T: Any + Send>(mut self, data: T) -> Self {
        self.data = Some(Box::new(data));
        self.state = ResourceState::Loaded;
        self.loaded_at = Instant::now();
        self
    }

    pub fn with_metadata(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn id(&self) -> ResourceId {
        self.id
    }

    pub fn resource_type(&self) -> &str {
        &self.resource_type
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn state(&self) -> ResourceState {
        self.state
    }

    pub fn data<T: Any + Send>(&self) -> Option<&T> {
        self.data.as_ref().and_then(|d| d.downcast_ref::<T>())
    }

    pub fn has_data(&self) -> bool {
        self.data.is_some()
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    pub fn loaded_at(&self) -> Instant {
        self.loaded_at
    }

    // ----- Internal mutation (used by ResourceManager) -----

    pub(crate) fn set_data_box(&mut self, data: Box<dyn Any + Send>) {
        self.data = Some(data);
        self.state = ResourceState::Loaded;
        self.loaded_at = Instant::now();
    }
		#[allow(dead_code)]
    pub(crate) fn set_state(&mut self, state: ResourceState) {
        self.state = state;
    }
		#[allow(dead_code)]
    pub(crate) fn take_data(&mut self) -> Option<Box<dyn Any + Send>> {
        self.data.take()
    }
}

impl std::fmt::Debug for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Resource")
            .field("id", &self.id)
            .field("resource_type", &self.resource_type)
            .field("source", &self.source)
            .field("state", &self.state)
            .field("has_data", &self.data.is_some())
            .field("metadata_count", &self.metadata.len())
            .finish()
    }
}