//! Public boundary of the App Engine.

pub mod commands;
pub mod context;
pub mod errors;
pub mod events;
pub mod history;
pub mod lifecycle;
pub mod resources;
pub mod runtime;
pub mod scheduling;
pub mod signals;
pub mod state;
pub mod tasks;
pub mod services;
pub mod plugins;
pub mod configuration;
pub mod capabilities;
pub mod execution;
pub mod persistence;
pub mod concurrency;
pub mod metadata;
pub mod testing;
pub mod serialization;

// Commands
pub use commands::command::{Command, CommandId, CommandInput};
pub use commands::command_context::CommandContext;
pub use commands::command_definition::{CommandDefinition, CommandDefinitionId};
pub use commands::command_executor::{CommandExecutor, CommandHandler};
pub use commands::command_registry::CommandRegistry;
pub use commands::command_result::CommandResult;

// Context
pub use context::application_context::ApplicationContext;
pub use context::context::{ApplicationId, Context, ExecutionId, SessionId};
pub use context::context_manager::ContextManager;
pub use context::execution_context::ExecutionContext;
pub use context::session_context::SessionContext;

// Concurrency
pub use concurrency::async_command_handler::{AsyncCommandFuture, AsyncCommandHandler};
pub use concurrency::async_task_handler::{AsyncTaskFuture, AsyncTaskHandler};
pub use concurrency::task_pool::TaskPool;

// Errors
pub use errors::command_error::CommandError;
pub use errors::communication_error::CommunicationError;
pub use errors::context_error::ContextError;
pub use errors::engine_error::EngineError;
pub use errors::history_error::HistoryError;
pub use errors::resource_error::ResourceError;
pub use errors::scheduling_error::SchedulingError;
pub use errors::state_error::StateError;
pub use errors::task_error::TaskError;
pub use errors::service_error::ServiceError;
pub use errors::plugin_error::PluginError;
pub use errors::configuration_error::ConfigurationError;
pub use errors::persistence_error::PersistenceError;
pub use errors::plugin_security_error::PluginSecurityError;
pub use errors::serialization_error::SerializationError;

// Events
pub use events::event::{Event, EventId, EventType};
pub use events::event_bus::EventBus;
pub use events::event_dispatcher::{EventDispatcher, HandlerId};
pub use events::event_handler::EventHandler;
pub use events::event_dispatcher::EventPriority;

// History
pub use history::history_entry::{HistoryEntry, HistoryEntryId};
pub use history::history_store::HistoryStore;
pub use history::redo::RedoResult;
pub use history::replay::Replay;
pub use history::transaction::{TransactionId, TransactionManager};
pub use history::undo::UndoResult;

// Lifecycle
pub use lifecycle::lifecycle::Lifecycle;
pub use lifecycle::lifecycle_manager::LifecycleManager;
pub use lifecycle::lifecycle_state::LifecycleState;

// Resources
pub use resources::resource::{Resource, ResourceId, ResourceKey, ResourceState};
pub use resources::resource_cache::ResourceCache;
pub use resources::resource_handle::ResourceHandle;
pub use resources::resource_loader::ResourceLoader;
pub use resources::resource_manager::ResourceManager;
pub use resources::resource_dependency::ResourceDependencyGraph;

// Runtime
pub use runtime::bootstrap::Bootstrap;
pub use runtime::runtime::{AppEngine, AppRuntime};
pub use runtime::runtime_state::RuntimeState;

// Scheduling
pub use scheduling::job::{Job, JobId, JobState};
pub use scheduling::queue::JobQueue;
pub use scheduling::schedule::Schedule;
pub use scheduling::scheduler::Scheduler;
pub use scheduling::trigger::Trigger;
pub use scheduling::retry_policy::RetryPolicy;

// Signals
pub use signals::signal::{Signal, SignalId, SignalType};
pub use signals::signal_bus::SignalBus;
pub use signals::subscription::{SignalCallback, Subscription, SubscriptionId};

// State
pub use state::state::StateId;
pub use state::state_manager::StateManager;
pub use state::state_store::StateStore;
pub use state::state_transition::StateTransition;

// Tasks
pub use tasks::task::{Task, TaskId, TaskInput};
pub use tasks::task_context::TaskContext;
pub use tasks::task_definition::{TaskDefinition, TaskDefinitionId};
pub use tasks::task_executor::{TaskExecutor, TaskHandler};
pub use tasks::task_manager::TaskManager;
pub use tasks::task_result::{TaskResult, TaskResultStatus};
pub use tasks::task_state::TaskState;

// Services
pub use services::service::{Service, ServiceDefinition, ServiceId, ServiceState};
pub use services::service_manager::ServiceManager;
pub use services::service_registry::ServiceRegistry;

// Plugins
pub use plugins::plugin::{Plugin, PluginId, PluginState};
pub use plugins::plugin_context::PluginContext;
pub use plugins::plugin_loader::PluginLoader;
pub use plugins::plugin_manager::PluginManager;
pub use plugins::plugin_manifest::PluginManifest;
pub use plugins::plugin_registry::PluginRegistry;
pub use plugins::plugin_permissions::PluginPermissions;
pub use plugins::dynamic_loader::DynamicPluginLoader;
pub use plugins::plugin_permissions::{
    REGISTER_COMMANDS, REGISTER_TASKS, LOAD_RESOURCES, REGISTER_SERVICES,
    SUBSCRIBE_EVENTS, PUBLISH_EVENTS, READ_CONFIG, WRITE_CONFIG, FULL_ACCESS,
};

// Configuration
pub use configuration::config::Config;
pub use configuration::config_schema::{ConfigPropertyDefinition, ConfigSchema};
pub use configuration::config_store::ConfigStore;
pub use configuration::preferences::Preferences;

// Capabilities
pub use capabilities::engine_ref::EngineRef;
pub use capabilities::permission_checked_ref::PermissionCheckedEngineRef;

// Execution Control
pub use execution::cancellation_token::CancellationToken;
pub use execution::deadline::Deadline;
pub use execution::output_channel::TaskOutputChannel;

// Persistence
pub use persistence::file_backend::FileStorageBackend;
pub use persistence::memory_backend::MemoryStorageBackend;
pub use persistence::storage_backend::StorageBackend;

// Metadata
pub use metadata::payload_schema::PayloadSchema;
pub use metadata::version::{Version, Versioned};

// Testing
pub use testing::mocks::{MockCommandExecutor, MockEventBus, MockSignalBus};
pub use testing::test_engine::TestEngine;
pub use testing::assertions::{
    assert_command_executed, assert_command_not_executed,
    assert_event_published, assert_event_not_published,
    assert_signal_emitted,
};

// Serialization
pub use serialization::serialization_registry::SerializationRegistry;
pub use serialization::serializer::Serializer;

// Async (only with feature)
#[cfg(feature = "async-runtime")]
pub use concurrency::async_runtime::AsyncRuntime;
#[cfg(feature = "async-runtime")]
pub use errors::async_error::AsyncError;
#[cfg(feature = "async-runtime")]
pub use tasks::task_executor::HandlerKind;