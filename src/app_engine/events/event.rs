//! `Event` — a message representing something that happened.
//!
//! An `Event` is a notification of a past occurrence. It is a **message**,
//! not a handler — it carries no behavior, only data.

use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

static EVENT_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EventId(u64);

impl EventId {
    pub fn new() -> Self {
        Self(EVENT_ID_COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

/// The type/category of an event (e.g. "task.completed", "command.executed").
pub type EventType = String;

/// A generic event with a type-erased payload.
///
/// The App Engine provides the structure; domain-specific payload types are
/// supplied by the Domain Engine. App Engine never inspects the payload.
///
/// Phase 22: now supports optional serialized payload bytes for IPC and persistence.
pub struct Event {
    id: EventId,
    event_type: EventType,
    timestamp: Instant,
    source: Option<String>,
    payload: Option<Box<dyn Any + Send>>,
    serialized_payload: Option<Vec<u8>>,
    payload_type_name: &'static str,
}

impl Event {
    /// Create an event with no payload.
    pub fn new(event_type: impl Into<EventType>) -> Self {
        Self {
            id: EventId::new(),
            event_type: event_type.into(),
            timestamp: Instant::now(),
            source: None,
            payload: None,
            serialized_payload: None,
            payload_type_name: "()",
        }
    }

    /// Create an event with a typed payload.
    pub fn with_payload<T: Any + Send>(mut self, data: T) -> Self {
        self.payload_type_name = std::any::type_name::<T>();
        self.payload = Some(Box::new(data));
        self.serialized_payload = None;
        self
    }

    /// Create an event with both typed and serialized payload.
    pub fn with_payload_and_serialized<T: Any + Send>(mut self, data: T, bytes: Vec<u8>) -> Self {
        self.payload_type_name = std::any::type_name::<T>();
        self.payload = Some(Box::new(data));
        self.serialized_payload = Some(bytes);
        self
    }

    /// Create an event from serialized payload bytes (for IPC / deserialization).
    pub fn from_serialized(
        event_type: impl Into<EventType>,
        payload_type_name: &'static str,
        serialized: Vec<u8>,
    ) -> Self {
        Self {
            id: EventId::new(),
            event_type: event_type.into(),
            timestamp: Instant::now(),
            source: None,
            payload: None,
            serialized_payload: Some(serialized),
            payload_type_name,
        }
    }

    /// Create an event from serialized bytes with a source.
    pub fn from_serialized_with_source(
        event_type: impl Into<EventType>,
        payload_type_name: &'static str,
        serialized: Vec<u8>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            id: EventId::new(),
            event_type: event_type.into(),
            timestamp: Instant::now(),
            source: Some(source.into()),
            payload: None,
            serialized_payload: Some(serialized),
            payload_type_name,
        }
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn id(&self) -> EventId {
        self.id
    }

    pub fn event_type(&self) -> &str {
        &self.event_type
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

    // ----- Phase 22: Serialized payload -----

    /// Get serialized payload bytes (for IPC / persistence).
    pub fn serialized_payload(&self) -> Option<&[u8]> {
        self.serialized_payload.as_deref()
    }

    /// Check if this event has serialized payload bytes.
    pub fn has_serialized_payload(&self) -> bool {
        self.serialized_payload.is_some()
    }

    /// Serialize the typed payload using a registry (if not already serialized).
    pub fn ensure_serialized(
        &mut self,
        registry: &crate::app_engine::serialization::serialization_registry::SerializationRegistry,
    ) -> Result<(), crate::app_engine::errors::serialization_error::SerializationError> {
        if self.serialized_payload.is_none() {
            if let Some(payload) = &self.payload {
                let bytes = registry.serialize(payload.as_ref(), self.payload_type_name)?;
                self.serialized_payload = Some(bytes);
            }
        }
        Ok(())
    }

    /// Deserialize from bytes using a registry (if not already typed).
    pub fn ensure_deserialized(
        &mut self,
        registry: &crate::app_engine::serialization::serialization_registry::SerializationRegistry,
    ) -> Result<(), crate::app_engine::errors::serialization_error::SerializationError> {
        if self.payload.is_none() {
            if let Some(bytes) = &self.serialized_payload {
                let data = registry.deserialize(bytes, self.payload_type_name)?;
                self.payload = Some(data);
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Event")
            .field("id", &self.id)
            .field("event_type", &self.event_type)
            .field("source", &self.source)
            .field("payload_type", &self.payload_type_name)
            .field("has_payload", &self.payload.is_some())
            .field("has_serialized_payload", &self.serialized_payload.is_some())
            .finish()
    }
}