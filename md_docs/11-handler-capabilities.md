# `11-handler-capabilities.md` — Handler Capabilities (EngineRef)

## Overview

`EngineRef` is the controlled environment through which handlers access engine systems. It is a **read-only bundle of references** passed to every `TaskHandler::execute()`, `CommandHandler::execute()`, `Service::initialize()/start()`, and `Plugin::initialize()/start()`.

```text
Handler (TaskHandler / CommandHandler)
    │
    ▼
  EngineRef (&T references)
    │
    ├── event_bus         → publish() ✅
    ├── signal_bus        → emit() ✅, subscribe() ✅ (Mutex)
    ├── resource_manager  → load() ✅, with_resource() ✅ (Mutex)
    ├── config_store      → get() ✅, set() ✅ (Mutex)
    ├── preferences       → get() ✅, set() ✅ (Mutex)
    ├── state_manager     → current() ✅, set() ✅ (Mutex)
    ├── command_executor  → execute() ✅, list() ✅
    ├── scheduler         → submit() ✅, tick() ✅, dispatch() ✅ (Mutex)
    ├── history_store     → append() ✅, undo() ✅, redo() ✅ (Mutex)
    │
    └── (TaskManager deliberately excluded for disjoint borrows)
```

---

## What Handlers CAN Do

### Publish Events

```rust
fn execute(&self, _: &TaskInput, _: &TaskContext, engine: &EngineRef) -> Result<TaskResult, TaskError> {
    engine.event_bus.publish(
        &Event::new("task.completed")
            .with_payload("task_42".to_string())
            .with_source("MyHandler")
    );
    Ok(TaskResult::completed())
}
```

Events are delivered to all registered handlers (priority-ordered, panic-isolated).

### Emit Signals

```rust
fn execute(&self, _: &TaskInput, _: &TaskContext, engine: &EngineRef) -> Result<TaskResult, TaskError> {
    engine.signal_bus.emit(
        &Signal::new("task.progress_changed")
            .with_payload(0.75_f32)
    );
    Ok(TaskResult::completed())
}
```

Signals are delivered synchronously to all subscribers.

### Subscribe to Signals (at runtime)

```rust
fn execute(&self, _: &TaskInput, _: &TaskContext, engine: &EngineRef) -> Result<TaskResult, TaskError> {
    // SignalBus uses Mutex internally — subscribe works through &SignalBus
    engine.signal_bus.subscribe(
        "state.changed",
        Box::new(|signal: &Signal| {
            println!("State changed: {}", signal.signal_type());
        }),
    );
    Ok(TaskResult::completed())
}
```

**Note:** Event subscription requires `&mut EventBus` (setup-time only). Signal subscription works at runtime because `SignalBus` uses `Mutex` internally.

### Load Resources (RAII handles auto-release)

```rust
fn execute(&self, _: &TaskInput, _: &TaskContext, engine: &EngineRef) -> Result<TaskResult, TaskError> {
    // Load a resource — returns RAII handle (Phase 25)
    let handle = engine.resource_manager.load("document", "file:///docs/my.doc")
        .map_err(|e| TaskError::ExecutionFailed { id: 0, reason: e.to_string() })?;

    // Access resource data through the closure pattern (Phase 23)
    let data = engine.resource_manager.with_resource(handle.resource_id(), |r| {
        r.data::<String>().map(|s| s.clone())
    }).flatten();

    // Handle auto-releases on drop when this function returns
    Ok(TaskResult::completed_with(data.unwrap_or_default()))
}
```

### Read and Write Configuration

```rust
fn execute(&self, _: &TaskInput, _: &TaskContext, engine: &EngineRef) -> Result<TaskResult, TaskError> {
    // Read config (thread-safe — Phase 23)
    let worker_count = engine.config_store.get_i64("worker_count").unwrap_or(4);

    // Write config (thread-safe, persists to backend if configured — Phase 15)
    engine.config_store.set("last_export", "2024-01-15").map_err(|e| {
        TaskError::ExecutionFailed { id: 0, reason: e.to_string() }
    })?;

    Ok(TaskResult::completed())
}
```

### Read and Write Preferences

```rust
fn execute(&self, _: &TaskInput, _: &TaskContext, engine: &EngineRef) -> Result<TaskResult, TaskError> {
    // Read preferences (thread-safe — Phase 23)
    let theme = engine.preferences.get("theme").as_deref().unwrap_or("light");

    // Write preferences (thread-safe)
    engine.preferences.set("last_task", "export");

    Ok(TaskResult::completed())
}
```

### Read and Write State

```rust
fn execute(&self, _: &TaskInput, _: &TaskContext, engine: &EngineRef) -> Result<TaskResult, TaskError> {
    // Read state (thread-safe)
    let current = engine.state_manager.current(StateId::new(1));

    // Write state (thread-safe)
    engine.state_manager.set(StateId::new(2), LifecycleState::Running);

    Ok(TaskResult::completed())
}
```

### Execute Sub-Commands

```rust
fn execute(&self, _: &TaskInput, _: &TaskContext, engine: &EngineRef) -> Result<TaskResult, TaskError> {
    let cmd = Command::new(
        "document.create",
        CommandContext::new(ExecutionId::new()),
        CommandInput::new("Sub-Document".to_string()),
    );

    let result = engine.command_executor.execute(&cmd, engine)
        .map_err(|e| TaskError::ExecutionFailed { id: 0, reason: e.to_string() })?;

    Ok(TaskResult::completed_with(result))
}
```

**Note:** `CommandExecutor::execute()` takes `&EngineRef` as its second parameter. Since `EngineRef` is in scope, the handler can pass it through.

### Submit Work to Scheduler

```rust
fn execute(&self, _: &TaskInput, _: &TaskContext, engine: &EngineRef) -> Result<TaskResult, TaskError> {
    // Submit a follow-up task to the scheduler
    let job_id = engine.scheduler.submit(
        TaskId::new(),
        Schedule::Delayed(Duration::from_secs(60)),
        5,  // priority
    );

    // Or submit with retry policy (Phase 27)
    let retry_job = engine.scheduler.submit_with_retry(
        TaskId::new(),
        Schedule::Immediate,
        0,
        RetryPolicy::exponential(3, Duration::from_secs(1), 2.0),
    );

    Ok(TaskResult::completed())
}
```

### Append to History

```rust
fn execute(&self, _: &TaskInput, _: &TaskContext, engine: &EngineRef) -> Result<TaskResult, TaskError> {
    // Append a history entry (thread-safe — Phase 22)
    engine.history_store.append(
        HistoryEntry::new("object.move")
            .undoable()
            .with_data("moved_data".to_string())
            .with_source("MyHandler"),
    );

    // Undo
    let undo_result = engine.history_store.undo();

    // Redo
    let redo_result = engine.history_store.redo();

    Ok(TaskResult::completed())
}
```

### Access History Entries (closure pattern)

```rust
fn execute(&self, _: &TaskInput, _: &TaskContext, engine: &EngineRef) -> Result<TaskResult, TaskError> {
    // Current entry (Phase 22 — Mutex-safe access)
    let current_op = engine.history_store.current();
    if let Some((_, op, undoable)) = current_op {
        println!("Current operation: {} (undoable: {})", op, undoable);
    }

    // Full entry access through closure
    let data = engine.history_store.with_current(|e| {
        e.data::<String>().map(|s| s.clone())
    }).flatten();

    // By ID
    let entry = engine.history_store.with_entry(some_id, |e| {
        (e.operation().to_string(), e.is_undoable())
    });

    Ok(TaskResult::completed())
}
```

---

## What Handlers CANNOT Do

### Cannot Subscribe to Events at Runtime

```rust
// This WON'T compile:
engine.event_bus.subscribe("task.completed", handler);
// Error: subscribe() requires &mut EventBus
// EventBus is &EventBus in EngineRef (read-only)
```

Event subscription requires `&mut EventBus`, which is only available through `engine.event_bus_mut()` (setup time, before `engine.start()`).

### Cannot Register Commands at Runtime

```rust
// This WON'T compile:
engine.command_executor.register(definition, handler);
// Error: register() requires &mut CommandExecutor
```

Command registration is a setup-time operation.

### Cannot Access TaskManager

```rust
// TaskManager is deliberately NOT in EngineRef.
// This is to allow disjoint borrows:
//   engine.task_manager_mut().start(task_id, &engine_ref)
// works because TaskManager borrows a different field than EngineRef.
```

If a handler needs to create a sub-task, it should use the `CommandExecutor` to execute a command that creates a task, or use the `Scheduler` to submit work.

### Cannot Access ServiceManager or PluginManager

These are managed by the runtime lifecycle, not by handlers.

### Cannot Access ContextManager

Contexts are created during setup, not during execution.

---

## Why TaskManager Is Excluded

The exclusion of `TaskManager` from `EngineRef` is a deliberate design choice for **disjoint borrows**:

```rust
// In AppRuntime::start_task():
pub fn start_task(&self, task_id: TaskId) -> Result<TaskResult, TaskError> {
    // Connect output channel to SignalBus
    let signal_bus_arc = self.signal_bus.clone();
    self.task_manager.connect_output(task_id, signal_bus_arc);

    // Construct EngineRef (borrows 9 fields, NOT task_manager)
    let engine_ref = EngineRef::new(
        &self.event_bus,
        self.signal_bus.as_ref(),
        &self.resource_manager,
        // ... 9 fields, but NOT task_manager
    );

    // Call start() — borrows task_manager mutably
    // This works because engine_ref borrows different fields!
    self.task_manager.start(task_id, &engine_ref)
}
```

If `TaskManager` were in `EngineRef`, then `self.task_manager.start()` would conflict with `&engine_ref` (which would borrow `task_manager` immutably). By excluding it, Rust's split borrow allows simultaneous mutable access to `task_manager` and immutable access to the other 9 fields.

---

## How EngineRef Is Constructed

`EngineRef` is constructed **internally** by the runtime:

```rust
// In AppRuntime:
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
    // EngineRef constructed inline — same 9 fields
    let engine_ref = EngineRef::new(/* same fields */);
    self.command_executor.execute(command, &engine_ref)
}

pub fn start_task(&self, task_id: TaskId) -> Result<TaskResult, TaskError> {
    // EngineRef constructed inline — same 9 fields
    let engine_ref = EngineRef::new(/* same fields */);
    self.task_manager.start(task_id, &engine_ref)
}
```

The caller never constructs `EngineRef` directly — the runtime handles it.

### EngineRef Fields Are Public

```rust
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
```

All fields are `pub` — handlers access them directly:

```rust
// Field access (not method calls):
engine.event_bus.publish(&event);       // field, then method
engine.config_store.set("key", "val"); // field, then method
engine.scheduler.submit(task_id, schedule, priority);  // field, then method
```

---

## Thread Safety Guarantees

All fields in `EngineRef` point to **thread-safe** managers:

| Field | Lock Type | Methods |
|---|---|---|
| `event_bus` | No lock | `publish(&self)` — safe for concurrent reads |
| `signal_bus` | `Mutex` | `emit(&self)`, `subscribe(&self)` — all thread-safe |
| `resource_manager` | `Mutex` + `RwLock` | `load(&self)`, `with_resource(&self)` — all thread-safe |
| `config_store` | `Mutex` | `get(&self)`, `set(&self)` — all thread-safe |
| `preferences` | `Mutex` | `get(&self)`, `set(&self)` — all thread-safe |
| `state_manager` | `Mutex` | `current(&self)`, `set(&self)` — all thread-safe |
| `command_executor` | No lock | `execute(&self)` — safe for concurrent reads |
| `scheduler` | `Mutex` + `RwLock` | `submit(&self)`, `tick(&self)`, `dispatch(&self)` — all thread-safe |
| `history_store` | `Mutex` | `append(&self)`, `undo(&self)`, `redo(&self)` — all thread-safe |

**Key rule:** `Mutex` is used when the contained type has `Box<dyn Any + Send>` (Send but not Sync). `RwLock` is used when the contained type is `Send + Sync`. No-lock is used when only `&self` methods exist (read-only).

---

## PermissionCheckedEngineRef (Phase 28)

For plugins that want to enforce permissions, wrap `EngineRef`:

```rust
use app_shell::app_engine::PermissionCheckedEngineRef;

// In a plugin's initialize():
fn initialize(&mut self, _ctx: &PluginContext, engine: &EngineRef) -> Result<(), PluginError> {
    let checked = PermissionCheckedEngineRef::new(
        engine,
        PluginPermissions::new(LOAD_RESOURCES | SUBSCRIBE_EVENTS),
        "my_plugin",
    );

    // Read-only access — always allowed
    let config = checked.config_store();
    let prefs = checked.preferences();

    // Permission-checked access
    let mgr = checked.try_load_resource()?;     // OK — has LOAD_RESOURCES
    let result = checked.try_write_config();     // Err — no WRITE_CONFIG
    assert!(matches!(result, Err(PluginSecurityError::PermissionDenied { .. })));

    Ok(())
}
```

### `try_*` Methods

| Method | Permission Required | Returns |
|---|---|---|
| `try_write_config()` | `WRITE_CONFIG` | `Result<&ConfigStore, PluginSecurityError>` |
| `try_load_resource()` | `LOAD_RESOURCES` | `Result<&ResourceManager, PluginSecurityError>` |
| `try_publish_event()` | `PUBLISH_EVENTS` | `Result<&EventBus, PluginSecurityError>` |
| `try_subscribe_events()` | `SUBSCRIBE_EVENTS` | `Result<&EventBus, PluginSecurityError>` |
| `try_register_commands()` | `REGISTER_COMMANDS` | `Result<&CommandExecutor, PluginSecurityError>` |
| `try_register_tasks()` | `REGISTER_TASKS` | `Result<&CommandExecutor, PluginSecurityError>` |
| `try_register_services()` | `REGISTER_SERVICES` | `Result<&CommandExecutor, PluginSecurityError>` |

Read-only access is **always allowed** regardless of permissions.

### Default Permissions

```rust
// Default = FULL_ACCESS — all try_* methods succeed
let checked = PermissionCheckedEngineRef::new(
    engine,
    PluginPermissions::default(),  // = FULL
    "full_access_plugin",
);

assert!(checked.try_load_resource().is_ok());
assert!(checked.try_write_config().is_ok());
assert!(checked.try_publish_event().is_ok());
```

---

## Complete Handler Example

```rust
use app_shell::app_engine::*;

struct ExportHandler;

impl TaskHandler for ExportHandler {
    fn execute(
        &self,
        input: &TaskInput,
        ctx: &TaskContext,
        engine: &EngineRef,
    ) -> Result<TaskResult, TaskError> {
        let input: &ExportInput = input.get::<ExportInput>()
            .ok_or_else(|| TaskError::ExecutionFailed {
                id: 0, reason: "expected ExportInput".into()
            })?;

        // Phase 14: Check cancellation
        if ctx.is_cancelled() {
            return Ok(TaskResult::cancelled());
        }

        // Phase 14: Check deadline
        if ctx.is_expired() {
            return Err(TaskError::ExecutionFailed {
                id: 0, reason: "timeout".into()
            });
        }

        // Phase 14: Report progress
        ctx.output().progress(0.0);

        // Phase 13: Read config
        let worker_count = engine.config_store.get_i64("worker_count").unwrap_or(4);

        // Phase 13: Load a resource (RAII handle auto-releases on drop)
        let handle = engine.resource_manager.load("document", &input.source)
            .map_err(|e| TaskError::ExecutionFailed {
                id: 0, reason: e.to_string()
            })?;

        // Phase 23: Access resource data
        let doc_data = engine.resource_manager.with_resource(handle.resource_id(), |r| {
            r.data::<String>().map(|s| s.clone())
        }).flatten().unwrap_or_default();

        // Phase 13: Publish event
        engine.event_bus.publish(
            &Event::new("document.exported")
                .with_payload(doc_data.clone())
                .with_source("ExportHandler")
        );

        // Phase 13: Emit progress signal
        engine.signal_bus.emit(
            &Signal::new("export.progress")
                .with_payload(0.5_f32)
        );

        // Phase 13: Append to history
        engine.history_store.append(
            HistoryEntry::new("document.export")
                .with_source("ExportHandler"),
        );

        // Phase 14: Report completion progress
        ctx.output().progress(1.0);

        Ok(TaskResult::completed_with(doc_data))
    }
}
```

---

## EngineRef in TestEngine

For testing, `TestEngine` provides the same `EngineRef`:

```rust
use app_shell::app_engine::TestEngine;

let engine = TestEngine::new();
let engine_ref = engine.engine_ref();

// All the same capabilities
let _ = engine_ref.config_store;
let _ = engine_ref.signal_bus;
let _ = engine_ref.resource_manager;
```

`TestEngine` provides real (empty) subsystems so handlers can access `EngineRef` without a full `AppEngine`.

---

## Quick Reference

### What's in EngineRef

```rust
// Field access (not method calls):
engine.event_bus         // &EventBus — publish()
engine.signal_bus        // &SignalBus — emit(), subscribe()
engine.resource_manager  // &ResourceManager — load(), with_resource()
engine.config_store      // &ConfigStore — get(), set()
engine.preferences      // &Preferences — get(), set()
engine.state_manager     // &StateManager — current(), set()
engine.command_executor  // &CommandExecutor — execute(), list()
engine.scheduler         // &Scheduler — submit(), tick(), dispatch()
engine.history_store     // &HistoryStore — append(), undo(), redo()
```

### What's NOT in EngineRef

```text
✗ TaskManager       (excluded for disjoint borrows)
✗ ServiceManager    (managed by runtime)
✗ PluginManager    (managed by runtime)
✗ ContextManager   (setup-time only)
✗ TransactionManager (setup-time only)
```

### PermissionCheckedEngineRef

```rust
let checked = PermissionCheckedEngineRef::new(engine, permissions, "plugin_id");

// Read-only (always allowed)
checked.config_store();
checked.preferences();
checked.state_manager();

// Permission-checked
checked.try_write_config()?;
checked.try_load_resource()?;
checked.try_publish_event()?;
checked.try_subscribe_events()?;
checked.try_register_commands()?;
```

---

## Next Steps

- [12-execution-control.md](12-execution-control.md) — Cancellation, deadlines, and progress streaming.
- [05-commands-and-tasks.md](05-commands-and-tasks.md) — Deep dive on commands and tasks.
- [17-plugin-security.md](17-plugin-security.md) — Permissions and enforcement in depth.
- [16-testing-and-metadata.md](16-testing-and-metadata.md) — Mocks, TestEngine, and testing patterns.