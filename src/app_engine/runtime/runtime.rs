//! The main App Engine runtime/orchestrator.

use std::sync::Arc;

use crate::app_engine::capabilities::engine_ref::EngineRef;
use crate::app_engine::commands::command_executor::CommandExecutor;
use crate::app_engine::commands::command::Command;
use crate::app_engine::commands::command_result::CommandResult;
use crate::app_engine::errors::command_error::CommandError;
use crate::app_engine::errors::engine_error::EngineError;
use crate::app_engine::errors::task_error::TaskError;
use crate::app_engine::configuration::config_schema::ConfigSchema;
use crate::app_engine::configuration::config_store::ConfigStore;
use crate::app_engine::configuration::preferences::Preferences;
use crate::app_engine::context::application_context::ApplicationContext;
use crate::app_engine::context::context_manager::ContextManager;
use crate::app_engine::events::event_bus::EventBus;
use crate::app_engine::history::history_store::HistoryStore;
use crate::app_engine::history::transaction::TransactionManager;
use crate::app_engine::lifecycle::lifecycle::Lifecycle;
use crate::app_engine::lifecycle::lifecycle_manager::LifecycleManager;
use crate::app_engine::lifecycle::lifecycle_state::LifecycleState;
use crate::app_engine::plugins::plugin_manager::PluginManager;
use crate::app_engine::resources::resource_manager::ResourceManager;
use crate::app_engine::runtime::runtime_state::RuntimeState;
use crate::app_engine::scheduling::scheduler::Scheduler;
use crate::app_engine::services::service_manager::ServiceManager;
use crate::app_engine::signals::signal_bus::SignalBus;
use crate::app_engine::state::state::StateId;
use crate::app_engine::state::state_manager::StateManager;
use crate::app_engine::tasks::task::TaskId;
use crate::app_engine::tasks::task_definition::TaskDefinition;
use crate::app_engine::tasks::task_manager::TaskManager;
use crate::app_engine::tasks::task_result::TaskResult;

pub const RUNTIME_STATE_ID_VALUE: u64 = 0;

pub struct AppRuntime {
    state: RuntimeState,
    lifecycle_manager: LifecycleManager,
    stop_requested: bool,

    context_manager: ContextManager,
    state_manager: StateManager<LifecycleState>,
    runtime_state_id: StateId,

    command_executor: CommandExecutor,
    task_manager: TaskManager,
    scheduler: Scheduler,

    event_bus: EventBus,
    signal_bus: Arc<SignalBus>,

    history_store: HistoryStore,
    transaction_manager: TransactionManager,

    resource_manager: Arc<ResourceManager>,

    service_manager: ServiceManager,

    plugin_manager: PluginManager,

    config_store: ConfigStore,
    preferences: Preferences,
}

impl AppRuntime {
    pub fn new() -> Result<Self, EngineError> {
        Self::with_application_id("app_shell")
    }

    pub fn with_context(context: ApplicationContext) -> Result<Self, EngineError> {
        let mut context_manager = ContextManager::new();
        context_manager.set_application_context(context);
        Self::finish_construction(context_manager)
    }

    pub fn with_application_id(application_id: impl Into<String>) -> Result<Self, EngineError> {
        let mut context_manager = ContextManager::new();
        context_manager.create_application_context(application_id);
        Self::finish_construction(context_manager)
    }

    fn finish_construction(context_manager: ContextManager) -> Result<Self, EngineError> {
        let state_manager = StateManager::new();
        let runtime_state_id = StateId::new(RUNTIME_STATE_ID_VALUE);
        state_manager.set(runtime_state_id, LifecycleState::Created);

        let config_store = ConfigStore::new(ConfigSchema::new());

        let resource_manager = Arc::new(ResourceManager::new());
        resource_manager.set_self_ref(Arc::downgrade(&resource_manager));

        Ok(Self {
            state: RuntimeState::Created,
            lifecycle_manager: LifecycleManager::new(),
            stop_requested: false,
            context_manager,
            state_manager,
            runtime_state_id,
            command_executor: CommandExecutor::new(),
            task_manager: TaskManager::new(),
            scheduler: Scheduler::new(),
            event_bus: EventBus::new(),
            signal_bus: Arc::new(SignalBus::new()),
            history_store: HistoryStore::new(),
            transaction_manager: TransactionManager::new(),
            resource_manager,
            service_manager: ServiceManager::new(),
            plugin_manager: PluginManager::new(),
            config_store,
            preferences: Preferences::new(),
        })
    }

    pub fn run(&mut self) -> Result<(), EngineError> {
        self.state = self.lifecycle_manager.run(self.state)?;
        Ok(())
    }

    pub fn stop_requested(&self) -> bool {
        self.stop_requested
    }

    pub fn context(&self) -> &ApplicationContext {
        self.context_manager
            .application_context()
            .expect("ApplicationContext must exist after construction")
    }

    pub fn state(&self) -> RuntimeState {
        self.state
    }

    pub fn context_manager(&self) -> &ContextManager {
        &self.context_manager
    }

    pub fn context_manager_mut(&mut self) -> &mut ContextManager {
        &mut self.context_manager
    }

    pub fn state_manager(&self) -> &StateManager<LifecycleState> {
        &self.state_manager
    }

    pub fn runtime_state_id(&self) -> StateId {
        self.runtime_state_id
    }

    pub fn command_executor(&self) -> &CommandExecutor {
        &self.command_executor
    }

    pub fn command_executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.command_executor
    }

    pub fn task_manager(&self) -> &TaskManager {
        &self.task_manager
    }

    pub fn task_manager_mut(&mut self) -> &mut TaskManager {
        &mut self.task_manager
    }

    pub fn scheduler(&self) -> &Scheduler {
        &self.scheduler
    }

    pub fn scheduler_mut(&mut self) -> &mut Scheduler {
        &mut self.scheduler
    }

    pub fn event_bus(&self) -> &EventBus {
        &self.event_bus
    }

    pub fn event_bus_mut(&mut self) -> &mut EventBus {
        &mut self.event_bus
    }

    pub fn signal_bus(&self) -> &SignalBus {
        self.signal_bus.as_ref()
    }

    pub fn signal_bus_mut(&mut self) -> &SignalBus {
        self.signal_bus.as_ref()
    }

    pub fn history_store(&self) -> &HistoryStore {
        &self.history_store
    }

    pub fn history_store_mut(&mut self) -> &mut HistoryStore {
        &mut self.history_store
    }

    pub fn transaction_manager(&self) -> &TransactionManager {
        &self.transaction_manager
    }

    pub fn transaction_manager_mut(&mut self) -> &mut TransactionManager {
        &mut self.transaction_manager
    }

    pub fn resource_manager(&self) -> &ResourceManager {
        &self.resource_manager
    }

    pub fn resource_manager_mut(&self) -> &ResourceManager {
        &self.resource_manager
    }

    pub fn service_manager(&self) -> &ServiceManager {
        &self.service_manager
    }

    pub fn service_manager_mut(&mut self) -> &mut ServiceManager {
        &mut self.service_manager
    }

    pub fn plugin_manager(&self) -> &PluginManager {
        &self.plugin_manager
    }

    pub fn plugin_manager_mut(&mut self) -> &mut PluginManager {
        &mut self.plugin_manager
    }

    pub fn config_store(&self) -> &ConfigStore {
        &self.config_store
    }

    pub fn config_store_mut(&mut self) -> &ConfigStore {
        &self.config_store
    }

    pub fn preferences(&self) -> &Preferences {
        &self.preferences
    }

    pub fn preferences_mut(&mut self) -> &Preferences {
        &self.preferences
    }

    pub fn engine_ref(&self) -> EngineRef<'_> {
        EngineRef::new(
            &self.event_bus,
            self.signal_bus.as_ref(),
            &self.resource_manager,
            &self.config_store,
            &self.preferences,
            &self.state_manager,
            &self.command_executor,
            &self.scheduler,
            &self.history_store,
        )
    }

    pub fn execute_command(&self, command: &Command) -> Result<CommandResult, CommandError> {
        let engine_ref = EngineRef::new(
            &self.event_bus,
            self.signal_bus.as_ref(),
            &self.resource_manager,
            &self.config_store,
            &self.preferences,
            &self.state_manager,
            &self.command_executor,
            &self.scheduler,
            &self.history_store,
        );
        self.command_executor.execute(command, &engine_ref)
    }

    pub fn start_task(&self, task_id: TaskId) -> Result<TaskResult, TaskError> {
        let signal_bus_arc = self.signal_bus.clone();
        self.task_manager.connect_output(task_id, signal_bus_arc);

        let engine_ref = EngineRef::new(
            &self.event_bus,
            self.signal_bus.as_ref(),
            &self.resource_manager,
            &self.config_store,
            &self.preferences,
            &self.state_manager,
            &self.command_executor,
            &self.scheduler,
            &self.history_store,
        );
        self.task_manager.start(task_id, &engine_ref)
    }

    #[cfg(feature = "async-runtime")]
    pub fn start_task_async(
        &self,
        task_id: TaskId,
        async_runtime: &crate::app_engine::concurrency::async_runtime::AsyncRuntime,
    ) -> Result<TaskResult, TaskError> {
        let signal_bus_arc = self.signal_bus.clone();
        self.task_manager.connect_output(task_id, signal_bus_arc);

        let engine_ref = EngineRef::new(
            &self.event_bus,
            self.signal_bus.as_ref(),
            &self.resource_manager,
            &self.config_store,
            &self.preferences,
            &self.state_manager,
            &self.command_executor,
            &self.scheduler,
            &self.history_store,
        );
        self.task_manager.start_async(task_id, &engine_ref, async_runtime)
    }

    #[cfg(feature = "async-runtime")]
    pub fn register_async_task(
        &self,
        definition: TaskDefinition,
        handler: Box<dyn crate::app_engine::concurrency::async_task_handler::AsyncTaskHandler>,
    ) -> Result<(), TaskError> {
        self.task_manager.register_async(definition, handler)
    }
}

impl Lifecycle for AppRuntime {
    fn initialize(&mut self) -> Result<(), EngineError> {
        self.state = self.lifecycle_manager.initialize(self.state)?;
        self.state_manager.set(self.runtime_state_id, self.state);

        let engine_ref = EngineRef::new(
            &self.event_bus,
            self.signal_bus.as_ref(),
            &self.resource_manager,
            &self.config_store,
            &self.preferences,
            &self.state_manager,
            &self.command_executor,
            &self.scheduler,
            &self.history_store,
        );
        self.service_manager.initialize_all(&engine_ref)?;
        self.plugin_manager.initialize_all(&engine_ref)?;

        Ok(())
    }

    fn start(&mut self) -> Result<(), EngineError> {
        self.state = self.lifecycle_manager.start(self.state)?;
        self.stop_requested = false;
        self.state_manager.set(self.runtime_state_id, self.state);

        let engine_ref = EngineRef::new(
            &self.event_bus,
            self.signal_bus.as_ref(),
            &self.resource_manager,
            &self.config_store,
            &self.preferences,
            &self.state_manager,
            &self.command_executor,
            &self.scheduler,
            &self.history_store,
        );
        self.service_manager.start_all(&engine_ref)?;
        self.plugin_manager.start_all(&engine_ref)?;

        Ok(())
    }

    fn stop(&mut self) -> Result<(), EngineError> {
        self.stop_requested = true;

        // Phase 26: Cancel all running tasks
        self.task_manager.cancel_all();

        // Stop plugins, then services (reverse order)
        self.plugin_manager.stop_all()?;
        self.service_manager.stop_all()?;

        self.state = self.lifecycle_manager.stop(self.state)?;
        self.state_manager.set(self.runtime_state_id, self.state);
        Ok(())
    }

    fn dispose(&mut self) -> Result<(), EngineError> {
        // Dispose plugins, then services
        self.plugin_manager.dispose_all()?;
        self.service_manager.dispose_all()?;

        // Phase 26: Unload all non-in-use resources
        self.resource_manager.unload_all();

        self.state = self.lifecycle_manager.dispose(self.state)?;
        self.state_manager.set(self.runtime_state_id, self.state);
        Ok(())
    }

    fn state(&self) -> LifecycleState {
        self.state
    }
}

impl Default for AppRuntime {
    fn default() -> Self {
        Self::new().expect("Default AppRuntime must construct")
    }
}

/// The public composition root for the App Engine.
pub type AppEngine = AppRuntime;