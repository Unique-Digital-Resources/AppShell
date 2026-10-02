//! Bootstrap — creates and wires the initial App Engine environment.

use crate::app_engine::context::application_context::ApplicationContext;
use crate::app_engine::errors::engine_error::EngineError;
use crate::app_engine::runtime::runtime::{AppEngine, AppRuntime};

#[derive(Debug, Clone, Default)]
pub struct Bootstrap;

impl Bootstrap {
    pub fn new() -> Self {
        Self
    }

    /// Construct an `AppEngine` with the default application context.
    pub fn create() -> Result<AppEngine, EngineError> {
        Self::create_with_context(ApplicationContext::new("app_shell"))
    }

    /// Construct an `AppEngine` with a specific application context.
    pub fn create_with_context(context: ApplicationContext) -> Result<AppEngine, EngineError> {
        AppRuntime::with_context(context)
    }
}