# Migration Guide

This guide covers how to handle changes to the Domain Foundation — adding new enum variants, adding fields to structs, deprecating APIs, and maintaining backward compatibility.

---

## Semver and Breaking Changes

The Domain Foundation follows semantic versioning:

| Change type | Version bump | Breaking? |
|---|---|---|
| New environment, utility, or method | Minor (0.x.0) | No |
| New enum variant | Minor | No (with `#[non_exhaustive]`) |
| New struct field with default | Minor | No (with `#[serde(default)]`) |
| Remove or rename a type | Major (x.0.0) | Yes |
| Change a method signature | Major | Yes |
| Change enum variant structure | Major | Yes |

---

## Adding New Enum Variants

Adding a variant to `CanvasOperation` or `CanvasOutcome` is the most common change. It breaks all exhaustive `match` statements on the enum.

### Step 1: Mark enums as `#[non_exhaustive]`

```rust
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum CanvasOperation {
    Create { transform: Transform },
    Move { id: ObjectId, translation: Vec2 },
    // ...
    // Future variants can be added without breaking downstream matches
}
```

With `#[non_exhaustive]`, downstream crates must include a `_` arm in their match statements. This allows adding variants in minor versions.

### Step 2: Add the variant

```rust
#[non_exhaustive]
pub enum CanvasOperation {
    // ... existing ...
    SetColor { id: ObjectId, color: String },  // ← NEW
}
```

### Step 3: Update all match arms

Search for every file that matches on the enum:

```sh
# Find all files that match on CanvasOperation
grep -rn "match.*CanvasOperation" src/ tests/
```

Update each match to include the new variant or a `_` wildcard:

```rust
// In operations.rs apply()
match self {
    CanvasOperation::Create { .. } => { /* ... */ },
    CanvasOperation::SetColor { .. } => { /* ... */ },  // ← ADD
    // If #[non_exhaustive] is used, _ is required for downstream crates
}

// In domain_engine/mod.rs
fn event_name_for_canvas_outcome(outcome: &CanvasOutcome) -> &'static str {
    match outcome {
        // ... existing ...
        CanvasOutcome::ColorSet { .. } => "canvas.object.color_set",  // ← ADD
    }
}
```

### Files to update when adding a variant

| File | What to update |
|---|---|
| `environments/xxx/operations.rs` | `apply()` method |
| `domain_engine/operations/canvas_operations.rs` | `undo()`, `redo()`, `history_entry()`, `event()` |
| `domain_engine/mod.rs` | Name helper functions |
| `domain_engine/operations/canvas_command_handler.rs` | Match arms |
| `tests/.../xxx_operations_test.rs` | New test for the variant |

---

## Adding New Struct Fields

Adding a field to an existing struct is breaking if the struct is constructed externally.

### Option A: Add with `Default` (backward compatible)

```rust
pub struct Object {
    pub id: ObjectId,
    pub transform: Transform,
    pub color: String,  // ← NEW
}

impl Object {
    pub fn new(id: ObjectId, transform: Transform) -> Self {
        Self {
            id,
            transform,
            color: String::new(),  // ← Default value
        }
    }
}
```

Existing code calling `Object::new(id, transform)` still works — `color` is set to default.

### Option B: Use a builder

```rust
pub struct Object {
    pub id: ObjectId,
    pub transform: Transform,
    pub color: String,
}

impl Object {
    pub fn new(id: ObjectId, transform: Transform) -> Self {
        Self { id, transform, color: String::new() }
    }

    pub fn with_color(mut self, color: String) -> Self {
        self.color = color;
        self
    }
}
```

### Serde compatibility

When adding a field, use `#[serde(default)]` so old serialized data can still be deserialized:

```rust
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Object {
    pub id: ObjectId,
    pub transform: Transform,
    #[cfg_attr(feature = "serde", serde(default))]
    pub color: String,  // Old data without "color" field deserializes with default
}
```

---

## Deprecating APIs

### Step 1: Add `#[deprecated]` attribute

```rust
#[deprecated(note = "use execute_canvas() instead", since = "0.2.0")]
pub fn execute(&mut self, operation: CanvasOperation) -> Result<CanvasOutcome, CanvasError> {
    self.execute_canvas(operation).map_err(|e| match e {
        DomainError::Canvas(ce) => ce,
        _ => unreachable!(),
    })
}
```

### Step 2: Keep the old API working

The deprecated method should call the new method internally. Don't remove it until a major version.

### Step 3: Update tests to use the new API

```rust
// Old (deprecated)
let outcome = engine.execute(CanvasOperation::Create { ... }).unwrap();

// New
let outcome = engine.execute_canvas(CanvasOperation::Create { ... }).unwrap();
```

### Step 4: Remove in next major version

When bumping to a major version (e.g., 1.0.0), remove deprecated methods entirely.

---

## Adding New Environments

Adding a new environment is **non-breaking** — no existing code changes:

1. Create the environment under `environments/`
2. Add `pub mod xxx;` to `environments/mod.rs`
3. Add the field, getter, and `execute_xxx()` to `DomainEngine`
4. Add the `DomainError` variant
5. Add tests

No existing match statements need updating because the new environment has its own types.

---

## Adding New Utilities

Adding a new utility is **non-breaking**:

1. Create the utility under `utilities/`
2. Add `pub mod xxx;` to `utilities/mod.rs`
3. Add tests

No existing code changes — utilities are opt-in and don't modify environments.

---

## Serde Compatibility

### Adding serde derives to existing types

Adding `#[cfg_attr(feature = "serde", derive(...))]` is non-breaking — without the feature, the attribute is a no-op.

### Changing field structure

If you change a field's type or structure:

| Change | Old serialized data | Solution |
|---|---|---|
| Add field | Missing in old data | `#[serde(default)]` |
| Remove field | Extra in old data | Serde ignores unknown fields by default |
| Rename field | Different key name | `#[serde(rename = "old_name")]` |
| Change type | Incompatible | Custom `deserialize_with` or version field |

### Version field pattern

For major structural changes, add a version field:

```rust
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CanvasV2 {
    #[serde(default = "default_version")]
    version: u32,
    objects: HashMap<ObjectId, Object>,
    draw_order: Vec<ObjectId>,
    next_id: u64,
}

fn default_version() -> u32 { 1 }
```

---

## Common Migration Scenarios

### Scenario 1: New operation variant added

```text
Before: CanvasOperation has 6 variants
After:  CanvasOperation has 7 variants (added SetColor)

Affected files:
1. operations.rs — add apply arm
2. canvas_operations.rs — add undo/redo/history_entry/event arms
3. domain_engine/mod.rs — add name helper arms
4. canvas_command_handler.rs — add match arms
5. tests — add test for new variant
```

### Scenario 2: New field on a data struct

```text
Before: Object has { id, transform }
After:  Object has { id, transform, color }

Affected files:
1. object.rs — add field, update new(), add with_color()
2. canvas.rs — update create_object (if it sets the field)
3. serde — add #[serde(default)] if applicable
4. tests — update assertions
```

### Scenario 3: Renaming a method

```text
Before: engine.execute(op)
After:  engine.execute_canvas(op)

Steps:
1. Add execute_canvas() as the new method
2. Mark execute() as #[deprecated]
3. Make execute() call execute_canvas() internally
4. Update all internal callers to use execute_canvas()
5. Update tests to use execute_canvas()
6. In next major version, remove execute()
```

### Scenario 4: Removing an operation variant

```text
Before: CanvasOperation has SetColor
After:  SetColor removed (color moved to a different system)

Steps:
1. Mark SetColor as #[deprecated]
2. In next major version, remove the variant
3. Update all match arms
4. Update tests
```

---

## Versioning Strategy

| Version | When to bump | Example |
|---|---|---|
| Patch (0.0.x) | Bug fixes, no API changes | Fix undo bug |
| Minor (0.x.0) | New features, new variants, new environments | Add Measurement environment |
| Major (x.0.0) | Breaking changes (remove types, change signatures) | Remove deprecated execute() |

---

## Checklist for Safe Changes

- [ ] New enum variants: add `#[non_exhaustive]` or update all match arms
- [ ] New struct fields: provide default in `new()` + `#[serde(default)]`
- [ ] Deprecated methods: add `#[deprecated]` + keep working + update tests
- [ ] Serde changes: test round-trip with old and new format
- [ ] All existing tests still pass
- [ ] New tests added for new functionality
- [ ] Documentation updated

---

## Next Steps

- [01-creating-environment.md](01-creating-environment.md) — Creating new environments (non-breaking).
- [02-adding-operations.md](02-adding-operations.md) — Adding operation variants.
- [07-adding-serde.md](07-adding-serde.md) — Serde compatibility.
- [18-public-api-reference.md](../18-public-api-reference.md) — Current API surface.