//! `ResourceLoader` — defines how a resource is obtained or constructed.
//!
//! Domain code provides concrete implementations (e.g. `ProjectLoader`,
//! `ImageLoader`). The App Engine calls `load()` and wraps the result
//! in a `Resource` struct.

use std::any::Any;

use crate::app_engine::errors::resource_error::ResourceError;

/// Implemented by anything that can load resources of a given type.
pub trait ResourceLoader: Send + Sync {
    /// The resource type this loader handles (e.g. "image", "font", "project").
    fn resource_type(&self) -> &str;

    /// Load the resource data from the given source.
    fn load(&self, source: &str) -> Result<Box<dyn Any + Send>, ResourceError>;
}