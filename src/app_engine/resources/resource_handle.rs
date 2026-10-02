//! `ResourceHandle` — a safe reference to a managed resource.
//!
//! The handle does not own the resource — the `ResourceManager` does.
//! The handle is a lightweight, copyable reference.

//! `ResourceHandle` — a RAII reference to a managed resource.
//!
//! Phase 16: When a handle is dropped, it automatically decrements the
//! resource's reference count via a release callback. This prevents
//! resource leaks from forgotten `release()` calls.

use crate::app_engine::resources::resource::ResourceId;

/// A RAII handle to a managed resource.
///
/// When dropped, the handle calls its release callback (if set) to
/// decrement the reference count in the ResourceManager.
pub struct ResourceHandle {
    resource_id: ResourceId,
    release_callback: Option<Box<dyn FnOnce(ResourceId) + Send + Sync>>,
}

impl ResourceHandle {
    /// Create a handle without a release callback (for tests / manual management).
    pub fn new(resource_id: ResourceId) -> Self {
        Self {
            resource_id,
            release_callback: None,
        }
    }

    /// Create a handle with a release callback for RAII auto-release.
    pub fn with_release_callback<F>(resource_id: ResourceId, callback: F) -> Self
    where
        F: FnOnce(ResourceId) + Send + Sync + 'static,
    {
        Self {
            resource_id,
            release_callback: Some(Box::new(callback)),
        }
    }

    pub fn resource_id(&self) -> ResourceId {
        self.resource_id
    }
}

impl Drop for ResourceHandle {
    fn drop(&mut self) {
        if let Some(cb) = self.release_callback.take() {
            cb(self.resource_id);
        }
    }
}

impl std::fmt::Debug for ResourceHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceHandle")
            .field("resource_id", &self.resource_id)
            .field("has_release_callback", &self.release_callback.is_some())
            .finish()
    }
}

// Manual PartialEq and Eq — can't derive because of the callback
impl PartialEq for ResourceHandle {
    fn eq(&self, other: &Self) -> bool {
        self.resource_id == other.resource_id
    }
}

impl Eq for ResourceHandle {}

impl std::hash::Hash for ResourceHandle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.resource_id.hash(state);
    }
}

impl PartialOrd for ResourceHandle {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ResourceHandle {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.resource_id.cmp(&other.resource_id)
    }
}