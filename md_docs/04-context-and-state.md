# `04-context-and-state.md` — Context & State

## Overview

Phase 2 of the App Engine introduced two foundational systems that every other subsystem builds on:

```text
Runtime
   │
   ├── ContextManager  ──→ Context (identity + environment)
   │
   └── StateManager    ──→ State (current condition)
```

These are **deliberately separate**:

```text
Context = "Where am I executing?"
State   = "What is the current condition?"
```

Context describes the **environment** in which something is executing. State describes the **condition** of that environment. They are not the same thing, and merging them causes architectural confusion.

---

## Context Hierarchy

Contexts form a parent-child hierarchy:

```text
ApplicationContext
       │
       └── SessionContext
                │
                └── ExecutionContext
```

| Context | Lifetime | Answers |
|---|---|---|
| `ApplicationContext` | Entire application run | "Which application instance is this?" |
| `SessionContext` | A logical session within the app | "Which session/user/workspace is this?" |
| `ExecutionContext` | One particular operation | "Which execution is this?" |

A session belongs to an application. An execution belongs to a session. This hierarchy is enforced by the `ContextManager`.

---

## Creating Contexts

```rust
use app_shell::app_engine::{
    AppEngine, Bootstrap, Lifecycle,
    ApplicationId, SessionId, ExecutionId,
};

let mut engine = Bootstrap::create()?;

// ApplicationContext is created automatically by Bootstrap
let app_id = engine.context().id();
println!("Application: {}", app_id.value());

// Create a session under the application
let session_id = engine
    .context_manager_mut()
    .create_session_context()?;

// Create an execution under the session
let exec_id = engine
    .context_manager_mut()
    .create_execution_context(session_id)?;

// Verify hierarchy
let session = engine.context_manager().session_context(session_id).unwrap();
let execution = engine.context_manager().execution_context(exec_id).unwrap();

assert_eq!(session.application_id(), app_id);   // session → app
assert_eq!(execution.session_id(), session_id); // execution → session
```

### Multiple Sessions

An application can have multiple sessions:

```rust
let session_a = engine.context_manager_mut().create_session_context()?;
let session_b = engine.context_manager_mut().create_session_context()?;
let session_c = engine.context_manager_mut().create_session_context()?;

assert_eq!(engine.context_manager().session_count(), 3);
assert_ne!(session_a, session_b);
assert_ne!(session_b, session_c);
```

Each session could represent a different user, workspace, CLI invocation, or UI window — the App Engine doesn't need to know which.

### Multiple Executions Per Session

```rust
let session = engine.context_manager_mut().create_session_context()?;

let exec_1 = engine.context_manager_mut().create_execution_context(session)?;
let exec_2 = engine.context_manager_mut().create_execution_context(session)?;
let exec_3 = engine.context_manager_mut().create_execution_context(session)?;

assert_eq!(engine.context_manager().execution_count(), 3);

// All executions belong to the same session
let e1 = engine.context_manager().execution_context(exec_1).unwrap();
let e2 = engine.context_manager().execution_context(exec_2).unwrap();
assert_eq!(e1.session_id(), session);
assert_eq!(e2.session_id(), session);
```

### Error: Execution Without Session

```rust
use app_shell::app_engine::ContextError;

// Can't create a session without an application context
let mut mgr = ContextManager::new();
let result = mgr.create_session_context();
assert!(matches!(result, Err(ContextError::NoApplicationContext)));

// Can't create an execution without a valid session
let mut mgr = ContextManager::new();
mgr.create_application_context("app");
let result = mgr.create_execution_context(SessionId::new()); // bogus session
assert!(matches!(result, Err(ContextError::SessionNotFound)));
```

---

## Context Identity

Each context has a typed, stable, copyable ID:

```rust
let app_id: ApplicationId = engine.context().id();
let session_id: SessionId = engine.context_manager_mut().create_session_context()?;
let exec_id: ExecutionId = engine.context_manager_mut().create_execution_context(session_id)?;

// IDs are Copy — pass them around freely
let copied = app_id;
assert_eq!(app_id, copied);

// IDs are unique
let another_app = ApplicationId::new();
assert_ne!(app_id, another_app);

// IDs are hashable (usable as HashMap keys)
let mut map = std::collections::HashMap::new();
map.insert(app_id, "root");
```

IDs are **not** sequential or predictable — they are auto-generated from a global counter. This means they remain stable across reboots of the counter (they're always unique within a process).

---

## Specialized Contexts

Beyond the core hierarchy, other subsystems define their own context types that **reference** the core context IDs:

```text
ApplicationContext
       │
       └── SessionContext
                │
                └── ExecutionContext
                         │
                ┌────────┼────────┐
                ▼        ▼        ▼
          TaskContext  CommandContext  PluginContext
```

### TaskContext

```rust
use app_shell::app_engine::{ExecutionId, TaskContext, TaskId};

let exec_id = ExecutionId::new();
let task_context = TaskContext::new(exec_id)
    .with_application(app_id)
    .with_session(session_id)
    .with_metadata("source", "scheduler");

// The TaskManager sets the task_id automatically on the context
let task_id = engine.task_manager_mut().create(
    "export.project",
    task_context,
    TaskInput::new("project.file".to_string()),
)?;

let task = engine.task_manager().get(task_id).unwrap();
assert_eq!(task.context().execution_id(), exec_id);
assert_eq!(task.context().task_id(), Some(task_id));  // set by manager
```

### CommandContext

```rust
use app_shell::app_engine::{CommandContext, Command, CommandInput, ExecutionId};

let exec_id = ExecutionId::new();
let cmd_context = CommandContext::new(exec_id)
    .with_application(app_id)
    .with_session(session_id)
    .with_metadata("invoked_by", "cli");

let command = Command::new(
    "app.quit",
    cmd_context,
    CommandInput::empty(),
);

// The executor receives the context when executing
let result = engine.command_executor().execute(&command)?;
```

### PluginContext

```rust
// PluginContext is created by the PluginManager during initialize()
// The plugin receives it:

impl Plugin for MyPlugin {
    fn initialize(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        println!("Plugin '{}' initializing", ctx.plugin_id());
        Ok(())
    }
    // ...
}
```

`PluginContext` is intentionally minimal — it provides the plugin's identity and engine metadata, not unlimited access to the runtime. In later extensions, it can be expanded to provide controlled access to commands, events, resources, etc.

---

## What Context Should NOT Contain

```text
✓ execution_id, session_id, application_id
✓ cancellation flags (future)
✓ deadline/priority metadata (future)
✓ generic key-value metadata

✗ current document
✗ selected card
✗ active scene
✗ domain objects
✗ UI state
```

If you need to pass domain objects to a task, the domain code should resolve them through its own mechanisms — not stuff them into `TaskContext`.

---

## State System

While Context describes *where*, State describes *what is happening*.

```rust
use app_shell::app_engine::{StateId, StateStore, StateManager, StateTransition, LifecycleState};

// The runtime mirrors its lifecycle state into the StateManager
let runtime_state_id = engine.runtime_state_id();

// Read current state
let current = engine.state_manager().current(runtime_state_id);
assert_eq!(current, Some(LifecycleState::Created));

// The lifecycle updates this automatically:
engine.initialize()?;
assert_eq!(
    engine.state_manager().current(runtime_state_id),
    Some(LifecycleState::Initialized)
);
```

---

## StateStore

`StateStore<S>` is a generic in-memory key-value state store:

```rust
use app_shell::app_engine::{StateId, StateStore, LifecycleState};

let store: StateStore<LifecycleState> = StateStore::new();
let id = StateId::new(1);

// Write
store.write(id, LifecycleState::Running);

// Read
assert_eq!(store.read(id), Some(LifecycleState::Running));

// Check
assert!(store.contains(id));

// Remove
let removed = store.remove(id);
assert_eq!(removed, Some(LifecycleState::Running));
assert!(!store.contains(id));
```

Multiple independent state slots:

```rust
let store: StateStore<LifecycleState> = StateStore::new();

store.write(StateId::new(1), LifecycleState::Created);
store.write(StateId::new(2), LifecycleState::Running);
store.write(StateId::new(3), LifecycleState::Stopped);

assert_eq!(store.len(), 3);
assert_eq!(store.read(StateId::new(2)), Some(LifecycleState::Running));
```

---

## State Transitions

A `StateTransition<S>` represents a change from one state to another:

```rust
use app_shell::app_engine::{StateTransition, LifecycleState};

let transition = StateTransition::new(
    LifecycleState::Created,
    LifecycleState::Initialized,
);

assert_eq!(transition.from(), &LifecycleState::Created);
assert_eq!(transition.to(), &LifecycleState::Initialized);
assert!(!transition.is_no_op());

// A no-op transition
let noop = StateTransition::new(LifecycleState::Running, LifecycleState::Running);
assert!(noop.is_no_op());
```

---

## StateManager

`StateManager<S>` sits above `StateStore` and validates transitions:

```rust
use app_shell::app_engine::{StateManager, StateId, StateTransition, LifecycleState, StateError};

let mgr: StateManager<LifecycleState> = StateManager::new();
let id = StateId::new(1);

// Set initial state
mgr.set(id, LifecycleState::Created);

// Valid transition: Created → Initialized
mgr.transition(id, LifecycleState::Created, LifecycleState::Initialized)?;
assert_eq!(mgr.current(id), Some(LifecycleState::Initialized));

// Invalid: from doesn't match current
let result = mgr.transition(id, LifecycleState::Created, LifecycleState::Running);
assert!(matches!(result, Err(StateError::TransitionMismatch { .. })));

// State is unchanged after failed transition
assert_eq!(mgr.current(id), Some(LifecycleState::Initialized));
```

### Apply a StateTransition object

```rust
let transition = StateTransition::new(LifecycleState::Initialized, LifecycleState::Running);
mgr.apply_transition(id, &transition)?;
assert_eq!(mgr.current(id), Some(LifecycleState::Running));
```

### Error: State Not Found

```rust
let result = mgr.apply_transition(StateId::new(999), &transition);
assert!(matches!(result, Err(StateError::StateNotFound)));
```

---

## Context vs State

This distinction is critical:

```text
Context                          State
───────                          ─────
"Execution #42 belongs           "Execution #42 is
 to session #3"                   currently running"

ApplicationContext                RuntimeState = Running
SessionContext                   TaskState = Pending
ExecutionContext                 JobState = Queued
TaskContext                      LifecycleState = Initialized
CommandContext                   ResourceState = Loaded
PluginContext                    ServiceState = Started
```

### When to use Context

Use Context when you need to:

- **Identify** where an operation is happening (which session, which execution).
- **Track parent relationships** (this execution belongs to this session).
- **Pass execution metadata** to handlers (source, priority hints).
- **Create isolated execution scopes** (different sessions for different users).

### When to use State

Use State when you need to:

- **Track the current condition** of something (is the runtime running? is the task pending?).
- **Validate transitions** (can a completed task be restarted?).
- **Store generic runtime values** that systems need to query.
- **Observe lifecycle changes** (the StateManager mirrors runtime lifecycle).

### When NOT to use either

| Don't put in Context | Don't put in State |
|---|---|
| Current document | "The document has 37 layers" |
| Selected card | "The selected card is Ace of Spades" |
| Active scene | "The scene contains 1200 objects" |
| UI window reference | "The window is 1920×1080" |

Those are **Domain State** — they belong in the Domain Engine.

---

## Domain State Boundary

```text
✓ App Engine State
    "Task #42 is running"
    "Resource #7 is loaded"
    "Service #3 is started"
    "Runtime is in Running state"

✗ Domain State
    "Document Foo has 37 shapes"
    "Selected object is Rectangle"
    "Card database has 1200 entries"
    "Scene contains 5 lights"
```

The App Engine's `StateManager` can store **generic** state values (any type `S: Clone + PartialEq + Debug`). It can store `LifecycleState`, `TaskState`, `ResourceState`, etc. — but it should never store domain-specific data like "which document is open."

If a task needs to know "which document to export," that information comes through `TaskInput` (the task's data payload), not through `TaskContext` or `StateManager`:

```rust
// Good: domain data in TaskInput
let task_id = engine.task_manager_mut().create(
    "export.project",
    TaskContext::new(exec_id)
        .with_application(app_id)
        .with_session(session_id),
    TaskInput::new(ExportInput {
        document_path: "/docs/my_project.dp".to_string(),
        output_format: "pdf".to_string(),
    }),
)?;

// Bad: domain data in TaskContext
let context = TaskContext::new(exec_id)
    .with_metadata("document", "/docs/my_project.dp");  // ← domain leak!
    .with_metadata("format", "pdf");                      // ← domain leak!
```

---

## Integration with Runtime

The runtime owns both `ContextManager` and `StateManager`:

```rust
let mut engine = Bootstrap::create()?;

// Context: create sessions and executions
let session_id = engine.context_manager_mut().create_session_context()?;
let exec_id = engine.context_manager_mut().create_execution_context(session_id)?;

// State: the runtime's own lifecycle is mirrored
let state_id = engine.runtime_state_id();
engine.initialize()?;
assert_eq!(
    engine.state_manager().current(state_id),
    Some(LifecycleState::Initialized)
);
engine.start()?;
assert_eq!(
    engine.state_manager().current(state_id),
    Some(LifecycleState::Running)
);
```

---

## Quick Reference

### Context API

```rust
// Create
let app_id = engine.context().id();
let session_id = engine.context_manager_mut().create_session_context()?;
let exec_id = engine.context_manager_mut().create_execution_context(session_id)?;

// Query
engine.context_manager().application_context()           // → &ApplicationContext
engine.context_manager().session_context(session_id)      // → Option<&SessionContext>
engine.context_manager().execution_context(exec_id)      // → Option<&ExecutionContext>
engine.context_manager().session_count()                 // → usize
engine.context_manager().execution_count()               // → usize
```

### State API

```rust
// Store
let store: StateStore<LifecycleState> = StateStore::new();
store.write(StateId::new(1), LifecycleState::Running);
store.read(StateId::new(1));          // → Option<LifecycleState>
store.contains(StateId::new(1));      // → bool
store.remove(StateId::new(1));        // → Option<LifecycleState>

// Manager
let mgr: StateManager<LifecycleState> = StateManager::new();
mgr.set(StateId::new(1), LifecycleState::Created);
mgr.current(StateId::new(1));         // → Option<LifecycleState>
mgr.transition(id, from, to)?;        // validates + applies
mgr.apply_transition(id, &transition)?;

// Transition
let t = StateTransition::new(from, to);
t.from();   // → &S
t.to();     // → &S
t.is_no_op(); // → bool
```

---

## Next Steps

- [05-commands-and-tasks.md](05-commands-and-tasks.md) — Registering and executing work.
- [06-scheduling-and-communication.md](06-scheduling-and-communication.md) — Events, signals, and scheduling.
- [08-services-and-plugins.md](08-services-and-plugins.md) — How services and plugins use context.