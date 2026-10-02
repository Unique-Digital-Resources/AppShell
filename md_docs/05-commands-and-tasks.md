# `05-commands-and-tasks.md` — Application Operations

## Overview

The App Engine provides two execution mechanisms:

```text
Command = an application intent — "do X"
Task    = a managed work instance — "X is being performed with lifecycle"
```

They are **not interchangeable**. Each serves a different purpose:

| | Command | Task |
|---|---|---|
| **Represents** | Intent / capability | Work instance |
| **Lifecycle** | Fire-and-forget | Pending → Running → Completed/Failed |
| **Has progress** | No | Yes |
| **Cancellable** | No (just don't execute) | Yes |
| **Pausable** | No | Yes (if supported) |
| **Scheduled** | No (directly) | Yes (via Scheduler) |
| **Good for** | Short, synchronous operations | Long, managed, observable operations |

---

## Two Execution Paths

### Simple Operation

For short, synchronous operations:

```text
Command
   ↓
CommandExecutor
   ↓
Domain Function (via CommandHandler)
   ↓
CommandResult
```

### Managed Operation

For work that needs lifecycle, progress, cancellation, or scheduling:

```text
Command
   ↓
Task (created by CommandHandler or domain code)
   ↓
TaskManager.start()
   ↓
TaskExecutor
   ↓
Domain Function (via TaskHandler)
   ↓
TaskResult
```

---

## Commands

### Defining a Command

A `Command` is an executable intent — it carries a definition ID, a context, and type-erased input:

```rust
use app_shell::app_engine::{
    Command, CommandContext, CommandInput, ExecutionId,
};

let exec_id = ExecutionId::new();
let context = CommandContext::new(exec_id)
    .with_application(app_id)
    .with_session(session_id);

let command = Command::new(
    "document.create",
    context,
    CommandInput::new("MyDocument".to_string()),
);
```

### Command Definition

Before a command can be executed, its definition must be registered. A `CommandDefinition` describes the command's metadata:

```rust
use app_shell::app_engine::CommandDefinition;

let definition = CommandDefinition::new(
    "document.create",           // id
    "Create Document",           // name
    "Creates a new document",    // description
).with_metadata("category", "file");
```

### Registering a Command Handler

Commands are executed through `CommandHandler` implementations. Domain code provides the handler:

```rust
use app_shell::app_engine::{CommandHandler, CommandInput, CommandContext, CommandResult, CommandError};

struct CreateDocumentHandler;

impl CommandHandler for CreateDocumentHandler {
    fn execute(
        &self,
        input: &CommandInput,
        _context: &CommandContext,
    ) -> Result<CommandResult, CommandError> {
        let name: &String = input.get::<String>()
            .ok_or_else(|| CommandError::InvalidArguments("expected document name".to_string()))?;

        // Call domain function
        let document_id = create_document_in_domain(name);

        Ok(CommandResult::success_with(document_id))
    }
}

// Register with the engine
engine.command_executor_mut().register(
    CommandDefinition::new("document.create", "Create Document", "Creates a new document"),
    Box::new(CreateDocumentHandler),
)?;
```

### Executing a Command

```rust
let command = Command::new(
    "document.create",
    CommandContext::new(ExecutionId::new()),
    CommandInput::new("MyDocument".to_string()),
);

let result = engine.command_executor().execute(&command)?;

assert!(result.is_success());
assert_eq!(result.output::<String>(), Some(&"doc_42".to_string()));
```

### Command Result

```rust
// Success with output
let result = CommandResult::success_with("doc_42".to_string());

// Success without output
let result = CommandResult::success();

// Failure
let result = CommandResult::failure();

// Failure with details
let result = CommandResult::failure_with("insufficient permissions".to_string());

// With metadata
let result = CommandResult::success()
    .with_metadata("duration_ms", "42")
    .with_metadata("trace_id", "abc123");

// Inspect
result.is_success();      // true
result.is_failure();     // false
result.has_output();     // true
result.output::<String>(); // Some("doc_42")
```

### Command Errors

```rust
match engine.command_executor().execute(&command) {
    Ok(result) => { /* handle result */ }
    Err(CommandError::UnknownCommand { id }) => {
        eprintln!("No command registered: {}", id);
    }
    Err(CommandError::InvalidArguments(msg)) => {
        eprintln!("Bad input: {}", msg);
    }
    Err(CommandError::ExecutionFailed(msg)) => {
        eprintln!("Handler failed: {}", msg);
    }
    Err(e) => eprintln!("Command error: {}", e),
}
```

### Type-Erased Input

`CommandInput` holds `Box<dyn Any + Send>`. The handler downcasts to its expected type:

```rust
// Pass a struct as input
#[derive(Debug)]
struct ExportInput {
    document_id: String,
    format: String,
    output_path: String,
}

let input = CommandInput::new(ExportInput {
    document_id: "doc_42".to_string(),
    format: "pdf".to_string(),
    output_path: "/tmp/export.pdf".to_string(),
});

// In the handler:
let export_input: &ExportInput = input.get::<ExportInput>()
    .ok_or_else(|| CommandError::InvalidArguments("expected ExportInput".to_string()))?;
```

The App Engine **never inspects** what's inside `CommandInput`. It just stores and forwards it.

---

## Tasks

### When to Use a Task Instead of a Command

| Situation | Use |
|---|---|
| Short, synchronous operation | Command |
| Needs progress tracking | Task |
| Needs cancellation | Task |
| Needs pause/resume | Task |
| Long-running (seconds+) | Task |
| Needs scheduling (delayed, periodic) | Task |
| Needs state (Pending, Running, Failed) | Task |
| Should appear in history as one operation | Task |

### Defining a Task Type

```rust
use app_shell::app_engine::{TaskDefinition, TaskHandler, TaskInput, TaskContext, TaskResult, TaskError, ExecutionId};

let definition = TaskDefinition::new(
    "project.export",                    // id
    "Export Project",                    // name
    "Exports the current project",       // description
)
.with_pause_support()
.with_cancel_support()
.with_metadata("category", "file");
```

### Registering a Task Handler

```rust
struct ExportProjectHandler;

impl TaskHandler for ExportProjectHandler {
    fn execute(
        &self,
        input: &TaskInput,
        _context: &TaskContext,
    ) -> Result<TaskResult, TaskError> {
        let input_data: &ExportInput = input.get::<ExportInput>()
            .ok_or_else(|| TaskError::ExecutionFailed {
                id: 0,
                reason: "expected ExportInput".to_string(),
            })?;

        // Call domain function
        let result = domain_engine.export_project(
            &input_data.document_id,
            &input_data.format,
            &input_data.output_path,
        )?;

        Ok(TaskResult::completed_with(result.output_path))
    }
}

// Register
engine.task_manager_mut().register(
    TaskDefinition::new("project.export", "Export Project", "Exports the project")
        .with_cancel_support(),
    Box::new(ExportProjectHandler),
)?;
```

### Creating a Task Instance

```rust
let exec_id = ExecutionId::new();
let task_context = TaskContext::new(exec_id)
    .with_application(app_id)
    .with_session(session_id);

let task_id = engine.task_manager_mut().create(
    "project.export",
    task_context,
    TaskInput::new(ExportInput {
        document_id: "doc_42".to_string(),
        format: "pdf".to_string(),
        output_path: "/tmp/export.pdf".to_string(),
    }),
)?;

// Task starts in Pending state
assert_eq!(
    engine.task_manager().get(task_id).unwrap().state(),
    TaskState::Pending,
);
```

### Starting a Task

```rust
let result = engine.task_manager_mut().start(task_id)?;

assert!(result.is_completed());
assert_eq!(result.output::<String>(), Some(&"/tmp/export.pdf".to_string()));

// Task state updated to Completed
assert_eq!(
    engine.task_manager().get(task_id).unwrap().state(),
    TaskState::Completed,
);
```

### Task Lifecycle States

```text
Pending
   ↓ start()
Running ──────────→ Completed (success)
   │           ────→ Failed (error)
   │           ────→ Cancelled
   │     ↕
   └──→ Paused
          ↓ resume()
        Running
```

```rust
use app_shell::app_engine::TaskState;

assert!(TaskState::Pending.can_start());
assert!(TaskState::Running.can_pause());
assert!(TaskState::Paused.can_resume());
assert!(TaskState::Running.can_cancel());
assert!(TaskState::Completed.is_terminal());
assert!(TaskState::Failed.is_terminal());
```

### Task Result

```rust
// Completed with output
let result = TaskResult::completed_with("/tmp/export.pdf".to_string());

// Completed without output
let result = TaskResult::completed();

// Cancelled
let result = TaskResult::cancelled();

// Failed with error message
let result = TaskResult::failed("disk full");

// Failed with partial output
let result = TaskResult::failed_with("timeout", partial_log_data);

// With metadata
let result = TaskResult::completed()
    .with_metadata("duration_ms", "4200")
    .with_metadata("file_size", "2.4MB");

// Inspect
result.is_completed();   // true
result.is_cancelled();   // false
result.is_failed();      // false
result.status();         // TaskResultStatus::Completed
result.output::<String>(); // Some("/tmp/export.pdf")
result.error();          // None
```

---

## Cancellation

Tasks that support cancellation can be cancelled before or during execution:

### Cancel Before Start

```rust
let definition = TaskDefinition::new("export", "Export", "Export")
    .with_cancel_support(); // ← required

engine.task_manager_mut().register(definition, Box::new(ExportProjectHandler))?;

let task_id = engine.task_manager_mut().create(
    "export",
    TaskContext::new(ExecutionId::new()),
    TaskInput::new(export_data),
)?;

// Cancel while still Pending
engine.task_manager_mut().cancel(task_id)?;

assert_eq!(
    engine.task_manager().get(task_id).unwrap().state(),
    TaskState::Cancelled,
);
```

### Cancel Non-Cancellable Task

If the task definition doesn't declare cancel support:

```rust
let definition = TaskDefinition::new("rigid", "Rigid", "Not cancellable");
// .with_cancel_support() NOT called

engine.task_manager_mut().register(definition, Box::new(SomeHandler))?;

let task_id = engine.task_manager_mut().create(
    "rigid",
    TaskContext::new(ExecutionId::new()),
    TaskInput::empty(),
)?;

let result = engine.task_manager_mut().cancel(task_id);
assert!(matches!(result, Err(TaskError::NotCancellable { .. })));
```

---

## Pause and Resume

Tasks that support pausing can be paused while running and resumed later:

```rust
let definition = TaskDefinition::new("render", "Render", "Render scene")
    .with_pause_support()
    .with_cancel_support();

engine.task_manager_mut().register(definition, Box::new(RenderHandler))?;

let task_id = engine.task_manager_mut().create(
    "render",
    TaskContext::new(ExecutionId::new()),
    TaskInput::new(scene_data),
)?;

// Start the task
engine.task_manager_mut().start(task_id)?;
assert_eq!(
    engine.task_manager().get(task_id).unwrap().state(),
    TaskState::Running,
);

// Can't pause in Pending state (before start)
// Can pause in Running state
engine.task_manager_mut().pause(task_id)?;
assert_eq!(
    engine.task_manager().get(task_id).unwrap().state(),
    TaskState::Paused,
);

// Resume
engine.task_manager_mut().resume(task_id)?;
assert_eq!(
    engine.task_manager().get(task_id).unwrap().state(),
    TaskState::Running,
);
```

### Capability Checks

```rust
let task = engine.task_manager().get(task_id).unwrap();

if task.supports_pause() {
    engine.task_manager_mut().pause(task_id)?;
}

if task.supports_cancel() {
    engine.task_manager_mut().cancel(task_id)?;
}
```

---

## When a Command Should Create a Task

The most powerful pattern is a **command that creates a task** — combining intent with managed execution:

```rust
struct StartExportCommand;

impl CommandHandler for StartExportCommand {
    fn execute(&self, input: &CommandInput, context: &CommandContext) -> Result<CommandResult, CommandError> {
        let export_input: &ExportInput = input.get::<ExportInput>()
            .ok_or_else(|| CommandError::InvalidArguments("expected ExportInput".to_string()))?;

        // Instead of exporting directly, create a task:
        // (The command handler would need access to the TaskManager)
        
        Ok(CommandResult::success_with("task_scheduled".to_string())
            .with_metadata("task_definition", "project.export"))
    }
}
```

### Decision Flowchart

```text
Is the operation short and synchronous?
    ├── Yes → Command (direct execution)
    └── No
        │
        Does it need progress tracking?
        ├── Yes → Task
        └── No
            │
            Does it need cancellation?
            ├── Yes → Task
            └── No
                │
                Does it need scheduling (delayed/periodic)?
                ├── Yes → Task + Scheduler
                └── No
                    │
                    Does it need pause/resume?
                    ├── Yes → Task
                    └── No → Command (sufficient)
```

### Examples

| Operation | Mechanism | Why |
|---|---|---|
| Set zoom level | Command | Instant, no lifecycle needed |
| Change color | Command | Instant |
| Export project | Task | Long-running, cancellable, has progress |
| Render scene | Task | Long-running, pausable |
| Autosave | Task + Scheduler | Periodic, scheduled |
| Load font | Resource (via loader) | Not a command or task |
| Undo last operation | Command → History | Triggers history undo, then re-executes |

---

## Managing Multiple Tasks

```rust
// Create multiple tasks
let task_a = engine.task_manager_mut().create(
    "project.export",
    TaskContext::new(ExecutionId::new()),
    TaskInput::new(export_a),
)?;

let task_b = engine.task_manager_mut().create(
    "project.export",
    TaskContext::new(ExecutionId::new()),
    TaskInput::new(export_b),
)?;

let task_c = engine.task_manager_mut().create(
    "render.scene",
    TaskContext::new(ExecutionId::new()),
    TaskInput::new(render_data),
)?;

assert_eq!(engine.task_manager().count(), 3);

// Start them
engine.task_manager_mut().start(task_a)?;
engine.task_manager_mut().start(task_b)?;
engine.task_manager_mut().start(task_c)?;

// Verify states
for id in engine.task_manager().list().iter().map(|t| t.id()) {
    let task = engine.task_manager().get(id).unwrap();
    assert!(task.is_terminal()); // Completed, Failed, or Cancelled
}

// Remove completed tasks
engine.task_manager_mut().remove(task_a);
assert_eq!(engine.task_manager().count(), 2);
```

---

## Error Handling

### Command Errors

```rust
// Unknown command
let cmd = Command::with_empty_input("unknown.cmd", context);
match engine.command_executor().execute(&cmd) {
    Err(CommandError::UnknownCommand { id }) => {
        eprintln!("No handler for: {}", id);
    }
    _ => {}
}

// Invalid input (handler rejects)
let cmd = Command::new("document.create", context, CommandInput::new(42_i64)); // wrong type
match engine.command_executor().execute(&cmd) {
    Err(CommandError::InvalidArguments(msg)) => {
        eprintln!("Handler rejected input: {}", msg);
    }
    _ => {}
}
```

### Task Errors

```rust
// Task not found
match engine.task_manager_mut().start(TaskId::new()) { // bogus id
    Err(TaskError::NotFound { id }) => {
        eprintln!("Task {} not found", id);
    }
    _ => {}
}

// Invalid state transition
engine.task_manager_mut().start(task_id)?; // → Completed
match engine.task_manager_mut().start(task_id) { // can't start completed task
    Err(TaskError::InvalidState { current, requested, .. }) => {
        eprintln!("Can't start: {} → {}", current, requested);
    }
    _ => {}
}

// Handler failure
struct FailingHandler;
impl TaskHandler for FailingHandler {
    fn execute(&self, _: &TaskInput, _: &TaskContext) -> Result<TaskResult, TaskError> {
        Err(TaskError::ExecutionFailed { id: 0, reason: "disk full".into() })
    }
}

// When start() fails, the task transitions to Failed
let result = engine.task_manager_mut().start(failing_task_id);
assert!(result.is_err());
assert_eq!(
    engine.task_manager().get(failing_task_id).unwrap().state(),
    TaskState::Failed,
);
```

---

## Complete Example: Command → Task → Result

```rust
use app_shell::app_engine::*;

// --- Domain data types ---

#[derive(Debug)]
struct ExportInput {
    project_path: String,
    format: String,
    output_path: String,
}

// --- Task handler (executes the actual work) ---

struct ExportProjectHandler;

impl TaskHandler for ExportProjectHandler {
    fn execute(
        &self,
        input: &TaskInput,
        _ctx: &TaskContext,
    ) -> Result<TaskResult, TaskError> {
        let input: &ExportInput = input.get::<ExportInput>()
            .ok_or_else(|| TaskError::ExecutionFailed {
                id: 0, reason: "expected ExportInput".into()
            })?;

        // Domain function call
        let result = domain_export_project(&input.project_path, &input.format, &input.output_path);

        match result {
            Ok(path) => Ok(TaskResult::completed_with(path)
                .with_metadata("format", &input.format)),
            Err(e) => Err(TaskError::ExecutionFailed {
                id: 0, reason: e.to_string()
            }),
        }
    }
}

// --- Setup ---

fn setup_engine() -> AppEngine {
    let mut engine = Bootstrap::create().unwrap();

    // Register the task type
    engine.task_manager_mut().register(
        TaskDefinition::new("project.export", "Export Project", "Export the project")
            .with_cancel_support(),
        Box::new(ExportProjectHandler),
    ).unwrap();

    engine
}

// --- Usage ---

fn main() {
    let mut engine = setup_engine();
    engine.initialize().unwrap();
    engine.start().unwrap();

    // Create a task
    let task_id = engine.task_manager_mut().create(
        "project.export",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new(ExportInput {
            project_path: "/projects/demo.proj".to_string(),
            format: "pdf".to_string(),
            output_path: "/tmp/demo.pdf".to_string(),
        }),
    ).unwrap();

    // Execute
    let result = engine.task_manager_mut().start(task_id).unwrap();

    assert!(result.is_completed());
    assert_eq!(result.output::<String>(), Some(&"/tmp/demo.pdf".to_string()));

    engine.stop().unwrap();
    engine.dispose().unwrap();
}
```

---

## Quick Reference

### Command API

```rust
// Register
engine.command_executor_mut().register(definition, handler)?;

// Execute
let result: CommandResult = engine.command_executor().execute(&command)?;

// Query
engine.command_executor().contains(&definition_id);  // bool
engine.command_executor().get(&definition_id);        // Option<&CommandDefinition>
engine.command_executor().list();                    // Vec<&CommandDefinition>

// Unregister
engine.command_executor_mut().unregister(&definition_id); // Option<CommandDefinition>
```

### Task API

```rust
// Register
engine.task_manager_mut().register(definition, handler)?;

// Create
let task_id = engine.task_manager_mut().create(definition_id, context, input)?;

// Lifecycle
engine.task_manager_mut().start(task_id)?;     // Pending → Running → Completed/Failed
engine.task_manager_mut().cancel(task_id)?;    // → Cancelled
engine.task_manager_mut().pause(task_id)?;     // Running → Paused
engine.task_manager_mut().resume(task_id)?;    // Paused → Running

// Query
engine.task_manager().get(task_id);            // Option<&Task>
engine.task_manager().count();                 // usize
engine.task_manager().list();                  // Vec<&Task>

// Remove
engine.task_manager_mut().remove(task_id);    // Option<Task>
```

---

## Next Steps

- [06-scheduling-and-communication.md](06-scheduling-and-communication.md) — Scheduling tasks and decoupled communication.
- [07-history-and-resources.md](07-history-and-resources.md) — Undo/redo and resource management.
- [09-domain-engine-integration.md](09-domain-engine-integration.md) — How domain functions become commands and tasks.