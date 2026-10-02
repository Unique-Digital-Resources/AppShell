# `06-scheduling-and-communication.md` — Scheduling, Events & Signals

## Overview

Two concerns are covered in this chapter:

1. **Scheduling** — *when* does work run?
2. **Communication** — *how do components talk without direct dependencies?*

```text
SCHEDULING                          COMMUNICATION
                                    
Task                                 Command = request
  ↓                                   
Job                                 Event = something happened
  ↓                                    
Scheduler                           Signal = something changed
  ↓                                    
Queue                                      │
  ↓                                        ├──→ History
Executor                                     ├──→ Plugin
  │                                          ├──→ UI
  ├── Event                                  └──→ Domain
  └── Signal
```

Together, these systems let the App Engine coordinate activity over time and between components without coupling them.

---

## Part 1: Scheduling

### The Scheduling Pipeline

```text
Task (what work exists)
   ↓
Scheduler (when should it run?)
   ↓
Job (one scheduled occurrence)
   ↓
Queue (ordered, by priority)
   ↓
Executor (how is it executed?)
```

The key separation:

```text
Task       = What work exists?
Scheduler  = When should it happen?
Executor   = How is it executed?
```

The scheduler **never executes tasks**. It decides when a job becomes eligible and hands it off.

---

### Submitting Work to the Scheduler

```rust
use app_shell::app_engine::{Schedule, Scheduler, TaskId};
use std::time::Duration;

// Assuming a task has been created:
let task_id = engine.task_manager_mut().create(
    "project.backup",
    TaskContext::new(ExecutionId::new()),
    TaskInput::new("backup_data".to_string()),
)?;

// Submit to scheduler with Immediate schedule
let job_id = engine.scheduler_mut().submit(
    task_id,
    Schedule::Immediate,
    0,  // priority (higher = sooner)
);

// Immediate jobs go straight to the queue
assert_eq!(engine.scheduler().queue_len(), 1);

// Dispatch + execute
let job = engine.scheduler_mut().dispatch().unwrap();
let result = engine.start_task(job.task_id())?;
assert!(result.is_completed());
```

### Immediate Execution

```rust
let job_id = engine.scheduler_mut().submit(
    task_id,
    Schedule::Immediate,
    0,
);

// Immediate jobs go straight to the queue
assert_eq!(engine.scheduler().queue_len(), 1);

// Dispatch + execute
let job = engine.scheduler_mut().dispatch().unwrap();
let result = engine.start_task(job.task_id())?;
assert!(result.is_completed());
```

### Delayed Execution

```rust
let job_id = engine.scheduler_mut().submit(
    task_id,
    Schedule::Delayed(Duration::from_secs(30)),
    0,
);

// Not in queue yet — waiting in scheduled
assert_eq!(engine.scheduler().queue_len(), 0);
assert_eq!(engine.scheduler().scheduled_count(), 1);

// After 30 seconds (in a real app, tick() is called periodically):
std::thread::sleep(Duration::from_millis(35));
engine.scheduler_mut().tick();  // moves eligible jobs to queue

// Now it's in the queue
assert_eq!(engine.scheduler().queue_len(), 1);
```

### Periodic Execution

```rust
// Schedule a backup every hour
let job_id = engine.scheduler_mut().submit(
    backup_task_id,
    Schedule::Interval(Duration::from_secs(3600)),
    5,  // priority
);

// Check that the schedule is recurring
let job = engine.scheduler().get_job(job_id).unwrap();
assert!(job.is_recurring());

// The Schedule type itself reports this:
assert!(Schedule::Interval(Duration::from_secs(60)).is_recurring());
assert!(!Schedule::Immediate.is_recurring());

// Compute the next occurrence
let last = std::time::Instant::now();
let next = Schedule::Interval(Duration::from_secs(60)).next_occurrence(last);
assert!(next.is_some());
```

### At-Time Execution

```rust
use std::time::{Duration, Instant};

let target = Instant::now() + Duration::from_secs(10);

let job_id = engine.scheduler_mut().submit(
    task_id,
    Schedule::AtTime(target),
    0,
);

// Not eligible until the target time
assert_eq!(engine.scheduler().queue_len(), 0);
```

### Priorities

The queue orders jobs by priority (higher first), then by creation time (FIFO within same priority):

```rust
// Three tasks with different priorities
let low_id = engine.scheduler_mut().submit(low_priority_task, Schedule::Immediate, 1);
let high_id = engine.scheduler_mut().submit(high_priority_task, Schedule::Immediate, 10);
let mid_id = engine.scheduler_mut().submit(mid_priority_task, Schedule::Immediate, 5);

// Dispatch order: high (10), mid (5), low (1)
assert_eq!(engine.scheduler_mut().dispatch().unwrap().priority(), 10);
assert_eq!(engine.scheduler_mut().dispatch().unwrap().priority(), 5);
assert_eq!(engine.scheduler_mut().dispatch().unwrap().priority(), 1);
```

### Triggers

A `Trigger` is a condition-based activation — complementary to time-based `Schedule`:

```rust
use app_shell::app_engine::Trigger;

// Manual trigger: always satisfied (just use the schedule)
let trigger = Trigger::Manual;

// Event-based: waits for a named event to occur
let trigger = Trigger::OnEvent("app.idle".to_string());

// Signal-based: waits for a signal
let trigger = Trigger::OnSignal("queue.empty".to_string());

// Condition-based: waits for a named condition
let trigger = Trigger::OnCondition("ready".to_string());
```

Submitting with a trigger:

```rust
// Job won't be queued until the trigger is satisfied
let job_id = engine.scheduler_mut().submit_with_trigger(
    task_id,
    Schedule::Immediate,
    Trigger::OnEvent("app.ready".to_string()),
    0,
);

assert_eq!(engine.scheduler().queue_len(), 0);       // not yet
assert_eq!(engine.scheduler().scheduled_count(), 1); // waiting

// Satisfy the trigger
engine.scheduler_mut().satisfy_trigger("app.ready");

// Tick moves it to the queue
engine.scheduler_mut().tick();
assert_eq!(engine.scheduler().queue_len(), 1);
```

Clearing a trigger:

```rust
engine.scheduler_mut().unsatisfy_trigger("app.ready");
```

### Cancelling a Scheduled Job

```rust
let job_id = engine.scheduler_mut().submit(
    task_id,
    Schedule::Delayed(Duration::from_secs(60)),
    0,
);

// Cancel before it runs
engine.scheduler_mut().cancel(job_id)?;

assert_eq!(engine.scheduler().scheduled_count(), 0);
assert_eq!(engine.scheduler().queue_len(), 0);
```

### Pause and Resume the Scheduler

```rust
engine.scheduler_mut().pause();
// tick() is now a no-op — no jobs move to the queue

engine.scheduler_mut().resume();
// tick() works again
```

### Retry Policies (Phase 27)

When a dispatched task fails, the scheduler can create a new delayed job based on the retry policy:

```rust
use app_shell::app_engine::RetryPolicy;

// Submit with retry policy
let job_id = engine.scheduler_mut().submit_with_retry(
    task_id,
    Schedule::Immediate,
    0,
    RetryPolicy::exponential(3, Duration::from_secs(1), 2.0),
);

// Dispatch and execute
let job = engine.scheduler_mut().dispatch().unwrap();
let result = engine.start_task(job.task_id());

// If failed, ask scheduler to retry
if result.is_err() {
    if let Some(retry_job_id) = engine.scheduler_mut().handle_task_failure(&job) {
        println!("Task scheduled for retry as job {}", retry_job_id.value());
        // The retry job is delayed (Schedule::Delayed with backoff delay)
        assert_eq!(engine.scheduler().scheduled_count(), 1);
    }
}
```

**Retry policies:**

```rust
// No retry
RetryPolicy::none()

// Fixed delay between attempts
RetryPolicy::fixed(3, Duration::from_secs(5))  // 3 retries, 5s delay each

// Exponential backoff
RetryPolicy::exponential(3, Duration::from_secs(1), 2.0)  // 1s, 2s, 4s
```

**Key rules:**
- Retry creates a **NEW job** (new `JobId`), not a re-run of the same job
- After `max_attempts`, no more retries are scheduled
- The retry job has `RetryPolicy::None` (created via `submit()`, not `submit_with_retry()`)
- Priority is preserved from the original job

### The Main Loop Pattern

```rust
engine.start()?;

while !engine.stop_requested() {
    // 1. Evaluate scheduler (move eligible jobs to queue)
    engine.scheduler_mut().tick();

    // 2. Dispatch and execute ready jobs
    while let Some(job) = engine.scheduler_mut().dispatch() {
        let task_id = job.task_id();
        match engine.start_task(task_id) {
            Ok(result) => {
                if result.is_completed() {
                    engine.event_bus().publish(
                        &Event::new("task.completed")
                            .with_payload(task_id.value())
                            .with_source("TaskExecutor"),
                    );
                } else if result.is_failed() && job.retry_policy().should_retry() {
                    // Retry on failure (Phase 27)
                    if let Some(retry_id) = engine.scheduler_mut().handle_task_failure(&job) {
                        // Retry job is delayed — will be eligible after backoff
                    }
                }
            }
            Err(e) => {
                eprintln!("Task {} failed: {}", task_id.value(), e);
            }
        }
    }

    // 3. Small sleep
    std::thread::sleep(Duration::from_millis(10));
}

engine.stop()?;
```

### Thread-Safe Scheduler (Phase 23)

The scheduler is fully thread-safe — all methods take `&self`:

```rust
use std::sync::Arc;
use std::thread;

let scheduler = Arc::new(Scheduler::new());

// Concurrent submits from multiple threads
let mut handles = vec![];
for _ in 0..5 {
    let scheduler_clone = Arc::clone(&scheduler);
    handles.push(thread::spawn(move || {
        scheduler_clone.submit(TaskId::new(), Schedule::Immediate, 0)
    }));
}

for h in handles {
    h.join().unwrap();
}

assert_eq!(scheduler.queue_len(), 5);
```

---

## Part 2: Events

### What Is an Event?

```text
Event = "Something happened."
```

Events are **past occurrences** — they announce that something already took place. They are:
- **One-to-many**: multiple consumers can subscribe.
- **Immutable**: the event describes what happened; it doesn't request action.
- **Decoupled**: the producer doesn't know who the consumers are.
- **Priority-ordered**: High before Normal before Low (Phase 19).
- **Panic-isolated**: one crashing handler doesn't kill the engine (Phase 19).

### Event Structure

```rust
use app_shell::app_engine::Event;

let event = Event::new("task.completed")
    .with_payload("task_42".to_string())
    .with_source("TaskExecutor");

assert_eq!(event.event_type(), "task.completed");
assert_eq!(event.source(), Some("TaskExecutor"));
assert!(event.has_payload());
assert_eq!(event.payload::<String>(), Some(&"task_42".to_string()));
```

### Event Priority (Phase 19)

```rust
use app_shell::app_engine::EventPriority;

// Subscribe with priority
engine.event_bus_mut().subscribe_with_priority(
    "task.completed",
    Box::new(HighPriorityHandler),
    EventPriority::High,      // delivered first
)?;

engine.event_bus_mut().subscribe_with_priority(
    "task.completed",
    Box::new(NormalHandler),
    EventPriority::Normal,    // default
)?;

engine.event_bus_mut().subscribe_with_priority(
    "task.completed",
    Box::new(LowPriorityHandler),
    EventPriority::Low,       // delivered last
)?;

// When published, order is: High → Normal → Low
engine.event_bus().publish(&Event::new("task.completed"));
```

### The EventHandler Trait

```rust
use app_shell::app_engine::{Event, EventHandler};

struct Logger;

impl EventHandler for Logger {
    fn handle(&self, event: &Event) {
        println!("[LOG] {} from {}", event.event_type(), event.source().unwrap_or("unknown"));
    }
}
```

### Publishing and Subscribing

```rust
// Subscribe
engine.event_bus_mut().subscribe(
    "task.completed",
    Box::new(Logger),
)?;

// Publish
engine.event_bus().publish(
    &Event::new("task.completed").with_source("TaskExecutor")
);

// Output: [LOG] task.completed from TaskExecutor
```

### Multiple Consumers

```rust
use std::sync::{Arc, Mutex};

struct Counter { count: Arc<Mutex<usize>> }
impl EventHandler for Counter {
    fn handle(&self, _event: &Event) {
        *self.count.lock().unwrap() += 1;
    }
}

let count_a = Arc::new(Mutex::new(0usize));
let count_b = Arc::new(Mutex::new(0usize));

engine.event_bus_mut().subscribe("task.completed", Box::new(Counter { count: count_a.clone() }));
engine.event_bus_mut().subscribe("task.completed", Box::new(Counter { count: count_b.clone() }));

engine.event_bus().publish(&Event::new("task.completed"));
engine.event_bus().publish(&Event::new("task.completed"));

assert_eq!(*count_a.lock().unwrap(), 2);
assert_eq!(*count_b.lock().unwrap(), 2);
```

### Unsubscribing

```rust
let handler_id = engine.event_bus_mut().subscribe(
    "task.completed",
    Box::new(Logger),
)?;

engine.event_bus_mut().unsubscribe("task.completed", handler_id);
// No more events delivered to Logger
```

### Panic Isolation (Phase 19)

If an event handler panics, it doesn't crash the engine or block other handlers:

```rust
struct PanickingHandler;
impl EventHandler for PanickingHandler {
    fn handle(&self, _event: &Event) {
        panic!("handler crashed!");
    }
}

struct SurvivingHandler { received: Arc<Mutex<bool>> }
impl EventHandler for SurvivingHandler {
    fn handle(&self, _event: &Event) {
        *self.received.lock().unwrap() = true;
    }
}

let received = Arc::new(Mutex::new(false));
engine.event_bus_mut().subscribe("test.isolation", Box::new(PanickingHandler));
engine.event_bus_mut().subscribe("test.isolation", Box::new(SurvivingHandler { received: received.clone() }));

// This should not panic
engine.event_bus().publish(&Event::new("test.isolation"));

// The surviving handler should still have been called
assert!(*received.lock().unwrap());
```

### Closure-Based Handlers

For simple handlers, use a closure wrapper:

```rust
struct ClosureEventHandler<F: Fn(&Event) + Send + Sync + 'static> { f: F }
impl<F: Fn(&Event) + Send + Sync + 'static> EventHandler for ClosureEventHandler<F> {
    fn handle(&self, event: &Event) { (self.f)(event); }
}
fn event_handler<F: Fn(&Event) + Send + Sync + 'static>(f: F) -> Box<dyn EventHandler> {
    Box::new(ClosureEventHandler { f })
}

// Usage:
let received = Arc::new(Mutex::new(false));
let r = received.clone();
engine.event_bus_mut().subscribe(
    "document.changed",
    event_handler(move |event: &Event| {
        *r.lock().unwrap() = true;
        println!("Document changed by {}", event.source().unwrap_or("?"));
    }),
);
```

### Serialized Event Payload (Phase 22)

For IPC, events can carry serialized bytes:

```rust
// Sender creates event with typed payload
let mut event = Event::new("document.exported")
    .with_payload(ExportInput {
        project_id: "doc_42".to_string(),
        format: "pdf".to_string(),
    });

// Serialize for IPC
event.ensure_serialized(&registry)?;
let type_name = event.payload_type_name();
let bytes = event.serialized_payload().unwrap().to_vec();

// Receiver reconstructs from bytes
let mut received = Event::from_serialized("document.exported", type_name, bytes);
assert!(!received.has_payload());

// Deserialize back to typed data
received.ensure_deserialized(&registry)?;
let payload = received.payload::<ExportInput>().unwrap();
assert_eq!(payload.project_id, "doc_42");
```

### The Decoupling Pattern

```text
TaskExecutor                    EventBus
     │                            │
     │ publish(TaskCompleted)     │
     └────────────────────────→   │
                                  ├──→ HistoryHandler (records)
                                  ├──→ PluginHandler (reacts)
                                  ├──→ UIHandler (refreshes view)
                                  └──→ DomainHandler (updates state)
```

The TaskExecutor has **no reference** to History, Plugin, Domain, or UI. It just publishes an event.

---

## Part 3: Signals

### What Is a Signal?

```text
Signal = "Something changed."
```

Signals are **lightweight change notifications** — they notify interested listeners that a value or state has changed. They are:
- **Lighter than events**: no formal handler trait, just callbacks.
- **Immediate**: typically consumed synchronously.
- **Change-focused**: "progress went from 0.5 to 0.7", not "operation completed".
- **Thread-safe**: `SignalBus` uses `Mutex` internally.

### Signal Structure

```rust
use app_shell::app_engine::Signal;

let signal = Signal::new("task.progress_changed")
    .with_payload(0.75_f32)
    .with_source("RenderTask");

assert_eq!(signal.signal_type(), "task.progress_changed");
assert_eq!(signal.payload::<f32>(), Some(&0.75));
```

### Emitting and Subscribing

```rust
let received = Arc::new(Mutex::new(vec![]));
let r = received.clone();

let sub_id = engine.signal_bus().subscribe(
    "task.progress_changed",
    Box::new(move |signal: &Signal| {
        r.lock().unwrap().push(signal.payload::<f32>().copied().unwrap_or(0.0));
    }),
);

// Emit signals
engine.signal_bus().emit(&Signal::new("task.progress_changed").with_payload(0.25_f32));
engine.signal_bus().emit(&Signal::new("task.progress_changed").with_payload(0.50_f32));
engine.signal_bus().emit(&Signal::new("task.progress_changed").with_payload(0.75_f32));

let values = received.lock().unwrap();
assert_eq!(*values, vec![0.25, 0.50, 0.75]);
```

### Unsubscribing

```rust
engine.signal_bus().unsubscribe("task.progress_changed", sub_id);
// No more deliveries
```

### Progress Streaming from Task Handlers (Phase 14)

Task handlers can emit progress signals through `TaskOutputChannel`, which is connected to the `SignalBus`:

```rust
struct RenderHandler;
impl TaskHandler for RenderHandler {
    fn execute(&self, _: &TaskInput, ctx: &TaskContext, _: &EngineRef) -> Result<TaskResult, TaskError> {
        for i in 0..100 {
            if ctx.is_cancelled() {
                return Ok(TaskResult::cancelled());
            }
            render_frame(i);
            // Report progress — emits "task.progress_changed" signal
            ctx.output().progress(i as f32 / 100.0);
            // Stream intermediate output — emits "task.output" signal
            ctx.output().push(format!("Frame {} rendered", i));
        }
        Ok(TaskResult::completed())
    }
}
```

The `TaskOutputChannel` is automatically connected to the engine's `SignalBus` when `start_task()` is called.

---

## Events vs Signals

| | Event | Signal |
|---|---|---|
| **Represents** | "Something happened" | "Something changed" |
| **Weight** | Heavier — may carry substantial data | Lighter — usually small |
| **Consumer** | `EventHandler` trait | `Fn(&Signal)` callback |
| **Priority** | Yes (High/Normal/Low) | No |
| **Panic isolation** | Yes (`catch_unwind`) | No (callbacks run directly) |
| **Typical use** | `TaskCompleted`, `ResourceLoaded`, `CommandExecuted` | `ProgressChanged`, `StateChanged`, `PropertyChanged` |

### When to Use Which

```rust
// Event: operation completed — a discrete occurrence
engine.event_bus().publish(&Event::new("task.completed").with_source("TaskExecutor"));

// Signal: progress changed — a continuous value update
engine.signal_bus().emit(
    &Signal::new("task.progress_changed").with_payload(0.5_f32)
);
```

A single operation can produce both:

```text
Task starts
  ↓
  ├── Signal: task.progress_changed (0.0)   ← continuous
  ├── Signal: task.progress_changed (0.25)
  ├── Signal: task.progress_changed (0.50)
  ├── Signal: task.progress_changed (0.75)
  ├── Signal: task.progress_changed (1.0)
  └── Event: task.completed                 ← discrete
```

---

## When NOT to Use Events

### Don't Use Events When You Need a Result

```rust
// BAD: using events to get a return value
engine.event_bus().publish(&Event::new("get.current.document")); // no way to get response!

// GOOD: use a direct call
let document = engine.state_manager().current(document_state_id);

// GOOD: use a command (if it's an operation)
let result = engine.execute_command(&get_document_command)?;
```

### Don't Use Events for Every Internal Call

```rust
// BAD: routing internal logic through events
engine.event_bus().publish(&Event::new("internal.sort_completed"));
// Another handler picks this up and calls the next step...

// GOOD: call directly
sort_data();
process_sorted_data();
```

### Don't Record Every Signal in History

```text
✓ History: CreateDocument, MoveObject, DeleteObject
✗ History: MouseMoved, FrameRendered, ProgressChanged
```

History records **meaningful application operations**. Signals are runtime notifications.

### Rule of Thumb

```text
Need an answer?          → Direct call (command, manager method)
Need to announce?        → Event
Need to notify a change? → Signal
Need to undo?           → History (via events)
```

---

## Integration: Scheduler + Events

The scheduler and event bus work together for reactive scheduling:

```rust
// A service reacts to a "document.changed" event
// by submitting a backup task to the scheduler
engine.event_bus_mut().subscribe(
    "document.changed",
    event_handler(|_event: &Event| {
        // In a real service, this would submit a backup task
        // engine.scheduler_mut().submit(backup_task_id, Schedule::Delayed(Duration::from_secs(10)), 3);
    }),
);
```

The scheduler doesn't know about the event bus, and the event bus doesn't know about the scheduler. They are connected through the application's logic.

---

## Integration: Events + History

```rust
// History subscribes to meaningful events
engine.event_bus_mut().subscribe(
    "document.created",
    event_handler(|event: &Event| {
        // Record in history for undo/redo
        // engine.history_store().append(
        //     HistoryEntry::new("document.create")
        //         .undoable()
        //         .with_data(event.payload::<String>().unwrap().clone())
        //         .with_source(event.source().unwrap_or("unknown")),
        // );
    }),
);

// Now when any code publishes "document.created", it's automatically recorded
engine.event_bus().publish(
    &Event::new("document.created")
        .with_payload("doc_42".to_string())
        .with_source("CreateDocumentCommand"),
);

// History now has the entry
assert_eq!(engine.history_store().entry_count(), 1);
```

---

## Integration: Signals + UI

```rust
// UI subscribes to progress signals
engine.signal_bus().subscribe(
    "task.progress_changed",
    Box::new(|signal: &Signal| {
        let progress = signal.payload::<f32>().unwrap_or(0.0);
        // update_progress_bar(progress * 100.0);
        println!("Progress: {:.0}%", progress * 100.0);
    }),
);

// A task handler emits progress
struct RenderHandler;
impl TaskHandler for RenderHandler {
    fn execute(&self, _: &TaskInput, ctx: &TaskContext, _: &EngineRef) -> Result<TaskResult, TaskError> {
        ctx.output().progress(0.5); // emits signal → UI updates
        Ok(TaskResult::completed())
    }
}
```

---

## Quick Reference

### Scheduler API

```rust
// Submit
let job_id = engine.scheduler_mut().submit(task_id, schedule, priority);
let job_id = engine.scheduler_mut().submit_with_trigger(task_id, schedule, trigger, priority);
let job_id = engine.scheduler_mut().submit_with_retry(task_id, schedule, priority, retry_policy);

// Process
engine.scheduler_mut().tick();                    // move eligible → queue
let job = engine.scheduler_mut().dispatch();      // dequeue + mark Dispatched

// Cancel
engine.scheduler_mut().cancel(job_id)?;

// Pause / Resume
engine.scheduler_mut().pause();
engine.scheduler_mut().resume();

// Triggers
engine.scheduler_mut().satisfy_trigger("app.ready");
engine.scheduler_mut().unsatisfy_trigger("app.ready");

// Retry
let retry_id = engine.scheduler_mut().handle_task_failure(&job)?;

// Query
engine.scheduler().queue_len();            // ready jobs
engine.scheduler().scheduled_count();      // waiting jobs
engine.scheduler().total_job_count();      // all jobs
engine.scheduler().get_job(job_id);       // Option<Job> (cloned)
```

### EventBus API

```rust
// Subscribe / Unsubscribe
let id = engine.event_bus_mut().subscribe(event_type, handler);
let id = engine.event_bus_mut().subscribe_with_priority(event_type, handler, EventPriority::High);
engine.event_bus_mut().unsubscribe(event_type, id);

// Publish
engine.event_bus().publish(&event);

// Query
engine.event_bus().handler_count(event_type);
engine.event_bus().total_handlers();
```

### SignalBus API

```rust
// Subscribe / Unsubscribe
let id = engine.signal_bus().subscribe(signal_type, callback);
engine.signal_bus().unsubscribe(signal_type, id);

// Emit
engine.signal_bus().emit(&signal);

// Query
engine.signal_bus().subscriber_count(signal_type);
engine.signal_bus().total_subscribers();
```

### RetryPolicy API

```rust
// Create
RetryPolicy::none()
RetryPolicy::fixed(max_attempts, delay)
RetryPolicy::exponential(max_attempts, base_delay, factor)

// Query
policy.should_retry()                  // bool
policy.max_attempts()                  // u32
policy.delay_for_attempt(attempt)      // Option<Duration>
```

---

## Next Steps

- [07-history-and-resources.md](07-history-and-resources.md) — Undo/redo and resource management.
- [11-handler-capabilities.md](11-handler-capabilities.md) — What handlers can access via `EngineRef`.
- [12-execution-control.md](12-execution-control.md) — Cancellation, deadlines, and progress streaming.
- [15-communication-reliability.md](15-communication-reliability.md) — Priority, panic isolation, and retry in depth.