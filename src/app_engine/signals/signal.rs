//! `Signal` — a lightweight notification that something changed.

use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

static SIGNAL_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SignalId(u64);

impl SignalId {
    pub fn new() -> Self {
        Self(SIGNAL_ID_COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Default for SignalId {
    fn default() -> Self {
        Self::new()
    }
}

/// The type/category of a signal (e.g. "task.progress_changed", "state.changed").
pub type SignalType = String;

/// A lightweight signal with an optional type-erased payload.
pub struct Signal {
    id: SignalId,
    signal_type: SignalType,
    timestamp: Instant,
    source: Option<String>,
    payload: Option<Box<dyn Any + Send>>,
    payload_type_name: &'static str,
}

impl Signal {
    pub fn new(signal_type: impl Into<SignalType>) -> Self {
        Self {
            id: SignalId::new(),
            signal_type: signal_type.into(),
            timestamp: Instant::now(),
            source: None,
            payload: None,
            payload_type_name: "()",
        }
    }

    pub fn with_payload<T: Any + Send>(mut self, data: T) -> Self {
        self.payload = Some(Box::new(data));
        self.payload_type_name = std::any::type_name::<T>();
        self
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn id(&self) -> SignalId {
        self.id
    }

    pub fn signal_type(&self) -> &str {
        &self.signal_type
    }

    pub fn timestamp(&self) -> Instant {
        self.timestamp
    }

    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    pub fn payload<T: Any + Send>(&self) -> Option<&T> {
        self.payload.as_ref().and_then(|p| p.downcast_ref::<T>())
    }

    pub fn has_payload(&self) -> bool {
        self.payload.is_some()
    }

    pub fn payload_type_name(&self) -> &'static str {
        self.payload_type_name
    }
}

impl std::fmt::Debug for Signal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Signal")
            .field("id", &self.id)
            .field("signal_type", &self.signal_type)
            .field("source", &self.source)
            .field("payload_type", &self.payload_type_name)
            .finish()
    }
}