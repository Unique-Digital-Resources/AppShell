//! `TaskOutputChannel` — progress reporting and streaming output from handlers.
//!
//! The channel emits signals on the SignalBus so that UI, monitors, and other
//! consumers can observe progress without direct coupling to the handler.

use std::sync::{Arc, Mutex};
use std::any::Any;

use crate::app_engine::signals::signal_bus::SignalBus;

/// A channel for handlers to report progress and stream intermediate output.
#[derive(Clone)]
pub struct TaskOutputChannel {
    signal_bus: Option<Arc<SignalBus>>,
    task_id_value: u64,
    progress: Arc<Mutex<f32>>,
}

impl TaskOutputChannel {
    /// Create a channel with no signal bus (for tests / headless).
    pub fn new(task_id_value: u64) -> Self {
        Self {
            signal_bus: None,
            task_id_value,
            progress: Arc::new(Mutex::new(0.0)),
        }
    }

    /// Create a channel connected to the signal bus.
    pub fn with_signal_bus(signal_bus: Arc<SignalBus>, task_id_value: u64) -> Self {
        Self {
            signal_bus: Some(signal_bus),
            task_id_value,
            progress: Arc::new(Mutex::new(0.0)),
        }
    }

    /// Report progress (0.0 to 1.0). Emits a `task.progress_changed` signal.
    pub fn progress(&self, value: f32) {
        let clamped = value.clamp(0.0, 1.0);
        *self.progress.lock().unwrap() = clamped;
        if let Some(bus) = &self.signal_bus {
            bus.emit(
                &crate::app_engine::signals::signal::Signal::new("task.progress_changed")
                    .with_payload(clamped)
                    .with_source(format!("task_{}", self.task_id_value)),
            );
        }
    }

    /// Get the current progress value.
    pub fn current_progress(&self) -> f32 {
        *self.progress.lock().unwrap()
    }

    /// Push intermediate output data. Emits a `task.output` signal with the data.
    pub fn push<T: Any + Send>(&self, data: T) {
        if let Some(bus) = &self.signal_bus {
            bus.emit(
                &crate::app_engine::signals::signal::Signal::new("task.output")
                    .with_payload(data)
                    .with_source(format!("task_{}", self.task_id_value)),
            );
        }
    }
}

impl std::fmt::Debug for TaskOutputChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskOutputChannel")
            .field("task_id", &self.task_id_value)
            .field("progress", &*self.progress.lock().unwrap())
            .field("has_signal_bus", &self.signal_bus.is_some())
            .finish()
    }
}