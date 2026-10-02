# `16-testing-and-metadata.md` — Testing & Metadata

## Overview

The App Engine provides two systems that improve developer experience:

```text
Testing     = Mocks, TestEngine, assertion helpers — test handlers without a full engine
Metadata    = Version, PayloadSchema — describe and version command/task/plugin definitions
```

---

## Part 1: Testing Infrastructure (Phase 20)

### Why Mocks?

Without mocks, testing a `CommandHandler` requires:
- Creating a full `AppEngine` via `Bootstrap`
- Registering the handler
- Constructing `EngineRef`
- Executing the command

With mocks, you can:
- Create a `MockCommandExecutor` that records executions
- Create a `MockEventBus` that records publications
- Verify specific commands were executed or events were published

### MockCommandExecutor

Records executed commands and provides assertion methods:

```rust
use app_shell::app_engine::{MockCommandExecutor, CommandDefinition, Command, CommandContext, ExecutionId, CommandInput, EngineRef, CommandHandler, CommandResult, CommandError};

struct EchoHandler;
impl CommandHandler for EchoHandler {
    fn execute(&self, _: &CommandInput, _: &CommandContext, _: &EngineRef) -> Result<CommandResult, CommandError> {
        Ok(CommandResult::success())
    }
}

// Create a TestEngine for EngineRef
let test_engine = app_shell::app_engine::TestEngine::new();

// Create mock executor
let mut exec = MockCommandExecutor::new();
exec.register(
    CommandDefinition::new("app.echo", "Echo", "Echo"),
    Box::new(EchoHandler),
)?;

// Execute a command
let cmd = Command::with_empty_input("app.echo", CommandContext::new(ExecutionId::new()));
let engine_ref = test_engine.engine_ref();
exec.execute(&cmd, &engine_ref)?;

// Assert it was executed
exec.assert_executed("app.echo");
assert_eq!(exec.executed_count(), 1);

// Assert something was NOT executed
exec.assert_not_executed("app.missing");
```

### MockEventBus

Records published events:

```rust
use app_shell::app_engine::MockEventBus;
use app_shell::app_engine::Event;

let bus = MockEventBus::new();

bus.publish(&Event::new("task.completed"));
bus.publish(&Event::new("document.changed"));

// Assert events were published
bus.assert_published("task.completed");
bus.assert_published("document.changed");

// Assert an event was NOT published
bus.assert_not_published("task.failed");

// Count
assert_eq!(bus.published_count(), 2);

// Get all published event types
let events = bus.published_events();
assert!(events.contains(&"task.completed".to_string()));
```

### MockSignalBus

Records emitted signals:

```rust
use app_shell::app_engine::{MockSignalBus, Signal};

let bus = MockSignalBus::new();

bus.emit(&Signal::new("task.progress_changed"));
bus.emit(&Signal::new("state.changed"));

// Assert signals were emitted
bus.assert_emitted("task.progress_changed");
bus.assert_emitted("state.changed");

// Count
assert_eq!(bus.emitted_count(), 2);
```

### TestEngine

A pre-configured, lightweight engine for fast test setup:

```rust
use app_shell::app_engine::TestEngine;

let engine = TestEngine::new();

// Provides real (empty) subsystems
let engine_ref = engine.engine_ref();

// All systems accessible
let _ = engine_ref.event_bus;
let _ = engine_ref.signal_bus;
let _ = engine_ref.resource_manager;
let _ = engine_ref.config_store;
let _ = engine_ref.preferences;
let _ = engine_ref.state_manager;
let _ = engine_ref.command_executor;
let _ = engine_ref.scheduler;
let _ = engine_ref.history_store;
```

### TestEngine with Real Handlers

```rust
let mut engine = TestEngine::new();

// Register a real handler
engine.command_executor.register(
    CommandDefinition::new("test.echo", "Test", "Test"),
    Box::new(EchoHandler),
)?;

// Execute
let cmd = Command::with_empty_input("test.echo", CommandContext::new(ExecutionId::new()));
let engine_ref = engine.engine_ref();
let result = engine.command_executor.execute(&cmd, &engine_ref).unwrap();
assert!(result.is_success());
```

### Assertion Helpers

Convenience functions for common assertions:

```rust
use app_shell::app_engine::{
    assert_command_executed,
    assert_command_not_executed,
    assert_event_published,
    assert_event_not_published,
    assert_signal_emitted,
};

// With MockCommandExecutor
assert_command_executed(&exec, "app.echo");
assert_command_not_executed(&exec, "app.missing");

// With MockEventBus
assert_event_published(&bus, "task.completed");
assert_event_not_published(&bus, "task.failed");

// With MockSignalBus
assert_signal_emitted(&bus, "task.progress_changed");
```

### Test Pattern: Verify Handler Publishes Events

```rust
use app_shell::app_engine::*;
use std::sync::{Arc, Mutex};

struct GreetHandler {
    received: Arc<Mutex<bool>>,
}

impl CommandHandler for GreetHandler {
    fn execute(&self, _: &CommandInput, _: &CommandContext, _: &EngineRef) -> Result<CommandResult, CommandError> {
        // In a real handler, this would publish via EngineRef.event_bus
        Ok(CommandResult::success())
    }
}

#[test]
fn handler_executes_and_succeeds() {
    let test_engine = TestEngine::new();
    let mut exec = MockCommandExecutor::new();
    exec.register(
        CommandDefinition::new("app.greet", "Greet", "Greet"),
        Box::new(GreetHandler { received: Arc::new(Mutex::new(false)) }),
    ).unwrap();

    let cmd = Command::with_empty_input("app.greet", CommandContext::new(ExecutionId::new()));
    let engine_ref = test_engine.engine_ref();
    let result = exec.execute(&cmd, &engine_ref).unwrap();

    assert!(result.is_success());
    assert_command_executed(&exec, "app.greet");
}
```

### Test Pattern: Verify Events Are Published

```rust
#[test]
fn event_is_published() {
    let bus = MockEventBus::new();

    // Simulate a handler publishing an event
    bus.publish(&Event::new("task.completed").with_source("TestHandler"));

    assert_event_published(&bus, "task.completed");
    assert_event_not_published(&bus, "task.failed");
    assert_eq!(bus.published_count(), 1);
}
```

### Test Pattern: Verify Signals Are Emitted

```rust
#[test]
fn signal_is_emitted() {
    let bus = MockSignalBus::new();

    // Simulate a handler emitting a signal
    bus.emit(&Signal::new("task.progress_changed").with_payload(0.5_f32));

    assert_signal_emitted(&bus, "task.progress_changed");
    assert_eq!(bus.emitted_count(), 1);
}
```

---

## Part 2: Metadata (Phase 20)

### Version

Semantic versioning for commands, tasks, and plugins:

```rust
use app_shell::app_engine::Version;

let v1 = Version::new(1, 2, 3);
assert_eq!(v1.major(), 1);
assert_eq!(v1.minor(), 2);
assert_eq!(v1.patch(), 3);

assert_eq!(v1.to_string(), "1.2.3");
```

### Version Compatibility

Same major version = compatible:

```rust
let v1 = Version::new(1, 0, 0);
let v2 = Version::new(1, 5, 3);
let v3 = Version::new(2, 0, 0);

assert!(v1.is_compatible_with(&v2));  // Same major
assert!(!v1.is_compatible_with(&v3));  // Different major
```

### Version Ordering

```rust
let v1 = Version::new(1, 0, 0);
let v2 = Version::new(1, 1, 0);
let v3 = Version::new(2, 0, 0);

assert!(v1 < v2);
assert!(v2 < v3);
assert!(v1 < v3);
```

### Version At Least

```rust
let v = Version::new(1, 2, 3);
assert!(v.is_at_least(&Version::new(1, 2, 0)));   // Yes — 1.2.3 >= 1.2.0
assert!(!v.is_at_least(&Version::new(1, 3, 0))); // No — 1.2.3 < 1.3.0
```

### Versioned Trait

```rust
use app_shell::app_engine::Versioned;

// Any type that exposes a Version can implement Versioned
pub trait Versioned {
    fn version(&self) -> &Version;
}
```

### PayloadSchema

Describes the expected type of a command/task input or output:

```rust
use app_shell::app_engine::PayloadSchema;

let schema = PayloadSchema::of::<String>();
assert!(schema.matches::<String>());
assert!(!schema.matches::<i32>());
assert!(schema.to_string().contains("String"));
```

### PayloadSchema with Description

```rust
let schema = PayloadSchema::of_with_description::<String>("The document title");
assert_eq!(schema.description(), "The document title");
```

### PayloadSchema Equality

Two schemas are equal if they describe the same type:

```rust
let s1 = PayloadSchema::of::<String>();
let s2 = PayloadSchema::of::<String>();
let s3 = PayloadSchema::of::<i32>();
assert_eq!(s1, s2);
assert_ne!(s1, s3);
```

---

## Part 3: Versioned Definitions

### CommandDefinition with Version and Schema

```rust
use app_shell::app_engine::CommandDefinition;

let def = CommandDefinition::new("project.export", "Export Project", "Exports the current project")
    .with_version(Version::new(1, 0, 0))
    .with_input_schema(PayloadSchema::of::<ExportProjectInput>())
    .with_output_schema(PayloadSchema::of::<ExportResult>());

assert_eq!(def.version(), &Version::new(1, 0, 0));
assert!(def.input_schema().is_some());
assert!(def.output_schema().is_some());
assert!(def.input_schema().unwrap().matches::<ExportProjectInput>());
assert!(def.output_schema().unwrap().matches::<ExportResult>());
```

### Compatibility Check

```rust
let def = CommandDefinition::new("project.export", "Export", "Export")
    .with_version(Version::new(1, 5, 0));

assert!(def.is_compatible_with(&Version::new(1, 0, 0)));   // Same major
assert!(!def.is_compatible_with(&Version::new(2, 0, 0)));  // Different major
```

### TaskDefinition with Version and Schema

```rust
use app_shell::app_engine::TaskDefinition;

let def = TaskDefinition::new("image.render", "Render Image", "Render an image")
    .with_version(Version::new(3, 1, 0))
    .with_input_schema(PayloadSchema::of::<RenderInput>())
    .with_output_schema(PayloadSchema::of::<RenderResult>());

assert_eq!(def.version(), &Version::new(3, 1, 0));
assert!(def.input_schema().is_some());
assert!(def.output_schema().is_some());
```

### PluginManifest with API Version

```rust
use app_shell::app_engine::PluginManifest;

let manifest = PluginManifest::new("image-plugin", "Image Plugin")
    .with_engine_api_version(Version::new(1, 0, 0));

assert_eq!(manifest.engine_api_version(), &Version::new(1, 0, 0));
assert!(manifest.is_compatible_with(&Version::new(1, 5, 0)));   // Same major
assert!(!manifest.is_compatible_with(&Version::new(2, 0, 0)));  // Different major
```

---

## Part 4: Integration — Testing with Metadata

### Verify Schema Matches Expected Type

```rust
#[test]
fn command_input_schema_matches() {
    let def = CommandDefinition::new("export", "Export", "Export")
        .with_input_schema(PayloadSchema::of::<ExportInput>())
        .with_output_schema(PayloadSchema::of::<ExportResult>());

    assert!(def.input_schema().unwrap().matches::<ExportInput>());
    assert!(def.output_schema().unwrap().matches::<ExportResult>());
    assert!(!def.input_schema().unwrap().matches::<i32>());
}
```

### Test Version Compatibility

```rust
#[test]
fn version_compatibility() {
    let def = CommandDefinition::new("export", "Export", "Export")
        .with_version(Version::new(2, 0, 0));

    assert!(def.is_compatible_with(&Version::new(2, 1, 0)));
    assert!(!def.is_compatible_with(&Version::new(3, 0, 0)));
}
```

### Complete Test Example

```rust
use app_shell::app_engine::*;

struct CountingHandler {
    order: Arc<Mutex<Vec<usize>>>,
    tag: usize,
}

impl EventHandler for CountingHandler {
    fn handle(&self, _: &Event) {
        self.order.lock().unwrap().push(self.tag);
    }
}

#[test]
fn full_test_with_mocks_and_metadata() {
    // --- Setup ---
    let test_engine = TestEngine::new();
    let engine_ref = test_engine.engine_ref();

    let mut exec = MockCommandExecutor::new();
    let def = CommandDefinition::new("app.test", "Test", "Test command")
        .with_version(Version::new(1, 0, 0))
        .with_input_schema(PayloadSchema::of::<String>());

    exec.register(def, Box::new(EchoHandler)).unwrap();

    // --- Execute ---
    let cmd = Command::with_empty_input("app.test", CommandContext::new(ExecutionId::new()));
    let result = exec.execute(&cmd, &engine_ref).unwrap();

    // --- Assertions ---
    assert!(result.is_success());
    assert_command_executed(&exec, "app.test");
    assert_eq!(exec.executed_count(), 1);

    // --- Verify metadata ---
    let def = exec.list().first().unwrap();
    assert_eq!(def.version(), &Version::new(1, 0, 0));
    assert!(def.input_schema().unwrap().matches::<String>());
}
```

---

## Quick Reference

### Mocks

```rust
// MockCommandExecutor
let mut exec = MockCommandExecutor::new();
exec.register(definition, handler)?;
exec.execute(&cmd, &engine_ref)?;
exec.assert_executed("cmd.id");
exec.assert_not_executed("cmd.missing");
exec.executed_count();

// MockEventBus
let bus = MockEventBus::new();
bus.publish(&event);
bus.assert_published("event.type");
bus.assert_not_published("event.missing");
bus.published_count();
bus.published_events();

// MockSignalBus
let bus = MockSignalBus::new();
bus.emit(&signal);
bus.assert_emitted("signal.type");
bus.emitted_count();
```

### TestEngine

```rust
let engine = TestEngine::new();
let engine_ref = engine.engine_ref();
// engine.command_executor — real CommandExecutor
// engine.task_manager — real TaskManager
// All other subsystems — real (empty)
```

### Assertion Helpers

```rust
assert_command_executed(&exec, "cmd.id");
assert_command_not_executed(&exec, "cmd.missing");
assert_event_published(&bus, "event.type");
assert_event_not_published(&bus, "event.missing");
assert_signal_emitted(&bus, "signal.type");
```

### Version

```rust
Version::new(major, minor, patch)
v.major() / v.minor() / v.patch()
v.to_string()                     // "1.2.3"
v.is_compatible_with(&other)      // Same major
v.is_at_least(&other)             // >=
v1 < v2                           // Ordering
Version::default()                // 1.0.0
```

### PayloadSchema

```rust
PayloadSchema::of::<T>()                          // Create
PayloadSchema::of_with_description::<T>("desc")    // With description
schema.matches::<T>()                              // Type check
schema.type_name()                                 // &'static str
schema.description()                               // &str
s1 == s2                                           // Same type
```

### Versioned Definitions

```rust
CommandDefinition::new(...)
    .with_version(Version::new(1, 0, 0))
    .with_input_schema(PayloadSchema::of::<InputType>())
    .with_output_schema(PayloadSchema::of::<OutputType>());

TaskDefinition::new(...)
    .with_version(Version::new(1, 0, 0))
    .with_input_schema(PayloadSchema::of::<InputType>())
    .with_output_schema(PayloadSchema::of::<OutputType>());

PluginManifest::new(...)
    .with_engine_api_version(Version::new(1, 0, 0));

def.is_compatible_with(&version);
```

---

## Next Steps

- [05-commands-and-tasks.md](05-commands-and-tasks.md) — Commands, tasks, and handler registration.
- [11-handler-capabilities.md](11-handler-capabilities.md) — What handlers can access via `EngineRef`.
- [17-plugin-security.md](17-plugin-security.md) — Plugin permissions and versioning.
- [09-domain-engine-integration.md](09-domain-engine-integration.md) — How domain code provides versioned definitions.