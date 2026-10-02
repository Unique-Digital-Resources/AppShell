# `03-bootstrap-and-lifecycle.md` — Creating and Running an App

## Overview

Every application starts the same way: create an `AppEngine`, transition it through its lifecycle, and shut it down cleanly. This chapter shows how.

```text
Bootstrap::create()
       ↓
    AppEngine (Created)
       ↓
    initialize()
       ↓
    AppEngine (Initialized)
       ↓
    start()
       ↓
    AppEngine (Running)
       ↓
    run()
       ↓
    stop()
       ↓
    AppEngine (Stopped)
       ↓
    dispose()
       ↓
    AppEngine (Disposed)
```

---

## Creating an AppEngine

The entry point is `Bootstrap::create()`:

```rust
use app_shell::app_engine::{AppEngine, Bootstrap, Lifecycle};

let mut engine: AppEngine = Bootstrap::create().expect("bootstrap failed");
```

`Bootstrap` wires all subsystems together:

```text
Bootstrap
    ↓
create ConfigStore (with schema + defaults)
    ↓
create ContextManager (with ApplicationContext)
    ↓
create StateManager (with runtime state slot)
    ↓
create all subsystems
    ↓
return AppEngine
```

At construction, the engine is in the `Created` state:

```rust
assert_eq!(engine.state(), RuntimeState::Created);
```

### Custom Application Identity

If you need a specific application ID:

```rust
let engine = Bootstrap::create_with_context(
    ApplicationContext::new("my_app")
)?;
```

---

## The Lifecycle Trait

`AppEngine` implements the `Lifecycle` trait:

```rust
pub trait Lifecycle {
    fn initialize(&mut self) -> Result<(), EngineError>;
    fn start(&mut self) -> Result<(), EngineError>;
    fn stop(&mut self) -> Result<(), EngineError>;
    fn dispose(&mut self) -> Result<(), EngineError>;
    fn state(&self) -> LifecycleState;
}
```

Every method returns `Result<(), EngineError>` — lifecycle failures are explicit, not silent.

---

## Initialization

```rust
engine.initialize()?;
```

What happens during `initialize()`:

```text
1. Runtime state transitions: Created → Initialized
2. StateManager updated
3. ServiceManager.initialize_all()  (services before plugins)
4. PluginManager.initialize_all()   (plugins after services)
```

Services and plugins receive their context during initialization. The ordering is intentional: services initialize first because plugins may depend on them.

After initialization:

```rust
assert_eq!(engine.state(), RuntimeState::Initialized);
```

### Invalid Transitions

```rust
let mut engine = Bootstrap::create()?;

engine.initialize()?;                           // OK
engine.initialize()?;                           // Error: already initialized
engine.start()?;                               // OK (from Initialized)
engine.initialize()?;                          // Error: already started
```

Invalid transitions return `EngineError`:

```rust
match engine.initialize() {
    Ok(()) => { /* lifecycle advanced */ }
    Err(EngineError::InvalidLifecycleTransition { from, to }) => {
        eprintln!("Can't transition from {} to {}", from, to);
    }
    Err(e) => {
        eprintln!("Initialization failed: {}", e);
    }
}
```

---

## Startup

```rust
engine.start()?;
```

What happens during `start()`:

```text
1. Runtime state transitions: Initialized → Running
2. StateManager updated
3. stop_requested = false
4. ServiceManager.start_all()  (services before plugins)
5. PluginManager.start_all()   (plugins after services)
```

After startup, the engine is `Running`:

```rust
assert_eq!(engine.state(), RuntimeState::Running);
```

All registered services and plugins are now active.

---

## Running

```rust
engine.run()?;
```

`run()` is an **operation while Running**, not a state transition. It validates that the engine is in the `Running` state and returns.

For applications with a continuous main loop:

```rust
engine.start()?;

while !engine.stop_requested() {
    // Process scheduler (move eligible jobs to queue)
    engine.scheduler_mut().tick();

    // Dispatch ready jobs
    while let Some(job) = engine.scheduler_mut().dispatch() {
        let result = engine.task_manager_mut().start(job.task_id())?;
        // Handle result...
    }

    // Small sleep to avoid busy-waiting
    std::thread::sleep(std::time::Duration::from_millis(10));
}

engine.stop()?;
```

For simpler applications, `run()` can just return immediately:

```rust
engine.start()?;
engine.run()?;  // no-op for Phase 1; real work happens via commands/tasks
engine.stop()?;
```

---

## Shutdown

```rust
engine.stop()?;
```

What happens during `stop()`:

```text
1. stop_requested = true
2. PluginManager.stop_all()   (plugins first — reverse startup order)
3. ServiceManager.stop_all()  (services second)
4. Runtime state transitions: Running → Stopped
5. StateManager updated
```

The reverse ordering is critical: plugins stop before services because plugins may depend on services being available during their shutdown.

---

## Disposal

```rust
engine.dispose()?;
```

What happens during `dispose()`:

```text
1. PluginManager.dispose_all()  (plugins first)
2. ServiceManager.dispose_all() (services second)
3. Runtime state transitions: Stopped → Disposed
4. StateManager updated
```

After disposal, the engine is terminal:

```rust
assert_eq!(engine.state(), RuntimeState::Disposed);
```

No further lifecycle operations are valid:

```rust
engine.initialize()?;  // Error: disposed
engine.start()?;       // Error: disposed
engine.stop()?;        // Error: disposed
engine.dispose()?;     // Error: disposed
```

---

## Complete Lifecycle Example

```rust
use app_shell::app_engine::{AppEngine, Bootstrap, Lifecycle, RuntimeState};

fn main() {
    let mut engine = Bootstrap::create().expect("bootstrap failed");

    // Created
    assert_eq!(engine.state(), RuntimeState::Created);

    // Initialize
    engine.initialize().expect("initialize failed");
    assert_eq!(engine.state(), RuntimeState::Initialized);

    // Start
    engine.start().expect("start failed");
    assert_eq!(engine.state(), RuntimeState::Running);

    // Run (operation while running)
    engine.run().expect("run failed");
    assert_eq!(engine.state(), RuntimeState::Running);

    // Stop
    engine.stop().expect("stop failed");
    assert_eq!(engine.state(), RuntimeState::Stopped);

    // Dispose
    engine.dispose().expect("dispose failed");
    assert_eq!(engine.state(), RuntimeState::Disposed);

    println!("Lifecycle complete.");
}
```

---

## Lifecycle with Subsystems

When services and plugins are registered before `initialize()`, they participate automatically:

```rust
use app_shell::app_engine::{
    AppEngine, Bootstrap, Lifecycle,
    Service, ServiceDefinition, ServiceError, ServiceState,
    Plugin, PluginContext, PluginError, PluginManifest, PluginState,
};
use std::sync::{Arc, Mutex};

// --- A simple service ---

struct LoggerService {
    active: Arc<Mutex<bool>>,
}

impl Service for LoggerService {
    fn service_type(&self) -> &str { "logger" }
    fn initialize(&mut self) -> Result<(), ServiceError> { Ok(()) }
    fn start(&mut self) -> Result<(), ServiceError> {
        *self.active.lock().unwrap() = true;
        println!("[Logger] Started");
        Ok(())
    }
    fn stop(&mut self) -> Result<(), ServiceError> {
        *self.active.lock().unwrap() = false;
        println!("[Logger] Stopped");
        Ok(())
    }
    fn dispose(&mut self) -> Result<(), ServiceError> {
        println!("[Logger] Disposed");
        Ok(())
    }
}

// --- A simple plugin ---

struct AnalyticsPlugin {
    active: Arc<Mutex<bool>>,
}

impl Plugin for AnalyticsPlugin {
    fn id(&self) -> &str { "analytics" }
    fn initialize(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        println!("[Analytics] Initialized");
        Ok(())
    }
    fn start(&mut self) -> Result<(), PluginError> {
        *self.active.lock().unwrap() = true;
        println!("[Analytics] Started");
        Ok(())
    }
    fn stop(&mut self) -> Result<(), PluginError> {
        *self.active.lock().unwrap() = false;
        println!("[Analytics] Stopped");
        Ok(())
    }
    fn dispose(&mut self) -> Result<(), PluginError> {
        println!("[Analytics] Disposed");
        Ok(())
    }
}

fn main() {
    let mut engine = Bootstrap::create().expect("bootstrap failed");

    let logger_active = Arc::new(Mutex::new(false));
    let analytics_active = Arc::new(Mutex::new(false));

    // Register before lifecycle starts
    engine.service_manager_mut().register(
        ServiceDefinition::new("logger", "Logger", "Logging service"),
        Box::new(LoggerService { active: logger_active.clone() }),
    ).unwrap();

    engine.plugin_manager_mut().register(
        PluginManifest::new("analytics", "Analytics Plugin"),
        Box::new(AnalyticsPlugin { active: analytics_active.clone() }),
    ).unwrap();

    // Lifecycle automatically manages subsystems
    engine.initialize().unwrap();
    // Output: [Logger] Initialized  (service first)
    //         [Analytics] Initialized (plugin second)

    engine.start().unwrap();
    // Output: [Logger] Started
    //         [Analytics] Started
    assert!(*logger_active.lock().unwrap());
    assert!(*analytics_active.lock().unwrap());

    engine.stop().unwrap();
    // Output: [Analytics] Stopped  (plugin first — reverse order)
    //         [Logger] Stopped     (service second)
    assert!(!*logger_active.lock().unwrap());
    assert!(!*analytics_active.lock().unwrap());

    engine.dispose().unwrap();
    // Output: [Analytics] Disposed
    //         [Logger] Disposed
}
```

---

## Lifecycle Ordering Summary

### Startup (forward)

```text
1. Runtime: Created → Initialized
2. Services: Registered → Initialized
3. Plugins: Loaded → Initialized
4. Runtime: Initialized → Running
5. Services: Initialized → Running
6. Plugins: Initialized → Started
```

### Shutdown (reverse)

```text
1. Plugins: Started → Stopped
2. Services: Running → Stopped
3. Runtime: Running → Stopped
4. Plugins: Stopped → Unloaded
5. Services: Stopped → Disposed
6. Runtime: Stopped → Disposed
```

---

## Configuration During Bootstrap

Configuration is available from the moment the engine is created:

```rust
use app_shell::app_engine::{
    AppEngine, Bootstrap, ConfigPropertyDefinition, ConfigSchema, ConfigStore, Lifecycle,
};

fn main() {
    // Option A: Default empty configuration
    let mut engine = Bootstrap::create().unwrap();
    
    // Option B: Define schema before construction
    let schema = ConfigSchema::new()
        .with_property(
            ConfigPropertyDefinition::new("app.name", "Application name")
                .with_default("MyApp")
                .required(),
        )
        .with_property(
            ConfigPropertyDefinition::new("worker_count", "Worker threads")
                .with_default("4")
                .with_min("1")
                .with_max("32"),
        );
    
    // The ConfigStore is created internally with defaults
    // Access it after construction:
    engine.config_store().get("app.name");  // None (no schema was set)
    
    // Set configuration values during or after initialization:
    engine.config_store_mut().set("app.name", "MyApp").unwrap();
    engine.config_store_mut().set("worker_count", "8").unwrap();
    
    // Services can read configuration during their initialize():
    // (The service would access config_store through the runtime)
    
    engine.initialize().unwrap();
    engine.start().unwrap();
    
    // ...
    
    engine.stop().unwrap();
    engine.dispose().unwrap();
}
```

For applications that need to load configuration from a file before construction, the pattern is:

```rust
// 1. Load configuration from file (application-specific)
// 2. Create ConfigSchema
// 3. Create ConfigStore with schema
// 4. Bootstrap the engine
// 5. Merge loaded config into engine's config store

let mut engine = Bootstrap::create()?;

// Load and merge configuration
let mut loaded_config = Config::new();
loaded_config.set("app.name", "MyApp");
loaded_config.set("worker_count", "8");
engine.config_store_mut().save(loaded_config)?;

engine.initialize()?;
engine.start()?;
```

---

## State Visibility

The runtime's lifecycle state is visible through two systems:

```rust
// Direct accessor
assert_eq!(engine.state(), RuntimeState::Running);

// Through StateManager (Phase 2 integration)
let state_id = engine.runtime_state_id();
assert_eq!(
    engine.state_manager().current(state_id),
    Some(LifecycleState::Running)
);
```

Both reflect the same state — the `StateManager` mirrors lifecycle transitions so that external systems (plugins, services) can observe the runtime's condition.

---

## Error Handling

```rust
use app_shell::app_engine::{AppEngine, Bootstrap, EngineError, Lifecycle};

let mut engine = Bootstrap::create()?;

// All lifecycle methods return Result<(), EngineError>
match engine.initialize() {
    Ok(()) => println!("Initialized"),
    Err(EngineError::AlreadyInitialized) => {
        eprintln!("Already initialized — continuing");
    }
    Err(EngineError::InvalidLifecycleTransition { from, to }) => {
        eprintln!("Invalid: {} -> {}", from, to);
    }
    Err(EngineError::SubsystemFailure { subsystem, reason }) => {
        eprintln!("{} failed: {}", subsystem, reason);
        // A service or plugin failed during initialization
    }
    Err(e) => {
        eprintln!("Unexpected: {}", e);
    }
}
```

Subsystem failures (service/plugin initialization, startup, etc.) are wrapped in `EngineError::SubsystemFailure` with the subsystem name and reason:

```text
service subsystem failure: Service 3 failed to start: database unreachable
plugin subsystem failure: Plugin 5 failed to initialize: dependency 'core' not met
```

---

## Complete Application Shell

```rust
use app_shell::app_engine::{AppEngine, Bootstrap, Lifecycle, RuntimeState};

fn main() {
    let mut engine: AppEngine = match Bootstrap::create() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to create engine: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = engine.initialize() {
        eprintln!("Initialization failed: {}", e);
        std::process::exit(1);
    }

    if let Err(e) = engine.start() {
        eprintln!("Startup failed: {}", e);
        // Try to dispose before exiting
        let _ = engine.dispose();
        std::process::exit(1);
    }

    // Main loop
    engine.run().unwrap();

    // Shutdown
    if let Err(e) = engine.stop() {
        eprintln!("Shutdown warning: {}", e);
    }

    if let Err(e) = engine.dispose() {
        eprintln!("Disposal warning: {}", e);
    }

    println!("Engine state: {:?}", engine.state());
}
```

---

## Quick Reference

| Method | From state | To state | Also does |
|---|---|---|---|
| `Bootstrap::create()` | — | `Created` | Wires all subsystems |
| `initialize()` | `Created` | `Initialized` | Init services → Init plugins |
| `start()` | `Initialized` / `Stopped` | `Running` | Start services → Start plugins |
| `run()` | `Running` | `Running` | Validates running state |
| `stop()` | `Running` | `Stopped` | Stop plugins → Stop services |
| `dispose()` | `Stopped` | `Disposed` | Dispose plugins → Dispose services |

---

## Next Steps

- [04-context-and-state.md](04-context-and-state.md) — Execution context and state management.
- [05-commands-and-tasks.md](05-commands-and-tasks.md) — Registering and executing work.
- [08-services-and-plugins.md](08-services-and-plugins.md) — Registering services and plugins with the lifecycle.