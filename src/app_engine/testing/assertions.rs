//! Test assertion helpers.

use crate::app_engine::testing::mocks::{MockCommandExecutor, MockEventBus, MockSignalBus};

/// Assert that a command was executed on a MockCommandExecutor.
pub fn assert_command_executed(executor: &MockCommandExecutor, command_id: &str) {
    executor.assert_executed(command_id);
}

/// Assert that a command was NOT executed.
pub fn assert_command_not_executed(executor: &MockCommandExecutor, command_id: &str) {
    executor.assert_not_executed(command_id);
}

/// Assert that an event was published on a MockEventBus.
pub fn assert_event_published(bus: &MockEventBus, event_type: &str) {
    bus.assert_published(event_type);
}

/// Assert that an event was NOT published.
pub fn assert_event_not_published(bus: &MockEventBus, event_type: &str) {
    bus.assert_not_published(event_type);
}

/// Assert that a signal was emitted on a MockSignalBus.
pub fn assert_signal_emitted(bus: &MockSignalBus, signal_type: &str) {
    bus.assert_emitted(signal_type);
}