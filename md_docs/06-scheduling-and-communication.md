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
Queue
  ↓
Executor
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
    "project.export",
    TaskContext::new(ExecutionId::new()),
    TaskInput::new(export_data),
)?;

// Submit to scheduler with Immediate schedule
let job_id = engine.scheduler_mut().submit(
    task_id,
    Schedule::Immediate,
    0,  // priority (higher = sooner)
);
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
let result = engine.task_manager_mut().start(job.task_id())?;
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
std::thread::sleep(Duration::from_millis(10)); // simulate time passing
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
let low_id = engine.scheduler_mut().submit(
    low_priority_task, Schedule::Immediate, 1,
);
let high_id = engine.scheduler_mut().submit(
    high_priority_task, Schedule::Immediate, 10,
);
let mid_id = engine.scheduler_mut().submit(
    mid_priority_task, Schedule::Immediate, 5,
);

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

### The Main Loop Pattern

```rust
engine.start()?;

while !engine.stop_requested() {
    // 1. Evaluate scheduled jobs
    engine.scheduler_mut().tick();

    // 2. Dispatch and execute ready jobs
    while let Some(job) = engine.scheduler_mut().dispatch() {
        let task_id = job.task_id();
        match engine.task_manager_mut().start(task_id) {
            Ok(result) => {
                if result.is_completed() {
                    engine.event_bus().publish(
                        &Event::new("task.completed")
                            .with_payload(task_id.value())
                            .with_source("TaskExecutor"),
                    );
                } else if result.is_failed() {
                    engine.event_bus().publish(
                        &Event::new("task.failed")
                            .with_source("TaskExecutor"),
                    );
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
engine.event_bus().publish(&Event::new("task.completed").with_source("TaskExecutor"));

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

let count_a = Arc::new(Mutex::new(0));
let count_b = Arc::new(Mutex::new(0));

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

### Event Payload Reaches Handlers

```rust
struct CapturingHandler { captured: Arc<Mutex<Option<i32>>> }
impl EventHandler for CapturingHandler {
    fn handle(&self, event: &Event) {
        if let Some(val) = event.payload::<i32>() {
            *self.captured.lock().unwrap() = Some(*val);
        }
    }
}

let captured = Arc::new(Mutex::new(None));
engine.event_bus_mut().subscribe(
    "task.progress",
    Box::new(CapturingHandler { captured: captured.clone() }),
);

engine.event_bus().publish(&Event::new("task.progress").with_payload(75_i32));

assert_eq!(*captured.lock().unwrap(), Some(75));
```

### The Decoupling Pattern

```text
TaskExecutor                    EventBus
     │                            │
     │ publish(TaskCompleted)     │
     └────────────────────────→   │
                                  ├──→ HistoryHandler (records)
                                  ├──→ PluginHandler (reacts)
                                  ├──→ DomainHandler (updates state)
                                  └──→ UIHandler (refreshes view)
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

### Multiple Subscribers

```rust
let ui_count = Arc::new(Mutex::new(0));
let monitor_count = Arc::new(Mutex::new(0));

let ui = ui_count.clone();
engine.signal_bus().subscribe(
    "state.changed",
    Box::new(move |_signal: &Signal| { *ui.lock().unwrap() += 1; }),
);

let mon = monitor_count.clone();
engine.signal_bus().subscribe(
    "state.changed",
    Box::new(move |_signal: &Signal| { *mon.lock().unwrap() += 1; }),
);

engine.signal_bus().emit(&Signal::new("state.changed"));

assert_eq!(*ui_count.lock().unwrap(), 1);
assert_eq!(*monitor_count.lock().unwrap(), 1);
assert_eq!(engine.signal_bus().subscriber_count("state.changed"), 2);
```

---

## Events vs Signals

| | Event | Signal |
|---|---|---|
| **Represents** | "Something happened" | "Something changed" |
| **Weight** | Heavier — may carry substantial data | Lighter — usually small |
| **Consumer** | `EventHandler` trait | `Fn(&Signal)` callback |
| **Typical use** | `TaskCompleted`, `ResourceLoaded`, `CommandExecuted` | `ProgressChanged`, `StateChanged`, `PropertyChanged` |
| **History-worthy?** | Often | Rarely |

### When to Use Which

```rust
// Event: operation completed — a discrete occurrence
engine.event_bus().publish(&Event::new("task.completed").with_source("TaskExecutor"));

// Signal: progress changed — a continuous value update
engine.signal_bus().emit(&Signal::new("task.progress_changed").with_payload(0.5_f32));
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
let result = engine.command_executor().execute(&get_document_command)?;
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
// A periodic backup service reacts to a "document.changed" event
// by submitting a new backup task to the scheduler

engine.event_bus_mut().subscribe(
    "document.changed",
    event_handler(move |_event: &Event| {
        // Submit a backup task with a 10-second delay
        let _job_id = engine.scheduler_mut().submit(
            backup_task_id,
            Schedule::Delayed(Duration::from_secs(10)),
            3,  // priority
        );
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
    event_handler(move |event: &Event| {
        engine.history_store_mut().append(
            HistoryEntry::new("document.create")
                .undoable()
                .with_data(event.payload::<String>().unwrap().clone())
                .with_source(event.source().unwrap_or("unknown")),
        );
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
    fn execute(&self, _: &TaskInput, ctx: &TaskContext) -> Result<TaskResult, TaskError> {
        // The handler would need access to the SignalBus to emit progress.
        // In a real app, this would be injected or accessed through context.
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

// Query
engine.scheduler().queue_len();            // ready jobs
engine.scheduler().scheduled_count();      // waiting jobs
engine.scheduler().total_job_count();      // all jobs
engine.scheduler().get_job(job_id);       // Option<&Job>
```

### EventBus API

```rust
// Subscribe / Unsubscribe
let id = engine.event_bus_mut().subscribe(event_type, handler);
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

---

## Next Steps

- [07-history-and-resources.md](07-history-and-resources.md) — Undo/redo and resource management.
- [08-services-and-plugins.md](08-services-and-plugins.md) — Long-lived services and plugin extensions.