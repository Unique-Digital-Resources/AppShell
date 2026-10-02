//! `AsyncRuntime` — a wrapper around tokio's runtime for async task execution.
//!
//! Only available with the `async-runtime` feature.
//! Without it, the engine uses synchronous execution exclusively.

use crate::app_engine::errors::async_error::AsyncError;

/// A tokio-based async runtime for spawning and awaiting futures.
#[cfg(feature = "async-runtime")]
pub struct AsyncRuntime {
    runtime: tokio::runtime::Runtime,
}

#[cfg(feature = "async-runtime")]
impl AsyncRuntime {
    /// Create a multi-threaded async runtime.
    pub fn new() -> Result<Self, AsyncError> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|e| AsyncError::RuntimeCreationFailed(e.to_string()))?;
        Ok(Self { runtime })
    }

    /// Create a single-threaded async runtime (for testing / lightweight use).
    pub fn new_current_thread() -> Result<Self, AsyncError> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| AsyncError::RuntimeCreationFailed(e.to_string()))?;
        Ok(Self { runtime })
    }

    /// Spawn a future on the runtime and block on its result.
    ///
    /// This is the bridge between sync code (TaskManager) and async handlers.
    /// The future is spawned on tokio's runtime, and the calling thread blocks
    /// until it completes.
    /// Spawn a future on the runtime and block on its result.
    pub fn block_on<F>(&self, future: F) -> F::Output
    where
        F: std::future::Future + Send,
        F::Output: Send,
    {
        self.runtime.handle().block_on(future)
    }

    /// Spawn a future on the runtime without blocking.
    pub fn spawn<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.runtime.handle().spawn(future)
    }

    /// Get a handle to the underlying tokio runtime.
    pub fn handle(&self) -> &tokio::runtime::Handle {
        self.runtime.handle()
    }
}

#[cfg(feature = "async-runtime")]
impl Default for AsyncRuntime {
    fn default() -> Self {
        Self::new().expect("Failed to create async runtime")
    }
}

#[cfg(feature = "async-runtime")]
impl std::fmt::Debug for AsyncRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncRuntime")
            .field("runtime", &"tokio::Runtime")
            .finish()
    }
}