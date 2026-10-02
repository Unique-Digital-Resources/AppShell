# `19-error-reference.md` — Error Reference

## Overview

The App Engine has 15 error types. Each subsystem owns its errors. `EngineError` unifies them for top-level error handling via `From` impls.

```text
EngineError                     Top-level — converts from all subsystem errors
├── CommandError              Command execution
├── TaskError                 Task execution
├── ResourceError             Resource operations
├── ServiceError              Service lifecycle
├── PluginError               Plugin lifecycle/loading
├── SchedulingError          Scheduling operations
├── HistoryError              History operations
├── ConfigurationError       Config validation/persistence
├── PersistenceError         Storage backend
├── SerializationError       Serialization/deserialization
├── PluginSecurityError      Permission denial
├── ContextError             Context operations
├── StateError               State transitions
├── CommunicationError      Event handler errors
└── AsyncError               Async runtime (with async-runtime feature)
```

---

## 1. EngineError

The top-level engine error. All subsystem errors convert to `EngineError` via `From`.

### Variants

| Variant | Description |
|---|---|
| `InvalidLifecycleTransition { from: String, to: String }` | Invalid lifecycle state transition. |
| `AlreadyInitialized` | Engine is already initialized. |
| `NotInitialized` | Engine is not initialized. |
| `AlreadyRunning` | Engine is already running. |
| `AlreadyStopped` | Engine is already stopped. |
| `AlreadyDisposed` | Engine is already disposed. |
| `InitializationFailed(String)` | Initialization failed (reason). |
| `StartupFailed(String)` | Startup failed (reason). |
| `ShutdownFailed(String)` | Shutdown failed (reason). |
| `SubsystemFailure { subsystem: String, reason: String }` | A subsystem reported a failure. |

### Display Examples

```text
Invalid lifecycle transition: Created -> Running
Engine is already initialized
Initialization failed: database unreachable
service subsystem failure: Service 3 failed to start: db error
```

### Helpers

```rust
EngineError::invalid_transition("Created", "Running")
EngineError::subsystem("service", "Service 3 failed to start")
```

### From Conversions

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

### Usage

```rust
fn do_work(engine: &mut AppEngine) -> Result<(), EngineError> {
    engine.service_manager_mut().start_all(&engine.engine_ref())?;
    // ServiceError auto-converts to EngineError via From impl
    Ok(())
}

match engine.initialize() {
    Ok(()) => {},
    Err(EngineError::AlreadyInitialized) => { /* continue */ },
    Err(EngineError::InvalidLifecycleTransition { from, to }) => {
        eprintln!("Invalid: {} -> {}", from, to);
    },
    Err(EngineError::SubsystemFailure { subsystem, reason }) => {
        eprintln!("{} failed: {}", subsystem, reason);
    },
    Err(e) => eprintln!("Unexpected: {}", e),
}
```

---

## 2. CommandError

Command execution errors.

### Variants

| Variant | Description |
|---|---|
| `UnknownCommand { id: String }` | No handler registered for the command. |
| `InvalidArguments(String)` | Handler rejected the input. |
| `InvalidState` | Command executed in wrong state. |
| `NotAvailable(String)` | Command not available (reason). |
| `ExecutionFailed(String)` | Handler execution failed. |
| `Cancelled` | Command was cancelled. |
| `PermissionDenied(String)` | Permission denied (reason). |
| `NoHandler { id: String }` | Handler missing (internal inconsistency). |
| `AlreadyRegistered { id: String }` | Command already registered. |

### Display Examples

```text
Unknown command: app.missing
Invalid arguments: expected String
Command not available: requires admin
Command execution failed: disk full
Command cancelled
Permission denied: insufficient role
No handler registered for command: app.echo
Command already registered: app.echo
```

### When Each Occurs

| Variant | Triggered by |
|---|---|
| `UnknownCommand` | `execute()` with unregistered command ID |
| `InvalidArguments` | Handler rejects input (wrong type, missing field) |
| `ExecutionFailed` | Handler returned `Err` or panicked (Phase 24) |
| `Cancelled` | Command was cancelled |
| `AlreadyRegistered` | `register()` with duplicate ID |
| `NoHandler` | Internal inconsistency (should not happen) |

---

## 3. TaskError

Task execution and lifecycle errors.

### Variants

| Variant | Description |
|---|---|
| `NotFound { id: u64 }` | Task not found. |
| `UnknownDefinition { id: String }` | Task definition not registered. |
| `InvalidState { id: u64, current: String, requested: String }` | Invalid state transition. |
| `NotPausable { id: u64, current: String }` | Task doesn't support pause. |
| `NotCancellable { id: u64, current: String }` | Task doesn't support cancel. |
| `AlreadyRunning { id: u64 }` | Task is already running. |
| `AlreadyCompleted { id: u64 }` | Task is already completed. |
| `ExecutionFailed { id: u64, reason: String }` | Handler execution failed. |
| `Cancelled { id: u64 }` | Task was cancelled. |
| `InvalidDefinition { id: String }` | Invalid task definition. |
| `AlreadyRegistered { id: String }` | Task definition already registered. |

### Display Examples

```text
Task not found: 42
Unknown task definition: export
Task 3 invalid state transition: Pending -> Completed
Task 5 is not pausable (current: Pending)
Task 7 is not cancellable (current: Completed)
Task 9 execution failed: disk full
Task 11 was cancelled
Task definition already registered: render
```

### When Each Occurs

| Variant | Triggered by |
|---|---|
| `NotFound` | `start/pause/resume/cancel` with bogus ID |
| `UnknownDefinition` | `create()` with unregistered definition |
| `InvalidState` | `start()` on a non-Pending task, etc. |
| `NotPausable` | `pause()` on a task without pause support |
| `NotCancellable` | `cancel()` on a task without cancel support |
| `ExecutionFailed` | Handler returned `Err` or panicked (Phase 24) |
| `AlreadyRegistered` | `register()` with duplicate ID |

---

## 4. ResourceError

Resource operation errors.

### Variants

| Variant | Description |
|---|---|
| `NotFound { id: u64 }` | Resource not found. |
| `LoaderNotFound { resource_type: String }` | No loader for the resource type. |
| `LoadFailed { source: String, reason: String }` | Loading failed. |
| `InvalidResource { id: u64 }` | Invalid resource. |
| `AlreadyLoaded { id: u64 }` | Resource already loaded. |
| `AlreadyReleased { id: u64 }` | Resource already released. |
| `UnloadFailed { id: u64, reason: String }` | Unload failed. |
| `InvalidHandle { id: u64 }` | Invalid resource handle. |
| `NoData { id: u64 }` | Resource has no data. |
| `ReloadFailed { id: u64, reason: String }` | Reload failed. |
| `InUse { id: u64 }` | Resource is in use (can't unload). |

### Display Examples

```text
Resource not found: 7
No loader for resource type: image
Failed to load resource 'file:///missing.png': file not found
Resource 3 is in use and cannot be unloaded
Invalid resource handle: 99
Failed to reload resource 5: file changed
```

---

## 5. ServiceError

Service lifecycle errors.

### Variants

| Variant | Description |
|---|---|
| `NotFound { id: u64 }` | Service not found. |
| `InvalidState { id: u64, current: String, requested: String }` | Invalid state transition. |
| `AlreadyRegistered { id: String }` | Service already registered. |
| `StartFailed { id: u64, reason: String }` | Start failed (including panic — Phase 24). |
| `StopFailed { id: u64, reason: String }` | Stop failed (including panic). |
| `InitializeFailed { id: u64, reason: String }` | Initialize failed (including panic). |
| `DisposeFailed { id: u64, reason: String }` | Dispose failed (including panic). |

### Display Examples

```text
Service not found: 3
Service 3 invalid state transition: Running -> Initialized
Service already registered: logger
Service 5 failed to start: database unreachable
Service 7 failed to initialize: service panicked: init crashed!
```

---

## 6. PluginError

Plugin lifecycle and loading errors.

### Variants

| Variant | Description |
|---|---|
| `NotFound { id: u64 }` | Plugin not found. |
| `InvalidState { id: u64, current: String, requested: String }` | Invalid state transition. |
| `AlreadyRegistered { id: String }` | Plugin already registered. |
| `LoadFailed { source: String, reason: String }` | Loading failed. |
| `InitializeFailed { id: u64, reason: String }` | Initialize failed (including panic — Phase 24). |
| `StartFailed { id: u64, reason: String }` | Start failed (including panic). |
| `StopFailed { id: u64, reason: String }` | Stop failed (including panic). |
| `DisposeFailed { id: u64, reason: String }` | Dispose failed (including panic). |
| `DependencyNotMet { id: String, dependency: String }` | Dependency not registered. |

### Display Examples

```text
Plugin not found: 5
Plugin 5 invalid state transition: Loaded -> Unloaded
Plugin already registered: image-plugin
Failed to load plugin 'image.so': no suitable loader found
Plugin 3 failed to initialize: plugin panicked: init crashed!
Plugin 'a' requires dependency 'b' which is not registered
```

---

## 7. SchedulingError

Scheduling operation errors.

### Variants

| Variant | Description |
|---|---|
| `JobNotFound { id: u64 }` | Job not found. |
| `JobAlreadyCancelled { id: u64 }` | Job already cancelled. |
| `JobAlreadyCompleted { id: u64 }` | Job already completed. |
| `InvalidState { id: u64, current: String, requested: String }` | Invalid state transition. |

### Display Examples

```text
Job not found: 42
Job 42 is already cancelled
Job 42 invalid state: Scheduled -> Completed
```

---

## 8. HistoryError

History operation errors.

### Variants

| Variant | Description |
|---|---|
| `EntryNotFound { id: u64 }` | History entry not found. |
| `NoUndoAvailable` | No operations to undo. |
| `NoRedoAvailable` | No operations to redo. |
| `TransactionNotActive` | No active transaction. |
| `TransactionAlreadyActive` | Transaction already active. |
| `ReplayExhausted` | Replay entries exhausted. |

### Display Examples

```text
History entry not found: 42
No undo available
No redo available
No active transaction
Replay entries exhausted
```

---

## 9. ConfigurationError

Configuration validation and persistence errors.

### Variants

| Variant | Description |
|---|---|
| `ValidationFailed { key: String, reason: String }` | Schema validation failed. |
| `NotFound { key: String }` | Configuration key not found. |
| `InvalidType { key: String, expected: String, actual: String }` | Type mismatch. |
| `LoadFailed { reason: String }` | Configuration load failed. |
| `SaveFailed { reason: String }` | Configuration save failed. |

### Display Examples

```text
Configuration validation failed for 'worker_count': value 0 is below minimum 1
Configuration key not found: missing
Configuration 'count' has invalid type: expected integer, got string
Configuration load failed: file not found
Configuration save failed: permission denied
```

---

## 10. PersistenceError

Storage backend errors.

### Variants

| Variant | Description |
|---|---|
| `IoError { reason: String }` | I/O error. |
| `SerializationError { reason: String }` | Serialization error. |
| `DeserializationError { reason: String }` | Deserialization error. |
| `KeyNotFound { key: String }` | Key not found. |
| `BackendNotAvailable` | No backend configured. |

### Display Examples

```text
I/O error: permission denied
Serialization error: invalid format
Deserialization error: unexpected EOF
Key not found: app.name
No persistence backend available
```

### From Conversions

```rust
let _: PersistenceError = std::io::Error::from(...).into();
```

---

## 11. SerializationError

Serialization and deserialization errors.

### Variants

| Variant | Description |
|---|---|
| `NoSerializer { type_name: String }` | No serializer registered for type. |
| `SerializationFailed { type_name: String, reason: String }` | Serialization failed. |
| `DeserializationFailed { type_name: String, reason: String }` | Deserialization failed. |
| `TypeMismatch { expected: String, actual: String }` | Type mismatch during downcast. |

### Display Examples

```text
No serializer registered for type: my_domain::ExportInput
Failed to serialize my_domain::ExportInput: invalid format
Failed to deserialize my_domain::ExportInput: unexpected EOF
Type mismatch: expected ExportInput, got unknown
```

---

## 12. PluginSecurityError

Permission denial errors.

### Variants

| Variant | Description |
|---|---|
| `PermissionDenied { plugin_id: String, permission: String }` | Plugin lacks required permission. |
| `InsufficientPermissions { plugin_id: String, required: u32, actual: u32 }` | Insufficient permissions (bits). |

### Display Examples

```text
Plugin 'limited' lacks permission: LOAD_RESOURCES
Plugin 'limited' has insufficient permissions (has 00000001, needs 00000100)
```

---

## 13. ContextError

Context operation errors.

### Variants

| Variant | Description |
|---|---|
| `NoApplicationContext` | No application context exists. |
| `SessionNotFound` | Session not found. |
| `ExecutionNotFound` | Execution not found. |

### Display Examples

```text
No application context exists
Session not found
Execution not found
```

---

## 14. StateError

State transition errors.

### Variants

| Variant | Description |
|---|---|
| `StateNotFound` | State not found in store. |
| `TransitionMismatch { expected: String, actual: String }` | Current state doesn't match `from`. |

### Display Examples

```text
State not found in store
State transition mismatch: expected Created, got Running
```

---

## 15. CommunicationError

Event handler errors.

### Variants

| Variant | Description |
|---|---|
| `HandlerNotFound { id: u64 }` | Handler not found. |
| `SubscriptionNotFound { id: u64 }` | Signal subscription not found. |
| `AlreadySubscribed { topic: String }` | Already subscribed to topic. |

### Display Examples

```text
Event handler not found: 42
Signal subscription not found: 5
Already subscribed to: task.completed
```

---

## 16. AsyncError (with `async-runtime` feature)

Async runtime errors.

### Variants

| Variant | Description |
|---|---|
| `RuntimeCreationFailed(String)` | Runtime creation failed. |
| `TaskJoinError(String)` | Task join failed. |
| `AsyncExecutionFailed(String)` | Async execution failed. |

### Display Examples

```text
Async runtime creation failed: out of memory
Async task join error: task panicked
Async execution failed: future returned error
```

### From Conversions

```rust
let _: AsyncError = tokio::task::JoinError::from(...).into();
```

---

## Error Handling Patterns

### Top-Level Error Handling

```rust
use app_shell::app_engine::*;

fn main() {
    let mut engine = Bootstrap::create()
        .unwrap_or_else(|e| {
            eprintln!("Bootstrap failed: {}", e);
            std::process::exit(1);
        });

    if let Err(e) = engine.initialize() {
        match e {
            EngineError::AlreadyInitialized => {
                // Continue — already initialized
            },
            EngineError::SubsystemFailure { subsystem, reason } => {
                eprintln!("{} subsystem failed: {}", subsystem, reason);
                // A service or plugin failed during initialization
                // Engine continues — panic was caught
            },
            _ => {
                eprintln!("Initialization failed: {}", e);
                std::process::exit(1);
            }
        }
    }
}
```

### Subsystem-Specific Error Handling

```rust
// Command execution
match engine.execute_command(&cmd) {
    Ok(result) => { /* handle result */ },
    Err(CommandError::UnknownCommand { id }) => {
        eprintln!("No command: {}", id);
    },
    Err(CommandError::ExecutionFailed(msg)) => {
        // Could be handler error OR handler panic (Phase 24)
        eprintln!("Failed: {}", msg);
    },
    Err(e) => eprintln!("{}", e),
}

// Task execution
match engine.start_task(task_id) {
    Ok(result) => { /* handle result */ },
    Err(TaskError::NotFound { id }) => eprintln!("Task {} not found", id),
    Err(TaskError::InvalidState { current, requested, .. }) => {
        eprintln!("Can't {}: {} -> {}", requested, current, requested);
    },
    Err(TaskError::ExecutionFailed { reason, .. }) => {
        // Could be handler error OR handler panic (Phase 24)
        eprintln!("Failed: {}", reason);
    },
    Err(e) => eprintln!("{}", e),
}
```

### Converting to EngineError

```rust
fn do_work(engine: &mut AppEngine) -> Result<(), EngineError> {
    // All subsystem errors auto-convert via From
    engine.service_manager_mut().start_all(&engine.engine_ref())?;
    // ServiceError → EngineError::SubsystemFailure

    engine.execute_command(&cmd)?;
    // CommandError → EngineError::SubsystemFailure

    engine.start_task(task_id)?;
    // TaskError → EngineError::SubsystemFailure

    Ok(())
}
```

### Panic-Isolated Errors (Phase 24)

When a handler panics, the error message contains `"handler panicked: "` prefix:

```rust
// Task handler panic → TaskError::ExecutionFailed
// Message: "handler panicked: task handler crashed!"

// Command handler panic → CommandError::ExecutionFailed
// Message: "handler panicked: command handler crashed!"

// Service lifecycle panic → ServiceError::*Failed
// Message: "service panicked: service init crashed!"

// Plugin lifecycle panic → PluginError::*Failed
// Message: "plugin panicked: plugin init crashed!"
```

### Detecting Panic-Origin Errors

```rust
match engine.start_task(task_id) {
    Err(TaskError::ExecutionFailed { reason, .. }) if reason.contains("handler panicked") => {
        eprintln!("Handler panicked: {}", reason);
        // Can retry, log, or escalate
    },
    Err(TaskError::ExecutionFailed { reason, .. }) => {
        eprintln!("Handler failed: {}", reason);
    },
    Ok(result) => { /* ... */ },
    Err(e) => eprintln!("{}", e),
}
```

---

## Quick Reference: All Error Types

| Error Type | Owned By | Subsystem |
|---|---|---|
| `EngineError` | App Engine | Top-level |
| `CommandError` | Commands | Command execution |
| `TaskError` | Tasks | Task execution |
| `ResourceError` | Resources | Resource operations |
| `ServiceError` | Services | Service lifecycle |
| `PluginError` | Plugins | Plugin lifecycle |
| `SchedulingError` | Scheduling | Job operations |
| `HistoryError` | History | Undo/redo/replay |
| `ConfigurationError` | Configuration | Config validation |
| `PersistenceError` | Persistence | Storage backend |
| `SerializationError` | Serialization | Serializer |
| `PluginSecurityError` | Plugins | Permission enforcement |
| `ContextError` | Context | Context creation |
| `StateError` | State | State transitions |
| `CommunicationError` | Events/Signals | Handler/subscription |
| `AsyncError` | Concurrency | Async runtime (feature) |

---

## Next Steps

- [18-public-api-reference.md](18-public-api-reference.md) — Complete public API reference.
- [20-quick-start.md](20-quick-start.md) — 5-minute getting started guide.
- [02-architecture.md](02-architecture.md) — Architecture and error ownership.