# `02-architecture.md` — Architecture & Boundaries

## Architecture Overview

The App Engine is designed around a strict layering principle: each layer knows the layer below it, never the layer above.

```text
                 ┌─────────────────────────────┐
                 │        External Layer        │
                 │   UI / API / MCP / CLI       │
                 └──────────────┬──────────────┘
                                │
                         uses contracts
                                │
                 ┌──────────────▼──────────────┐
                 │        Integration Layer      │
                 │   (adapts engines to clients) │
                 └──────────────┬──────────────┘
                                │
                          uses commands
                                │
          ┌─────────────────────▼──────────────────────┐
          │              DOMAIN ENGINE                  │
          │     (application meaning and behavior)      │
          │                                             │
          │   ExportProject   CreateCard   RenderScene │
          └─────────────────────┬──────────────────────┘
                                │
                            uses contracts
                                │
          ┌─────────────────────▼──────────────────────┐
          │              APP ENGINE                     │
          │       (generic runtime infrastructure)       │
          │                                             │
          │  Runtime  Commands  Tasks  Scheduler        │
          │  Events   Signals   History  Resources      │
          │  Services Plugins   Configuration           │
          └─────────────────────────────────────────────┘
```

The dependency arrows always point downward. The App Engine never imports from the Domain Engine, the Integration Layer, or the UI.

---

## Project Structure

A real application built around the App Engine looks like:

```text
app/
├── app_engine/          ← generic runtime infrastructure
│   ├── runtime/
│   ├── lifecycle/
│   ├── context/
│   ├── state/
│   ├── commands/
│   ├── tasks/
│   ├── scheduling/
│   ├── events/
│   ├── signals/
│   ├── history/
│   ├── resources/
│   ├── services/
│   ├── plugins/
│   ├── configuration/
│   └── mod.rs            ← public boundary (re-exports)
│
├── domain_engine/        ← application meaning and behavior
│   ├── project/
│   │   ├── export_project.rs
│   │   └── export_command.rs
│   ├── document/
│   │   ├── create_document.rs
│   │   └── document_model.rs
│   └── commands/         ← domain commands registered with App Engine
│       └── mod.rs
│
├── api/                  ← external access layer
│   ├── local/            ← local library/CLI access
│   ├── web/              ← HTTP/WebSocket
│   └── mcp/              ← AI agent integration
│
└── ui/                   ← presentation
    ├── desktop/          ← native UI (Qt, GTK, etc.)
    ├── web/              ← web frontend
    └── terminal/         ← TUI
```

Each layer has a single, clear responsibility.

---

## Dependency Direction

The most important architectural rule:

```text
Domain Engine ──→ App Engine
App Engine     ✗→ Domain Engine
App Engine     ✗→ UI
App Engine     ✗→ API / MCP
```

This means:

- The Domain Engine can import `AppEngine`, `TaskManager`, `CommandExecutor`, `EventBus`, etc.
- The App Engine **never** imports `ExportProject`, `Document`, `Card`, `Scene`, or any domain type.
- The App Engine **never** imports `Window`, `Widget`, `Route`, or `McpTool`.

If you find yourself adding a domain type to the App Engine, **stop**. It belongs in the Domain Engine.

---

## Composition Root

`AppEngine` is the **composition root** — the single object that owns and exposes all subsystems.

```rust
use app_shell::app_engine::{AppEngine, Bootstrap};

let mut engine: AppEngine = Bootstrap::create().expect("bootstrap failed");
```

`AppEngine` owns:

```text
AppEngine
├── context_manager         (Context)
├── state_manager           (State)
├── command_executor        (Commands)
├── task_manager            (Tasks)
├── scheduler               (Scheduling)
├── event_bus               (Events)
├── signal_bus              (Signals)
├── history_store           (History)
├── resource_manager        (Resources)
├── service_manager         (Services)
├── plugin_manager          (Plugins)
├── config_store            (Configuration)
└── preferences             (User Preferences)
```

`AppEngine` is **not a god object**. It does not implement domain logic, execute tasks, or handle UI events. It **composes and exposes** subsystems; each subsystem owns its own behavior.

---

## Public vs Internal API

### Public API

Accessible through `app_shell::app_engine::*` — these are the types consumers depend on:

```rust
// Core
AppEngine, Bootstrap, Lifecycle, RuntimeState

// Context
ApplicationContext, SessionContext, ExecutionContext, ContextManager

// State
StateId, StateManager, StateStore, StateTransition

// Commands
Command, CommandDefinition, CommandExecutor, CommandHandler, CommandInput, CommandResult

// Tasks
Task, TaskDefinition, TaskManager, TaskHandler, TaskInput, TaskResult, TaskState

// Scheduling
Schedule, Scheduler, Job, JobQueue, Trigger

// Communication
Event, EventBus, EventHandler, Signal, SignalBus, Subscription

// History
HistoryEntry, HistoryStore, UndoResult, RedoResult, TransactionManager, Replay

// Resources
Resource, ResourceId, ResourceHandle, ResourceLoader, ResourceManager, ResourceCache

// Services
Service, ServiceDefinition, ServiceManager, ServiceState

// Plugins
Plugin, PluginManifest, PluginManager, PluginContext, PluginLoader

// Configuration
Config, ConfigSchema, ConfigStore, Preferences
```

### Internal API

Not re-exported — these are implementation details that can change without breaking consumers:

```text
- EventDispatcher (internal routing — use EventBus instead)
- ServiceEntry (internal wrapper — use ServiceManager instead)
- PluginEntry (internal wrapper — use PluginManager instead)
- JobQueue internals (use Scheduler instead)
- StateStore internals (use StateManager instead)
- Bootstrap wiring details
```

Consumers should never need to reach into internal modules. If they do, the public API needs improvement.

---

## Subsystem Boundaries

Each subsystem is a self-contained module with clear boundaries:

```text
┌─────────────┬───────────────────────┬──────────────────────────────┐
│ Subsystem   │ What it owns          │ What it does NOT do         │
├─────────────┼───────────────────────┼──────────────────────────────┤
│ Runtime     │ Lifetime, state       │ Domain logic, execution     │
│ Lifecycle   │ Transition rules      │ Side effects, domain state  │
│ Context     │ Identity hierarchy    │ Domain objects              │
│ State       │ Generic state store   │ Domain data                 │
│ Commands    │ Intent + execution    │ What commands mean          │
│ Tasks       │ Work lifecycle        │ What tasks compute          │
│ Scheduler   │ When work runs        │ How work executes           │
│ Events      │ Pub/sub notification  │ Consumer behavior           │
│ Signals     │ Change notification   │ Consumer behavior           │
│ History     │ Operation record      │ What operations mean        │
│ Resources   │ Load/cache/unload     │ What resources contain      │
│ Services    │ Long-lived lifecycle  │ Service-specific behavior   │
│ Plugins     │ Extension lifecycle   │ Plugin implementations      │
│ Config      │ Settings storage      │ What settings mean          │
└─────────────┴───────────────────────┴──────────────────────────────┘
```

---

## What Belongs Where

### In App Engine (generic)

```text
✓ "Run this command."
✓ "Schedule this task for 10 minutes from now."
✓ "Publish TaskCompleted."
✓ "Undo the last 3 operations."
✓ "Load resource from file:///assets/image.png."
✓ "Start the autosave service."
✓ "Register the image plugin."
✓ "Save configuration: worker_count = 4."
```

### In Domain Engine (semantic)

```text
✓ "Export the project to PDF."
✓ "Create a card with these stats."
✓ "Render the scene at 1920x1080."
✓ "The document has 37 layers."
✓ "The selected object is a Rectangle."
✓ "Card database contains 1200 entries."
```

### In UI Layer (presentation)

```text
✓ "Show the toolbar."
✓ "Display the progress bar at 50%."
✓ "User clicked the Export button."
✓ "Theme is dark mode."
✓ "Canvas is 1920x1080."
```

### In API/MCP Layer (external access)

```text
✓ "POST /api/export → ExportProjectCommand"
✓ "MCP tool: create_card(name, attack, defense)"
✓ "WebSocket: subscribe to TaskProgressChanged"
✓ "CLI: --worker-count 8"
```

---

## Avoiding Accidental Coupling

### Bad: Direct dependency

```rust
// DON'T: TaskExecutor directly depends on HistoryStore
impl TaskExecutor {
    fn execute(&self, task: &Task) {
        // ... execute task ...
        self.history_store.append(entry);  // ← coupling!
    }
}
```

### Good: Decoupled through events

```rust
// DO: TaskExecutor publishes an event; History subscribes
impl TaskExecutor {
    fn execute(&self, task: &Task) {
        // ... execute task ...
        self.event_bus.publish(&Event::new("task.completed"));
    }
}

// History is a separate consumer:
event_bus.subscribe("task.completed", Box::new(history_handler));
```

### Rules to prevent coupling

1. **Systems never import each other directly.** Use Events/Signals/Commands.
2. **No domain types in App Engine.** If you need `Document`, it's in the Domain Engine.
3. **No UI types in App Engine.** If you need `Window`, it's in the UI Layer.
4. **No API types in App Engine.** If you need `Route`, it's in the API Layer.
5. **Configuration ≠ State.** Don't store runtime state in `ConfigStore`.
6. **History ≠ Telemetry.** Don't record every internal function call in `HistoryStore`.

---

## Communication Topology

Systems communicate through established contracts, not direct references:

```text
                    ┌──────────────┐
                    │  CommandExecutor
                    └──────┬───────┘
                           │ executes
                           ▼
                    ┌──────────────┐
                    │  TaskManager  │
                    └──────┬───────┘
                           │ schedules
                           ▼
                    ┌──────────────┐
                    │   Scheduler   │
                    └──────┬───────┘
                           │ dispatches
                           ▼
                    ┌──────────────┐
                    │ TaskExecutor  │
                    └──────┬───────┘
                           │
                    ┌──────┴───────┐
                    │              │
                    ▼              ▼
              ┌──────────┐  ┌──────────┐
              │ EventBus │  │SignalBus │
              └────┬─────┘  └────┬─────┘
                   │             │
        ┌──────────┼─────────────┘
        │          │
        ▼          ▼
   ┌────────┐ ┌──────────┐
   │ History│ │ Plugins  │
   └────────┘ └──────────┘
```

Key principle: **the producer announces that something happened; it does not need to know who reacts to it.**

---

## Lifecycle Integration

All long-lived systems start and stop with the engine:

```text
Startup:
  Create → Initialize → Services → Plugins → Start → Running

Shutdown (reverse):
  Running → Stop Plugins → Stop Services → Dispose → Disposed
```

```rust
let mut engine = Bootstrap::create()?;

// Services and plugins are initialized/started automatically:
engine.initialize()?;  // initializes all services, then plugins
engine.start()?;       // starts all services, then plugins

// ... application runs ...

engine.stop()?;        // stops plugins first, then services
engine.dispose()?;     // disposes plugins first, then services
```

The ordering is intentional: services start before plugins (plugins may depend on services), and plugins stop before services on shutdown.

---

## Error Ownership

Each subsystem owns its errors:

```text
CommandExecutor   → CommandError
TaskManager       → TaskError
ResourceManager  → ResourceError
ServiceManager    → ServiceError
PluginManager     → PluginError
ConfigStore       → ConfigurationError
Scheduler         → SchedulingError
HistoryStore      → HistoryError
```

`EngineError` unifies them for top-level error handling:

```rust
fn do_work(engine: &mut AppEngine) -> Result<(), EngineError> {
    engine.service_manager_mut().start_all()?;  // ServiceError → EngineError
    engine.plugin_manager_mut().start_all()?;   // PluginError → EngineError
    Ok(())
}
```

This gives callers both specific error information and a single top-level boundary.

---

## Domain Engine Boundary

The final boundary is explicit:

```text
              DOMAIN ENGINE
             (application meaning)
                    │
                    │ uses contracts
                    ▼
              APP ENGINE
            (generic mechanisms)
                    ▲
                    │
              Integration Layer
             (API / MCP / UI)
```

The App Engine should **never** contain:

```text
✗ Project    ✗ Document    ✗ Scene
✗ Card       ✗ Layer       ✗ Node
✗ CADObject  ✗ ImageDocument
✗ Window     ✗ Widget      ✗ Theme
✗ Route      ✗ Endpoint   ✗ McpTool
```

If any of these appear in App Engine code, they need to move to the appropriate layer.

---

## Checklist: Am I Doing It Right?

| Question | Answer should be |
|---|---|
| Does App Engine import any domain type? | No |
| Does App Engine import any UI type? | No |
| Does App Engine import any API type? | No |
| Do subsystems import each other directly? | No (use events/signals) |
| Is `ConfigStore` used to store runtime state? | No (use `StateManager`) |
| Is `HistoryStore` recording every function call? | No (only meaningful operations) |
| Does `TaskExecutor` know about `HistoryStore`? | No (publish events instead) |
| Are plugins defining App Engine contracts? | No (they consume them) |
| Is `AppEngine` a god object with hundreds of methods? | No (it composes and exposes) |

If any answer is wrong, revisit the architecture. The App Engine's value comes from its **clean boundaries** — once those are violated, the entire layering benefit erodes.

---

## Next Steps

- [03-bootstrap-and-lifecycle.md](03-bootstrap-and-lifecycle.md) — Creating and running an engine.
- [09-domain-engine-integration.md](09-domain-engine-integration.md) — How the Domain Engine plugs in.
- [10-application-integration.md](10-application-integration.md) — UI, API, and MCP integration.