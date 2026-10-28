//! `EventDispatcher` — internal routing of published events to handlers.
//!
//! Phase 19: now supports priority delivery and handler error isolation.

use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::app_engine::events::event::Event;
use crate::app_engine::events::event_handler::EventHandler;

static HANDLER_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandlerId(u64);

impl HandlerId {
    pub fn new() -> Self {
        Self(HANDLER_ID_COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Default for HandlerId {
    fn default() -> Self {
        Self::new()
    }
}

/// Event delivery priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EventPriority {
    /// Delivered last (e.g., analytics, logging).
    Low,
    /// Default priority.
    Normal,
    /// Delivered first (e.g., history recording, critical reactions).
    High,
}

impl Default for EventPriority {
    fn default() -> Self {
        EventPriority::Normal
    }
}

impl EventPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventPriority::High => "High",
            EventPriority::Normal => "Normal",
            EventPriority::Low => "Low",
        }
    }
}

/// Internal handler entry with priority.
struct HandlerEntry {
    id: HandlerId,
    handler: Box<dyn EventHandler>,
    priority: EventPriority,
}

pub struct EventDispatcher {
    handlers: HashMap<String, Vec<HandlerEntry>>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn subscribe(
        &mut self,
        event_type: &str,
        handler: Box<dyn EventHandler>,
    ) -> HandlerId {
        self.subscribe_with_priority(event_type, handler, EventPriority::Normal)
    }

    pub fn subscribe_with_priority(
        &mut self,
        event_type: &str,
        handler: Box<dyn EventHandler>,
        priority: EventPriority,
    ) -> HandlerId {
        let id = HandlerId::new();
        self.handlers
            .entry(event_type.to_string())
            .or_default()
            .push(HandlerEntry {
                id,
                handler,
                priority,
            });
        id
    }

    pub fn unsubscribe(&mut self, event_type: &str, id: HandlerId) -> bool {
        if let Some(list) = self.handlers.get_mut(event_type) {
            let before = list.len();
            list.retain(|e| e.id != id);
            return list.len() != before;
        }
        false
    }

    /// Dispatch an event to all registered handlers, sorted by priority.
    /// Handler panics are caught — one crashing handler doesn't kill the engine.
    pub fn dispatch(&self, event: &Event) {
        let Some(list) = self.handlers.get(event.event_type()) else {
            return;
        };

        // Sort by priority (High first). Clone the references to avoid borrowing issues.
        let mut sorted: Vec<&HandlerEntry> = list.iter().collect();
        sorted.sort_by(|a, b| b.priority.cmp(&a.priority));

        for entry in sorted {
            // Isolate panics — one handler crash doesn't kill the engine
            let _ = catch_unwind(AssertUnwindSafe(|| {
                entry.handler.handle(event);
            }));
        }
    }

    pub fn handler_count(&self, event_type: &str) -> usize {
        self.handlers
            .get(event_type)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    pub fn total_handlers(&self) -> usize {
        self.handlers.values().map(|v| v.len()).sum()
    }

    pub fn event_types(&self) -> Vec<&str> {
        self.handlers.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for EventDispatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventDispatcher")
            .field("topics", &self.handlers.len())
            .field("total_handlers", &self.total_handlers())
            .finish()
    }
}