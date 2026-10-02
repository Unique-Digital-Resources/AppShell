# `10-application-integration.md` — UI, API, MCP & Complete Example

## Overview

The final layer wraps both engines and exposes them to the outside world:

```text
                         Application
                              │
        ┌─────────────────────┼─────────────────────┐
        ▼                     ▼                     ▼
       UI                    API                   MCP
     (presentation)      (local/web)          (AI agents)
        │                     │                     │
        └─────────────────────┼─────────────────────┘
                              ▼
                       Domain Engine
                     (application meaning)
                              │
                              ▼
                         App Engine
                    (runtime infrastructure)
```

Each external surface translates user/agent intents into **commands**, and translates engine **events/signals** back into user/agent-facing responses.

---

## The Integration Principle

All external layers follow the same pattern:

```text
Input (user action / HTTP request / MCP call)
       ↓
       Translate to Command
       ↓
       CommandExecutor.execute()
       ↓
       CommandResult / TaskResult
       ↓
       Translate to Output (UI update / HTTP response / MCP result)
```

And in reverse:

```text
EventBus / SignalBus
       ↓
       Handler in integration layer
       ↓
       Update UI / Push WebSocket / Notify agent
```

The integration layer **never** calls domain functions directly. It always goes through commands.

---

## Desktop UI

```text
┌─────────────────────────────────┐
│           Desktop UI             │
│  (Qt / GTK / Tauri / native)     │
│                                  │
│  ┌──────────┐  ┌──────────────┐ │
│  │ Toolbar  │  │ Canvas/View  │ │
│  └────┬─────┘  └──────┬───────┘ │
│       │ button click   │ render  │
│       ▼                ▲         │
│  ┌──────────────────────────────┐│
│  │    UI Integration Layer      ││
│  │  (translates UI → Commands)  ││
│  └──────────────┬───────────────┘│
└─────────────────┼────────────────┘
                  │ commands
                  ▼
            App Engine
```

### UI → App Engine

```rust
// User clicks "Export" button
fn on_export_clicked(engine: &mut AppEngine, project_id: &str) {
    let command = Command::new(
        "project.export",
        CommandContext::new(ExecutionId::new()),
        CommandInput::new(ExportProjectInput {
            project_id: project_id.to_string(),
            format: "pdf".to_string(),
            output_path: "/tmp/export.pdf".to_string(),
        }),
    );

    match engine.command_executor().execute(&command) {
        Ok(result) => {
            // Update UI with result
            show_status(&format!("Exported: {:?}", result.output::<String>()));
        }
        Err(e) => show_error(&e.to_string()),
    }
}
```

### App Engine → UI (via Signals)

```rust
// UI subscribes to progress signals during long operations
engine.signal_bus().subscribe(
    "task.progress_changed",
    Box::new(|signal: &Signal| {
        let progress = signal.payload::<f32>().unwrap_or(0.0);
        // update_progress_bar(progress);
        println!("Progress: {:.0}%", progress * 100.0);
    }),
);

// UI subscribes to events
engine.event_bus_mut().subscribe(
    "document.created",
    event_handler(|event: &Event| {
        // refresh_document_list();
        println!("Document created");
    }),
);
```

### Preferences

```rust
// UI reads and writes user preferences
engine.preferences_mut().set("theme", "dark");
engine.preferences_mut().set("show_toolbar", "true");
engine.preferences_mut().set("language", "en");

// UI reads preferences on startup
let theme = engine.preferences().get("theme").unwrap_or("light");
let show_toolbar = engine.preferences().get_bool("show_toolbar").unwrap_or(true);
```

---

## Web UI

```text
┌───────────────────────────┐
│         Web Frontend       │
│  (React / Vue / Svelte)    │
│                            │
│  ┌───────────────────────┐ │
│  │   WebSocket Client    │ │
│  └───────────┬───────────┘ │
└──────────────┼────────────┘
               │ JSON / binary
               ▼
┌───────────────────────────┐
│      Web Integration      │
│  (WebSocket / HTTP server) │
│                           │
│  translates:              │
│    HTTP → Commands        │
│    Events → WebSocket push│
│    Signals → WebSocket push│
└──────────────┬────────────┘
               │ commands
               ▼
          App Engine
```

### HTTP API → Commands

```rust
// POST /api/documents → document.create command
fn handle_create_document(req: HttpRequest, engine: &mut AppEngine) -> HttpResponse {
    let command = Command::new(
        "document.create",
        CommandContext::new(ExecutionId::new()),
        CommandInput::new(CreateDocumentInput {
            title: req.body["title"].clone(),
        }),
    );

    match engine.command_executor().execute(&command) {
        Ok(result) => HttpResponse::ok(json!({
            "document_id": result.output::<String>(),
        })),
        Err(e) => HttpResponse::error(&e.to_string()),
    }
}
```

### Events → WebSocket Push

```rust
// WebSocket clients receive events in real-time
engine.event_bus_mut().subscribe(
    "document.changed",
    event_handler(move |event: &Event| {
        let ws_message = json!({
            "type": event.event_type(),
            "source": event.source(),
            "payload": "document modified",
        });
        // websocket.broadcast(ws_message);
    }),
);

engine.signal_bus().subscribe(
    "task.progress_changed",
    Box::new(|signal: &Signal| {
        let ws_message = json!({
            "type": signal.signal_type(),
            "progress": signal.payload::<f32>().unwrap_or(0.0),
        });
        // websocket.broadcast(ws_message);
    }),
);
```

---

## CLI

```text
┌───────────────────────┐
│      CLI Frontend      │
│  (clap / structopt)    │
│                        │
│  app export --format pdf
│  app create "My Doc"   │
│  app --worker-count 8  │
└───────────┬────────────┘
            │ parses args → commands
            ▼
       App Engine
```

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand)]
enum CliCommand {
    Create { title: String },
    Export { project_id: String, format: String, output: String },
    List,
}

fn main() {
    let cli = Cli::parse();
    let mut engine: AppEngine = Bootstrap::create().unwrap();
    domain_engine::register_with_engine(&mut engine).unwrap();
    engine.initialize().unwrap();
    engine.start().unwrap();

    match cli.command {
        CliCommand::Create { title } => {
            let result = engine.command_executor().execute(&Command::new(
                "document.create",
                CommandContext::new(ExecutionId::new()),
                CommandInput::new(CreateDocumentInput { title }),
            )).unwrap();
            println!("Created: {}", result.output::<String>().unwrap());
        }
        CliCommand::Export { project_id, format, output } => {
            let task_id = engine.task_manager_mut().create(
                "project.export",
                TaskContext::new(ExecutionId::new()),
                TaskInput::new(ExportProjectInput {
                    project_id, format, output_path: output,
                }),
            ).unwrap();
            let result = engine.task_manager_mut().start(task_id).unwrap();
            println!("Export complete: {:?}", result.output::<String>());
        }
        CliCommand::List => {
            for def in engine.command_executor().list() {
                println!("  {} — {}", def.id(), def.description());
            }
        }
    }

    engine.stop().unwrap();
    engine.dispose().unwrap();
}
```

### CLI Configuration Override

```rust
// CLI flags override configuration
let cli = Cli::parse();

// Apply config overrides
if let Some(workers) = cli.worker_count {
    engine.config_store_mut().set("worker_count", &workers.to_string()).unwrap();
}

engine.initialize()?;
engine.start()?;
```

---

## Local API (Library Mode)

For applications that embed the engine as a library (other Rust code, FFI, embedded systems):

```rust
/// Public API for embedding the application as a library.
pub struct AppApi {
    engine: AppEngine,
}

impl AppApi {
    pub fn new() -> Self {
        let mut engine = Bootstrap::create().unwrap();
        domain_engine::register_with_engine(&mut engine).unwrap();
        engine.initialize().unwrap();
        engine.start().unwrap();
        Self { engine }
    }

    pub fn create_document(&mut self, title: &str) -> Result<String, String> {
        let result = self.engine.command_executor().execute(&Command::new(
            "document.create",
            CommandContext::new(ExecutionId::new()),
            CommandInput::new(CreateDocumentInput { title: title.to_string() }),
        )).map_err(|e| e.to_string())?;

        result.output::<String>().ok_or("no document id".to_string())
    }

    pub fn export_project(&mut self, project_id: &str, format: &str, path: &str) -> Result<(), String> {
        let task_id = self.engine.task_manager_mut().create(
            "project.export",
            TaskContext::new(ExecutionId::new()),
            TaskInput::new(ExportProjectInput {
                project_id: project_id.to_string(),
                format: format.to_string(),
                output_path: path.to_string(),
            }),
        ).map_err(|e| e.to_string())?;

        let result = self.engine.task_manager_mut().start(task_id)
            .map_err(|e| e.to_string())?;

        if result.is_completed() {
            Ok(())
        } else {
            Err("export failed".to_string())
        }
    }

    pub fn on_document_changed<F>(&mut self, handler: F)
    where
        F: Fn(&Event) + Send + Sync + 'static,
    {
        self.engine.event_bus_mut().subscribe(
            "document.changed",
            event_handler(handler),
        );
    }

    pub fn shutdown(&mut self) {
        self.engine.stop().unwrap();
        self.engine.dispose().unwrap();
    }
}
```

### Usage

```rust
let mut app = AppApi::new();

let doc_id = app.create_document("Hello World")?;
app.export_project("proj_1", "pdf", "/tmp/out.pdf")?;

app.on_document_changed(|event| {
    println!("Document changed: {}", event.event_type());
});

app.shutdown();
```

---

## MCP (Model Context Protocol)

For AI agent integration:

```text
┌───────────────────────────┐
│       AI Agent / LLM       │
│                            │
│  "Export the project       │
│   as PDF to /tmp/out"     │
└─────────────┬─────────────┘
              │ MCP tool call
              ▼
┌───────────────────────────┐
│     MCP Integration        │
│                            │
│  MCP Tool: export_project  │
│  MCP Tool: create_document │
│  MCP Tool: list_commands   │
│                            │
│  translates:               │
│    tool call → Command     │
│    CommandResult → MCP     │
│    response                │
└─────────────┬─────────────┘
              │ commands
              ▼
         App Engine
```

### MCP Tool Definition

```rust
/// MCP tool: create_document
/// 
/// Creates a new document with the given title.
/// 
/// Parameters:
///   - title (string, required): The document title
/// 
/// Returns:
///   - document_id (string): The created document's ID
fn mcp_create_document(params: serde_json::Value, engine: &mut AppEngine) -> serde_json::Value {
    let title = params["title"].as_str().expect("title required");

    let command = Command::new(
        "document.create",
        CommandContext::new(ExecutionId::new()),
        CommandInput::new(CreateDocumentInput {
            title: title.to_string(),
        }),
    );

    match engine.command_executor().execute(&command) {
        Ok(result) => json!({
            "document_id": result.output::<String>().unwrap_or_default(),
        }),
        Err(e) => json!({ "error": e.to_string() }),
    }
}

/// MCP tool: export_project
/// 
/// Exports a project to the specified format.
/// 
/// Parameters:
///   - project_id (string, required): The project to export
///   - format (string, required): Output format (pdf, png, etc.)
///   - output_path (string, required): Where to save the export
/// 
/// Returns:
///   - output_path (string): The path to the exported file
///   - page_count (number): Number of pages exported
fn mcp_export_project(params: serde_json::Value, engine: &mut AppEngine) -> serde_json::Value {
    let input = ExportProjectInput {
        project_id: params["project_id"].as_str().unwrap().to_string(),
        format: params["format"].as_str().unwrap().to_string(),
        output_path: params["output_path"].as_str().unwrap().to_string(),
    };

    let task_id = match engine.task_manager_mut().create(
        "project.export",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new(input),
    ) {
        Ok(id) => id,
        Err(e) => return json!({ "error": e.to_string() }),
    };

    match engine.task_manager_mut().start(task_id) {
        Ok(result) => {
            let export_result: &ExportResult = result.output::<ExportResult>().unwrap();
            json!({
                "output_path": export_result.output_path,
                "page_count": export_result.page_count,
            })
        }
        Err(e) => json!({ "error": e.to_string() }),
    }
}

/// MCP tool: list_available_commands
/// 
/// Lists all commands available in the application.
/// 
/// Returns:
///   - commands (array): List of {id, name, description}
fn mcp_list_commands(_params: serde_json::Value, engine: &AppEngine) -> serde_json::Value {
    let commands: Vec<serde_json::Value> = engine.command_executor()
        .list()
        .iter()
        .map(|def| json!({
            "id": def.id(),
            "name": def.name(),
            "description": def.description(),
        }))
        .collect();

    json!({ "commands": commands })
}
```

### Agent Subscription to Events

```rust
// Agent receives real-time updates via MCP notifications
engine.event_bus_mut().subscribe(
    "task.completed",
    event_handler(|event: &Event| {
        // Notify the agent that a task completed
        // mcp_server.notify("task.completed", event.payload());
    }),
);

engine.event_bus_mut().subscribe(
    "document.changed",
    event_handler(|event: &Event| {
        // Notify the agent that a document changed
        // mcp_server.notify("document.changed", {});
    }),
);
```

---

## Headless Operation

The App Engine runs perfectly without any UI, API, or MCP:

```rust
// Pure headless batch processing
fn main() {
    let mut engine: AppEngine = Bootstrap::create().unwrap();
    domain_engine::register_with_engine(&mut engine).unwrap();

    // Load configuration
    engine.config_store_mut().set("batch_mode", "true").unwrap();

    engine.initialize().unwrap();
    engine.start().unwrap();

    // Process a queue of work
    let tasks = load_batch_tasks(); // application-specific
    for task_input in tasks {
        let task_id = engine.task_manager_mut().create(
            "project.export",
            TaskContext::new(ExecutionId::new()),
            TaskInput::new(task_input),
        ).unwrap();

        let result = engine.task_manager_mut().start(task_id).unwrap();

        if result.is_completed() {
            println!("OK: {:?}", result.output::<String>());
        } else if result.is_failed() {
            eprintln!("FAILED: {}", result.error().unwrap_or("unknown"));
        }
    }

    engine.stop().unwrap();
    engine.dispose().unwrap();
}
```

No UI, no HTTP server, no MCP — just the engine doing work.

---

## Complete Application Example

```rust
use app_shell::app_engine::*;

fn main() {
    // ─── Bootstrap ───
    let mut engine: AppEngine = Bootstrap::create().expect("bootstrap failed");

    // ─── Domain Registration ───
    domain_engine::register_with_engine(&mut engine).expect("domain registration failed");

    // ─── Configuration ───
    engine.config_store_mut().set("app.name", "MyApp").unwrap();
    engine.config_store_mut().set("worker_count", "4").unwrap();

    // ─── Preferences ───
    engine.preferences_mut().set("theme", "dark");
    engine.preferences_mut().set("language", "en");

    // ─── Event Subscriptions (Integration Layer) ───
    // UI subscribes to progress
    engine.signal_bus().subscribe(
        "task.progress_changed",
        Box::new(|signal: &Signal| {
            let progress = signal.payload::<f32>().unwrap_or(0.0);
            println!("[UI] Progress: {:.0}%", progress * 100.0);
        }),
    );

    // History subscribes to meaningful events
    engine.event_bus_mut().subscribe(
        "document.created",
        event_handler(|event: &Event| {
            println!("[History] Recorded: {}", event.event_type());
        }),
    );

    // API/MCP subscribes to task events
    engine.event_bus_mut().subscribe(
        "task.completed",
        event_handler(|event: &Event| {
            println!("[API] Task completed: source={}", event.source().unwrap_or("?"));
        }),
    );

    // ─── Lifecycle ───
    engine.initialize().expect("initialize failed");
    engine.start().expect("start failed");

    // ─── Application Work ───
    // Create a document (command — synchronous)
    let create_result = engine.command_executor().execute(&Command::new(
        "document.create",
        CommandContext::new(ExecutionId::new()),
        CommandInput::new(CreateDocumentInput {
            title: "My Document".to_string(),
        }),
    )).unwrap();

    let doc_id = create_result.output::<String>().unwrap();
    println!("Created document: {}", doc_id);

    // Publish a domain event (consumed by History, UI, etc.)
    engine.event_bus().publish(
        &Event::new("document.created")
            .with_payload(doc_id.clone())
            .with_source("main"),
    );

    // Export a project (task — managed, cancellable)
    let task_id = engine.task_manager_mut().create(
        "project.export",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new(ExportProjectInput {
            project_id: "proj_1".to_string(),
            format: "pdf".to_string(),
            output_path: "/tmp/export.pdf".to_string(),
        }),
    ).unwrap();

    let result = engine.task_manager_mut().start(task_id).unwrap();
    if result.is_completed() {
        let export_result: &ExportResult = result.output::<ExportResult>().unwrap();
        println!("Exported {} pages to {}", export_result.page_count, export_result.output_path);

        // Publish completion event
        engine.event_bus().publish(
            &Event::new("task.completed")
                .with_source("TaskExecutor"),
        );
    }

    // Load a resource
    let handle = engine.resource_manager_mut().load(
        "document",
        "file:///docs/my_doc.dp",
    ).unwrap();
    println!("Loaded resource: {}", handle.resource_id().value());

    // ─── Shutdown ───
    engine.stop().expect("stop failed");
    engine.dispose().expect("dispose failed");

    println!("Application complete. Final state: {:?}", engine.state());
}
```

---

## Integration Layer Summary

```text
┌─────────────────────────────────────────────────────────┐
│                   Integration Layer                     │
│                                                         │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  │
│  │ Desktop │  │   Web   │  │   CLI   │  │   MCP   │  │
│  │   UI    │  │   API   │  │         │  │  Tools  │  │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘  │
│       │            │            │            │        │
│       ▼            ▼            ▼            ▼        │
│  ┌────────────────────────────────────────────────┐   │
│  │           Translate to Commands                │   │
│  │     Translate from Events / Signals            │   │
│  └──────────────────────┬─────────────────────────┘   │
└─────────────────────────┼─────────────────────────────┘
                          │
                          ▼
                   ┌──────────────┐
                   │ Domain Engine │
                   └──────┬───────┘
                          │
                          ▼
                   ┌──────────────┐
                   │  App Engine   │
                   └──────────────┘
```

| Layer | Translates | Direction |
|---|---|---|
| Desktop UI | Button clicks → Commands | Inbound |
| Desktop UI | Signals → UI updates | Outbound |
| Web API | HTTP requests → Commands | Inbound |
| Web API | Events → WebSocket pushes | Outbound |
| CLI | Args → Commands | Inbound |
| CLI | Results → stdout | Outbound |
| MCP | Tool calls → Commands | Inbound |
| MCP | Events → Agent notifications | Outbound |

All layers follow the same pattern: **translate external input into commands, translate events/signals into external output.**

---

## Complete Application Stack

```text
┌─────────────────────────────────────────────────┐
│                  User / Agent                    │
│  clicks, types, speaks, requests                │
└───────────────────────┬─────────────────────────┘
                        │
┌───────────────────────▼─────────────────────────┐
│              Integration Layer                   │
│                                                  │
│  Desktop UI  │  Web API  │  CLI  │  MCP Tools   │
│                                                  │
│  translates input → Commands                     │
│  translates Events → output                      │
│  reads/writes Preferences                        │
│  reads Configuration                             │
└───────────────────────┬─────────────────────────┘
                        │ commands + event subscriptions
┌───────────────────────▼─────────────────────────┐
│                Domain Engine                      │
│                                                  │
│  Project  │  Document  │  Card  │  Scene        │
│                                                  │
│  create_document()  │  export_project()          │
│                                                  │
│  Domain Commands  │  Domain Tasks                │
│  Domain Resources │  Domain Services             │
│  Domain Events                                    │
└───────────────────────┬─────────────────────────┘
                        │ uses contracts
┌───────────────────────▼─────────────────────────┐
│                  App Engine                      │
│                                                  │
│  Runtime + Lifecycle                             │
│  Context + State                                 │
│  Commands + Tasks + Scheduler                    │
│  Events + Signals + History                      │
│  Resources + Services + Plugins                  │
│  Configuration + Preferences                     │
└──────────────────────────────────────────────────┘
```

---

## Architecture Rules

1. **Integration layers never call domain functions directly.** Always through commands.
2. **Integration layers never access domain objects directly.** Always through command results or events.
3. **One command can serve all integration layers.** The same `document.create` command works for UI, API, CLI, and MCP.
4. **Events serve all integration layers.** The same `document.changed` event updates the UI, pushes to WebSocket, and notifies the AI agent.
5. **The App Engine doesn't know which integration layers exist.** It just provides commands and events.
6. **The Domain Engine doesn't know which integration layers exist.** It just provides command handlers and task handlers.
7. **Adding a new integration layer** (e.g., a new MCP endpoint) requires zero changes to the engines — just a new translation layer.

---

## Quick Reference

| Want to... | Do this |
|---|---|
| Add a UI button | Create a command, execute via `command_executor()` |
| Show progress | Subscribe to `signal_bus()` with `task.progress_changed` |
| Add HTTP endpoint | Translate request → `Command`, execute, return result |
| Add MCP tool | Define tool schema, translate call → `Command`/`Task` |
| Add CLI command | Parse args, translate to `Command`, execute |
| Get real-time updates | Subscribe to `event_bus()` or `signal_bus()` |
| Read user settings | `engine.preferences().get("key")` |
| Change settings | `engine.preferences_mut().set("key", "value")` |
| Read configuration | `engine.config_store().get("key")` |
| Run headless | Just don't start a UI/API/MCP server — engine works alone |

---

## Conclusion

The 12-phase App Engine is now a **complete, generic application runtime**:

```text
Phase 1:  Runtime + Lifecycle
Phase 2:  Context + State
Phase 3:  Commands
Phase 4:  Tasks + Execution
Phase 5:  Scheduling
Phase 6:  Events + Signals
Phase 7:  History (undo/redo/replay)
Phase 8:  Resources (load/cache/unload)
Phase 9:  Services (long-lived capabilities)
Phase 10: Plugins (extension packages)
Phase 11: Configuration + Preferences
Phase 12: Integration cleanup + AppEngine
```

The architecture provides:

- **Stable contracts** — commands, tasks, events, signals, resources, services, plugins.
- **Clean boundaries** — App Engine ↔ Domain Engine ↔ Integration Layer.
- **Decoupled communication** — producers don't know consumers.
- **Extensibility** — plugins add capabilities without modifying the core.
- **Observability** — events, signals, and history provide visibility into what the application is doing.
- **Headless operation** — no UI, API, or MCP required.

The Domain Engine provides **meaning**. The App Engine provides **machinery**. The Integration Layer provides **access**. Each layer is independently evolvable.

---

## Guide Index

- [01-overview.md](01-overview.md) — What App Engine Is
- [02-architecture.md](02-architecture.md) — Architecture & Boundaries
- [03-bootstrap-and-lifecycle.md](03-bootstrap-and-lifecycle.md) — Creating and Running an App
- [04-context-and-state.md](04-context-and-state.md) — Context & State
- [05-commands-and-tasks.md](05-commands-and-tasks.md) — Application Operations
- [06-scheduling-and-communication.md](06-scheduling-and-communication.md) — Scheduling, Events & Signals
- [07-history-and-resources.md](07-history-and-resources.md) — History & Resources
- [08-services-and-plugins.md](08-services-and-plugins.md) — Services & Plugins
- [09-domain-engine-integration.md](09-domain-engine-integration.md) — Domain Engine Integration
- [10-application-integration.md](10-application-integration.md) — UI, API, MCP & Complete Example