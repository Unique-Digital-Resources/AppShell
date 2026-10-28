//! `Subscription` — represents a consumer's subscription to a signal type.

use std::sync::atomic::{AtomicU64, Ordering};

static SUBSCRIPTION_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SubscriptionId(u64);

impl SubscriptionId {
    pub fn new() -> Self {
        Self(SUBSCRIPTION_ID_COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Default for SubscriptionId {
    fn default() -> Self {
        Self::new()
    }
}

/// A callback for signal notifications.
pub type SignalCallback = Box<dyn Fn(&crate::app_engine::signals::signal::Signal) + Send + Sync>;

/// Represents one subscription to a signal source.
pub struct Subscription {
    id: SubscriptionId,
    signal_type: String,
    callback: SignalCallback,
}

impl Subscription {
    pub fn new(signal_type: impl Into<String>, callback: SignalCallback) -> Self {
        Self {
            id: SubscriptionId::new(),
            signal_type: signal_type.into(),
            callback,
        }
    }

    pub fn id(&self) -> SubscriptionId {
        self.id
    }

    pub fn signal_type(&self) -> &str {
        &self.signal_type
    }

    pub fn invoke(&self, signal: &crate::app_engine::signals::signal::Signal) {
        (self.callback)(signal);
    }
}

impl std::fmt::Debug for Subscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Subscription")
            .field("id", &self.id)
            .field("signal_type", &self.signal_type)
            .finish()
    }
}