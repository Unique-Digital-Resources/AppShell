//! `EventBus` — the public event communication interface.
//!
//! Phase 19: now supports priority subscriptions.

use crate::app_engine::events::event::Event;
use crate::app_engine::events::event_dispatcher::{EventDispatcher, EventPriority, HandlerId};
use crate::app_engine::events::event_handler::EventHandler;

pub struct EventBus {
    dispatcher: EventDispatcher,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            dispatcher: EventDispatcher::new(),
        }
    }

    /// Subscribe a handler with normal priority.
    pub fn subscribe(
        &mut self,
        event_type: &str,
        handler: Box<dyn EventHandler>,
    ) -> HandlerId {
        self.dispatcher.subscribe(event_type, handler)
    }

    /// Subscribe a handler with a specific priority.
    pub fn subscribe_with_priority(
        &mut self,
        event_type: &str,
        handler: Box<dyn EventHandler>,
        priority: EventPriority,
    ) -> HandlerId {
        self.dispatcher
            .subscribe_with_priority(event_type, handler, priority)
    }

    /// Unsubscribe a handler by ID.
    pub fn unsubscribe(&mut self, event_type: &str, id: HandlerId) -> bool {
        self.dispatcher.unsubscribe(event_type, id)
    }

    /// Publish an event to all registered handlers.
    /// Handler panics are caught — one crashing handler doesn't kill the engine.
    pub fn publish(&self, event: &Event) {
        self.dispatcher.dispatch(event);
    }

    pub fn handler_count(&self, event_type: &str) -> usize {
        self.dispatcher.handler_count(event_type)
    }

    pub fn total_handlers(&self) -> usize {
        self.dispatcher.total_handlers()
    }

    pub fn dispatcher(&self) -> &EventDispatcher {
        &self.dispatcher
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for EventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventBus")
            .field("dispatcher", &self.dispatcher)
            .finish()
    }
}