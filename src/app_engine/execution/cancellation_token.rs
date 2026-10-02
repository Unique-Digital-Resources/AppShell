//! `CancellationToken` — an atomic flag that handlers can check during execution.
//!
//! When `cancel()` is called on a task, the token is set, and the running
//! handler can detect it by checking `is_cancelled()`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// A cancellation token backed by an atomic boolean.
///
/// The token is shared between the TaskManager (which sets it on `cancel()`)
/// and the TaskContext (which the handler checks during execution).
#[derive(Debug, Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Check if cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Request cancellation.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Reset the token (for reuse).
    pub fn reset(&self) {
        self.cancelled.store(false, Ordering::SeqCst);
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}