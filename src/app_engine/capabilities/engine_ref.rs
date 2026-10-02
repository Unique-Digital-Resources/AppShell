//! `EngineRef` — a read-only bundle of references to engine subsystems.
//!
//! Passed to handlers during execution so they can:
//! - Publish events
//! - Emit signals
//! - Load resources
//! - Read configuration
//! - Query state
//!
//! Handlers receive `&EngineRef` (immutable) — they can observe and use
//! engine systems but cannot reconfigure the engine itself.

//! `EngineRef` — read-only bundle of references to engine subsystems.

use crate::app_engine::configuration::config_store::ConfigStore;
use crate::app_engine::configuration::preferences::Preferences;
use crate::app_engine::events::event_bus::EventBus;
use crate::app_engine::resources::resource_manager::ResourceManager;
use crate::app_engine::signals::signal_bus::SignalBus;
use crate::app_engine::state::state_manager::StateManager;
use crate::app_engine::lifecycle::lifecycle_state::LifecycleState;
use crate::app_engine::commands::command_executor::CommandExecutor;
use crate::app_engine::scheduling::scheduler::Scheduler;
use crate::app_engine::history::history_store::HistoryStore;

/// Read-only access to engine subsystems.
///
/// `task_manager` is deliberately excluded to allow disjoint borrows
/// when calling `task_manager_mut().start(task_id, &engine_ref)`.
pub struct EngineRef<'a> {
    pub event_bus: &'a EventBus,
    pub signal_bus: &'a SignalBus,
    pub resource_manager: &'a ResourceManager,
    pub config_store: &'a ConfigStore,
    pub preferences: &'a Preferences,
    pub state_manager: &'a StateManager<LifecycleState>,
    pub command_executor: &'a CommandExecutor,
    pub scheduler: &'a Scheduler,
    pub history_store: &'a HistoryStore,
}

impl<'a> EngineRef<'a> {
    pub fn new(
        event_bus: &'a EventBus,
        signal_bus: &'a SignalBus,
        resource_manager: &'a ResourceManager,
        config_store: &'a ConfigStore,
        preferences: &'a Preferences,
        state_manager: &'a StateManager<LifecycleState>,
        command_executor: &'a CommandExecutor,
        scheduler: &'a Scheduler,
        history_store: &'a HistoryStore,
    ) -> Self {
        Self {
            event_bus,
            signal_bus,
            resource_manager,
            config_store,
            preferences,
            state_manager,
            command_executor,
            scheduler,
            history_store,
        }
    }
}

impl<'a> std::fmt::Debug for EngineRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EngineRef")
            .field("has_event_bus", &true)
            .field("has_signal_bus", &true)
            .field("has_resource_manager", &true)
            .field("has_config_store", &true)
            .finish()
    }
}