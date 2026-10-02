# `07-history-and-resources.md` — Persistence of Operations & Data

## Overview

Two systems provide application continuity:

```text
History   = "What happened?"   (record of meaningful operations)
Resource  = "What's available?" (managed data/content lifecycle)
```

They are deliberately separate:

```text
HistoryStore records operations    StateStore records conditions
ResourceCache stores loaded data  HistoryStore stores actions
```

History is about **the path** — how the application got to where it is. Resources are about **the content** — what data is available to use.

---

## Part 1: History

### What History Records

History records **meaningful application operations**, not every internal function call:

```text
✓ CreateDocument
✓ MoveObject
✓ ChangeProperty
✓ DeleteObject
✓ ExportProject

✗ update_position()
✗ invalidate_layout()
✗ sort_array()
✗ allocate_buffer()
```

If `MoveObjectCommand` internally calls `update_position()`, `update_bounds()`, and `invalidate_layout()`, history should contain **one entry**: `MoveObject`.

### History Entry

```rust
use app_shell::app_engine::{HistoryEntry, HistoryEntryId, TransactionId};
use std::time::Instant;

// Create a history entry with operation data and inverse data
let entry = HistoryEntry::new("object.move")
    .undoable()
    .with_data(MoveData {
        object: "rect_1".to_string(),
        from: (100.0, 200.0),
        to: (50.0, 50.0),
    })
    .with_inverse(MoveData {
        object: "rect_1".to_string(),
        from: (50.0, 50.0),
        to: (100.0, 200.0),
    })
    .with_source("MoveObjectCommand")
    .with_command_id("object.move_command");

assert!(entry.is_undoable());
assert_eq!(entry.operation(), "object.move");
assert_eq!(entry.source(), Some("MoveObjectCommand"));
assert!(entry.has_data());
assert!(entry.has_inverse_data());
```

### Undoable vs Non-Undoable

Not every operation can be reversed:

```rust
// Undoable: position change (has inverse)
let undoable = HistoryEntry::new("object.move")
    .undoable()
    .with_data(new_position)
    .with_inverse(old_position);

// Non-undoable: file export (no meaningful undo)
let non_undoable = HistoryEntry::new("file.export")
    .non_undoable()
    .with_data(output_path);

assert!(undoable.is_undoable());
assert!(!non_undoable.is_undoable());
```

### Storing History

```rust
// Append entries to the store
let id1 = engine.history_store_mut().append(
    HistoryEntry::new("object.create").undoable().with_data("rect_1".to_string()),
);
let id2 = engine.history_store_mut().append(
    HistoryEntry::new("object.move").undoable()
        .with_data(move_forward)
        .with_inverse(move_backward),
);
let id3 = engine.history_store_mut().append(
    HistoryEntry::new("object.resize").undoable(),
);

assert_eq!(engine.history_store().active_count(), 3);
assert_eq!(engine.history_store().position(), 3);
```

### Current Position

The store tracks a "current position" — how many entries are active:

```text
Entry 0  Entry 1  Entry 2  Entry 3
                            ↑
                         position (4 active)
```

After undo:

```text
Entry 0  Entry 1  Entry 2  Entry 3
                  ↑
               position (3 active)
```

```rust
assert_eq!(engine.history_store().position(), 3);
assert_eq!(
    engine.history_store().current().unwrap().operation(),
    "object.resize",
);
```

### Undo

```rust
use app_shell::app_engine::UndoResult;

let result: UndoResult = engine.history_store_mut().undo();

assert_eq!(result.count(), 1);
assert!(!result.is_empty());

// Position moved back
assert_eq!(engine.history_store().position(), 2);
assert_eq!(
    engine.history_store().current().unwrap().operation(),
    "object.move",
);

// Look up the undone entry to get inverse data
let undone_id = result.undone_entry_ids()[0];
let entry = engine.history_store().get(undone_id).unwrap();
let inverse: &MoveData = entry.inverse_data::<MoveData>().unwrap();
// Use inverse to create an undo command
```

### Redo

```rust
use app_shell::app_engine::RedoResult;

let result: RedoResult = engine.history_store_mut().redo();

assert_eq!(result.count(), 1);
assert_eq!(engine.history_store().position(), 3);
```

### Redo Branch Invalidation

If you undo and then perform a **new** operation, the redo branch is discarded:

```rust
// State: [A, B, C] position=3
engine.history_store_mut().undo();  // position=2, [A, B, C] — C is redoable
engine.history_store_mut().undo();  // position=1, [A, B, C] — B, C are redoable

// New operation — invalidates redo branch
engine.history_store_mut().append(HistoryEntry::new("object.rotate").undoable());

// Now: [A, D] position=2 — B and C are gone
assert_eq!(engine.history_store().entry_count(), 2);
assert!(!engine.history_store().can_redo());
```

### Transactions

Group multiple operations into one undo/redo unit:

```rust
use app_shell::app_engine::TransactionManager;

// Begin a transaction
let tx = engine.transaction_manager_mut().begin();

// Append multiple entries under the same transaction
engine.history_store_mut().append(
    HistoryEntry::new("object.move").with_transaction(tx),
);
engine.history_store_mut().append(
    HistoryEntry::new("object.resize").with_transaction(tx),
);
engine.history_store_mut().append(
    HistoryEntry::new("object.rotate").with_transaction(tx),
);

// Commit (entries stay in history)
engine.transaction_manager_mut().commit();

// Now: one undo undoes all three
let result = engine.history_store_mut().undo();
assert_eq!(result.count(), 3);  // all three undone
assert!(result.transaction_undone());
assert_eq!(engine.history_store().position(), 0);
```

### Transaction Rollback

If the transaction fails, remove its entries:

```rust
let tx = engine.transaction_manager_mut().begin();

engine.history_store_mut().append(
    HistoryEntry::new("step.a").with_transaction(tx),
);
engine.history_store_mut().append(
    HistoryEntry::new("step.b").with_transaction(tx),
);

// Something went wrong — rollback
engine.transaction_manager_mut().rollback();
engine.history_store_mut().remove_transaction(tx);  // remove entries

assert_eq!(engine.history_store().entry_count(), 0);
```

### Replay

Replay re-executes recorded operations in sequence:

```rust
// Record operations
engine.history_store_mut().append(
    HistoryEntry::new("object.create")
        .with_data("rect_1".to_string()),
);
engine.history_store_mut().append(
    HistoryEntry::new("object.move")
        .with_data("rect_1_moved".to_string()),
);
engine.history_store_mut().append(
    HistoryEntry::new("object.resize")
        .with_data("rect_1_resized".to_string()),
);

// Replay: iterate over active entries
let replayed: Vec<&str> = engine.history_store()
    .replay()
    .map(|e| e.operation())
    .collect();

assert_eq!(replayed, vec!["object.create", "object.move", "object.resize"]);
```

Replay **reuses existing execution mechanisms** — the caller creates commands from history entries and executes them via `CommandExecutor`. The history system is **not** a second execution engine.

### Replay Excludes Undone Entries

```rust
engine.history_store_mut().undo();  // "object.resize" undone

let replayed: Vec<&str> = engine.history_store()
    .replay()
    .map(|e| e.operation())
    .collect();

// Only active entries are replayed
assert_eq!(replayed, vec!["object.create", "object.move"]);
```

### Command → History Integration

The cleanest pattern: commands produce events, and a history handler subscribes:

```text
Command → Execution → Event → HistoryHandler → HistoryStore
```

```rust
// History handler subscribes to "meaningful" events
let store = Arc::new(Mutex::new(HistoryStore::new()));

let s = store.clone();
engine.event_bus_mut().subscribe(
    "document.created",
    event_handler(move |event: &Event| {
        s.lock().unwrap().append(
            HistoryEntry::new("document.create")
                .undoable()
                .with_data(event.payload::<String>().unwrap().clone())
                .with_source(event.source().unwrap_or("unknown")),
        );
    }),
);

// When a command executes and publishes an event, history records it
engine.event_bus().publish(
    &Event::new("document.created")
        .with_payload("doc_42".to_string())
        .with_source("CreateDocumentCommand"),
);

assert_eq!(store.lock().unwrap().entry_count(), 1);
```

### What NOT to Record in History

```text
✗ MouseMoved, FrameRendered, ProgressChanged → signals, not history
✗ ResourceLoaded, ResourceReleased → runtime infrastructure
✗ JobCreated, JobQueued, JobDispatched → scheduling internals
✗ TaskStarted → record TaskCompleted instead (one meaningful entry per operation)
```

History is for **user-visible, undoable application operations**.

---

## Part 2: Resources

### What Is a Resource?

A resource is a managed piece of data whose **lifetime matters** to the application:

```text
files, images, fonts, audio, documents, database connections, GPU assets...
```

The App Engine doesn't know what any of these mean. It only knows:

```text
Resource
├── identity (ResourceId)
├── type (string, e.g. "image", "font")
├── source (string, e.g. "file:///assets/logo.png")
├── state (Unloaded, Loading, Loaded, Failed, Released)
└── data (type-erased Box<dyn Any + Send>)
```

### Resource Lifecycle

```text
           ┌──────────┐
           │ Unloaded │
           └────┬─────┘
                │ load()
                ▼
           ┌──────────┐
           │  Loaded   │ ←─── reload()
           └────┬─────┘
                │ release()
                ▼
           ┌──────────┐
           │  Cached   │ (in cache, no active handles)
           └────┬─────┘
                │ unload()
                ▼
           ┌──────────┐
           │ Unloaded  │ (can be reloaded)
           └──────────┘
```

### Defining a Resource

```rust
use app_shell::app_engine::{Resource, ResourceId, ResourceState, ResourceKey};

let resource = Resource::new("image", "file:///assets/logo.png")
    .with_data("pixel_data_here".to_string());

assert_eq!(resource.resource_type(), "image");
assert_eq!(resource.source(), "file:///assets/logo.png");
assert_eq!(resource.state(), ResourceState::Loaded);
assert!(resource.has_data());
assert_eq!(resource.data::<String>(), Some(&"pixel_data_here".to_string()));
```

### Resource States

```rust
assert!(ResourceState::Loaded.is_loaded());
assert!(!ResourceState::Unloaded.is_loaded());
assert!(ResourceState::Loading.is_active());
assert!(ResourceState::Unloaded.is_terminal());
```

### ResourceHandle

A handle is a lightweight, copyable reference to a managed resource:

```rust
use app_shell::app_engine::ResourceHandle;

let handle = ResourceHandle::new(resource_id);

// Handles are Copy — pass them around freely
let copied = handle;
assert_eq!(handle, copied);
assert_eq!(handle.resource_id(), resource_id);
```

Multiple systems can hold handles to the same resource. The `ResourceManager` tracks reference counts to know when it's safe to unload.

### ResourceLoader

Domain code provides loaders that know how to obtain specific resource types:

```rust
use app_shell::app_engine::{ResourceLoader, ResourceError};
use std::any::Any;

struct ImageLoader;

impl ResourceLoader for ImageLoader {
    fn resource_type(&self) -> &str {
        "image"
    }

    fn load(&self, source: &str) -> Result<Box<dyn Any + Send>, ResourceError> {
        // Domain-specific loading logic
        if source.contains("missing") {
            return Err(ResourceError::LoadFailed {
                source: source.to_string(),
                reason: "file not found".to_string(),
            });
        }
        // Simulate loading image data
        Ok(Box::new(format!("pixels:{}", source)))
    }
}

// Register the loader
engine.resource_manager_mut().register_loader(Box::new(ImageLoader));
```

### Loading a Resource

```rust
let handle = engine.resource_manager_mut().load(
    "image",
    "file:///assets/logo.png",
)?;

let resource = engine.resource_manager().get(&handle).unwrap();
assert_eq!(resource.resource_type(), "image");
assert_eq!(resource.state(), ResourceState::Loaded);
assert_eq!(
    resource.data::<String>(),
    Some(&"pixels:file:///assets/logo.png".to_string()),
);
```

### Cache Reuse

Loading the same source twice returns the **same** resource:

```rust
let h1 = engine.resource_manager_mut().load("image", "file:///logo.png")?;
let h2 = engine.resource_manager_mut().load("image", "file:///logo.png")?;

// Same resource — deduplicated by (type, source)
assert_eq!(h1.resource_id(), h2.resource_id());
assert_eq!(engine.resource_manager().resource_count(), 1);
assert_eq!(engine.resource_manager().active_count(h1.resource_id()), 2);
```

### Acquire and Release

```rust
let handle = engine.resource_manager_mut().load("image", "file:///logo.png")?;
assert_eq!(engine.resource_manager().active_count(handle.resource_id()), 1);

// Acquire a second handle (increment ref count)
engine.resource_manager_mut().acquire(&handle)?;
assert_eq!(engine.resource_manager().active_count(handle.resource_id()), 2);

// Release one handle
engine.resource_manager_mut().release(handle)?;
assert_eq!(engine.resource_manager().active_count(handle.resource_id()), 1);

// Release the other
let handle_copy = handle;
engine.resource_manager_mut().release(handle_copy)?;
assert_eq!(engine.resource_manager().active_count(handle.resource_id()), 0);

// Resource stays in cache (not in use, but still available)
assert!(engine.resource_manager().exists(handle.resource_id()));
```

### Unload

```rust
// Can't unload while in use
let handle = engine.resource_manager_mut().load("image", "file:///logo.png")?;
assert!(engine.resource_manager_mut().unload(handle.resource_id()).is_err());
// Error: InUse

// Release first, then unload
engine.resource_manager_mut().release(handle)?;
engine.resource_manager_mut().unload(handle.resource_id())?;
assert!(!engine.resource_manager().exists(handle.resource_id()));
```

### Reload

Replace a resource's data without changing its identity:

```rust
let handle = engine.resource_manager_mut().load("image", "file:///logo.png")?;
let id = handle.resource_id();

// ... file changes on disk ...

// Reload from source
engine.resource_manager_mut().reload(id)?;

// Same ResourceId, new data
let resource = engine.resource_manager().get(&handle).unwrap();
assert_eq!(resource.state(), ResourceState::Loaded);
assert!(resource.has_data());
```

### Loading Failure

```rust
let result = engine.resource_manager_mut().load("image", "file:///missing.png");
assert!(matches!(result, Err(ResourceError::LoadFailed { .. })));

// Manager state is clean — no partial resource
assert_eq!(engine.resource_manager().resource_count(), 0);
```

### Reloading After Unload

```rust
let handle = engine.resource_manager_mut().load("image", "file:///logo.png")?;
let id = handle.resource_id();

engine.resource_manager_mut().release(handle)?;
engine.resource_manager_mut().unload(id)?;
assert!(!engine.resource_manager().exists(id));

// Load again — same source, new resource instance
let handle2 = engine.resource_manager_mut().load("image", "file:///logo.png")?;
assert!(engine.resource_manager().exists(handle2.resource_id()));
assert_eq!(engine.resource_manager().active_count(handle2.resource_id()), 1);
```

---

## Practical Example: DocumentResource

A complete example showing how a Domain Engine provides a document resource:

```rust
use app_shell::app_engine::*;
use std::any::Any;

// --- Domain data type ---

#[derive(Debug, Clone)]
struct DocumentData {
    title: String,
    content: String,
    word_count: usize,
}

// --- Domain loader ---

struct DocumentLoader;

impl ResourceLoader for DocumentLoader {
    fn resource_type(&self) -> &str {
        "document"
    }

    fn load(&self, source: &str) -> Result<Box<dyn Any + Send>, ResourceError> {
        // In a real app, this would read from a file
        let doc = DocumentData {
            title: source.to_string(),
            content: "Lorem ipsum dolor sit amet.".to_string(),
            word_count: 5,
        };
        Ok(Box::new(doc))
    }
}

// --- Setup ---

fn main() {
    let mut engine = Bootstrap::create().unwrap();
    engine.initialize().unwrap();
    engine.start().unwrap();

    // Register the domain loader
    engine.resource_manager_mut().register_loader(Box::new(DocumentLoader));

    // Load a document
    let handle = engine.resource_manager_mut().load(
        "document",
        "file:///docs/intro.dp",
    ).unwrap();

    // Access the document data
    let resource = engine.resource_manager().get(&handle).unwrap();
    let doc: &DocumentData = resource.data::<DocumentData>().unwrap();
    println!("Title: {}", doc.title);
    println!("Words: {}", doc.word_count);

    // A task can acquire the resource handle
    let handle_copy = handle;
    engine.resource_manager_mut().acquire(&handle_copy).unwrap();

    // ... task uses the document ...

    // Release when done
    engine.resource_manager_mut().release(handle_copy).unwrap();
    engine.resource_manager_mut().release(handle).unwrap();

    // Optionally unload
    engine.resource_manager_mut().unload(handle.resource_id()).unwrap();

    engine.stop().unwrap();
    engine.dispose().unwrap();
}
```

The App Engine managed the resource lifecycle (load, cache, reference count, unload). The Domain Engine provided the `DocumentData` type and `DocumentLoader`. Neither knew about the other's internals.

---

## History + Resources Integration

Don't record resource lifecycle operations in history:

```text
✓ History: ImportDocument (meaningful user operation)
✗ History: LoadResource (runtime infrastructure)
✗ History: CacheHit (performance detail)
✗ History: UnloadResource (cleanup)
```

But a resource operation can **trigger** a history entry through events:

```text
User clicks "Import Document"
       ↓
ImportDocumentCommand
       ↓
Task loads DocumentResource
       ↓
Publishes "document.imported" event
       ↓
HistoryHandler records: ImportDocument
```

The resource system and history system don't know about each other — they're connected through the event bus.

---

## Quick Reference

### History API

```rust
// Store
let id = engine.history_store_mut().append(entry);
engine.history_store().get(id);               // Option<&HistoryEntry>
engine.history_store().list();                // &[HistoryEntry]
engine.history_store().active();              // &[HistoryEntry] (up to position)
engine.history_store().current();             // Option<&HistoryEntry>
engine.history_store().position();            // usize
engine.history_store().can_undo();           // bool
engine.history_store().can_redo();           // bool

// Undo / Redo
let result = engine.history_store_mut().undo();  // UndoResult
let result = engine.history_store_mut().redo();  // RedoResult

// Transactions
let tx = engine.transaction_manager_mut().begin();
engine.transaction_manager_mut().commit();
engine.transaction_manager_mut().rollback();
engine.history_store_mut().remove_transaction(tx);

// Replay
for entry in engine.history_store().replay() {
    // Create command from entry data and execute
}
```

### Resource API

```rust
// Register loader
engine.resource_manager_mut().register_loader(loader);

// Load
let handle = engine.resource_manager_mut().load("type", "source")?;

// Access
let resource = engine.resource_manager().get(&handle)?;

// Ref counting
engine.resource_manager_mut().acquire(&handle)?;
engine.resource_manager_mut().release(handle)?;

// Unload / Reload
engine.resource_manager_mut().unload(id)?;
engine.resource_manager_mut().reload(id)?;

// Query
engine.resource_manager().resource_count();
engine.resource_manager().active_count(id);
engine.resource_manager().is_in_use(id);
engine.resource_manager().exists(id);
```

---

## Next Steps

- [08-services-and-plugins.md](08-services-and-plugins.md) — Long-lived services and plugin extensions.
- [09-domain-engine-integration.md](09-domain-engine-integration.md) — How domain code provides resources, commands, and tasks.