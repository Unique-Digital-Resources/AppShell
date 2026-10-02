# `01-overview.md` — What App Engine Is

## Overview

The **App Engine** is a generic application runtime infrastructure library. It provides the mechanisms that most applications need — lifecycle, commands, tasks, scheduling, events, signals, history, resources, services, plugins, and configuration — without knowing what the application actually *does*.

```text
┌─────────────────────────────────────────────────┐
│                  APP ENGINE                     │
│                                                 │
│  Execution       Communication     Extension    │
│  ├─ Commands     ├─ Events         ├─ Plugins   │
│  ├─ Tasks        ├─ Signals        └─ Services  │
│  ├─ Scheduler    └─ History                    │
│  └─ Resources                                   │
│                                                 │
│  Supporting:                                    │
│  ├─ Runtime + Lifecycle                         │
│  ├─ Context + State                             │
│  └─ Configuration + Preferences                  │
└───────────────────────┬─────────────────────────┘
                        │
                        │ contracts
                        ▼
               ┌─────────────────┐
               │  DOMAIN ENGINE  │
               │  (app meaning)  │
               └────────┬────────┘
                        │
                        │ integration
                        ▼
               ┌─────────────────┐
               │  API / MCP / UI │
               └─────────────────┘
```

---

## What App Engine Provides

| System | Purpose |
|---|---|
| **Runtime** | Owns the application lifetime and coordinates all systems. |
| **Lifecycle** | Deterministic `Created → Initialized → Running → Stopped → Disposed` transitions. |
| **Context** | Identity hierarchy: `Application → Session → Execution`. |
| **State** | Generic state store with validated transitions. |
| **Commands** | Intent/capability execution — "the app can do X." |
| **Tasks** | Managed work instances with `Pending → Running → Completed/Failed` lifecycle. |
| **Scheduler** | Decides *when* work runs (immediate, delayed, interval, triggered). |
| **Events** | "Something happened" — decoupled one-to-many notifications. |
| **Signals** | "Something changed" — lightweight change notifications. |
| **History** | Records meaningful operations for undo/redo/replay. |
| **Resources** | Load, cache, track, and release generic data/content. |
| **Services** | Long-lived capabilities that start and stop with the app. |
| **Plugins** | Extension packages that contribute commands, tasks, resources, etc. |
| **Configuration** | Persistent application settings separate from runtime state. |
| **Preferences** | User-facing settings (theme, language, etc.). |

---

## What App Engine Does NOT Provide

| Concern | Who owns it |
|---|---|
| **Domain logic** (what "export project" means) | Domain Engine |
| **Domain state** (which document is open, which card is selected) | Domain Engine |
| **UI** (windows, widgets, rendering) | UI Layer |
| **API/MCP** (HTTP endpoints, AI agent contracts) | Integration Layer |
| **Persistence** (database, file system) | Future extension / Domain Engine |
| **Networking** (HTTP client, WebSocket) | Future extension / Domain Engine |

The App Engine is deliberately **semantic-agnostic**. It knows that a `Task` exists and has a lifecycle, but it does not know whether that task exports a project or renders a scene.

---

## App Engine vs Domain Engine

```text
  Domain Engine                  App Engine
  ┌──────────────┐              ┌──────────────┐
  │              │              │              │
  │  ExportProject│   uses →   │  TaskManager │
  │  CreateCard  │             │  Scheduler   │
  │  RenderScene │             │  EventBus    │
  │              │              │  Resources   │
  └──────────────┘              └──────────────┘
       meaning                   mechanisms
```

- **Domain Engine** defines *what the application can do*.
- **App Engine** provides *how work is managed, scheduled, and communicated*.
- **Dependency direction**: `Domain Engine → App Engine` (never the reverse).

Example: The Domain Engine defines `ExportProjectCommand`. The App Engine provides the `CommandExecutor` that runs it, the `TaskManager` that manages it, the `Scheduler` that decides when it runs, and the `EventBus` that announces `ProjectExported`.

---

## App Engine vs UI

The App Engine has **no UI types**. No `Window`, `Widget`, `Theme`, `Layout`, `Renderer`, or `Canvas`.

- The UI reads application state through context, events, and signals.
- The UI sends intents through commands.
- The UI changes settings through preferences.
- The App Engine never imports or depends on a UI framework.

```text
UI Layer                       App Engine
┌──────────────┐              ┌──────────────┐
│  Window      │   commands   │  CommandExecutor
│  Toolbar     │ ──────────→  │  EventBus     │
│  StatusBar   │ ←────────── │  SignalBus    │
└──────────────┘  events     └──────────────┘
```

---

## App Engine vs API/MCP

The App Engine has **no API types**. No `Endpoint`, `Request`, `Response`, `Route`, or `McpTool`.

- API/MCP is an **external integration layer** that exposes App Engine capabilities to clients.
- The App Engine's `Command` system is the natural integration point: API requests can be translated into commands.

```text
External Client          API Layer              App Engine
┌──────────┐          ┌──────────┐           ┌──────────┐
│  HTTP    │  request │  Route   │  command  │ Command  │
│  AI Agent│ ───────→ │  Handler│ ────────→ │ Executor │
│  CLI     │          │  MCP Tool│          │          │
└──────────┘          └──────────┘           └──────────┘
```

---

## When to Use App Engine

Use the App Engine when your application needs:

- **Managed execution** — commands, tasks, scheduling (not just function calls).
- **Undo/redo** — meaningful operation history.
- **Plugin extensibility** — third-party or optional extensions.
- **Decoupled communication** — events and signals instead of direct calls.
- **Resource management** — load, cache, and release data with reference tracking.
- **Long-lived services** — background workers that start/stop with the app.
- **Stable contracts** — a clean boundary between runtime infrastructure and domain logic.

You might **not** need it if:
- Your application is a simple script with no lifecycle, undo, or extensibility needs.
- You have no domain/UI/API separation concerns.

---

## Minimal Example

```rust
use app_shell::app_engine::{AppEngine, Bootstrap, Lifecycle};

fn main() {
    let mut engine: AppEngine = Bootstrap::create().expect("bootstrap failed");

    engine.initialize().expect("initialize failed");
    engine.start().expect("start failed");
    engine.run().expect("run failed");
    engine.stop().expect("stop failed");
    engine.dispose().expect("dispose failed");

    println!("Engine lifecycle complete: {:?}", engine.state());
}
```

This creates an `AppEngine`, runs through its full lifecycle, and shuts down cleanly. No UI, no API, no domain logic — just the runtime.

---

## Core Vocabulary

| Term | Definition |
|---|---|
| **AppEngine** | The composition root. Owns all subsystems. |
| **Bootstrap** | Constructs a fully wired `AppEngine`. |
| **Lifecycle** | `Created → Initialized → Running → Stopped → Disposed`. |
| **Context** | Identity: `Application → Session → Execution`. |
| **State** | Generic key-value state store with transitions. |
| **Command** | An application intent — "do X." Registered and executed. |
| **Task** | A managed work instance with its own lifecycle. |
| **Scheduler** | Decides *when* tasks run (immediate, delayed, triggered). |
| **Event** | "Something happened" — one-to-many, past occurrence. |
| **Signal** | "Something changed" — lightweight notification. |
| **History** | Ordered operations with undo/redo/replay. |
| **Resource** | Managed data with load/cache/unload lifecycle. |
| **Service** | Long-lived capability that starts/stops with the app. |
| **Plugin** | Extension package contributing commands, tasks, resources, etc. |
| **Configuration** | Persistent application settings (not runtime state). |
| **Preferences** | User-selected settings (theme, language, etc.). |

---

## Architecture at a Glance

```text
App
├── App Engine          ← runtime infrastructure (this library)
├── Domain Engine       ← application meaning and behavior
└── UI / API / MCP      ← external access and presentation
```

The App Engine is the **middle layer** — it provides the runtime machinery that the Domain Engine builds on, and that UI/API layers expose to users and external systems.

---

## Next Steps

- [02-architecture.md](02-architecture.md) — Internal architecture and module layout.
- [03-bootstrap-and-lifecycle.md](03-bootstrap-and-lifecycle.md) — Creating and running an engine.
- [09-domain-engine-integration.md](09-domain-engine-integration.md) — How the Domain Engine uses App Engine.