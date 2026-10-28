//! `TestEngine` — pre-configured lightweight engine for testing.
//!
//! Provides real (empty) subsystems for integration tests,
//! or mock subsystems for unit tests.

use crate::app_engine::capabilities::engine_ref::EngineRef;
use crate::app_engine::commands::command_executor::CommandExecutor;
use crate::app_engine::configuration::config_schema::ConfigSchema;
use crate::app_engine::configuration::config_store::ConfigStore;
use crate::app_engine::configuration::preferences::Preferences;
use crate::app_engine::events::event_bus::EventBus;
use crate::app_engine::history::history_store::HistoryStore;
use crate::app_engine::lifecycle::lifecycle_state::LifecycleState;
use crate::app_engine::resources::resource_manager::ResourceManager;
use crate::app_engine::scheduling::scheduler::Scheduler;
use crate::app_engine::state::state_manager::StateManager;
use crate::app_engine::tasks::task_manager::TaskManager;

/// A lightweight engine for testing.
///
/// Uses real (empty) subsystems so handlers can access EngineRef.
/// For pure unit tests, use the individual mock types.
pub struct TestEngine {
    pub event_bus: EventBus,
    pub signal_bus: crate::app_engine::signals::signal_bus::SignalBus,
    pub resource_manager: ResourceManager,
    pub config_store: ConfigStore,
    pub preferences: Preferences,
    pub state_manager: StateManager<LifecycleState>,
    pub command_executor: CommandExecutor,
    pub task_manager: TaskManager,
    pub scheduler: Scheduler,
    pub history_store: HistoryStore,
}

impl TestEngine {
    pub fn new() -> Self {
        Self {
            event_bus: EventBus::new(),
            signal_bus: crate::app_engine::signals::signal_bus::SignalBus::new(),
            resource_manager: ResourceManager::new(),
            config_store: ConfigStore::new(ConfigSchema::new()),
            preferences: Preferences::new(),
            state_manager: StateManager::new(),
            command_executor: CommandExecutor::new(),
            task_manager: TaskManager::new(),
            scheduler: Scheduler::new(),
            history_store: HistoryStore::new(),
        }
    }

    /// Construct an EngineRef for passing to handlers.
    pub fn engine_ref(&self) -> EngineRef<'_> {
        EngineRef::new(
            &self.event_bus,
            &self.signal_bus,
            &self.resource_manager,
            &self.config_store,
            &self.preferences,
            &self.state_manager,
            &self.command_executor,
            &self.scheduler,
            &self.history_store,
        )
    }
}

impl Default for TestEngine {
    fn default() -> Self {
        Self::new()
    }
}