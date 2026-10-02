//! `EventHandler` — contract for anything that consumes events.

use crate::app_engine::events::event::Event;

/// Implemented by anything that can react to events.
///
/// Handlers can belong to App Engine, Domain Engine, UI, Plugins, History,
/// etc. — the event producer never needs to know which.
pub trait EventHandler: Send + Sync {
    fn handle(&self, event: &Event);
}