# `17-plugin-security.md` — Plugin Security

## Overview

The plugin security system controls what plugins can access and do within the App Engine. It consists of two layers:

```text
PluginPermissions              PermissionCheckedEngineRef
(bitflags on manifest)         (runtime enforcement wrapper)
    ↓                              ↓
"Can this plugin               "try_load_resource()? → Err if no permission"
register commands?"              
```

- **PluginPermissions** — declared in the manifest, queried by the manager
- **PermissionCheckedEngineRef** — wraps `EngineRef` with `try_*` methods that return `Result`

Default permissions are `FULL_ACCESS` — backward compatible.

---

## Part 1: Permission Flags (Phase 17)

### All Permissions

```rust
use app_shell::app_engine::PluginPermissions;
use app_shell::app_engine::{
    REGISTER_COMMANDS,
    REGISTER_TASKS,
    LOAD_RESOURCES,
    REGISTER_SERVICES,
    SUBSCRIBE_EVENTS,
    PUBLISH_EVENTS,
    READ_CONFIG,
    WRITE_CONFIG,
    FULL_ACCESS,
};
```

| Flag | Permission |
|---|---|
| `REGISTER_COMMANDS` | Can register command handlers |
| `REGISTER_TASKS` | Can register task handlers |
| `LOAD_RESOURCES` | Can load resources via ResourceManager |
| `REGISTER_SERVICES` | Can register services |
| `SUBSCRIBE_EVENTS` | Can subscribe to events |
| `PUBLISH_EVENTS` | Can publish events |
| `READ_CONFIG` | Can read configuration |
| `WRITE_CONFIG` | Can write configuration |
| `FULL_ACCESS` | All permissions (default) |

### Creating Permissions

```rust
// No permissions
let none = PluginPermissions::new(0);
assert!(!none.can_register_commands());
assert!(!none.can_load_resources());

// Full access
let full = PluginPermissions::FULL;
assert!(full.can_register_commands());
assert!(full.can_load_resources());

// Default = full access (backward compatible)
let default = PluginPermissions::default();
assert!(default.can_load_resources());
assert!(default.can_write_config());

// Combined permissions
let limited = PluginPermissions::new(
    REGISTER_COMMANDS | SUBSCRIBE_EVENTS,
);
assert!(limited.can_register_commands());
assert!(limited.can_subscribe_events());
assert!(!limited.can_load_resources());
assert!(!limited.can_write_config());
```

### Bitwise Operations

```rust
let combined = PluginPermissions::new(REGISTER_COMMANDS)
    | PluginPermissions::new(SUBSCRIBE_EVENTS);

assert!(combined.can_register_commands());
assert!(combined.can_subscribe_events());
assert!(!combined.can_publish_events());
```

### Display

```rust
let perms = PluginPermissions::new(REGISTER_COMMANDS);
assert!(perms.to_string().contains("00000001"));

let full = PluginPermissions::FULL;
assert!(full.to_string().contains("11111111"));
```

---

## Part 2: Manifest Permissions

### Declaring Permissions in Manifest

```rust
use app_shell::app_engine::{PluginManifest, PluginPermissions};

// Full access (default — backward compatible)
let manifest = PluginManifest::new("full_plugin", "Full Plugin");
assert_eq!(manifest.permissions(), PluginPermissions::FULL);

// Limited permissions
let manifest = PluginManifest::new("limited", "Limited Plugin")
    .with_permissions(PluginPermissions::new(
        REGISTER_COMMANDS | SUBSCRIBE_EVENTS,
    ));
let perms = manifest.permissions();
assert!(perms.can_register_commands());
assert!(perms.can_subscribe_events());
assert!(!perms.can_load_resources());
assert!(!perms.can_write_config());

// No permissions
let manifest = PluginManifest::new("restricted", "Restricted Plugin")
    .with_permissions(PluginPermissions::new(0));
let perms = manifest.permissions();
assert!(!perms.can_register_commands());
assert!(!perms.can_load_resources());
assert!(!perms.can_publish_events());
```

---

## Part 3: Manager Permission Queries

### Checking Permissions via PluginManager

```rust
let mgr = PluginManager::new();

let id = mgr.register(
    PluginManifest::new("limited", "Limited")
        .with_permissions(PluginPermissions::new(REGISTER_COMMANDS)),
    Box::new(StubPlugin),
)?;

assert!(mgr.can_register_commands(id));
assert!(!mgr.can_load_resources(id));
assert!(!mgr.can_register_tasks(id));
assert!(!mgr.can_publish_events(id));
assert!(!mgr.can_write_config(id));
```

### Default Plugin Has Full Access

```rust
let mgr = PluginManager::new();

let id = mgr.register(
    PluginManifest::new("full", "Full"),  // No permissions specified → FULL
    Box::new(StubPlugin),
)?;

assert!(mgr.can_register_commands(id));
assert!(mgr.can_load_resources(id));
assert!(mgr.can_publish_events(id));
assert!(mgr.can_write_config(id));
```

### Querying Unregistered Plugin

```rust
let mgr = PluginManager::new();
let bogus = PluginId::new();
assert!(!mgr.has_permission(bogus, REGISTER_COMMANDS));
assert!(!mgr.can_register_commands(bogus));
```

---

## Part 4: PermissionCheckedEngineRef (Phase 28)

### What It Is

A wrapper around `EngineRef` that checks permissions before allowing access to restricted operations. Read-only access is always allowed.

```rust
use app_shell::app_engine::PermissionCheckedEngineRef;

let engine = TestEngine::new();
let engine_ref = engine.engine_ref();

let checked = PermissionCheckedEngineRef::new(
    &engine_ref,
    PluginPermissions::new(LOAD_RESOURCES | SUBSCRIBE_EVENTS),
    "limited_plugin",
);
```

### Read-Only Access (Always Allowed)

Regardless of permissions, these are always accessible:

```rust
let _ = checked.config_store();        // &ConfigStore (read)
let _ = checked.preferences();        // &Preferences (read)
let _ = checked.state_manager();      // &StateManager (read)
let _ = checked.history_store();     // &HistoryStore (read)
let _ = checked.scheduler();          // &Scheduler (read)
let _ = checked.event_bus();          // &EventBus (read)
let _ = checked.signal_bus();        // &SignalBus (read)
let _ = checked.command_executor();  // &CommandExecutor (read)
let _ = checked.resource_manager();   // &ResourceManager (read)
```

### Permission-Checked Access

Each `try_*` method returns `Result` — `Ok` if permission granted, `Err(PluginSecurityError)` if denied:

```rust
// With LOAD_RESOURCES permission
let result = checked.try_load_resource()?;
assert!(result.is_ok());
let mgr = result.unwrap();
assert_eq!(mgr.resource_count(), 0);

// Without WRITE_CONFIG permission
let result = checked.try_write_config();
assert!(result.is_err());
let err = result.unwrap_err();
assert!(matches!(err, PluginSecurityError::PermissionDenied { .. }));
assert!(err.to_string().contains("WRITE_CONFIG"));
assert!(err.to_string().contains("limited_plugin"));
```

### All try_* Methods

| Method | Permission Required | Returns |
|---|---|---|
| `try_write_config()` | `WRITE_CONFIG` | `Result<&ConfigStore, PluginSecurityError>` |
| `try_load_resource()` | `LOAD_RESOURCES` | `Result<&ResourceManager, PluginSecurityError>` |
| `try_publish_event()` | `PUBLISH_EVENTS` | `Result<&EventBus, PluginSecurityError>` |
| `try_subscribe_events()` | `SUBSCRIBE_EVENTS` | `Result<&EventBus, PluginSecurityError>` |
| `try_register_commands()` | `REGISTER_COMMANDS` | `Result<&CommandExecutor, PluginSecurityError>` |
| `try_register_tasks()` | `REGISTER_TASKS` | `Result<&CommandExecutor, PluginSecurityError>` |
| `try_register_services()` | `REGISTER_SERVICES` | `Result<&CommandExecutor, PluginSecurityError>` |

### Full Access (All try_* Succeed)

```rust
let checked = PermissionCheckedEngineRef::new(
    &engine_ref,
    PluginPermissions::FULL,
    "full_access_plugin",
);

assert!(checked.try_load_resource().is_ok());
assert!(checked.try_write_config().is_ok());
assert!(checked.try_publish_event().is_ok());
assert!(checked.try_subscribe_events().is_ok());
assert!(checked.try_register_commands().is_ok());
assert!(checked.try_register_tasks().is_ok());
assert!(checked.try_register_services().is_ok());
```

### Default Permissions (Backward Compatible)

```rust
let checked = PermissionCheckedEngineRef::new(
    &engine_ref,
    PluginPermissions::default(),  // = FULL
    "default_perms_plugin",
);

assert!(checked.try_load_resource().is_ok());
assert!(checked.try_write_config().is_ok());
assert!(checked.try_publish_event().is_ok());
```

### Combined Permissions

```rust
let checked = PermissionCheckedEngineRef::new(
    &engine_ref,
    PluginPermissions::new(LOAD_RESOURCES | PUBLISH_EVENTS),
    "combined_plugin",
);

assert!(checked.try_load_resource().is_ok());     // Has LOAD_RESOURCES
assert!(checked.try_publish_event().is_ok());     // Has PUBLISH_EVENTS
assert!(checked.try_write_config().is_err());     // No WRITE_CONFIG
assert!(checked.try_register_commands().is_err()); // No REGISTER_COMMANDS
```

---

## Part 5: Using the Wrapper in Plugins

The `Plugin` trait still receives `&EngineRef` (not `PermissionCheckedEngineRef`) for backward compatibility. Domain plugins that want enforcement construct the wrapper themselves:

```rust
use app_shell::app_engine::*;

struct MyPlugin {
    permissions: PluginPermissions,
}

impl Plugin for MyPlugin {
    fn id(&self) -> &str { "my_plugin" }

    fn initialize(&mut self, ctx: &PluginContext, engine: &EngineRef) -> Result<(), PluginError> {
        // Wrap engine with permission checking
        let checked = PermissionCheckedEngineRef::new(
            engine,
            self.permissions,
            "my_plugin",
        );

        // Use try_* methods for restricted operations
        let config = checked.try_write_config()
            .map_err(|e| PluginError::InitializeFailed {
                id: 0,
                reason: e.to_string(),
            })?;

        config.set("plugin.initialized", "true").map_err(|e| {
            PluginError::InitializeFailed {
                id: 0,
                reason: e.to_string(),
            }
        })?;

        Ok(())
    }

    fn start(&mut self, engine: &EngineRef) -> Result<(), PluginError> {
        let checked = PermissionCheckedEngineRef::new(
            engine,
            self.permissions,
            "my_plugin",
        );

        // Try to load a resource
        if let Ok(mgr) = checked.try_load_resource() {
            let _handle = mgr.load("config", "file:///plugin_config.json")
                .map_err(|e| PluginError::StartFailed {
                    id: 0,
                    reason: e.to_string(),
                })?;
        }

        Ok(())
    }

    fn stop(&mut self) -> Result<(), PluginError> { Ok(()) }
    fn dispose(&mut self) -> Result<(), PluginError> { Ok(()) }
}

// Register with limited permissions
let manifest = PluginManifest::new("my_plugin", "My Plugin")
    .with_permissions(PluginPermissions::new(
        LOAD_RESOURCES | WRITE_CONFIG | SUBSCRIBE_EVENTS,
    ));

engine.plugin_manager_mut().register(manifest, Box::new(MyPlugin {
    permissions: PluginPermissions::new(LOAD_RESOURCES | WRITE_CONFIG | SUBSCRIBE_EVENTS),
}))?;
```

---

## Part 6: Plugin Lifecycle Is Not Blocked by Permissions

A plugin with **no permissions** still has a normal lifecycle:

```rust
let id = engine.plugin_manager_mut().register(
    PluginManifest::new("restricted", "Restricted")
        .with_permissions(PluginPermissions::new(0)),
    Box::new(StubPlugin),
)?;

// Lifecycle works — permissions don't block lifecycle
engine.plugin_manager_mut().initialize(id, &engine_ref)?;
assert_eq!(engine.plugin_manager().state(id), Some(PluginState::Initialized));

engine.plugin_manager_mut().start(id, &engine_ref)?;
assert_eq!(engine.plugin_manager().state(id), Some(PluginState::Started));

engine.plugin_manager_mut().stop(id)?;
engine.plugin_manager_mut().dispose(id)?;
```

Permissions only affect **runtime capabilities** (what the plugin can do with `EngineRef`), not whether the plugin itself can be initialized/started/stopped/disposed.

---

## Part 7: Error Messages

Permission denied errors contain the plugin ID and permission name:

```rust
let checked = PermissionCheckedEngineRef::new(
    &engine_ref,
    PluginPermissions::new(0),
    "my_plugin",
);

let err = checked.try_load_resource().unwrap_err();
let msg = err.to_string();

assert!(msg.contains("my_plugin"));
assert!(msg.contains("LOAD_RESOURCES"));
```

### PluginSecurityError

```rust
use app_shell::app_engine::PluginSecurityError;

// PermissionDenied variant
let err = PluginSecurityError::PermissionDenied {
    plugin_id: "limited".to_string(),
    permission: "LOAD_RESOURCES".to_string(),
};
assert!(err.to_string().contains("limited"));
assert!(err.to_string().contains("LOAD_RESOURCES"));

// Implements std::error::Error
fn assert_std_error<E: std::error::Error>(_: E) {}
assert_std_error(PluginSecurityError::PermissionDenied {
    plugin_id: "x".to_string(),
    permission: "y".to_string(),
});
```

---

## Part 8: Dynamic Plugin Loading (Phase 17)

### DynamicPluginLoader

A contract stub for loading plugins from shared libraries:

```rust
use app_shell::app_engine::DynamicPluginLoader;

let loader = DynamicPluginLoader::new();

// Detect shared libraries
assert!(DynamicPluginLoader::is_shared_library("plugin.so"));
assert!(DynamicPluginLoader::is_shared_library("plugin.dll"));
assert!(DynamicPluginLoader::is_shared_library("plugin.dylib"));
assert!(!DynamicPluginLoader::is_shared_library("plugin.rs"));
```

### With Search Paths

```rust
let mut loader = DynamicPluginLoader::new();
loader.add_search_path("/tmp/plugins");
assert_eq!(loader.search_paths_len(), 1);
```

### Loading (Stub)

Actual dynamic loading requires the `libloading` crate. The stub returns a descriptive error:

```rust
let result = loader.load("nonexistent.so");
assert!(matches!(result, Err(PluginError::LoadFailed { .. })));
```

### Contract for Dynamic Plugins

```text
1. The shared library exports create_plugin() -> Box<dyn Plugin>
2. The shared library exports create_manifest() -> PluginManifest
3. The loader calls both, verifies the manifest, and returns them
```

---

## Quick Reference

### PluginPermissions

```rust
PluginPermissions::new(bits)           // Create from raw bits
PluginPermissions::FULL                // All permissions
PluginPermissions::default()          // = FULL (backward compatible)
perms.can_register_commands()         // bool
perms.can_load_resources()            // bool
perms.can_publish_events()           // bool
perms.can_subscribe_events()         // bool
perms.can_write_config()             // bool
perms.has(flag)                      // bool
perms.bits()                         // u32
perms1 | perms2                      // BitOr
```

### Manifest with Permissions

```rust
PluginManifest::new("id", "name")
    .with_permissions(PluginPermissions::new(flags))
    .with_engine_api_version(Version::new(1, 0, 0))
manifest.permissions()               // PluginPermissions
manifest.is_compatible_with(&version) // bool
```

### PermissionCheckedEngineRef

```rust
let checked = PermissionCheckedEngineRef::new(engine, permissions, "plugin_id");

// Read-only (always allowed)
checked.config_store()
checked.preferences()
checked.state_manager()
checked.history_store()
checked.scheduler()
checked.event_bus()
checked.signal_bus()
checked.command_executor()
checked.resource_manager()

// Permission-checked
checked.try_write_config()?        // WRITE_CONFIG
checked.try_load_resource()?       // LOAD_RESOURCES
checked.try_publish_event()?       // PUBLISH_EVENTS
checked.try_subscribe_events()?   // SUBSCRIBE_EVENTS
checked.try_register_commands()?  // REGISTER_COMMANDS
checked.try_register_tasks()?     // REGISTER_TASKS
checked.try_register_services()?   // REGISTER_SERVICES
```

### PluginManager Queries

```rust
engine.plugin_manager().permissions(id)          // Option<PluginPermissions>
engine.plugin_manager().has_permission(id, flag) // bool
engine.plugin_manager().can_register_commands(id) // bool
engine.plugin_manager().can_load_resources(id)  // bool
engine.plugin_manager().can_publish_events(id)  // bool
engine.plugin_manager().can_write_config(id)    // bool
```

### DynamicPluginLoader

```rust
DynamicPluginLoader::is_shared_library("plugin.so")  // bool
loader.add_search_path("/path")
loader.search_paths_len()                             // usize
loader.can_load("plugin.so")                         // bool
loader.load("plugin.so")                             // Result<(manifest, plugin), PluginError>
```

---

## Next Steps

- [08-services-and-plugins.md](08-services-and-plugins.md) — Services and plugins overview.
- [11-handler-capabilities.md](11-handler-capabilities.md) — `EngineRef` and `PermissionCheckedEngineRef` in context.
- [16-testing-and-metadata.md](16-testing-and-metadata.md) — Testing infrastructure and versioning.
- [09-domain-engine-integration.md](09-domain-engine-integration.md) — How domain code registers plugins.