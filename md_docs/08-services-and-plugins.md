# `08-services-and-plugins.md` — Long-Lived Capabilities & Extensions

## Overview

Two systems provide extensibility at different levels:

```text
Service = long-lived runtime capability (starts with app, stays running)
Plugin  = extension package (contributes commands, tasks, resources, services, handlers)
```

They are complementary:

```text
Plugin
 ├── may provide → Service
 ├── may provide → Command
 ├── may provide → Task
 ├── may provide → Resource
 ├── may provide → EventHandler
 └── may provide → Signal subscription
```

A plugin is the **package**. A service is one **capability** inside it.

---

## Part 1: Services

### What Is a Service?

```text
Task    = finite work instance (has a start and end)
Service = long-lived capability (starts with app, stays alive, stops with app)
```

| | Task | Service |
|---|---|---|
| **Lifetime** | Finite — Pending → Completed | Long — starts with app, stays running |
| **Created by** | Command or direct call | Registration before `engine.start()` |
| **Lifecycle** | Task lifecycle (6 states) | Service lifecycle (5 states) |
| **Example** | Export project | Autosave service |

### The Service Trait

```rust
use app_shell::app_engine::{Service, ServiceError};

struct AutosaveService {
    active: bool,
}

impl Service for AutosaveService {
    fn service_type(&self) -> &str { "autosave" }

    fn initialize(&mut self) -> Result<(), ServiceError> {
        println!("[Autosave] Initializing");
        Ok(())
    }

    fn start(&mut self) -> Result<(), ServiceError> {
        self.active = true;
        println!("[Autosave] Started — monitoring for changes");
        Ok(())
    }

    fn stop(&mut self) -> Result<(), ServiceError> {
        self.active = false;
        println!("[Autosave] Stopped");
        Ok(())
    }

    fn dispose(&mut self) -> Result<(), ServiceError> {
        println!("[Autosave] Disposed");
        Ok(())
    }
}
```

### Service Registration

```rust
use app_shell::app_engine::ServiceDefinition;

let service_id = engine.service_manager_mut().register(
    ServiceDefinition::new("autosave", "Autosave", "Automatic document saving")
        .with_metadata("interval", "300"),
    Box::new(AutosaveService { active: false }),
)?;

assert_eq!(engine.service_manager().count(), 1);
assert!(engine.service_manager().contains_type("autosave"));
assert_eq!(
    engine.service_manager().state(service_id),
    Some(ServiceState::Registered),
);
```

### Service Lifecycle

```text
Registered → Initialized → Running → Stopped → Disposed
```

```rust
// Individual lifecycle
engine.service_manager_mut().initialize(service_id)?;
assert_eq!(engine.service_manager().state(service_id), Some(ServiceState::Initialized));

engine.service_manager_mut().start(service_id)?;
assert_eq!(engine.service_manager().state(service_id), Some(ServiceState::Running));

engine.service_manager_mut().stop(service_id)?;
assert_eq!(engine.service_manager().state(service_id), Some(ServiceState::Stopped));

engine.service_manager_mut().dispose(service_id)?;
assert_eq!(engine.service_manager().state(service_id), Some(ServiceState::Disposed));
```

### Automatic Lifecycle (Via Engine)

Services participate automatically in the engine's lifecycle:

```rust
// Register services BEFORE calling engine.initialize()
engine.service_manager_mut().register(
    ServiceDefinition::new("autosave", "Autosave", "Autosave"),
    Box::new(AutosaveService { active: false }),
)?;
engine.service_manager_mut().register(
    ServiceDefinition::new("worker", "Background Worker", "Background processing"),
    Box::new(BackgroundWorkerService { active: false }),
)?;

// Engine lifecycle manages services automatically:
engine.initialize()?;  // → initialize_all() (services → Initialized)
engine.start()?;       // → start_all() (services → Running)

// ... application runs — services are active ...

engine.stop()?;        // → stop_all() (services → Stopped, reverse order)
engine.dispose()?;     // → dispose_all() (services → Disposed, reverse order)
```

### Service State Validation

```rust
// Can't start before initialize
assert!(engine.service_manager_mut().start(service_id).is_err());

engine.service_manager_mut().initialize(service_id)?;

// Can't initialize twice
assert!(engine.service_manager_mut().initialize(service_id).is_err());

engine.service_manager_mut().start(service_id)?;

// Can't dispose while running (must stop first)
assert!(engine.service_manager_mut().dispose(service_id).is_err());
```

### Service: Autosave Example

A real autosave service that reacts to events and schedules tasks:

```rust
use app_shell::app_engine::*;
use std::sync::{Arc, Mutex};

struct AutosaveService {
    save_count: Arc<Mutex<usize>>,
}

impl Service for AutosaveService {
    fn service_type(&self) -> &str { "autosave" }

    fn initialize(&mut self) -> Result<(), ServiceError> {
        Ok(())
    }

    fn start(&mut self) -> Result<(), ServiceError> {
        *self.save_count.lock().unwrap() = 0;
        // In a real service, this would subscribe to "document.changed" events
        // and schedule backup tasks via the scheduler.
        Ok(())
    }

    fn stop(&mut self) -> Result<(), ServiceError> {
        // Final save on shutdown
        Ok(())
    }

    fn dispose(&mut self) -> Result<(), ServiceError> {
        Ok(())
    }
}
```

### Service vs Task

```text
AutosaveService (long-lived)
       │
       │ detects document change (via event)
       ▼
     Command
       │
       ▼
     Task (finite — one save operation)
       │
       ▼
   TaskExecutor → domain save function
       │
       ▼
   TaskResult
```

The service doesn't perform the save itself. It **coordinates**: detects when a save should happen and creates/schedules a task to do it.

### Service Dependencies

Services can depend on other services or App Engine systems:

```text
BackgroundWorkerService
       │
       ├── uses Scheduler (to dispatch work)
       ├── uses ResourceManager (to acquire data)
       └── uses EventBus (to publish results)
```

Prefer services that depend on App Engine capabilities rather than services depending on other services:

```text
✓ Service → Scheduler / EventBus / ResourceManager
✗ Service A → Service B → Service C → Service D  (chain)
```

If shared behavior is needed, it should live in an **App Engine subsystem**, not be chained through services.

---

## Part 2: Plugins

### What Is a Plugin?

```text
Service = one long-lived capability
Plugin  = an extension package that may provide multiple capabilities
```

A plugin can contribute:

```text
ImagePlugin
 ├── Commands (ImportImage, ExportImage)
 ├── Tasks (RenderImage)
 ├── Resources (ImageResource, ImageLoader)
 ├── Services (ImageCacheService)
 └── Event handlers (subscribe to DocumentChanged)
```

### The Plugin Trait

```rust
use app_shell::app_engine::{Plugin, PluginContext, PluginError};

struct ImagePlugin {
    initialized: bool,
}

impl Plugin for ImagePlugin {
    fn id(&self) -> &str { "image-plugin" }

    fn initialize(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        println!("[ImagePlugin] Initializing (context: {})", ctx.plugin_id());
        self.initialized = true;
        Ok(())
    }

    fn start(&mut self) -> Result<(), PluginError> {
        println!("[ImagePlugin] Started");
        Ok(())
    }

    fn stop(&mut self) -> Result<(), PluginError> {
        println!("[ImagePlugin] Stopped");
        Ok(())
    }

    fn dispose(&mut self) -> Result<(), PluginError> {
        println!("[ImagePlugin] Disposed");
        Ok(())
    }
}
```

### Plugin Manifest

A manifest describes a plugin without loading its implementation:

```rust
use app_shell::app_engine::PluginManifest;

let manifest = PluginManifest::new("image-plugin", "Image Plugin")
    .with_version("2.0.0")
    .with_description("Image import, export, and processing")
    .with_author("Alice")
    .with_dependency("core-plugin")    // requires another plugin
    .with_capability("commands")       // provides commands
    .with_capability("resources")      // provides resource loaders
    .with_capability("services")      // provides services
    .with_metadata("license", "MIT");
```

### Plugin Registration

```rust
let plugin_id = engine.plugin_manager_mut().register(
    PluginManifest::new("image-plugin", "Image Plugin"),
    Box::new(ImagePlugin { initialized: false }),
)?;

assert_eq!(engine.plugin_manager().count(), 1);
assert!(engine.plugin_manager().contains_type("image-plugin"));
assert_eq!(
    engine.plugin_manager().state(plugin_id),
    Some(PluginState::Loaded),
);
```

### Plugin Lifecycle

```text
Discovered → Loaded → Initialized → Started → Stopped → Unloaded
```

```rust
engine.plugin_manager_mut().initialize(plugin_id)?;
assert_eq!(engine.plugin_manager().state(plugin_id), Some(PluginState::Initialized));

engine.plugin_manager_mut().start(plugin_id)?;
assert_eq!(engine.plugin_manager().state(plugin_id), Some(PluginState::Started));

engine.plugin_manager_mut().stop(plugin_id)?;
assert_eq!(engine.plugin_manager().state(plugin_id), Some(PluginState::Stopped));

engine.plugin_manager_mut().dispose(plugin_id)?;
assert_eq!(engine.plugin_manager().state(plugin_id), Some(PluginState::Unloaded));
```

### Automatic Lifecycle (Via Engine)

Plugins participate in the engine's lifecycle, **after** services:

```rust
// Register plugins BEFORE engine.initialize()
engine.plugin_manager_mut().register(
    PluginManifest::new("image-plugin", "Image Plugin"),
    Box::new(ImagePlugin { initialized: false }),
)?;

engine.initialize()?;  // → init services, then init plugins
engine.start()?;       // → start services, then start plugins

// ... application runs ...

engine.stop()?;        // → stop plugins first, then stop services
engine.dispose()?;     // → dispose plugins first, then dispose services
```

### PluginContext

The `PluginContext` is the controlled environment a plugin receives during initialization:

```rust
impl Plugin for ImagePlugin {
    fn initialize(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        // The context provides the plugin's identity
        assert_eq!(ctx.plugin_id(), "image-plugin");

        // The context provides engine metadata
        // (In future phases, it can provide controlled access to
        //  commands, events, resources, etc.)

        Ok(())
    }
    // ...
}
```

### Plugin Dependencies

```rust
// Plugin "a" depends on plugin "b"
let a_manifest = PluginManifest::new("a", "A")
    .with_dependency("b");

engine.plugin_manager_mut().register(a_manifest, Box::new(PluginA))?;

// "b" is not registered → validation fails
assert!(engine.plugin_manager_mut().validate_all_dependencies().is_err());

// Register "b"
engine.plugin_manager_mut().register(
    PluginManifest::new("b", "B"),
    Box::new(PluginB),
)?;

// Now validation passes
engine.plugin_manager_mut().validate_all_dependencies()?;
```

`initialize_all()` validates dependencies before initializing:

```rust
// If dependencies are not met, initialize_all() fails:
engine.plugin_manager_mut().initialize_all()
    .expect_err("dependency not met");
```

### Plugin Loading via Loaders

For dynamically loaded plugins (future: dynamic libraries, WASM, etc.):

```rust
use app_shell::app_engine::PluginLoader;

struct StaticLoader;

impl PluginLoader for StaticLoader {
    fn can_load(&self, source: &str) -> bool {
        source.starts_with("static:")
    }

    fn load(&self, source: &str) -> Result<(PluginManifest, Box<dyn Plugin>), PluginError> {
        let id = source.strip_prefix("static:").unwrap_or(source);
        Ok((
            PluginManifest::new(id, id),
            Box::new(ImagePlugin { initialized: false }),
        ))
    }
}

// Register the loader
engine.plugin_manager_mut().register_loader(Box::new(StaticLoader));

// Load a plugin from a "source" string
let plugin_id = engine.plugin_manager_mut().load("static:image-plugin")?;
assert!(engine.plugin_manager().contains(plugin_id));
```

The loading mechanism is independent of the plugin contract — future dynamic loaders use the same `PluginLoader` trait.

---

## Plugin → App Engine Integration

A plugin contributes to the engine through established contracts. Here's a complete example:

```rust
use app_shell::app_engine::*;
use std::any::Any;
use std::sync::{Arc, Mutex};

// --- Plugin-provided command handler ---

struct ImportImageHandler;
impl CommandHandler for ImportImageHandler {
    fn execute(&self, input: &CommandInput, _ctx: &CommandContext) -> Result<CommandResult, CommandError> {
        let path: &String = input.get::<String>()
            .ok_or_else(|| CommandError::InvalidArguments("expected path".to_string()))?;
        Ok(CommandResult::success_with(format!("imported:{}", path)))
    }
}

// --- Plugin-provided resource loader ---

struct ImageResourceLoader;
impl ResourceLoader for ImageResourceLoader {
    fn resource_type(&self) -> &str { "image" }
    fn load(&self, source: &str) -> Result<Box<dyn Any + Send>, ResourceError> {
        Ok(Box::new(format!("pixels:{}", source)))
    }
}

// --- Plugin-provided service ---

struct ImageCacheService;
impl Service for ImageCacheService {
    fn service_type(&self) -> &str { "image_cache" }
    fn initialize(&mut self) -> Result<(), ServiceError> { Ok(()) }
    fn start(&mut self) -> Result<(), ServiceError> { Ok(()) }
    fn stop(&mut self) -> Result<(), ServiceError> { Ok(()) }
    fn dispose(&mut self) -> Result<(), ServiceError> { Ok(()) }
}

// --- Plugin-provided task handler ---

struct RenderImageTask;
impl TaskHandler for RenderImageTask {
    fn execute(&self, input: &TaskInput, _ctx: &TaskContext) -> Result<TaskResult, TaskError> {
        let path: &String = input.get::<String>()
            .ok_or_else(|| TaskError::ExecutionFailed { id: 0, reason: "expected path".into() })?;
        Ok(TaskResult::completed_with(format!("rendered:{}", path)))
    }
}

// --- The plugin itself ---

struct ImagePlugin;

impl Plugin for ImagePlugin {
    fn id(&self) -> &str { "image-plugin" }

    fn initialize(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        println!("[ImagePlugin] init — context id: {}", ctx.plugin_id());
        Ok(())
    }

    fn start(&mut self) -> Result<(), PluginError> {
        // In a real plugin, the start() method would register commands,
        // resource loaders, services, and task handlers with the engine.
        // The engine reference would need to be provided to the plugin
        // (e.g., through an expanded PluginContext or by passing the
        // engine to the plugin's start method).
        Ok(())
    }

    fn stop(&mut self) -> Result<(), PluginError> { Ok(()) }
    fn dispose(&mut self) -> Result<(), PluginError> { Ok(()) }
}

// --- Setup ---

fn main() {
    let mut engine = Bootstrap::create().unwrap();

    // Register the plugin
    let pid = engine.plugin_manager_mut().register(
        PluginManifest::new("image-plugin", "Image Plugin")
            .with_capability("commands")
            .with_capability("resources")
            .with_capability("services")
            .with_capability("tasks"),
        Box::new(ImagePlugin),
    ).unwrap();

    // Initialize and start the plugin
    engine.plugin_manager_mut().initialize(pid).unwrap();
    engine.plugin_manager_mut().start(pid).unwrap();

    // Register the plugin's contributions with the engine
    engine.command_executor_mut().register(
        CommandDefinition::new("image.import", "Import Image", "Import an image file"),
        Box::new(ImportImageHandler),
    ).unwrap();

    engine.resource_manager_mut().register_loader(Box::new(ImageResourceLoader));

    engine.service_manager_mut().register(
        ServiceDefinition::new("image_cache", "Image Cache", "Image cache service"),
        Box::new(ImageCacheService),
    ).unwrap();

    engine.task_manager_mut().register(
        TaskDefinition::new("image.render", "Render Image", "Render an image"),
        Box::new(RenderImageTask),
    ).unwrap();

    // Start the engine
    engine.initialize().unwrap();
    engine.start().unwrap();

    // Use the plugin's command
    let result = engine.command_executor().execute(&Command::new(
        "image.import",
        CommandContext::new(ExecutionId::new()),
        CommandInput::new("/assets/logo.png".to_string()),
    )).unwrap();
    assert_eq!(result.output::<String>(), Some(&"imported:/assets/logo.png".to_string()));

    // Use the plugin's resource
    let handle = engine.resource_manager_mut().load("image", "file:///logo.png").unwrap();
    let resource = engine.resource_manager().get(&handle).unwrap();
    assert_eq!(resource.data::<String>(), Some(&"pixels:file:///logo.png".to_string()));

    engine.stop().unwrap();
    engine.dispose().unwrap();
}
```

The App Engine never knew that `ImportImageHandler` came from a plugin. It just saw a `CommandHandler` registered with the `CommandExecutor`.

---

## When to Create a Service vs a Plugin vs a New Subsystem

### When to Use a Service

```text
✓ Long-lived capability that starts/stops with the app
✓ Coordinates other systems (scheduler, events, resources)
✓ Does NOT contain the core domain logic
```

Example: `AutosaveService` — watches for changes, schedules save tasks. Doesn't implement saving itself.

### When to Use a Plugin

```text
✓ Extension package that bundles multiple capabilities
✓ Can be added/removed independently
✓ May provide commands, tasks, resources, services, and event handlers
```

Example: `ImagePlugin` — bundles image import/export commands, image resource loader, image cache service, and render task.

### When to Create a New App Engine Subsystem

```text
✓ The behavior is fundamental to all applications
✓ Every domain needs it, not just one
✓ It has its own complex internal architecture
```

If something starts as a service and grows to have its own managers, registries, loaders, and lifecycle — it may need to become a subsystem.

```text
Plugin → Service → Subsystem (growth path)

Example:
  ImageCacheService (simple)
    → grows into
  ResourceManager (full subsystem with loaders, caches, handles)
```

| Scale | Solution |
|---|---|
| One capability, short code | Service |
| Bundle of related capabilities | Plugin |
| Complex internal architecture used by all apps | App Engine subsystem |

---

## Quick Reference

### Service API

```rust
// Register
let id = engine.service_manager_mut().register(definition, service)?;

// Lifecycle
engine.service_manager_mut().initialize(id)?;
engine.service_manager_mut().start(id)?;
engine.service_manager_mut().stop(id)?;
engine.service_manager_mut().dispose(id)?;

// Bulk (called automatically by engine lifecycle)
engine.service_manager_mut().initialize_all()?;
engine.service_manager_mut().start_all()?;
engine.service_manager_mut().stop_all()?;
engine.service_manager_mut().dispose_all()?;

// Query
engine.service_manager().get(id);                 // Option<&dyn Service>
engine.service_manager().state(id);               // Option<ServiceState>
engine.service_manager().contains_type("type");   // bool
engine.service_manager().get_definition(id);      // Option<&ServiceDefinition>
engine.service_manager().count();                 // usize
```

### Plugin API

```rust
// Register
let id = engine.plugin_manager_mut().register(manifest, plugin)?;

// Loader
engine.plugin_manager_mut().register_loader(loader);
let id = engine.plugin_manager_mut().load("source")?;

// Lifecycle
engine.plugin_manager_mut().initialize(id)?;
engine.plugin_manager_mut().start(id)?;
engine.plugin_manager_mut().stop(id)?;
engine.plugin_manager_mut().dispose(id)?;

// Bulk (called automatically by engine lifecycle)
engine.plugin_manager_mut().initialize_all()?;
engine.plugin_manager_mut().start_all()?;
engine.plugin_manager_mut().stop_all()?;
engine.plugin_manager_mut().dispose_all()?;

// Dependencies
engine.plugin_manager_mut().validate_dependencies(id)?;
engine.plugin_manager_mut().validate_all_dependencies()?;

// Query
engine.plugin_manager().get(id);                  // Option<&dyn Plugin>
engine.plugin_manager().state(id);               // Option<PluginState>
engine.plugin_manager().contains_type("id");     // bool
engine.plugin_manager().get_manifest(id);        // Option<&PluginManifest>
engine.plugin_manager().count();                 // usize
```

---

## Lifecycle Ordering Summary

```text
Startup:
  Engine.initialize()
    ↓
  ServiceManager.initialize_all()  ← services first
    ↓
  PluginManager.initialize_all()   ← plugins second (may depend on services)
    ↓
  Engine.start()
    ↓
  ServiceManager.start_all()       ← services first
    ↓
  PluginManager.start_all()        ← plugins second

Shutdown (reverse):
  Engine.stop()
    ↓
  PluginManager.stop_all()         ← plugins first
    ↓
  ServiceManager.stop_all()        ← services second
    ↓
  Engine.dispose()
    ↓
  PluginManager.dispose_all()      ← plugins first
    ↓
  ServiceManager.dispose_all()     ← services second
```

---

## Next Steps

- [09-domain-engine-integration.md](09-domain-engine-integration.md) — How the Domain Engine uses all of these systems.
- [10-application-integration.md](10-application-integration.md) — UI, API, and MCP integration.