# Public API Reference

## Overview

All types, traits, and functions re-exported from `app_shell::app_engine::*`. This is the complete public surface that consumers depend on.

```rust
use app_shell::app_engine::*;
```

---

## Core

| Type | Description |
|---|---|
| `AppEngine` | The composition root. Type alias for `AppRuntime`. Owns all subsystems. `Send + Sync`. |
| `Bootstrap` | Constructs a fully wired `AppEngine`. |
| `Lifecycle` | Trait: `initialize()`, `start()`, `stop()`, `dispose()`, `state()`. |
| `RuntimeState` | Enum: `Created`, `Initialized`, `Running`, `Stopped`, `Disposed`. |

### Bootstrap Methods

```rust
Bootstrap::create() -> Result<AppEngine, EngineError>
Bootstrap::create_with_context(ctx: ApplicationContext) -> Result<AppEngine, EngineError>
```

### AppEngine Methods

```rust
// Lifecycle (from Lifecycle trait)
engine.initialize() -> Result<(), EngineError>
engine.start() -> Result<(), EngineError>
engine.run() -> Result<(), EngineError>
engine.stop() -> Result<(), EngineError>
engine.dispose() -> Result<(), EngineError>
engine.state() -> RuntimeState

// Command execution
engine.execute_command(cmd: &Command) -> Result<CommandResult, CommandError>

// Task execution
engine.start_task(task_id: TaskId) -> Result<TaskResult, TaskError>

// Async (with async-runtime feature)
engine.start_task_async(task_id, &runtime) -> Result<TaskResult, TaskError>
engine.register_async_task(def, handler) -> Result<(), TaskError>

// EngineRef construction
engine.engine_ref() -> EngineRef<'_>

// Subsystem accessors
engine.context_manager() -> &ContextManager
engine.state_manager() -> &StateManager<LifecycleState>
engine.command_executor() -> &CommandExecutor
engine.task_manager() -> &TaskManager
engine.scheduler() -> &Scheduler
engine.event_bus() -> &EventBus
engine.signal_bus() -> &SignalBus
engine.history_store() -> &HistoryStore
engine.transaction_manager() -> &TransactionManager
engine.resource_manager() -> &ResourceManager
engine.service_manager() -> &ServiceManager
engine.plugin_manager() -> &PluginManager
engine.config_store() -> &ConfigStore
engine.preferences() -> &Preferences

// Mutable accessors (*_mut variants for setup-time operations)
engine.command_executor_mut() -> &mut CommandExecutor
engine.task_manager_mut() -> &mut TaskManager
engine.scheduler_mut() -> &mut Scheduler
engine.event_bus_mut() -> &mut EventBus
engine.context_manager_mut() -> &mut ContextManager
engine.config_store_mut() -> &ConfigStore         // &self (Mutex internally)
engine.preferences_mut() -> &Preferences          // &self (Mutex internally)
engine.resource_manager_mut() -> &ResourceManager // &self (Mutex internally)
```

---

## Context

| Type | Description |
|---|---|
| `ApplicationContext` | Application-level identity. |
| `SessionContext` | Session within an application. |
| `ExecutionContext` | One particular operation. |
| `ContextManager` | Creates/tracks contexts. |
| `ApplicationId` | Typed ID for `ApplicationContext`. `Copy`, `Hash`, `Ord`. |
| `SessionId` | Typed ID for `SessionContext`. `Copy`, `Hash`, `Ord`. |
| `ExecutionId` | Typed ID for `ExecutionContext`. `Copy`, `Hash`, `Ord`. |
| `ContextError` | Errors from context operations. |

### ContextManager Methods

```rust
mgr.create_application_context(id: impl Into<String>) -> ApplicationId
mgr.set_application_context(ctx: ApplicationContext) -> ApplicationId
mgr.create_session_context() -> Result<SessionId, ContextError>
mgr.create_execution_context(session_id: SessionId) -> Result<ExecutionId, ContextError>
mgr.application_context() -> Option<&ApplicationContext>
mgr.session_context(id: SessionId) -> Option<&SessionContext>
mgr.execution_context(id: ExecutionId) -> Option<&ExecutionContext>
mgr.session_count() -> usize
mgr.execution_count() -> usize
```

---

## State

| Type | Description |
|---|---|
| `StateId` | Typed key for state slots. `Copy`, `Hash`, `Ord`. |
| `StateManager<S>` | Validates transitions. Thread-safe (`Mutex`). |
| `StateStore<S>` | In-memory key-value store. Thread-safe (`Mutex`). |
| `StateTransition<S>` | Represents a change from one state to another. |
| `StateError` | Errors from state operations. |

### StateManager Methods

```rust
mgr.set(id: StateId, state: S)
mgr.current(id: StateId) -> Option<S>
mgr.transition(id, from: S, to: S) -> Result<(), StateError>
mgr.apply_transition(id, &transition) -> Result<(), StateError>
mgr.contains(id: StateId) -> bool
mgr.remove(id: StateId) -> Option<S>
mgr.store() -> &StateStore<S>
```

---

## Commands

| Type | Description |
|---|---|
| `Command` | An executable intent instance. |
| `CommandDefinition` | Metadata describing a command. |
| `CommandDefinitionId` | Typed ID for command definitions. |
| `CommandHandler` | Trait: `execute(&self, input, context, engine) -> Result<CommandResult, CommandError>`. |
| `CommandInput` | Type-erased input with optional serialized bytes. |
| `CommandResult` | Output of command execution. |
| `CommandError` | Command execution errors. |
| `CommandExecutor` | Executes commands. `execute(&self)`. |
| `CommandRegistry` | Stores command definitions and handlers. |
| `CommandContext` | Execution environment for a command. Has `Deadline`. |

### CommandDefinition Builder

```rust
CommandDefinition::new(id, name, description)
    .with_version(Version::new(1, 0, 0))
    .with_input_schema(PayloadSchema::of::<InputType>())
    .with_output_schema(PayloadSchema::of::<OutputType>())
    .with_metadata("key", "value")
    .is_compatible_with(&version) -> bool
```

### CommandInput Serialization

```rust
CommandInput::new(data: T) -> CommandInput
CommandInput::empty() -> CommandInput
CommandInput::from_bytes(type_name, bytes) -> CommandInput
CommandInput::with_serialized(data, bytes) -> CommandInput
input.get::<T>() -> Option<&T>
input.serialized() -> Option<&[u8]>
input.ensure_serialized(&registry) -> Result<(), SerializationError>
input.ensure_deserialized(&registry) -> Result<(), SerializationError>
```

---

## Tasks

| Type | Description |
|---|---|
| `Task` | A managed work instance. |
| `TaskDefinition` | Metadata describing a task type. |
| `TaskDefinitionId` | Typed ID for task definitions. |
| `TaskHandler` | Trait: `execute(&self, input, context, engine) -> Result<TaskResult, TaskError>`. |
| `TaskInput` | Type-erased input with optional serialized bytes. |
| `TaskResult` | Output of task execution. |
| `TaskResultStatus` | Enum: `Completed`, `Cancelled`, `Failed`. |
| `TaskState` | Enum: `Pending`, `Running`, `Paused`, `Cancelled`, `Completed`, `Failed`. |
| `TaskError` | Task execution errors. |
| `TaskManager` | Creates/tracks/executes tasks. Thread-safe (`Mutex`). |
| `TaskExecutor` | Stores handlers and executes tasks. |
| `TaskContext` | Execution environment. Has `CancellationToken`, `Deadline`, `TaskOutputChannel`. |
| `TaskId` | Typed ID for tasks. `Copy`, `Hash`, `Ord`. |
| `HandlerKind` | Enum: `Sync`, `Async` (with feature). |

### TaskDefinition Builder

```rust
TaskDefinition::new(id, name, description)
    .with_pause_support()
    .with_cancel_support()
    .with_version(Version::new(1, 0, 0))
    .with_input_schema(PayloadSchema::of::<InputType>())
    .with_output_schema(PayloadSchema::of::<OutputType>())
    .with_metadata("key", "value")
    .is_compatible_with(&version) -> bool
```

### TaskManager Methods

```rust
mgr.register(definition, handler) -> Result<(), TaskError>
mgr.create(def_id, context, input) -> Result<TaskId, TaskError>
mgr.start(id, &engine_ref) -> Result<TaskResult, TaskError>
mgr.cancel(id) -> Result<(), TaskError>
mgr.pause(id) -> Result<(), TaskError>
mgr.resume(id) -> Result<(), TaskError>
mgr.cancel_all() -> Vec<TaskId>
mgr.get_state(id) -> Option<TaskState>
mgr.count() -> usize
mgr.contains(id) -> bool
mgr.remove(id) -> Option<Task>
mgr.connect_output(id, signal_bus_arc)
mgr.task_execution_id(id) -> Option<ExecutionId>
mgr.task_context_task_id(id) -> Option<TaskId>

// Async (with feature)
mgr.register_async(definition, handler) -> Result<(), TaskError>
mgr.start_async(id, &engine_ref, &async_runtime) -> Result<TaskResult, TaskError>
```

---

## Scheduling

| Type | Description |
|---|---|
| `Schedule` | Enum: `Immediate`, `Delayed(Duration)`, `AtTime(Instant)`, `Interval(Duration)`. |
| `Scheduler` | Coordinates scheduled execution. Thread-safe (`Mutex`/`RwLock`). |
| `Job` | One scheduled occurrence. `Clone`. |
| `JobId` | Typed ID for jobs. `Copy`, `Hash`, `Ord`. |
| `JobState` | Enum: `Scheduled`, `Waiting`, `Queued`, `Dispatched`, `Executing`, `Completed`, `Cancelled`, `Expired`. |
| `JobQueue` | Priority-ordered queue. |
| `Trigger` | Enum: `Manual`, `OnEvent(String)`, `OnSignal(String)`, `OnCondition(String)`. |
| `RetryPolicy` | Enum: `None`, `Fixed{max, delay}`, `Exponential{max, base, factor}`. |
| `SchedulingError` | Scheduling errors. |

### Scheduler Methods

```rust
sched.submit(task_id, schedule, priority) -> JobId
sched.submit_with_trigger(task_id, schedule, trigger, priority) -> JobId
sched.submit_with_retry(task_id, schedule, priority, retry_policy) -> JobId
sched.tick()
sched.dispatch() -> Option<Job>
sched.cancel(job_id) -> Result<(), SchedulingError>
sched.pause()
sched.resume()
sched.is_paused() -> bool
sched.satisfy_trigger(name)
sched.unsatisfy_trigger(name)
sched.is_trigger_satisfied(&trigger) -> bool
sched.handle_task_failure(&job) -> Option<JobId>
sched.get_job(id) -> Option<Job>   // Cloned
sched.list_jobs() -> Vec<Job>      // Cloned
sched.queue_len() -> usize
sched.scheduled_count() -> usize
sched.total_job_count() -> usize
```

### RetryPolicy Methods

```rust
policy.should_retry() -> bool
policy.max_attempts() -> u32
policy.delay_for_attempt(attempt) -> Option<Duration>
```

---

## Events

| Type | Description |
|---|---|
| `Event` | A past occurrence notification. |
| `EventId` | Typed ID. |
| `EventType` | Type alias: `String`. |
| `EventBus` | Pub/sub mechanism. |
| `EventHandler` | Trait: `handle(&self, event: &Event)`. |
| `EventPriority` | Enum: `High`, `Normal`, `Low`. |
| `HandlerId` | Typed ID for subscriptions. `Copy`, `Hash`. |
| `EventDispatcher` | Internal routing (not typically used directly). |

### EventBus Methods

```rust
bus.subscribe(event_type, handler) -> HandlerId
bus.subscribe_with_priority(event_type, handler, priority) -> HandlerId
bus.unsubscribe(event_type, id) -> bool
bus.publish(&event)
bus.handler_count(event_type) -> usize
bus.total_handlers() -> usize
```

### Event Serialization

```rust
Event::new(event_type) -> Event
Event::from_serialized(event_type, payload_type_name, bytes) -> Event
Event::with_payload(data: T) -> Event
Event::with_payload_and_serialized(data, bytes) -> Event
event.ensure_serialized(&registry) -> Result<(), SerializationError>
event.ensure_deserialized(&registry) -> Result<(), SerializationError>
event.serialized_payload() -> Option<&[u8]>
```

---

## Signals

| Type | Description |
|---|---|
| `Signal` | A lightweight change notification. |
| `SignalId` | Typed ID. |
| `SignalType` | Type alias: `String`. |
| `SignalBus` | Pub/sub mechanism for signals. Thread-safe (`Mutex`). |
| `SubscriptionId` | Typed ID. `Copy`, `Hash`. |
| `Subscription` | Represents one subscription. |
| `SignalCallback` | Type alias: `Box<dyn Fn(&Signal) + Send + Sync>`. |

### SignalBus Methods

```rust
bus.subscribe(signal_type, callback) -> SubscriptionId
bus.unsubscribe(signal_type, id) -> bool
bus.emit(&signal)
bus.subscriber_count(signal_type) -> usize
bus.total_subscribers() -> usize
```

---

## History

| Type | Description |
|---|---|
| `HistoryEntry` | One recorded operation. |
| `HistoryEntryId` | Typed ID. `Copy`, `Hash`. |
| `HistoryStore` | Ordered entries with undo/redo. Thread-safe (`Mutex`). |
| `UndoResult` | Result of undo. |
| `RedoResult` | Result of redo. |
| `TransactionId` | Typed ID. `Copy`, `Hash`. |
| `TransactionManager` | Groups operations into transactions. |
| `Replay` | Iterator over active entries. |
| `HistoryError` | History errors. |

### HistoryStore Methods

```rust
store.append(entry) -> HistoryEntryId
store.get(id) -> Option<(HistoryEntryId, String, bool)>
store.current() -> Option<(HistoryEntryId, String, bool)>
store.list() -> Vec<(HistoryEntryId, String, bool)>
store.active() -> Vec<(HistoryEntryId, String, bool)>
store.undo() -> UndoResult
store.redo() -> RedoResult
store.replay() -> Replay
store.position() -> usize
store.entry_count() -> usize
store.active_count() -> usize
store.can_undo() -> bool
store.can_redo() -> bool
store.clear()
store.remove_transaction(tx_id)
store.has_backend() -> bool
store.with_entry(id, |e| ...) -> Option<R>
store.with_current(|e| ...) -> Option<R>
```

---

## Resources

| Type | Description |
|---|---|
| `Resource` | Managed data with lifecycle. |
| `ResourceId` | Typed ID. `Copy`, `Hash`, `Ord`. |
| `ResourceHandle` | RAII reference. Auto-releases on drop. |
| `ResourceLoader` | Trait: `resource_type()`, `load(source)`. |
| `ResourceManager` | Load/cache/track resources. Thread-safe (`Mutex`/`RwLock`). |
| `ResourceCache` | In-memory cache. |
| `ResourceDependencyGraph` | Tracks resource dependencies. |
| `ResourceKey` | Composite key (type, source). |
| `ResourceState` | Enum: `Unloaded`, `Loading`, `Loaded`, `Failed`, `Released`. |
| `ResourceError` | Resource errors. |

### ResourceManager Methods

```rust
mgr.register_loader(loader)
mgr.register_dependency(resource_key, dependency_key)
mgr.load(resource_type, source) -> Result<ResourceHandle, ResourceError>
mgr.acquire(&handle) -> Result<(), ResourceError>
mgr.release(handle) -> Result<(), ResourceError>
mgr.release_by_id(id) -> Result<(), ResourceError>
mgr.unload(id) -> Result<(), ResourceError>
mgr.reload(id) -> Result<(), ResourceError>
mgr.unload_all() -> usize
mgr.exists(id) -> bool
mgr.resource_count() -> usize
mgr.active_count(id) -> usize
mgr.is_in_use(id) -> bool
mgr.with_resource(id, |r| ...) -> Option<R>
mgr.dependencies_of(key) -> Vec<String>
mgr.has_dependents(key) -> bool
mgr.set_self_ref(weak: Weak<ResourceManager>)
```

---

## Services

| Type | Description |
|---|---|
| `Service` | Trait: `initialize()`, `start()`, `stop()`, `dispose()`. |
| `ServiceDefinition` | Metadata. |
| `ServiceManager` | Lifecycle coordinator. Panic-isolated. |
| `ServiceState` | Enum: `Registered`, `Initialized`, `Running`, `Stopped`, `Disposed`. |
| `ServiceId` | Typed ID. `Copy`, `Hash`. |
| `ServiceRegistry` | Discovery. |
| `ServiceError` | Service errors. |

### Service Trait

```rust
trait Service: Send + Sync {
    fn service_type(&self) -> &str;
    fn initialize(&mut self, engine: &EngineRef) -> Result<(), ServiceError>;
    fn start(&mut self, engine: &EngineRef) -> Result<(), ServiceError>;
    fn stop(&mut self) -> Result<(), ServiceError>;
    fn dispose(&mut self) -> Result<(), ServiceError>;
}
```

---

## Plugins

| Type | Description |
|---|---|
| `Plugin` | Trait: `initialize()`, `start()`, `stop()`, `dispose()`. |
| `PluginManifest` | Static description. |
| `PluginManager` | Lifecycle coordinator. Panic-isolated. |
| `PluginLoader` | Trait: `can_load()`, `load()`. |
| `PluginPermissions` | Bitflags for capabilities. |
| `PluginContext` | Identity + metadata. |
| `PluginRegistry` | Discovery. |
| `DynamicPluginLoader` | Stub for `.so`/`.dll` loading. |
| `PluginId` | Typed ID. `Copy`, `Hash`. |
| `PluginState` | Enum: `Discovered`, `Loaded`, `Initialized`, `Started`, `Stopped`, `Unloaded`. |
| `PluginError` | Plugin errors. |
| `PluginSecurityError` | Permission denied errors. |

### Plugin Trait

```rust
trait Plugin: Send + Sync {
    fn id(&self) -> &str;
    fn initialize(&mut self, ctx: &PluginContext, engine: &EngineRef) -> Result<(), PluginError>;
    fn start(&mut self, engine: &EngineRef) -> Result<(), PluginError>;
    fn stop(&mut self) -> Result<(), PluginError>;
    fn dispose(&mut self) -> Result<(), PluginError>;
}
```

### Permission Constants

```rust
REGISTER_COMMANDS, REGISTER_TASKS, LOAD_RESOURCES, REGISTER_SERVICES,
SUBSCRIBE_EVENTS, PUBLISH_EVENTS, READ_CONFIG, WRITE_CONFIG, FULL_ACCESS
```

---

## Configuration

| Type | Description |
|---|---|
| `Config` | Key-value configuration. `Clone`. |
| `ConfigSchema` | Validation rules. |
| `ConfigPropertyDefinition` | Single property definition. |
| `ConfigStore` | Persistent, thread-safe, change-notifying store. |
| `Preferences` | User-facing settings. Thread-safe. |
| `ConfigurationError` | Config errors. |

### ConfigStore Methods

```rust
ConfigStore::new(schema) -> ConfigStore
ConfigStore::with_backend(schema, backend) -> ConfigStore
ConfigStore::with_memory_backend(schema) -> ConfigStore
store.load() -> Config               // Clone
store.save(config) -> Result<(), ConfigurationError>
store.get(key) -> Option<String>     // Owned
store.set(key, value) -> Result<(), ConfigurationError>
store.remove(key) -> Option<String>  // Owned
store.reset()
store.contains(key) -> bool
store.key_count() -> usize
store.has_backend() -> bool
store.on_changed(|key, old, new| { ... })
```

---

## Capabilities

| Type | Description |
|---|---|
| `EngineRef<'a>` | Read-only bundle of subsystem references passed to handlers. |
| `PermissionCheckedEngineRef<'a>` | Wrapper with `try_*` permission checks. |

### EngineRef Fields (all `pub`)

```rust
pub event_bus: &'a EventBus
pub signal_bus: &'a SignalBus
pub resource_manager: &'a ResourceManager
pub config_store: &'a ConfigStore
pub preferences: &'a Preferences
pub state_manager: &'a StateManager<LifecycleState>
pub command_executor: &'a CommandExecutor
pub scheduler: &'a Scheduler
pub history_store: &'a HistoryStore
```

### PermissionCheckedEngineRef Methods

```rust
PermissionCheckedEngineRef::new(engine, permissions, plugin_id)

// Read-only (always allowed)
checked.config_store() -> &ConfigStore
checked.preferences() -> &Preferences
checked.state_manager() -> &StateManager<LifecycleState>
checked.history_store() -> &HistoryStore
checked.scheduler() -> &Scheduler
checked.event_bus() -> &EventBus
checked.signal_bus() -> &SignalBus
checked.command_executor() -> &CommandExecutor
checked.resource_manager() -> &ResourceManager

// Permission-checked
checked.try_write_config() -> Result<&ConfigStore, PluginSecurityError>
checked.try_load_resource() -> Result<&ResourceManager, PluginSecurityError>
checked.try_publish_event() -> Result<&EventBus, PluginSecurityError>
checked.try_subscribe_events() -> Result<&EventBus, PluginSecurityError>
checked.try_register_commands() -> Result<&CommandExecutor, PluginSecurityError>
checked.try_register_tasks() -> Result<&CommandExecutor, PluginSecurityError>
checked.try_register_services() -> Result<&CommandExecutor, PluginSecurityError>
```

---

## Execution Control

| Type | Description |
|---|---|
| `CancellationToken` | Atomic flag. `Clone` (shared state). |
| `Deadline` | Optional time limit. |
| `TaskOutputChannel` | Progress + streaming output. `Clone`. |

### On TaskContext

```rust
ctx.is_cancelled() -> bool
ctx.is_expired() -> bool
ctx.deadline() -> &Deadline
ctx.cancellation_token() -> CancellationToken
ctx.output() -> &TaskOutputChannel
ctx.output().progress(value: f32)      // Emits task.progress_changed signal
ctx.output().push(data: T)             // Emits task.output signal
ctx.output().current_progress() -> f32
```

### On CommandContext

```rust
ctx.is_expired() -> bool
ctx.deadline() -> &Deadline
ctx.with_deadline(deadline) -> CommandContext  // Builder
```

---

## Persistence

| Type | Description |
|---|---|
| `StorageBackend` | Trait: `load()`, `save()`, `remove()`, `exists()`, `keys()`. |
| `MemoryStorageBackend` | In-memory. |
| `FileStorageBackend` | One file per key. |
| `PersistenceError` | Persistence errors. |

---

## Serialization

| Type | Description |
|---|---|
| `Serializer` | Trait: `type_name()`, `serialize()`, `deserialize()`. |
| `SerializationRegistry` | Stores serializers by type name. `RwLock`. |
| `SerializationError` | Serialization errors. |

### SerializationRegistry Methods

```rust
registry.register(serializer)
registry.has_serializer(type_name) -> bool
registry.serialize(&data, type_name) -> Result<Vec<u8>, SerializationError>
registry.deserialize(&bytes, type_name) -> Result<Box<dyn Any + Send>, SerializationError>
registry.registered_types() -> Vec<String>
registry.len() -> usize
```

---

## Concurrency

| Type | Description |
|---|---|
| `TaskPool` | Minimal thread pool. |
| `AsyncTaskHandler` | Async trait (with `async-runtime` feature). |
| `AsyncCommandHandler` | Async trait (with `async-runtime` feature). |
| `AsyncTaskFuture` | `Pin<Box<dyn Future<Output = Result<TaskResult, TaskError> + Send>>>` |
| `AsyncCommandFuture` | `Pin<Box<dyn Future<Output = Result<CommandResult, CommandError> + Send>>>` |
| `AsyncRuntime` | Tokio wrapper (with `async-runtime` feature). |
| `AsyncError` | Async errors (with `async-runtime` feature). |

### TaskPool Methods

```rust
TaskPool::new(size) -> TaskPool
pool.size() -> usize
pool.spawn(f: F)  // F: FnOnce() + Send + 'static
```

### AsyncRuntime Methods (with feature)

```rust
AsyncRuntime::new() -> Result<AsyncRuntime, AsyncError>
AsyncRuntime::new_current_thread() -> Result<AsyncRuntime, AsyncError>
runtime.block_on(future) -> F::Output
runtime.spawn(future) -> tokio::task::JoinHandle<F::Output>
runtime.handle() -> &tokio::runtime::Handle
```

---

## Testing

| Type | Description |
|---|---|
| `MockCommandExecutor` | Records executed commands. |
| `MockEventBus` | Records published events. |
| `MockSignalBus` | Records emitted signals. |
| `TestEngine` | Pre-configured lightweight engine. |

### Assertion Helpers

```rust
assert_command_executed(&exec, "cmd.id")
assert_command_not_executed(&exec, "cmd.missing")
assert_event_published(&bus, "event.type")
assert_event_not_published(&bus, "event.missing")
assert_signal_emitted(&bus, "signal.type")
```

---

## Metadata

| Type | Description |
|---|---|
| `Version` | Semantic version (`major.minor.patch`). `Ord`, `Display`. |
| `Versioned` | Trait: `version() -> &Version`. |
| `PayloadSchema` | Type description via `TypeId`. `PartialEq`, `Display`. |

### Version Methods

```rust
Version::new(major, minor, patch)
v.major() -> u32
v.minor() -> u32
v.patch() -> u32
v.is_compatible_with(&other) -> bool  // Same major
v.is_at_least(&other) -> bool         // >=
Version::default()                    // 1.0.0
```

### PayloadSchema Methods

```rust
PayloadSchema::of::<T>() -> PayloadSchema
PayloadSchema::of_with_description::<T>("desc") -> PayloadSchema
schema.matches::<T>() -> bool
schema.type_name() -> &'static str
schema.description() -> &str
```

---

## Errors (Complete List)

| Type | Scope |
|---|---|
| `EngineError` | Top-level engine errors. `From` for all subsystem errors. |
| `CommandError` | Command execution. |
| `TaskError` | Task execution. |
| `ResourceError` | Resource operations. |
| `ServiceError` | Service lifecycle. |
| `PluginError` | Plugin lifecycle/loading. |
| `SchedulingError` | Scheduling operations. |
| `HistoryError` | History operations. |
| `ConfigurationError` | Config validation/persistence. |
| `PersistenceError` | Storage backend. |
| `SerializationError` | Serialization/deserialization. |
| `PluginSecurityError` | Permission denial. |
| `ContextError` | Context operations. |
| `StateError` | State transitions. |
| `AsyncError` | Async runtime (with feature). |

### EngineError Unification

All subsystem errors convert to `EngineError` via `From`:

```rust
let _: EngineError = ServiceError::NotFound { id: 1 }.into();
let _: EngineError = PluginError::NotFound { id: 1 }.into();
let _: EngineError = CommandError::UnknownCommand { id: "x".into() }.into();
let _: EngineError = TaskError::NotFound { id: 1 }.into();
let _: EngineError = ResourceError::NotFound { id: 1 }.into();
let _: EngineError = ConfigurationError::NotFound { key: "x".into() }.into();
let _: EngineError = SchedulingError::JobNotFound { id: 1 }.into();
let _: EngineError = HistoryError::NoUndoAvailable.into();
let _: EngineError = CommunicationError::HandlerNotFound { id: 1 }.into();
let _: EngineError = ContextError::NoApplicationContext.into();
```

---

## Next Steps

- [19-error-reference.md](19-error-reference.md) — All error types in detail.
- [20-quick-start.md](20-quick-start.md) — 5-minute getting started guide.
- [02-architecture.md](02-architecture.md) — Architecture and boundaries.