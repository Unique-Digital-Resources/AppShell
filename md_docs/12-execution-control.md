# `12-execution-control.md` — Execution Control

## Overview

Phase 14 added three mechanisms that make long-running tasks controllable:

```text
CancellationToken  — "Has the user/system requested cancellation?"
Deadline           — "Has the time limit expired?"
TaskOutputChannel  — "Report progress and stream intermediate results"
```

These live on `TaskContext` and are automatically injected by `TaskManager::create()`:

```rust
impl TaskHandler for MyHandler {
    fn execute(&self, input: &TaskInput, context: &TaskContext, engine: &EngineRef) -> Result<TaskResult, TaskError> {
        // context.is_cancelled()  ← CancellationToken
        // context.is_expired()     ← Deadline
        // context.output()        ← TaskOutputChannel
    }
}
```

---

## CancellationToken

### What It Is

An atomic boolean flag shared between the `TaskManager` (which sets it) and the handler (which checks it). When `cancel()` is called on a task, the token is set, and the running handler can detect it.

```rust
use app_shell::app_engine::CancellationToken;

let token = CancellationToken::new();
assert!(!token.is_cancelled());

token.cancel();
assert!(token.is_cancelled());
```

### How It Works

When a task is created, `TaskManager` creates a `CancellationToken` and injects it into the `TaskContext`:

```rust
// In TaskManager::create():
let token = CancellationToken::new();
context.set_cancellation_token(token.clone());
self.cancellation_tokens.insert(task_id, token);
```

When `cancel()` is called:

```rust
// In TaskManager::cancel():
if let Some(token) = self.cancellation_tokens.get(&id) {
    token.cancel();  // Sets atomic flag
}
task.set_state(TaskState::Cancelled);
```

The handler checks the flag periodically:

```rust
impl TaskHandler for RenderHandler {
    fn execute(&self, _: &TaskInput, ctx: &TaskContext, _: &EngineRef) -> Result<TaskResult, TaskError> {
        for i in 0..1000 {
            if ctx.is_cancelled() {
                return Ok(TaskResult::cancelled());
            }
            render_frame(i);
        }
        Ok(TaskResult::completed())
    }
}
```

### Cloning and Sharing

`CancellationToken` is `Clone` — the cloned copy shares the same atomic flag:

```rust
let token = CancellationToken::new();
let clone = token.clone();

token.cancel();
assert!(clone.is_cancelled());  // Shared state
```

### Reset

```rust
let token = CancellationToken::new();
token.cancel();
assert!(token.is_cancelled());
token.reset();
assert!(!token.is_cancelled());
```

### Cancel Before Start

```rust
let definition = TaskDefinition::new("export", "Export", "Export")
    .with_cancel_support();  // ← required

let task_id = engine.task_manager_mut().create(
    "export",
    TaskContext::new(ExecutionId::new()),
    TaskInput::new("data".to_string()),
)?;

// Cancel while still Pending
engine.task_manager_mut().cancel(task_id)?;
assert_eq!(
    engine.task_manager().get_state(task_id),
    Some(TaskState::Cancelled),
);
```

### Cancel Non-Cancellable Task

```rust
let definition = TaskDefinition::new("rigid", "Rigid", "Not cancellable");
// .with_cancel_support() NOT called

let task_id = engine.task_manager_mut().create(
    "rigid",
    TaskContext::new(ExecutionId::new()),
    TaskInput::empty(),
)?;

let result = engine.task_manager_mut().cancel(task_id);
assert!(matches!(result, Err(TaskError::NotCancellable { .. })));
```

### Capability Check

```rust
let task = engine.task_manager().get(task_id);  // Not available through Mutex — use state
// Instead check via definition or state
let state = engine.task_manager().get_state(task_id);
```

---

## Deadline

### What It Is

A point in time after which an operation should stop. If `None`, there is no deadline.

```rust
use app_shell::app_engine::{Deadline, Duration};

// No deadline — runs forever
let none = Deadline::none();
assert!(!none.is_expired());
assert!(none.remaining().is_none());

// Deadline after a duration from now
let deadline = Deadline::after(Duration::from_secs(30));
assert!(!deadline.is_expired());
assert!(deadline.remaining().is_some());

// Deadline at a specific instant
let target = std::time::Instant::now() + Duration::from_secs(10);
let at = Deadline::at(target);
```

### How It Works

The handler checks `context.is_expired()` during long loops:

```rust
impl TaskHandler for ExportHandler {
    fn execute(&self, _: &TaskInput, ctx: &TaskContext, _: &EngineRef) -> Result<TaskResult, TaskError> {
        if ctx.is_expired() {
            return Err(TaskError::ExecutionFailed {
                id: 0, reason: "timeout".to_string()
            });
        }
        Ok(TaskResult::completed())
    }
}
```

### Setting a Deadline

```rust
let task_id = engine.task_manager_mut().create(
    "project.export",
    TaskContext::new(ExecutionId::new())
        .with_deadline(Deadline::after(Duration::from_secs(30))),  // ← deadline
    TaskInput::new(export_data),
)?;

// In the handler:
if ctx.is_expired() {
    return Err(TaskError::ExecutionFailed {
        id: 0, reason: "timeout".to_string()
    });
}
```

### Checking Remaining Time

```rust
if let Some(remaining) = ctx.deadline().remaining() {
    println!("{}ms remaining", remaining.as_millis());
}
```

### Deadline on CommandContext

`CommandContext` also supports deadlines:

```rust
let cmd_context = CommandContext::new(ExecutionId::new())
    .with_deadline(Deadline::after(Duration::from_secs(10)));

let cmd = Command::new("app.quit", cmd_context, CommandInput::empty());
let result = engine.execute_command(&cmd)?;
```

### Deadline Expiration

```rust
let deadline = Deadline::after(Duration::from_millis(1));
std::thread::sleep(Duration::from_millis(5));
assert!(deadline.is_expired());
```

---

## TaskOutputChannel

### What It Is

A channel for handlers to report progress and stream intermediate output. When connected to the `SignalBus`, progress reports emit `task.progress_changed` signals and intermediate output emits `task.output` signals.

### Creating a Channel

The `TaskManager` creates and injects the channel automatically:

```rust
// In TaskManager::create():
let channel = TaskOutputChannel::new(task_id.value());
context.set_output_channel(channel);
```

When `start_task()` is called, the runtime connects the channel to the `SignalBus`:

```rust
// In AppRuntime::start_task():
let signal_bus_arc = self.signal_bus.clone();
self.task_manager.connect_output(task_id, signal_bus_arc);
```

### Reporting Progress

```rust
impl TaskHandler for RenderHandler {
    fn execute(&self, _: &TaskInput, ctx: &TaskContext, _: &EngineRef) -> Result<TaskResult, TaskError> {
        for i in 0..100 {
            // Report progress (0.0 to 1.0)
            ctx.output().progress(i as f32 / 100.0);
            render_frame(i);
        }
        ctx.output().progress(1.0);  // 100%
        Ok(TaskResult::completed())
    }
}
```

Progress is clamped to `[0.0, 1.0]`:

```rust
ctx.output().progress(-1.0);  // clamped to 0.0
ctx.output().progress(2.0);   // clamped to 1.0
```

### Checking Current Progress

```rust
let progress = ctx.output().current_progress();
assert_eq!(progress, 0.5);
```

### Streaming Intermediate Output

```rust
impl TaskHandler for StreamingHandler {
    fn execute(&self, _: &TaskInput, ctx: &TaskContext, _: &EngineRef) -> Result<TaskResult, TaskError> {
        for i in 0..3 {
            ctx.output().push(format!("Frame {} rendered", i));
        }
        Ok(TaskResult::completed())
    }
}
```

### Receiving Progress via SignalBus

When the channel is connected to the `SignalBus`, progress reports emit signals:

```rust
// Subscribe to progress signals
engine.signal_bus().subscribe(
    "task.progress_changed",
    Box::new(|signal: &Signal| {
        let progress = signal.payload::<f32>().unwrap_or(0.0);
        println!("Progress: {:.0}%", progress * 100.0);
    }),
);

// Subscribe to intermediate output
engine.signal_bus().subscribe(
    "task.output",
    Box::new(|signal: &Signal| {
        if let Some(s) = signal.payload::<String>() {
            println!("Output: {}", s);
        }
    }),
);

// Execute task — signals are emitted automatically
let result = engine.start_task(task_id)?;
```

### Channel Without SignalBus

When the channel is not connected to a `SignalBus`, calls to `progress()` and `push()` are silent no-ops (no crash):

```rust
// Create a standalone channel (no SignalBus)
let channel = TaskOutputChannel::new(42);
channel.progress(0.5);  // No crash, no signal emitted
channel.push("data".to_string());  // No crash, no signal emitted
assert_eq!(channel.current_progress(), 0.5);
```

---

## Complete Example

```rust
use app_shell::app_engine::*;
use std::time::Duration;

struct RenderHandler {
    frames: usize,
}

impl TaskHandler for RenderHandler {
    fn execute(
        &self,
        _input: &TaskInput,
        ctx: &TaskContext,
        engine: &EngineRef,
    ) -> Result<TaskResult, TaskError> {
        for i in 0..self.frames {
            // Phase 14: Check cancellation
            if ctx.is_cancelled() {
                engine.signal_bus.emit(
                    &Signal::new("render.cancelled")
                );
                return Ok(TaskResult::cancelled());
            }

            // Phase 14: Check deadline
            if ctx.is_expired() {
                return Err(TaskError::ExecutionFailed {
                    id: 0,
                    reason: "render timeout".to_string(),
                });
            }

            // Phase 14: Report progress (emits task.progress_changed signal)
            ctx.output().progress(i as f32 / self.frames as f32);

            // Phase 14: Stream intermediate output (emits task.output signal)
            ctx.output().push(format!("Frame {} rendered", i));

            // Phase 13: Publish progress event (separate from signals)
            engine.event_bus.publish(
                &Event::new("render.frame_completed")
                    .with_payload(i)
                    .with_source("RenderHandler")
            );

            // Simulate work
            std::thread::sleep(Duration::from_millis(1));
        }

        // Final progress
        ctx.output().progress(1.0);

        // Publish completion event
        engine.event_bus.publish(
            &Event::new("task.completed")
                .with_source("RenderHandler")
        );

        Ok(TaskResult::completed())
    }
}

// Setup
fn main() {
    let mut engine = Bootstrap::create().unwrap();
    engine.initialize().unwrap();
    engine.start().unwrap();

    // Subscribe to signals (progress + output)
    let progress_values = Arc::new(Mutex::new(vec![]));
    let pv = progress_values.clone();
    engine.signal_bus().subscribe(
        "task.progress_changed",
        Box::new(move |signal: &Signal| {
            if let Some(p) = signal.payload::<f32>() {
                pv.lock().unwrap().push(*p);
            }
        }),
    );

    let output_messages = Arc::new(Mutex::new(vec![]));
    let om = output_messages.clone();
    engine.signal_bus().subscribe(
        "task.output",
        Box::new(move |signal: &Signal| {
            if let Some(s) = signal.payload::<String>() {
                om.lock().unwrap().push(s.clone());
            }
        }),
    );

    // Register and create task
    engine.task_manager_mut().register(
        TaskDefinition::new("render", "Render", "Render frames")
            .with_cancel_support(),
        Box::new(RenderHandler { frames: 100 }),
    ).unwrap();

    let task_id = engine.task_manager_mut().create(
        "render",
        TaskContext::new(ExecutionId::new())
            .with_deadline(Deadline::after(Duration::from_secs(10))),
        TaskInput::empty(),
    ).unwrap();

    // Execute — progress and output signals are emitted during execution
    let result = engine.start_task(task_id).unwrap();
    assert!(result.is_completed());

    // Verify progress was tracked
    let progress = progress_values.lock().unwrap();
    assert!(!progress.is_empty());
    assert_eq!(*progress.last().unwrap(), 1.0);  // final progress

    // Verify intermediate output was streamed
    let messages = output_messages.lock().unwrap();
    assert_eq!(messages.len(), 100);  // one per frame
    assert_eq!(messages[0], "Frame 0 rendered");
    assert_eq!(messages[99], "Frame 99 rendered");

    engine.stop().unwrap();
    engine.dispose().unwrap();
}
```

---

## Integration with TaskManager

### Cancellation Propagation (Phase 14 + 26)

When `TaskManager::cancel()` is called:
1. The `CancellationToken` is set (atomic flag)
2. The handler checks `ctx.is_cancelled()` and returns `TaskResult::cancelled()`
3. The task state transitions to `Cancelled`

When `engine.stop()` is called (Phase 26):
1. `TaskManager::cancel_all()` is called
2. All running tasks' tokens are cancelled
3. All running tasks' states are set to `Cancelled`
4. The handler (if running in a loop) will detect cancellation on the next check

### Output Channel Connection (Phase 14)

When `AppRuntime::start_task()` is called:
1. `TaskOutputChannel` is created with a `Weak<SignalBus>` callback
2. `progress()` and `push()` emit signals on the `SignalBus`
3. Subscribers (UI, monitors, etc.) receive the signals in real-time

---

## Quick Reference

### CancellationToken API

```rust
CancellationToken::new()           // Create
token.is_cancelled()               // Check (bool)
token.cancel()                     // Set flag
token.reset()                      // Clear flag
// On TaskContext:
ctx.is_cancelled()                 // Check from handler
```

### Deadline API

```rust
Deadline::none()                  // No deadline
Deadline::after(duration)         // Relative
Deadline::at(instant)             // Absolute
deadline.is_expired()             // Check (bool)
deadline.remaining()             // Option<Duration>
// On TaskContext:
ctx.is_expired()                  // Check from handler
ctx.deadline()                    // &Deadline
```

### TaskOutputChannel API

```rust
// On TaskContext:
ctx.output().progress(0.5)       // Report progress (0.0-1.0, clamped)
ctx.output().push(data)           // Stream intermediate output
ctx.output().current_progress()    // Get current progress value
```

### CommandContext Deadline

```rust
// On CommandContext:
ctx.with_deadline(Deadline::after(duration))   // Builder
ctx.is_expired()                                 // Check
ctx.deadline()                                   // &Deadline
```

---

## Next Steps

- [05-commands-and-tasks.md](05-commands-and-tasks.md) — Deep dive on commands and tasks.
- [11-handler-capabilities.md](11-handler-capabilities.md) — What handlers can access via `EngineRef`.
- [06-scheduling-and-communication.md](06-scheduling-and-communication.md) — Scheduling, events, and signals.
- [15-communication-reliability.md](15-communication-reliability.md) — Priority, panic isolation, and retry in depth.