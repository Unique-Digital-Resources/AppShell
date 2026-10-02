# `20-quick-start.md` — 5-Minute Getting Started

## Overview

This guide takes you from zero to a working App Engine in 5 minutes. By the end, you'll have:
- Created an `AppEngine`
- Registered and executed a command
- Created and executed a task
- Published and subscribed to events
- Loaded a resource
- Done undo/redo
- Shut down cleanly

---

## Step 1: Create the Engine

```rust
use app_shell::app_engine::{AppEngine, Bootstrap, Lifecycle};

let mut engine: AppEngine = Bootstrap::create().expect("bootstrap failed");
```

That's it. All subsystems are wired and ready.

---

## Step 2: Register a Command

Define a handler, register it:

```rust
use app_shell::app_engine::*;
use std::any::Any;

// Your domain input type
#[derive(Debug, Clone)]
struct GreetInput {
    name: String,
}

// Your command handler
struct GreetCommand;

impl CommandHandler for GreetCommand {
    fn execute(
        &self,
        input: &CommandInput,
        _context: &CommandContext,
        _engine: &EngineRef,
    ) -> Result<CommandResult, CommandError> {
        let input: &GreetInput = input.get::<GreetInput>()
            .ok_or_else(|| CommandError::InvalidArguments("expected GreetInput".to_string()))?;

        Ok(CommandResult::success_with(format!("Hello, {}!", input.name)))
    }
}

// Register
engine.command_executor_mut().register(
    CommandDefinition::new("app.greet", "Greet", "Greet someone by name"),
    Box::new(GreetCommand),
)?;
```

---

## Step 3: Execute the Command

```rust
// Start the engine
engine.initialize()?;
engine.start()?;

// Create and execute the command
let cmd = Command::new(
    "app.greet",
    CommandContext::new(ExecutionId::new()),
    CommandInput::new(GreetInput { name: "World".to_string() }),
);

let result = engine.execute_command(&cmd)?;

assert!(result.is_success());
assert_eq!(result.output::<String>(), Some(&"Hello, World!".to_string()));
```

---

## Step 4: Register and Execute a Task

For longer operations with lifecycle:

```rust
// Your task handler
struct CountTask;

impl TaskHandler for CountTask {
    fn execute(
        &self,
        _input: &TaskInput,
        ctx: &TaskContext,
        _engine: &EngineRef,
    ) -> Result<TaskResult, TaskError> {
        // Check cancellation (Phase 14)
        if ctx.is_cancelled() {
            return Ok(TaskResult::cancelled());
        }

        // Report progress (Phase 14 — emits signal)
        ctx.output().progress(0.5);

        // Do work
        let count = 42;

        ctx.output().progress(1.0);

        Ok(TaskResult::completed_with(count))
    }
}

// Register
engine.task_manager_mut().register(
    TaskDefinition::new("app.count", "Count", "Count to 42")
        .with_cancel_support(),
    Box::new(CountTask),
)?;

// Create and execute
let task_id = engine.task_manager_mut().create(
    "app.count",
    TaskContext::new(ExecutionId::new()),
    TaskInput::empty(),
)?;

let result = engine.start_task(task_id)?;
assert!(result.is_completed());
assert_eq!(result.output::<i32>(), Some(&42));
```

---

## Step 5: Publish and Subscribe to Events

```rust
use std::sync::{Arc, Mutex};

let received = Arc::new(Mutex::new(false));
let r = received.clone();

// Subscribe BEFORE publishing
engine.event_bus_mut().subscribe(
    "task.completed",
    event_handler(move |_event: &Event| {
        *r.lock().unwrap() = true;
    }),
);

// Publish (from a handler or directly)
engine.event_bus().publish(
    &Event::new("task.completed").with_source("QuickStart")
);

assert!(*received.lock().unwrap());
```

### Closure Wrapper for Events

If you want to use closures (not just `EventHandler` impl):

```rust
struct ClosureEventHandler<F: Fn(&Event) + Send + Sync + 'static> { f: F }
impl<F: Fn(&Event) + Send + Sync + 'static> EventHandler for ClosureEventHandler<F> {
    fn handle(&self, event: &Event) { (self.f)(event); }
}
fn event_handler<F: Fn(&Event) + Send + Sync + 'static>(f: F) -> Box<dyn EventHandler> {
    Box::new(ClosureEventHandler { f })
}

// Usage:
engine.event_bus_mut().subscribe("my.event", event_handler(|event| {
    println!("Got: {}", event.event_type());
}));
```

---

## Step 6: Load a Resource

```rust
// Define a resource loader
struct TextLoader;
impl ResourceLoader for TextLoader {
    fn resource_type(&self) -> &str { "text" }
    fn load(&self, source: &str) -> Result<Box<dyn Any + Send>, ResourceError> {
        Ok(Box::new(format!("content:{}", source)))
    }
}

// Register the loader
engine.resource_manager().register_loader(Box::new(TextLoader));

// Load (RAII handle auto-releases on drop — Phase 25)
let handle = engine.resource_manager().load("text", "file:///welcome.txt")?;

// Access the data
let data = engine.resource_manager()
    .with_resource(handle.resource_id(), |r| {
        r.data::<String>().map(|s| s.clone())
    })
    .flatten();

assert_eq!(data.as_deref(), Some("content:file:///welcome.txt"));
// Handle drops here → ref count = 0, resource stays in cache
```

---

## Step 7: Undo / Redo

```rust
// Record a history entry
engine.history_store().append(
    HistoryEntry::new("object.create")
        .undoable()
        .with_data("rect_1".to_string())
        .with_source("QuickStart")
);

engine.history_store().append(
    HistoryEntry::new("object.move")
        .undoable()
        .with_data("moved".to_string())
        .with_inverse("unmoved".to_string())
);

assert_eq!(engine.history_store().active_count(), 2);

// Undo
let undo_result = engine.history_store().undo();
assert_eq!(undo_result.count(), 1);
assert_eq!(engine.history_store().position(), 1);

// Redo
let redo_result = engine.history_store().redo();
assert_eq!(redo_result.count(), 1);
assert_eq!(engine.history_store().position(), 2);
```

---

## Step 8: Shut Down Cleanly

```rust
// Stop — cancels all running tasks (Phase 26)
engine.stop()?;

// Dispose — unloads all non-in-use resources (Phase 26)
engine.dispose()?;

assert_eq!(engine.state(), RuntimeState::Disposed);
```

---

## Complete Example

```rust
use app_shell::app_engine::*;
use std::any::Any;
use std::sync::{Arc, Mutex};

// --- Stubs ---

struct GreetCommand;

impl CommandHandler for GreetCommand {
    fn execute(&self, input: &CommandInput, _: &CommandContext, _: &EngineRef) -> Result<CommandResult, CommandError> {
        let name: &String = input.get::<String>()
            .ok_or_else(|| CommandError::InvalidArguments("expected name".to_string()))?;
        Ok(CommandResult::success_with(format!("Hello, {}!", name)))
    }
}

struct TextLoader;
impl ResourceLoader for TextLoader {
    fn resource_type(&self) -> &str { "text" }
    fn load(&self, source: &str) -> Result<Box<dyn Any + Send>, ResourceError> {
        Ok(Box::new(format!("content:{}", source)))
    }
}

// --- Closure wrapper for events ---

struct ClosureEventHandler<F: Fn(&Event) + Send + Sync + 'static> { f: F }
impl<F: Fn(&Event) + Send + Sync + 'static> EventHandler for ClosureEventHandler<F> {
    fn handle(&self, event: &Event) { (self.f)(event); }
}
fn event_handler<F: Fn(&Event) + Send + Sync + 'static>(f: F) -> Box<dyn EventHandler> {
    Box::new(ClosureEventHandler { f })
}

// --- Main ---

fn main() {
    // 1. Create the engine
    let mut engine: AppEngine = Bootstrap::create().expect("bootstrap failed");

    // 2. Register a command
    engine.command_executor_mut().register(
        CommandDefinition::new("app.greet", "Greet", "Greet someone"),
        Box::new(GreetCommand),
    ).unwrap();

    // 3. Register a resource loader
    engine.resource_manager().register_loader(Box::new(TextLoader));

    // 4. Subscribe to events
    let received = Arc::new(Mutex::new(false));
    let r = received.clone();
    engine.event_bus_mut().subscribe(
        "greet.done",
        event_handler(move |_event: &Event| { *r.lock().unwrap() = true; }),
    );

    // 5. Start the engine
    engine.initialize().unwrap();
    engine.start().unwrap();

    // 6. Execute a command
    let cmd = Command::new(
        "app.greet",
        CommandContext::new(ExecutionId::new()),
        CommandInput::new("World".to_string()),
    );
    let result = engine.execute_command(&cmd).unwrap();
    println!("{}", result.output::<String>().unwrap());

    // 7. Publish an event
    engine.event_bus().publish(&Event::new("greet.done").with_source("main"));
    assert!(*received.lock().unwrap());

    // 8. Load a resource (RAII handle auto-releases on drop)
    let handle = engine.resource_manager()
        .load("text", "file:///welcome.txt").unwrap();
    let data = engine.resource_manager()
        .with_resource(handle.resource_id(), |r| r.data::<String>().map(|s| s.clone()))
        .flatten();
    println!("Loaded: {}", data.unwrap());

    // 9. Undo/redo
    engine.history_store().append(
        HistoryEntry::new("greet").undoable().with_source("main"),
    );
    engine.history_store().undo();
    engine.history_store().redo();

    // 10. Shut down (cancels tasks, unloads resources)
    engine.stop().unwrap();
    engine.dispose().unwrap();

    println!("Done! State: {:?}", engine.state());
}
```

Output:
```text
Hello, World!
Loaded: content:file:///welcome.txt
Done! State: Disposed
```

---

## Quick Reference Card

```rust
// Create
let mut engine = Bootstrap::create()?;

// Register command
engine.command_executor_mut().register(def, handler)?;

// Register task
engine.task_manager_mut().register(def, handler)?;

// Register resource loader
engine.resource_manager().register_loader(loader);

// Subscribe to event
engine.event_bus_mut().subscribe("type", handler);

// Lifecycle
engine.initialize()?;
engine.start()?;
engine.run()?;
engine.stop()?;
engine.dispose()?;

// Execute command
let result = engine.execute_command(&cmd)?;

// Execute task
let task_id = engine.task_manager_mut().create(def_id, ctx, input)?;
let result = engine.start_task(task_id)?;

// Publish event
engine.event_bus().publish(&Event::new("type"));

// Load resource
let handle = engine.resource_manager().load("type", "source")?;

// Undo/redo
engine.history_store().undo();
engine.history_store().redo();
```

---

## Next Steps

- [01-overview.md](01-overview.md) — What App Engine is.
- [03-bootstrap-and-lifecycle.md](03-bootstrap-and-lifecycle.md) — Lifecycle in depth.
- [05-commands-and-tasks.md](05-commands-and-tasks.md) — Commands and tasks.
- [09-domain-engine-integration.md](09-domain-engine-integration.md) — Building a Domain Engine.
- [16-testing-and-metadata.md](16-testing-and-metadata.md) — Testing your handlers.