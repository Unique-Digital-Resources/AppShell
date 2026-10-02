# `09-domain-engine-integration.md` — Building a Domain Engine on App Engine

## Overview

This chapter answers the central question:

> **How do I build my application's domain layer using the App Engine?**

The answer is a single architectural pattern applied consistently:

```text
Domain Function
      ↑
      │ wrapped by
      │
Command / Task
      ↑
      │ exposed through
      │
   App Engine
```

The Domain Engine provides the **meaning**. The App Engine provides the **machinery**. Together, they form a complete application.

---

## Ownership Boundary

### Domain Engine Owns

```text
✓ Meaning — what "export project" actually does
✓ Rules — validation, business logic, constraints
✓ Objects — Project, Document, Card, Scene, Layer
✓ Operations — create_document(), export_project(), render_scene()
✓ Domain data — document contents, card stats, scene hierarchy
✓ Domain resource types — DocumentResource, ProjectResource
✓ Domain-specific commands — ExportProjectCommand, CreateCardCommand
✓ Domain-specific tasks — ExportProjectTask, RenderSceneTask
✓ Domain events — DocumentCreated, ProjectExported, CardMoved
```

### App Engine Owns

```text
✓ Lifecycle — Created → Initialized → Running → Stopped → Disposed
✓ Execution — CommandExecutor, TaskExecutor
✓ Scheduling — Scheduler, JobQueue, Schedule, Trigger
✓ Communication — EventBus, SignalBus
✓ Resource management — ResourceManager, ResourceCache, ResourceHandle
✓ History infrastructure — HistoryStore, Undo, Redo, Replay
✓ Plugin infrastructure — PluginManager, PluginLoader
✓ Configuration — ConfigStore, Preferences
✓ Context — Application, Session, Execution hierarchy
✓ State — generic state store
```

The key rule: **App Engine never imports Domain Engine types.** The dependency goes one way:

```text
Domain Engine ──→ App Engine
App Engine     ✗──→ Domain Engine
```

---

## Domain Engine Structure

```text
domain_engine/
├── project/
│   ├── mod.rs
│   ├── project.rs              ← domain object
│   ├── document.rs             ← domain object
│   ├── create_document.rs      ← domain function
│   ├── save_document.rs        ← domain function
│   └── export_project.rs       ← domain function
│
├── commands/                   ← domain commands (App Engine handlers)
│   ├── mod.rs
│   ├── create_document.rs      ← CommandHandler impl
│   ├── save_document.rs        ← CommandHandler impl
│   └── export_project.rs       ← CommandHandler impl
│
├── tasks/                     ← domain tasks (App Engine handlers)
│   ├── mod.rs
│   └── export_project.rs       ← TaskHandler impl
│
├── resources/                 ← domain resources (App Engine loaders)
│   ├── mod.rs
│   ├── document_loader.rs      ← ResourceLoader impl
│   └── document_data.rs        ← resource data type
│
├── services/                  ← domain services (App Engine services)
│   ├── mod.rs
│   └── autosave_service.rs     ← Service impl
│
└── events/                    ← domain event definitions
    ├── mod.rs
    └── document_events.rs
```

Each subdirectory maps to an App Engine contract.

---

## Pattern 1: Domain Function → Command

For short, synchronous operations:

```text
Domain Function: create_document(name) → Document
                          ↑
                          │ wrapped by
                          │
               CreateDocumentCommand (CommandHandler)
                          ↑
                          │ registered with
                          │
                    CommandExecutor
```

### Domain Function

```rust
// domain_engine/project/document.rs

pub struct Document {
    pub id: String,
    pub title: String,
    pub content: String,
}

// domain_engine/project/create_document.rs

pub fn create_document(title: &str) -> Document {
    Document {
        id: format!("doc_{}", uuid::new_v4()),
        title: title.to_string(),
        content: String::new(),
    }
}
```

### Domain Command (CommandHandler)

```rust
// domain_engine/commands/create_document.rs

use app_shell::app_engine::*;
use crate::project::{create_document, Document};

pub struct CreateDocumentCommand;

#[derive(Debug, Clone)]
pub struct CreateDocumentInput {
    pub title: String,
}

impl CommandHandler for CreateDocumentCommand {
    fn execute(
        &self,
        input: &CommandInput,
        _context: &CommandContext,
    ) -> Result<CommandResult, CommandError> {
        let input: &CreateDocumentInput = input.get::<CreateDocumentInput>()
            .ok_or_else(|| CommandError::InvalidArguments("expected CreateDocumentInput".to_string()))?;

        // Call domain function
        let document = create_document(&input.title);

        Ok(CommandResult::success_with(document.id.clone())
            .with_metadata("title", &document.title))
    }
}
```

### Registration

```rust
// During application setup:
engine.command_executor_mut().register(
    CommandDefinition::new("document.create", "Create Document", "Creates a new document"),
    Box::new(CreateDocumentCommand),
)?;

// Usage:
let result = engine.command_executor().execute(&Command::new(
    "document.create",
    CommandContext::new(ExecutionId::new()),
    CommandInput::new(CreateDocumentInput { title: "My Doc".to_string() }),
))?;

let doc_id = result.output::<String>().unwrap();
```

The App Engine executed the command. It never knew that the command creates a document — it just called `handler.execute()` and returned a `CommandResult`.

---

## Pattern 2: Domain Function → Task

For long-running, cancellable, or schedulable operations:

```text
Domain Function: export_project(project, format, path) → ExportResult
                          ↑
                          │ wrapped by
                          │
               ExportProjectTask (TaskHandler)
                          ↑
                          │ registered with
                          │
                     TaskManager
```

### Domain Function

```rust
// domain_engine/project/export_project.rs

pub struct ExportResult {
    pub output_path: String,
    pub file_size: u64,
    pub page_count: usize,
}

pub fn export_project(
    project: &Project,
    format: &str,
    output_path: &str,
) -> Result<ExportResult, ExportError> {
    // Domain-specific export logic: serialize, render, write file...
    Ok(ExportResult {
        output_path: output_path.to_string(),
        file_size: 1024 * 1024,
        page_count: 42,
    })
}
```

### Domain Task (TaskHandler)

```rust
// domain_engine/tasks/export_project.rs

use app_shell::app_engine::*;
use crate::project::{export_project, Project, ExportResult};

pub struct ExportProjectTask;

#[derive(Debug, Clone)]
pub struct ExportProjectInput {
    pub project_id: String,
    pub format: String,
    pub output_path: String,
}

impl TaskHandler for ExportProjectTask {
    fn execute(
        &self,
        input: &TaskInput,
        _context: &TaskContext,
    ) -> Result<TaskResult, TaskError> {
        let input: &ExportProjectInput = input.get::<ExportProjectInput>()
            .ok_or_else(|| TaskError::ExecutionFailed {
                id: 0,
                reason: "expected ExportProjectInput".to_string(),
            })?;

        // Load the project (domain-specific lookup)
        let project = load_project(&input.project_id)
            .map_err(|e| TaskError::ExecutionFailed {
                id: 0,
                reason: e.to_string(),
            })?;

        // Call domain function
        let result = export_project(&project, &input.format, &input.output_path)
            .map_err(|e| TaskError::ExecutionFailed {
                id: 0,
                reason: e.to_string(),
            })?;

        Ok(TaskResult::completed_with(result)
            .with_metadata("format", &input.format))
    }
}
```

### Registration and Usage

```rust
// Register
engine.task_manager_mut().register(
    TaskDefinition::new("project.export", "Export Project", "Exports the project")
        .with_cancel_support(),
    Box::new(ExportProjectTask),
)?;

// Create and execute
let task_id = engine.task_manager_mut().create(
    "project.export",
    TaskContext::new(ExecutionId::new())
        .with_application(app_id)
        .with_session(session_id),
    TaskInput::new(ExportProjectInput {
        project_id: "proj_42".to_string(),
        format: "pdf".to_string(),
        output_path: "/tmp/export.pdf".to_string(),
    }),
)?;

let result = engine.task_manager_mut().start(task_id)?;
assert!(result.is_completed());
let export_result: &ExportResult = result.output::<ExportResult>().unwrap();
println!("Exported {} pages ({:?} bytes)", export_result.page_count, export_result.file_size);
```

---

## Pattern 3: Command → Task (Delegation)

A command can create and schedule a task instead of executing directly:

```text
User clicks "Export"
       ↓
ExportProjectCommand (CommandHandler)
       ↓
Creates Task via TaskManager
       ↓
Schedules via Scheduler (optional)
       ↓
ExportProjectTask (TaskHandler)
       ↓
export_project() (domain function)
       ↓
TaskResult
```

```rust
// domain_engine/commands/start_export.rs

pub struct StartExportCommand;

impl CommandHandler for StartExportCommand {
    fn execute(
        &self,
        input: &CommandInput,
        _context: &CommandContext,
    ) -> Result<CommandResult, CommandError> {
        let input: &ExportProjectInput = input.get::<ExportProjectInput>()
            .ok_or_else(|| CommandError::InvalidArguments("expected ExportProjectInput".to_string()))?;

        // Instead of exporting directly, create a task
        // (The command handler would need access to the TaskManager
        //  — this would be provided through the application context
        //  or by passing the engine to the handler)

        Ok(CommandResult::success()
            .with_metadata("task_definition", "project.export")
            .with_metadata("project_id", &input.project_id))
    }
}
```

This pattern separates **intent** (the command) from **managed execution** (the task).

---

## Pattern 4: Domain Resource

The Domain Engine defines resource types and loaders. The App Engine manages their lifecycle.

```text
DocumentLoader (ResourceLoader)
       ↓
 loads from file
       ↓
 DocumentData (domain type)
       ↓
 stored in Resource (generic App Engine wrapper)
       ↓
 accessed via ResourceHandle
```

### Domain Resource Type

```rust
// domain_engine/resources/document_data.rs

#[derive(Debug, Clone)]
pub struct DocumentData {
    pub title: String,
    pub content: String,
    pub modified_at: u64,
}
```

### Domain Loader

```rust
// domain_engine/resources/document_loader.rs

use app_shell::app_engine::{ResourceLoader, ResourceError};
use std::any::Any;

pub struct DocumentLoader;

impl ResourceLoader for DocumentLoader {
    fn resource_type(&self) -> &str {
        "document"
    }

    fn load(&self, source: &str) -> Result<Box<dyn Any + Send>, ResourceError> {
        // Domain-specific file reading
        let data = DocumentData {
            title: source.to_string(),
            content: "Document content here...".to_string(),
            modified_at: 0,
        };
        Ok(Box::new(data))
    }
}
```

### Registration and Usage

```rust
// Register the domain loader
engine.resource_manager_mut().register_loader(Box::new(DocumentLoader));

// Load a document resource
let handle = engine.resource_manager_mut().load(
    "document",
    "file:///docs/my_doc.dp",
)?;

// Access domain data through the generic resource
let resource = engine.resource_manager().get(&handle).unwrap();
let doc: &DocumentData = resource.data::<DocumentData>().unwrap();
println!("Title: {}", doc.title);

// Acquire for a task
engine.resource_manager_mut().acquire(&handle)?;

// ... use the document in a task ...

// Release when done
engine.resource_manager_mut().release(handle)?;
```

The App Engine managed loading, caching, and reference counting. The Domain Engine provided the `DocumentData` type and `DocumentLoader` implementation.

---

## Pattern 5: Domain Service

A domain service coordinates domain operations using App Engine systems:

```text
AutosaveService
       ↓
 subscribes to "document.changed" event
       ↓
 creates a SaveDocumentTask
       ↓
 schedules it with a delay
       ↓
 TaskExecutor runs the save
```

```rust
// domain_engine/services/autosave_service.rs

use app_shell::app_engine::*;
use std::sync::{Arc, Mutex};

pub struct AutosaveService {
    save_count: Arc<Mutex<usize>>,
}

impl AutosaveService {
    pub fn new() -> Self {
        Self {
            save_count: Arc::new(Mutex::new(0)),
        }
    }
}

impl Service for AutosaveService {
    fn service_type(&self) -> &str { "autosave" }

    fn initialize(&mut self) -> Result<(), ServiceError> {
        Ok(())
    }

    fn start(&mut self) -> Result<(), ServiceError> {
        // In a real service, this would subscribe to events
        // and schedule tasks through the engine.
        // The engine reference would be provided through
        // an expanded PluginContext or service context.
        *self.save_count.lock().unwrap() = 0;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), ServiceError> {
        // Perform a final save
        Ok(())
    }

    fn dispose(&mut self) -> Result<(), ServiceError> {
        Ok(())
    }
}
```

### Registration

```rust
engine.service_manager_mut().register(
    ServiceDefinition::new("autosave", "Autosave", "Automatic document saving"),
    Box::new(AutosaveService::new()),
)?;

// The service starts/stops automatically with engine lifecycle
engine.initialize()?;
engine.start()?;  // AutosaveService.start() is called
// ... app runs, autosave monitors for changes ...
engine.stop()?;   // AutosaveService.stop() is called
engine.dispose()?;
```

---

## Pattern 6: Domain Events

The Domain Engine can publish and subscribe to events through the App Engine's `EventBus`:

```rust
// domain_engine/events/document_events.rs

// Event type constants
pub const DOCUMENT_CREATED: &str = "document.created";
pub const DOCUMENT_CHANGED: &str = "document.changed";
pub const DOCUMENT_SAVED: &str = "document.saved";

// Event payload types
#[derive(Debug, Clone)]
pub struct DocumentChangedPayload {
    pub document_id: String,
    pub field: String,
    pub old_value: String,
    pub new_value: String,
}
```

### Publishing Domain Events

```rust
// Inside a command handler or task handler:
fn save_document_handler(/* ... */) {
    // ... save the document ...

    // Publish a domain event
    engine.event_bus().publish(
        &Event::new("document.saved")
            .with_payload(DocumentChangedPayload {
                document_id: "doc_42".to_string(),
                field: "content".to_string(),
                old_value: "old".to_string(),
                new_value: "new".to_string(),
            })
            .with_source("SaveDocumentCommand"),
    );
}
```

### Subscribing to Domain Events

```rust
// The autosave service subscribes to document changes:
engine.event_bus_mut().subscribe(
    "document.changed",
    event_handler(|event: &Event| {
        if let Some(payload) = event.payload::<DocumentChangedPayload>() {
            println!("Document {} changed: {} → {}",
                payload.document_id, payload.field, payload.new_value);
        }
    }),
);

// History subscribes to meaningful events:
engine.event_bus_mut().subscribe(
    "document.saved",
    event_handler(|event: &Event| {
        // Record in history for undo/redo
    }),
);
```

The App Engine's `EventBus` routes the event. The Domain Engine provides the event types and payloads. Neither knows about the other's internals.

---

## Complete Example: ProjectDomain

```rust
// domain_engine/project/mod.rs

use app_shell::app_engine::*;

// --- Domain Objects ---

pub struct Project {
    pub id: String,
    pub name: String,
    pub documents: Vec<Document>,
}

pub struct Document {
    pub id: String,
    pub title: String,
    pub content: String,
}

pub struct ExportResult {
    pub output_path: String,
    pub page_count: usize,
}

// --- Domain Functions ---

pub fn create_document(title: &str) -> Document {
    Document {
        id: format!("doc_{}", rand::random::<u32>()),
        title: title.to_string(),
        content: String::new(),
    }
}

pub fn save_document(doc: &Document, path: &str) -> Result<(), String> {
    // Domain-specific save logic
    Ok(())
}

pub fn export_project(project: &Project, format: &str, output_path: &str) -> Result<ExportResult, String> {
    // Domain-specific export logic
    Ok(ExportResult {
        output_path: output_path.to_string(),
        page_count: project.documents.len(),
    })
}

// --- Domain Command Input ---

#[derive(Debug, Clone)]
pub struct CreateDocumentInput {
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct ExportProjectInput {
    pub project_id: String,
    pub format: String,
    pub output_path: String,
}

// --- Domain Commands ---

pub struct CreateDocumentCommand;

impl CommandHandler for CreateDocumentCommand {
    fn execute(&self, input: &CommandInput, _ctx: &CommandContext) -> Result<CommandResult, CommandError> {
        let input: &CreateDocumentInput = input.get::<CreateDocumentInput>()
            .ok_or_else(|| CommandError::InvalidArguments("expected CreateDocumentInput".to_string()))?;

        let doc = create_document(&input.title);

        Ok(CommandResult::success_with(doc.id.clone())
            .with_metadata("title", &doc.title))
    }
}

// --- Domain Tasks ---

pub struct ExportProjectTask;

impl TaskHandler for ExportProjectTask {
    fn execute(&self, input: &TaskInput, _ctx: &TaskContext) -> Result<TaskResult, TaskError> {
        let input: &ExportProjectInput = input.get::<ExportProjectInput>()
            .ok_or_else(|| TaskError::ExecutionFailed {
                id: 0, reason: "expected ExportProjectInput".into()
            })?;

        // In a real app, load the project from domain state
        let project = Project {
            id: input.project_id.clone(),
            name: "Demo".to_string(),
            documents: vec![],
        };

        let result = export_project(&project, &input.format, &input.output_path)
            .map_err(|e| TaskError::ExecutionFailed {
                id: 0, reason: e
            })?;

        // Publish a domain event
        // (would need engine reference — in a real app, the task handler
        //  would receive it through context or dependency injection)

        Ok(TaskResult::completed_with(result))
    }
}

// --- Domain Resource ---

pub struct DocumentData {
    pub title: String,
    pub content: String,
}

pub struct DocumentLoader;

impl ResourceLoader for DocumentLoader {
    fn resource_type(&self) -> &str { "document" }
    fn load(&self, source: &str) -> Result<Box<dyn std::any::Any + Send>, ResourceError> {
        Ok(Box::new(DocumentData {
            title: source.to_string(),
            content: "Content...".to_string(),
        }))
    }
}

// --- Registration Function ---

pub fn register_with_engine(engine: &mut AppEngine) -> Result<(), Box<dyn std::error::Error>> {
    // Commands
    engine.command_executor_mut().register(
        CommandDefinition::new("document.create", "Create Document", "Create a new document"),
        Box::new(CreateDocumentCommand),
    )?;

    // Tasks
    engine.task_manager_mut().register(
        TaskDefinition::new("project.export", "Export Project", "Export the project")
            .with_cancel_support(),
        Box::new(ExportProjectTask),
    )?;

    // Resources
    engine.resource_manager_mut().register_loader(Box::new(DocumentLoader));

    Ok(())
}
```

### Using the Domain Engine

```rust
use app_shell::app_engine::*;
use domain_engine::project;

fn main() {
    let mut engine = Bootstrap::create().unwrap();

    // Register all domain commands, tasks, and resources
    project::register_with_engine(&mut engine).unwrap();

    // Start the engine
    engine.initialize().unwrap();
    engine.start().unwrap();

    // Create a document (command — short, synchronous)
    let create_result = engine.command_executor().execute(&Command::new(
        "document.create",
        CommandContext::new(ExecutionId::new()),
        CommandInput::new(project::CreateDocumentInput {
            title: "My First Document".to_string(),
        }),
    )).unwrap();

    let doc_id = create_result.output::<String>().unwrap();
    println!("Created document: {}", doc_id);

    // Export the project (task — long, managed, cancellable)
    let task_id = engine.task_manager_mut().create(
        "project.export",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new(project::ExportProjectInput {
            project_id: "proj_1".to_string(),
            format: "pdf".to_string(),
            output_path: "/tmp/export.pdf".to_string(),
        }),
    ).unwrap();

    let export_result = engine.task_manager_mut().start(task_id).unwrap();
    let result: &project::ExportResult = export_result.output::<project::ExportResult>().unwrap();
    println!("Exported {} pages to {}", result.page_count, result.output_path);

    // Load a document resource
    let handle = engine.resource_manager_mut().load(
        "document",
        "file:///docs/my_doc.dp",
    ).unwrap();

    let resource = engine.resource_manager().get(&handle).unwrap();
    let doc: &project::DocumentData = resource.data::<project::DocumentData>().unwrap();
    println!("Loaded document: {}", doc.title);

    engine.stop().unwrap();
    engine.dispose().unwrap();
}
```

---

## Integration Summary

```text
Domain Engine                      App Engine
─────────────                      ──────────

Domain Function                →   wrapped by CommandHandler
  create_document()

Domain Function                →   wrapped by TaskHandler
  export_project()

Domain Resource Type          →   loaded by ResourceLoader
  DocumentData

Domain Service                →   implements Service trait
  AutosaveService

Domain Event Payload          →   published via EventBus
  DocumentChangedPayload

Domain Command Input          →   passed via CommandInput
  ExportProjectInput

Domain Task Input             →   passed via TaskInput
  ExportProjectInput
```

Each domain concept maps to exactly one App Engine contract. The Domain Engine never reaches into App Engine internals — it uses only the public API.

---

## Checklist: Am I Integrating Correctly?

| Question | Correct Answer |
|---|---|
| Does the Domain Engine import App Engine types? | Yes (commands, tasks, events, resources, services) |
| Does the App Engine import Domain Engine types? | **No** — never |
| Are domain functions called directly from UI/API? | No — through commands/tasks |
| Are domain objects stored in TaskContext? | No — in TaskInput (type-erased) |
| Are domain events published through EventBus? | Yes |
| Are domain resources loaded through ResourceManager? | Yes |
| Is domain configuration stored in ConfigStore? | Yes — with domain-specific keys |
| Is domain history recorded through HistoryStore? | Yes — via event handlers |

---

## Next Steps

- [10-application-integration.md](10-application-integration.md) — UI, API, and MCP integration on top of both engines.
- [05-commands-and-tasks.md](05-commands-and-tasks.md) — Deep dive on commands and tasks.
- [08-services-and-plugins.md](08-services-and-plugins.md) — Services and plugins.