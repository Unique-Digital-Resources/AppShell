# Quick Start

## 5-Minute Guide

### 1. Add to Cargo.toml

```toml
[dependencies]
app_shell = { path = "path/to/AppShell" }

# Optional: serialization
app_shell = { path = "path/to/AppShell", features = ["serde"] }
```

### 2. Create a DomainEngine

```rust
use app_shell::domain_engine::DomainEngine;
use app_shell::domain_foundation::environments::canvas_2d::*;

let mut engine = DomainEngine::new();

// Create a canvas object
let id = engine.execute_canvas(CanvasOperation::Create {
    transform: Transform::from_translation(Vec2::new(100.0, 200.0)),
}).unwrap();

// Move it
engine.execute_canvas(CanvasOperation::Move {
    id,
    translation: Vec2::new(50.0, 50.0),
}).unwrap();

// Undo — restores the original position
engine.undo().unwrap();
assert_eq!(
    engine.canvas().get(id).unwrap().transform.translation,
    Vec2::new(100.0, 200.0),
);
```

### 3. Use Multiple Environments

```rust
use app_shell::domain_foundation::environments::timeline::*;

// Create a timeline
engine.execute_timeline(TimelineOperation::CreateTrack).unwrap();
let track = engine.timeline().track_ids()[0];

engine.execute_timeline(TimelineOperation::InsertClip {
    track_id: track,
    start: Time::new(0.0),
    duration: Duration::new(10.0),
}).unwrap();

// Query: what clips are active at time 5.0?
let active = engine.timeline().clips_at(Time::new(5.0));
assert_eq!(active.len(), 1);
```

```rust
use app_shell::domain_foundation::environments::layers::*;

// Create a layer hierarchy
engine.execute_layers(LayersOperation::CreateLayer {
    name: "Background".to_string(),
}).unwrap();
let root = engine.layers().roots()[0];

engine.execute_layers(LayersOperation::CreateChild {
    parent: root,
    name: "Shapes".to_string(),
}).unwrap();

// Hide the root — children become effectively invisible
engine.execute_layers(LayersOperation::SetVisibility {
    id: root,
    visible: false,
}).unwrap();
assert!(!engine.layers().effective_visibility(
    engine.layers().roots()[0]
));
```

```rust
use app_shell::domain_foundation::environments::text_editor::*;

// Edit text
engine.execute_text_editor(TextEditorOperation::Insert {
    position: Position::new(0),
    text: "Hello, World!".to_string(),
}).unwrap();

assert_eq!(engine.document().text(), "Hello, World!");
assert_eq!(engine.document().len(), 13);

// Find text
let matches = engine.document().find_all("o");
assert_eq!(matches.len(), 2);
```

### 4. Transactions

Group multiple operations into a single undoable unit:

```rust
engine.begin_transaction();

engine.execute_canvas(CanvasOperation::Create {
    transform: Transform::identity(),
}).unwrap();
engine.execute_canvas(CanvasOperation::Create {
    transform: Transform::identity(),
}).unwrap();

engine.commit_transaction();

// Undo reverses both creations as one unit
engine.undo().unwrap();
assert_eq!(engine.canvas().len(), 0);
```

### 5. Compound Operations (Cross-Environment)

```rust
// Group operations across different environments
engine.begin_compound();

engine.execute_canvas(CanvasOperation::Create {
    transform: Transform::identity(),
}).unwrap();
engine.execute_layers(LayersOperation::CreateLayer {
    name: "Group".to_string(),
}).unwrap();
engine.execute_timeline(TimelineOperation::CreateTrack).unwrap();

engine.commit_compound();

// One undo reverses all three
engine.undo().unwrap();
assert_eq!(engine.canvas().len(), 0);
assert_eq!(engine.layers().len(), 0);
```

### 6. Events

```rust
use app_shell::app_engine::{EventBus, EventHandler, Event};
use std::sync::{Arc, Mutex};

// Subscribe to events
struct PrintHandler;
impl EventHandler for PrintHandler {
    fn handle(&self, event: &Event) {
        println!("Event: {}", event.event_type());
    }
}

engine.event_bus_mut().subscribe(
    "canvas.object.created",
    Box::new(PrintHandler),
);

// Execute triggers event publication
engine.execute_canvas(CanvasOperation::Create {
    transform: Transform::identity(),
}).unwrap();
// Output: "Event: canvas.object.created"
```

### 7. Relations

```rust
use app_shell::domain_foundation::relations::*;
use app_shell::domain_foundation::relations::definition::*;
use std::sync::Arc;

// Create a registry
let mut registry = RelationRegistry::new();

// Define: A.property0 → B.property0 (copy value, cascading)
let src = Endpoint::source(PropertyId::new(ObjectId::new(1), 0));
let tgt = Endpoint::target(PropertyId::new(ObjectId::new(2), 0));

registry.register(
    RelationDefinitionBuilder::new(src, tgt)
        .priority(10)
        .propagation(PropagationMode::Cascading)
        .effect(Arc::new(|ctx: &EffectContext| {
            EffectOutcome::Apply(ctx.source_value.clone())
        }))
        .build(),
);

// Propagate a change
let mut ctx = EvaluationContext::new(&registry);
ctx.set_value(PropertyId::new(ObjectId::new(1), 0), PropertyValue::Int(42));

let steps = ctx.propagate_from(PropertyId::new(ObjectId::new(1), 0)).unwrap();
assert_eq!(steps.len(), 1);

// Target received the value
assert_eq!(
    ctx.get_value(PropertyId::new(ObjectId::new(2), 0)),
    &PropertyValue::Int(42),
);
```

### 8. Batch Propagation

```rust
let mut ctx = EvaluationContext::new(&registry);

ctx.begin_batch();
ctx.set_value(prop_a, PropertyValue::Int(42));
ctx.set_value(prop_b, PropertyValue::Text("hello".into()));

let steps = ctx.commit_batch().unwrap();
// Both changes propagated together, deduplicated
```

### 9. Utilities

```rust
use app_shell::domain_foundation::utilities::filtering::*;

// Filter Canvas2D objects by position
let objects: Vec<_> = engine.canvas().iter().copied().collect();
let right_half = filter(&objects, &criteria(|obj: &Object| {
    obj.transform.translation.x > 50.0
}));
```

```rust
use app_shell::domain_foundation::utilities::validation::*;

// Validate table cell values
let rule = rule(|v: &i64| {
    if *v >= 0 {
        ValidationResult::valid()
    } else {
        ValidationResult::invalid("must be non-negative")
    }
});

let failures = validate_all(&[1, -2, 3, -4], &rule);
assert_eq!(failures.len(), 2);
```

```rust
use app_shell::domain_foundation::utilities::tagging::*;

let mut tags = TagSet::new();
tags.add(Tag::new("visible"));
tags.add(Tag::new("locked"));

// Deterministic iteration (sorted)
for tag in &tags {
    println!("{}", tag.as_str());
}
```

### 10. Serde (Optional)

```toml
[dependencies]
app_shell = { path = "...", features = ["serde"] }
```

```rust
// Serialize a Canvas to JSON
let json = serde_json::to_string(engine.canvas()).unwrap();

// Deserialize back
let restored: Canvas = serde_json::from_str(&json).unwrap();
assert_eq!(engine.canvas(), &restored);
```

### 11. Dirty Tracking

```rust
engine.mark_saved();
assert!(!engine.is_dirty());

engine.execute_canvas(CanvasOperation::Create {
    transform: Transform::identity(),
}).unwrap();
assert!(engine.is_dirty());

engine.mark_saved();
assert!(!engine.is_dirty());

engine.undo().unwrap();
assert!(engine.is_dirty());
```

### 12. Command Handlers (App Engine Integration)

```rust
use std::sync::{Arc, Mutex};
use app_shell::app_engine::*;
use app_shell::domain_engine::operations::CanvasCommandHandler;

// Create shared canvas
let canvas = Arc::new(Mutex::new(Canvas::new()));

// Create and register handler
let handler = CanvasCommandHandler::new(Arc::clone(&canvas));
let mut app_engine: AppEngine = AppEngine::new().unwrap();
app_engine.command_executor_mut().register(
    CanvasCommandHandler::definition(),
    Box::new(handler),
).unwrap();

app_engine.initialize().unwrap();
app_engine.start().unwrap();

// Execute via App Engine's command system
let cmd = Command::new(
    "canvas.operation",
    CommandContext::new(ExecutionId::new()),
    CommandInput::new(CanvasOperation::Create {
        transform: Transform::identity(),
    }),
);
let result = app_engine.execute_command(&cmd).unwrap();
assert!(result.is_success());

app_engine.stop().unwrap();
app_engine.dispose().unwrap();
```

---

## Next Steps

- [01-overview.md](01-overview.md) — What Domain Foundation is.
- [02-architecture.md](02-architecture.md) — Architecture and design principles.
- [03-environments.md](03-environments.md) — All 7 environments.
- [11-relations.md](11-relations.md) — Relations system deep dive.
- [13-domain-engine-integration.md](13-domain-engine-integration.md) — Full DomainEngine API.
- [18-public-api-reference.md](18-public-api-reference.md) — Complete type listing.

---

<p align="center">
  <sub>Built with ❤️ in Rust. Zero dependencies. No compromise.</sub>
</p>