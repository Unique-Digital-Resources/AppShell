//! `SignalBus` — the communication mechanism for signals.
//!
//! Producers call `emit()`. Consumers call `subscribe()`. The bus routes
//! signals to all matching subscriptions.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::app_engine::signals::signal::Signal;
use crate::app_engine::signals::subscription::{Subscription, SubscriptionId};

pub struct SignalBus {
    subscriptions: Mutex<HashMap<String, Vec<Subscription>>>,
}

impl SignalBus {
    pub fn new() -> Self {
        Self {
            subscriptions: Mutex::new(HashMap::new()),
        }
    }

    /// Subscribe to a signal type. Returns a `SubscriptionId` that can be
    /// used to unsubscribe later.
    pub fn subscribe(
        &self,
        signal_type: impl Into<String>,
        callback: crate::app_engine::signals::subscription::SignalCallback,
    ) -> SubscriptionId {
        let sub = Subscription::new(signal_type, callback);
        let id = sub.id();
        let mut subs = self.subscriptions.lock().unwrap();
        subs.entry(sub.signal_type().to_string())
            .or_default()
            .push(sub);
        id
    }

    /// Unsubscribe by ID.
    pub fn unsubscribe(&self, signal_type: &str, id: SubscriptionId) -> bool {
        let mut subs = self.subscriptions.lock().unwrap();
        if let Some(list) = subs.get_mut(signal_type) {
            let before = list.len();
            list.retain(|s| s.id() != id);
            return list.len() != before;
        }
        false
    }

    /// Emit a signal to all matching subscribers.
    pub fn emit(&self, signal: &Signal) {
        let subs = self.subscriptions.lock().unwrap();
        if let Some(list) = subs.get(signal.signal_type()) {
            for sub in list {
                sub.invoke(signal);
            }
        }
    }

    pub fn subscriber_count(&self, signal_type: &str) -> usize {
        self.subscriptions
            .lock()
            .unwrap()
            .get(signal_type)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    pub fn total_subscribers(&self) -> usize {
        self.subscriptions
            .lock()
            .unwrap()
            .values()
            .map(|v| v.len())
            .sum()
    }

    pub fn signal_types(&self) -> Vec<String> {
        self.subscriptions
            .lock()
            .unwrap()
            .keys()
            .cloned()
            .collect()
    }
}

impl Default for SignalBus {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for SignalBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let subs = self.subscriptions.lock().unwrap();
        f.debug_struct("SignalBus")
            .field("topics", &subs.len())
            .field("total_subscribers", &subs.values().map(|v| v.len()).sum::<usize>())
            .finish()
    }
}