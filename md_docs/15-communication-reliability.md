# Communication & Reliability

## Overview

Three mechanisms make the App Engine's communication robust:

```text
Event Priority     — High before Normal before Low
Panic Isolation    — One crashing handler doesn't kill the engine
Retry Policies     — Failed tasks can be retried with configurable backoff
```

---

## Part 1: Event Priority (Phase 19)

### Priority Levels

```rust
use app_shell::app_engine::EventPriority;

// Three levels
EventPriority::High      // Delivered first (e.g., history recording)
EventPriority::Normal    // Default
EventPriority::Low       // Delivered last (e.g., analytics, logging)

// Ordering
assert!(EventPriority::High > EventPriority::Normal);
assert!(EventPriority::Normal > EventPriority::Low);

// Default
assert_eq!(EventPriority::default(), EventPriority::Normal);
```

### Subscribing with Priority

```rust
// Default (Normal)
let id1 = engine.event_bus_mut().subscribe("task.completed", Box::new(NormalHandler))?;

// With explicit priority
let id2 = engine.event_bus_mut().subscribe_with_priority(
    "task.completed",
    Box::new(HighHandler),
    EventPriority::High,
)?;

let id3 = engine.event_bus_mut().subscribe_with_priority(
    "task.completed",
    Box::new(LowHandler),
    EventPriority::Low,
)?;

// When published, order is: High → Normal → Low
engine.event_bus().publish(&Event::new("task.completed"));
```

### Priority Delivery Example

```rust
use std::sync::{Arc, Mutex};

struct CountingHandler {
    order: Arc<Mutex<Vec<usize>>>,
    tag: usize,
}

impl EventHandler for CountingHandler {
    fn handle(&self, _: &Event) {
        self.order.lock().unwrap().push(self.tag);
    }
}

let order = Arc::new(Mutex::new(vec![]));

engine.event_bus_mut().subscribe_with_priority(
    "test.priority",
    Box::new(CountingHandler { order: order.clone(), tag: 1 }),  // Normal
    EventPriority::Normal,
)?;
engine.event_bus_mut().subscribe_with_priority(
    "test.priority",
    Box::new(CountingHandler { order: order.clone(), tag: 0 }),  // High
    EventPriority::High,
)?;
engine.event_bus_mut().subscribe_with_priority(
    "test.priority",
    Box::new(CountingHandler { order: order.clone(), tag: 2 }),  // Low
    EventPriority::Low,
)?;

engine.event_bus().publish(&Event::new("test.priority"));

let order = order.lock().unwrap();
assert_eq!(*order, vec![0, 1, 2]);  // High(0), Normal(1), Low(2)
```

### When to Use Priority

| Priority | Use Case |
|---|---|
| `High` | History recording, critical state updates, safety checks |
| `Normal` | Default — UI refresh, domain reactions |
| `Low` | Analytics, logging, non-critical monitoring |

---

## Part 2: Panic Isolation (Phase 19 + 24)

### What Is Isolated

All handler and lifecycle calls are wrapped in `std::panic::catch_unwind`:

| Handler Type | Isolated? | Where |
|---|---|---|
| Event handlers | ✅ | `EventDispatcher::dispatch()` |
| Task handlers | ✅ | `TaskExecutor::execute()` |
| Command handlers | ✅ | `CommandExecutor::execute()` |
| Service lifecycle | ✅ | `ServiceManager::initialize/start/stop/dispose()` |
| Plugin lifecycle | ✅ | `PluginManager::initialize/start/stop/dispose()` |

### Event Handler Panic

```rust
struct PanickingHandler;
impl EventHandler for PanickingHandler {
    fn handle(&self, _: &Event) {
        panic!("handler crashed!");
    }
}

struct SurvivingHandler {
    received: Arc<Mutex<bool>>,
}
impl EventHandler for SurvivingHandler {
    fn handle(&self, _: &Event) {
        *self.received.lock().unwrap() = true;
    }
}

let received = Arc::new(Mutex::new(false));

engine.event_bus_mut().subscribe("test.isolation", Box::new(PanickingHandler));
engine.event_bus_mut().subscribe("test.isolation", Box::new(SurvivingHandler { received: received.clone() }));

// This does NOT crash the engine
engine.event_bus().publish(&Event::new("test.isolation"));

// The surviving handler was still called
assert!(*received.lock().unwrap());
```

### Task Handler Panic

```rust
struct PanickingTaskHandler;
impl TaskHandler for PanickingTaskHandler {
    fn execute(&self, _: &TaskInput, _: &TaskContext, _: &EngineRef) -> Result<TaskResult, TaskError> {
        panic!("task handler crashed!");
    }
}

engine.task_manager_mut().register(
    TaskDefinition::new("panic.task", "Panic", "Panics"),
    Box::new(PanickingTaskHandler),
)?;

let task_id = engine.task_manager_mut().create(
    "panic.task",
    TaskContext::new(ExecutionId::new()),
    TaskInput::empty(),
)?;

let result = engine.start_task(task_id);

// Panic is converted to an error
assert!(result.is_err());
assert!(result.unwrap_err().to_string().contains("handler panicked: task handler crashed!"));

// Task is in Failed state
assert_eq!(engine.task_manager().get_state(task_id), Some(TaskState::Failed));
```

### Command Handler Panic

```rust
struct PanickingCommandHandler;
impl CommandHandler for PanickingCommandHandler {
    fn execute(&self, _: &CommandInput, _: &CommandContext, _: &EngineRef) -> Result<CommandResult, CommandError> {
        panic!("command handler crashed!");
    }
}

engine.command_executor_mut().register(
    CommandDefinition::new("panic.cmd", "Panic", "Panics"),
    Box::new(PanickingCommandHandler),
)?;

let cmd = Command::with_empty_input("panic.cmd", CommandContext::new(ExecutionId::new()));
let result = engine.execute_command(&cmd);

assert!(result.is_err());
assert!(result.unwrap_err().to_string().contains("handler panicked: command handler crashed!"));
```

### Service Lifecycle Panic

```rust
struct PanickingService;
impl Service for PanickingService {
    fn service_type(&self) -> &str { "panic_svc" }
    fn initialize(&mut self, _: &EngineRef) -> Result<(), ServiceError> {
        panic!("service init crashed!");
    }
    fn start(&mut self, _: &EngineRef) -> Result<(), ServiceError> { Ok(()) }
    fn stop(&mut self) -> Result<(), ServiceError> { Ok(()) }
    fn dispose(&mut self) -> Result<(), ServiceError> { Ok(()) }
}

let id = engine.service_manager_mut().register(
    ServiceDefinition::new("panic_svc", "Panic", "Panics"),
    Box::new(PanickingService),
)?;

let result = engine.service_manager_mut().initialize(id, &engine_ref);
assert!(result.is_err());
assert!(result.unwrap_err().to_string().contains("service panicked: service init crashed!"));
```

### Plugin Lifecycle Panic

```rust
struct PanickingPlugin;
impl Plugin for PanickingPlugin {
    fn id(&self) -> &str { "panic_plugin" }
    fn initialize(&mut self, _: &PluginContext, _: &EngineRef) -> Result<(), PluginError> {
        panic!("plugin init crashed!");
    }
    fn start(&mut self, _: &EngineRef) -> Result<(), PluginError> { Ok(()) }
    fn stop(&mut self) -> Result<(), PluginError> { Ok(()) }
    fn dispose(&mut self) -> Result<(), PluginError> { Ok(()) }
}

let id = engine.plugin_manager_mut().register(
    PluginManifest::new("panic_plugin", "Panic Plugin"),
    Box::new(PanickingPlugin),
)?;

let result = engine.plugin_manager_mut().initialize(id, &engine_ref);
assert!(result.is_err());
assert!(result.unwrap_err().to_string().contains("plugin panicked: plugin init crashed!"));
```

### Mutex Not Poisoned

The critical guarantee: **after a panic, the Mutex is NOT poisoned.**

```rust
// After a panicking task handler:
let result = engine.start_task(task_id);  // Panics, caught
assert!(result.is_err());

// TaskManager still usable — Mutex NOT poisoned
assert_eq!(engine.task_manager().count(), 1);

// Can create more tasks
let task_id2 = engine.task_manager_mut().create(
    "panic.task",
    TaskContext::new(ExecutionId::new()),
    TaskInput::empty(),
)?;
assert_eq!(engine.task_manager().count(), 2);
```

**How it works:** The `catch_unwind` catches the panic **before** the `MutexGuard` is dropped. The guard is released normally (no panic), so the Mutex is not poisoned.

---

## Part 3: Retry Policies (Phase 27)

### RetryPolicy

```rust
use app_shell::app_engine::RetryPolicy;
use std::time::Duration;

// No retry
let none = RetryPolicy::none();
assert!(!none.should_retry());
assert_eq!(none.max_attempts(), 1);  // 1 total (initial only)
assert_eq!(none.delay_for_attempt(0), None);

// Fixed delay between attempts
let fixed = RetryPolicy::fixed(3, Duration::from_secs(5));
assert!(fixed.should_retry());
assert_eq!(fixed.max_attempts(), 4);  // 3 retries + 1 initial
assert_eq!(fixed.delay_for_attempt(0), Some(Duration::from_secs(5)));
assert_eq!(fixed.delay_for_attempt(2), Some(Duration::from_secs(5)));
assert_eq!(fixed.delay_for_attempt(3), None);  // Exceeded

// Exponential backoff
let exp = RetryPolicy::exponential(3, Duration::from_millis(100), 2.0);
assert!(exp.should_retry());
assert_eq!(exp.max_attempts(), 4);

assert_eq!(exp.delay_for_attempt(0), Some(Duration::from_millis(100)));   // 100ms
assert_eq!(exp.delay_for_attempt(1), Some(Duration::from_millis(200)));   // 200ms
assert_eq!(exp.delay_for_attempt(2), Some(Duration::from_millis(400)));   // 400ms
assert_eq!(exp.delay_for_attempt(3), None);  // Exceeded
```

### Submitting with Retry

```rust
let job_id = engine.scheduler_mut().submit_with_retry(
    task_id,
    Schedule::Immediate,
    0,  // priority
    RetryPolicy::exponential(3, Duration::from_secs(1), 2.0),
);

// The job has the retry policy attached
let job = engine.scheduler().get_job(job_id).unwrap();
assert!(job.retry_policy().should_retry());
assert_eq!(job.retry_count(), 0);
```

### Handling Task Failure

When a dispatched task fails, call `handle_task_failure()` on the scheduler:

```rust
// Dispatch and execute
let job = engine.scheduler_mut().dispatch().unwrap();
let result = engine.start_task(job.task_id());

if result.is_err() {
    // Ask scheduler to retry based on the job's retry policy
    if let Some(retry_job_id) = engine.scheduler_mut().handle_task_failure(&job) {
        println!("Task scheduled for retry as job {}", retry_job_id.value());
        // Retry job is delayed (Schedule::Delayed with backoff)
        assert_eq!(engine.scheduler().scheduled_count(), 1);
        assert_ne!(retry_job_id, job.id());  // New JobId
    }
}
```

### Key Rules

- Retry creates a **NEW job** (new `JobId`), not a re-run of the same job
- The retry job is `Schedule::Delayed` with the computed backoff delay
- After `max_attempts`, `handle_task_failure()` returns `None`
- The retry job has `RetryPolicy::None` (created via `submit()`, not `submit_with_retry()`)
- Priority is preserved from the original job

### Full Retry Flow

```rust
// 1. Submit with retry
let original_id = engine.scheduler_mut().submit_with_retry(
    task_id,
    Schedule::Immediate,
    7,  // priority 7
    RetryPolicy::fixed(3, Duration::from_millis(1)),
);

// 2. Dispatch
let job = engine.scheduler_mut().dispatch().unwrap();
assert_eq!(job.id(), original_id);
assert_eq!(job.priority(), 7);

// 3. Execute — fails
let result = engine.start_task(job.task_id());
assert!(result.is_err());

// 4. Request retry
let retry_id = engine.scheduler_mut().handle_task_failure(&job).unwrap();
assert_ne!(original_id, retry_id);  // New job

// 5. Wait for delay and tick
std::thread::sleep(Duration::from_millis(5));
engine.scheduler_mut().tick();
assert_eq!(engine.scheduler().queue_len(), 1);

// 6. Dispatch retry
let retry_job = engine.scheduler_mut().dispatch().unwrap();
assert_eq!(retry_job.id(), retry_id);
assert_eq!(retry_job.priority(), 7);  // Priority preserved
```

### Retry with No Policy

```rust
let job_id = engine.scheduler_mut().submit_with_retry(
    task_id,
    Schedule::Immediate,
    0,
    RetryPolicy::None,
);

let job = engine.scheduler_mut().dispatch().unwrap();
let result = engine.start_task(job.task_id());

if result.is_err() {
    let retry = engine.scheduler_mut().handle_task_failure(&job);
    assert!(retry.is_none());  // No retry with None policy
}
```

### Config Change Notification (Phase 19)

ConfigStore fires callbacks when values change — useful for reactive configuration:

```rust
use std::sync::{Arc, Mutex};

let store = ConfigStore::new(ConfigSchema::new()
    .with_property(ConfigPropertyDefinition::new("key", "Key").with_default("default")));

let received = Arc::new(Mutex::new(None::<String>));
let r = received.clone();
store.on_changed(move |key, old, new| {
    *r.lock().unwrap() = new.map(|s| s.to_string());
});

store.set("key", "new_value").unwrap();

assert_eq!(received.lock().unwrap().as_deref(), Some("new_value"));
```

---

## Integration: Priority + Isolation + Retry

All three mechanisms work together:

```rust
// Register a high-priority history handler (panic-isolated)
engine.event_bus_mut().subscribe_with_priority(
    "task.completed",
    event_handler(|event: &Event| {
        // History records the operation
        // This handler runs FIRST (High priority)
        // If it panics, other handlers still fire
    }),
    EventPriority::High,
)?;

// Register a normal-priority UI handler (panic-isolated)
engine.event_bus_mut().subscribe_with_priority(
    "task.completed",
    event_handler(|_event: &Event| {
        // UI refreshes the task list
        // This runs after history
        // If it panics, analytics still fires
    }),
    EventPriority::Normal,
)?;

// Register a low-priority analytics handler (panic-isolated)
engine.event_bus_mut().subscribe_with_priority(
    "task.completed",
    event_handler(|_event: &Event| {
        // Analytics records the completion
        // This runs last
    }),
    EventPriority::Low,
)?;

// Submit a task with retry policy
let job_id = engine.scheduler_mut().submit_with_retry(
    task_id,
    Schedule::Immediate,
    0,
    RetryPolicy::exponential(3, Duration::from_secs(1), 2.0),
);

// Main loop with retry
while !engine.stop_requested() {
    engine.scheduler_mut().tick();

    while let Some(job) = engine.scheduler_mut().dispatch() {
        match engine.start_task(job.task_id()) {
            Ok(result) => {
                if result.is_completed() {
                    engine.event_bus().publish(
                        &Event::new("task.completed").with_source("MainLoop")
                    );
                }
            }
            Err(e) => {
                eprintln!("Task failed: {}", e);
                // Retry with backoff
                if let Some(retry_id) = engine.scheduler_mut().handle_task_failure(&job) {
                    eprintln!("Scheduled for retry as job {}", retry_id.value());
                }
            }
        }
    }

    std::thread::sleep(Duration::from_millis(10));
}
```

---

## Quick Reference

### Event Priority

```rust
// Subscribe
engine.event_bus_mut().subscribe_with_priority(event_type, handler, EventPriority::High)?;

// Priority ordering
EventPriority::High > EventPriority::Normal > EventPriority::Low
EventPriority::default() == EventPriority::Normal
```

### Panic Isolation

```rust
// All handlers are automatically isolated:
// - Event handlers: catch_unwind in EventDispatcher
// - Task handlers: catch_unwind in TaskExecutor
// - Command handlers: catch_unwind in CommandExecutor
// - Service lifecycle: catch_unwind in ServiceManager
// - Plugin lifecycle: catch_unwind in PluginManager

// Error messages contain "handler panicked: ..." or "service panicked: ..." etc.
// Mutex is NOT poisoned after a panic
```

### RetryPolicy

```rust
// Create
RetryPolicy::none()
RetryPolicy::fixed(max_attempts, delay)
RetryPolicy::exponential(max_attempts, base_delay, factor)

// Query
policy.should_retry()                    // bool
policy.max_attempts()                    // u32 (including initial)
policy.delay_for_attempt(attempt)        // Option<Duration>

// Use with scheduler
let job_id = scheduler.submit_with_retry(task_id, schedule, priority, retry_policy)?;

// Handle failure
let retry_id = scheduler.handle_task_failure(&job)?;  // Option<JobId>
```

### Config Change Notification

```rust
store.on_changed(|key, old_value: Option<&str>, new_value: Option<&str>| {
    // Called when set(), remove(), or save() changes values
});
```

---

## Next Steps

- [06-scheduling-and-communication.md](06-scheduling-and-communication.md) — Scheduling, events, and signals overview.
- [12-execution-control.md](12-execution-control.md) — Cancellation, deadlines, and progress streaming.
- [14-concurrency-and-async.md](14-concurrency-and-async.md) — Thread safety and async execution.
- [08-services-and-plugins.md](08-services-and-plugins.md) — Services and plugins (panic-isolated lifecycle).