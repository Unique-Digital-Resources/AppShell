# Writing Tests

This guide covers the testing conventions, patterns, and categories used throughout the Domain Foundation. Follow these patterns when adding tests for new environments or operations.

---

## Test Organization

All Domain Foundation tests live under `tests/domain_foundation_tests/` as submodules of a single test binary:

```text
tests/
└── domain_foundation_tests/
    ├── main.rs                    ← Entry point: declares all submodules
    ├── canvas_2d_test.rs          ← Model tests
    ├── canvas_2d_operations_test.rs
    ├── ...
    ├── concurrency_test.rs
    ├── proptest_test.rs
    ├── stress_test.rs
    └── serde_roundtrip_test.rs
```

### Adding a new test file

1. Create `tests/domain_foundation_tests/my_thing_test.rs`
2. Add `mod my_thing_test;` to `main.rs`

```rust
// tests/domain_foundation_tests/main.rs

mod canvas_2d_test;
mod canvas_2d_operations_test;
mod my_thing_test;  // ← ADD
```

### File naming convention

Test files use the `_test` suffix: `canvas_2d_test.rs`, `relation_propagation_test.rs`. The module name in `main.rs` matches without the `.rs` extension.

---

## Test Categories

### 1. Model Tests

Verify the environment's types, queries, and constraints:

```rust
#[test]
fn create_returns_unique_ids() {
    let mut env = MyEnvironment::new();
    let a = env.create(/* ... */);
    let b = env.create(/* ... */);
    assert_ne!(a, b);
}

#[test]
fn remove_returns_object() {
    let mut env = MyEnvironment::new();
    let id = env.create(/* ... */);
    let removed = env.remove(id).unwrap();
    assert_eq!(removed.id, id);
}

#[test]
fn clear_preserves_id_counter() {
    let mut env = MyEnvironment::new();
    let first = env.create(/* ... */);
    env.clear();
    let second = env.create(/* ... */);
    assert_ne!(first, second);  // Ids don't collide
}
```

### 2. Operation Tests

Verify each operation variant, error cases, and composition:

```rust
#[test]
fn create_operation_adds_object() {
    let mut env = MyEnvironment::new();
    let outcome = MyOperation::Create { /* ... */ }
        .apply(&mut env)
        .unwrap();
    assert!(matches!(outcome, MyOutcome::Created { .. }));
    assert_eq!(env.len(), 1);
}

#[test]
fn operation_on_missing_id_fails() {
    let mut env = MyEnvironment::new();
    let fake_id = MyId::from_raw(999);
    let err = MyOperation::Remove { id: fake_id }
        .apply(&mut env)
        .unwrap_err();
    assert!(matches!(err, MyError::NotFound { .. }));
}

#[test]
fn sequence_of_operations_composes() {
    let mut env = MyEnvironment::new();
    // Execute multiple operations
    MyOperation::Create { /* ... */ }.apply(&mut env).unwrap();
    MyOperation::Create { /* ... */ }.apply(&mut env).unwrap();
    // Verify final state
    assert_eq!(env.len(), 2);
}
```

### 3. Integration Tests (DomainEngine + App Engine)

Verify undo/redo, transactions, events:

```rust
#[test]
fn undo_restores_state() {
    let mut engine = DomainEngine::new();
    let id = match engine.execute_canvas(CanvasOperation::Create { /* ... */ }).unwrap() {
        CanvasOutcome::Created { id } => id,
        _ => unreachable!(),
    };
    engine.execute_canvas(CanvasOperation::Move { id, /* ... */ }).unwrap();

    engine.undo().unwrap();  // Undo move
    assert_eq!(engine.canvas().get(id).unwrap().transform.translation, Vec2::new(0.0, 0.0));
}

#[test]
fn transaction_undoes_all_at_once() {
    let mut engine = DomainEngine::new();
    engine.begin_transaction();
    engine.execute_canvas(Create { /* ... */ }).unwrap();
    engine.execute_canvas(Create { /* ... */ }).unwrap();
    engine.commit_transaction();

    engine.undo().unwrap();  // Undoes both
    assert_eq!(engine.canvas().len(), 0);
}
```

### 4. Composition Tests

Verify architectural boundaries:

```rust
#[test]
fn my_env_works_without_others() {
    let mut env = MyEnvironment::new();
    let id = env.create(/* ... */);
    assert!(env.contains(id));
}

#[test]
fn composed_env_separate_concerns() {
    let mut graph = NodeGraph::new();
    let n = graph.create_node("N".into(), Transform::identity());

    // Spatial operation doesn't affect graph
    graph.move_node(n, Vec2::new(50.0, 50.0)).unwrap();
    assert_eq!(graph.edge_count(), 0);  // No edges affected
}
```

### 5. Concurrency Tests

Verify `Send + Sync` and thread-safe access:

```rust
#[test]
fn my_env_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<MyEnvironment>();
}

#[test]
fn my_env_shared_across_threads() {
    let env = Arc::new(Mutex::new(MyEnvironment::new()));
    let handles: Vec<_> = (0..4).map(|i| {
        let env = Arc::clone(&env);
        thread::spawn(move || {
            let mut e = env.lock().unwrap();
            e.create(/* ... */);
        })
    }).collect();
    for h in handles { h.join().unwrap(); }
    assert_eq!(env.lock().unwrap().len(), 4);
}
```

### 6. Property-Based Tests (proptest)

Verify invariants across random inputs:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn create_then_remove_preserves_empty(
        count in 0u32..20u32
    ) {
        let mut env = MyEnvironment::new();
        let mut ids = Vec::new();
        for _ in 0..count {
            ids.push(env.create(/* ... */));
        }
        for id in &ids {
            env.remove(*id).unwrap();
        }
        prop_assert!(env.is_empty());
    }

    #[test]
    fn undo_redo_preserves_state(
        ops in prop::collection::vec(create_op_strategy(), 1..10)
    ) {
        let mut engine = DomainEngine::new();
        let len_before = engine.canvas().len();
        for op in &ops {
            let _ = engine.execute(op.clone());
        }
        let len_after = engine.canvas().len();
        // Undo all
        while engine.can_undo() { engine.undo().unwrap(); }
        // Redo all
        while engine.can_redo() { engine.redo().unwrap(); }
        prop_assert_eq!(engine.canvas().len(), len_after);
    }
}
```

### 7. Stress Tests

Large-scale operations:

```rust
#[test]
fn my_env_100k_items() {
    let mut env = MyEnvironment::new();
    for i in 0..100_000u64 {
        env.create(/* ... */);
    }
    assert_eq!(env.len(), 100_000);
}
```

### 8. Serde Round-Trip Tests

```rust
#[cfg(feature = "serde")]
#[test]
fn my_env_roundtrip() {
    let mut env = MyEnvironment::new();
    env.create(/* ... */);
    env.create(/* ... */);

    let json = serde_json::to_string(&env).unwrap();
    let restored: MyEnvironment = serde_json::from_str(&json).unwrap();

    assert_eq!(env, restored);
}
```

---

## The Struct-Literal-in-Match Rule

!!! warning "Most common test compilation error"
    Never write `match Foo::Variant { fields }.method() { ... }`.

    The parser sees `match EXPR {` and can't tell if the `{` is the struct literal or the match body.

### Wrong

```rust
// ❌ Won't compile
let id = match MyOperation::Create { name: "x".into() }
    .apply(&mut env)
    .unwrap()
{
    MyOutcome::Created { id } => id,
    _ => unreachable!(),
};
```

### Correct

```rust
// ✅ Bind to a variable first
let outcome = MyOperation::Create { name: "x".into() }
    .apply(&mut env)
    .unwrap();
let id = match outcome {
    MyOutcome::Created { id } => id,
    _ => unreachable!(),
};
```

This is the **most common compilation error** in the Domain Foundation test suite. Always bind the `.apply().unwrap()` result to a variable before matching.

---

## Import Conventions

### Standard imports

```rust
use app_shell::domain_foundation::environments::canvas_2d::{
    Canvas, CanvasError, Object, ObjectId, Transform, Vec2,
};
```

### Trait imports for methods

If a type's method comes from a trait, import the trait:

```rust
// FilterCriteria provides .and(), .or(), .not()
use app_shell::domain_foundation::utilities::filtering::{criteria, FilterCriteria};

let c = criteria(|x: &i32| *x > 0).and(criteria(|x| *x < 10));
```

```rust
// ValidationRule provides .validate()
use app_shell::domain_foundation::utilities::validation::{rule, ValidationRule};

assert!(r.validate(&5).is_valid());
```

### DomainEngine imports

```rust
use app_shell::domain_engine::DomainEngine;
use app_shell::domain_foundation::environments::canvas_2d::CanvasOperation;
```

### Crate name

All test files import via the crate name `app_shell::...`, not `crate::...`:

```rust
// ✅ Correct (integration test)
use app_shell::domain_foundation::environments::canvas_2d::Canvas;

// ❌ Wrong (only works inside the crate, not in integration tests)
use crate::domain_foundation::environments::canvas_2d::Canvas;
```

---

## Using `PartialEq` for Assertions

All environment containers derive `PartialEq`. Use `assert_eq!` for value comparisons:

```rust
#[test]
fn partial_eq_works() {
    let mut a = Canvas::new();
    let mut b = Canvas::new();

    a.create_object(Transform::identity());
    b.create_object(Transform::identity());

    assert_eq!(a, b);  // Same content, same next_id
}
```

### When `PartialEq` fails

Two canvases with the same objects but different `next_id` are **not equal**:

```rust
let mut a = Canvas::new();
a.create_object(Transform::identity());  // next_id = 1
a.create_object(Transform::identity());  // next_id = 2
a.clear();                               // next_id still 2

let mut b = Canvas::new();
b.create_object(Transform::identity());  // next_id = 1

// a is empty (len=0), b has 1 object, but next_id differs
assert_ne!(a, b);  // a.next_id=2, b.next_id=1
```

### Comparing specific fields

When `PartialEq` fails due to `next_id` or id differences, compare specific fields:

```rust
assert_eq!(a.len(), b.len());
assert_eq!(a.get(id).unwrap().transform, b.get(id).unwrap().transform);
```

---

## Test Count Expectations

Each new environment should add approximately:

| Test category | Tests |
|---|---|
| Model tests | 10-15 |
| Operation tests | 8-12 (one per variant + errors + composition) |
| Integration tests | 3-5 (undo, redo, transaction) |
| Concurrency | 2 (Send+Sync, shared mutation) |
| Serde round-trip | 1 |
| Stress | 1 |
| **Total** | ~25-35 tests per environment |

---

## Running Tests

```sh
# All Domain Foundation tests
cargo test --test domain_foundation_tests

# With serde feature (enables round-trip tests)
cargo test --features serde --test domain_foundation_tests

# Specific test module
cargo test --test domain_foundation_tests canvas_2d_test

# Specific test function
cargo test --test domain_foundation_tests create_object_returns_unique_ids

# Stress tests only
cargo test --test domain_foundation_tests stress

# App Engine tests
cargo test --test app_engine_tests

# All tests
cargo test

# Benchmarks
cargo bench
```

---

## Checklist

- [ ] Test file created with `_test.rs` suffix
- [ ] Module declared in `main.rs`
- [ ] Model tests (create, remove, clear, queries)
- [ ] Operation tests (each variant + errors + composition)
- [ ] Integration test (undo/redo via DomainEngine)
- [ ] Concurrency test (Send + Sync + threads)
- [ ] Serde round-trip test (if applicable)
- [ ] Stress test (large scale)
- [ ] No struct-literal-in-match patterns
- [ ] All imports use `app_shell::...` (not `crate::...`)
- [ ] Trait imports included where needed

---

## Next Steps

- [01-creating-environment.md](01-creating-environment.md) — Creating a new environment (includes tests).
- [02-adding-operations.md](02-adding-operations.md) — Adding operations (includes test patterns).
- [17-testing.md](../17-testing.md) — Testing reference.
- [07-adding-serde.md](07-adding-serde.md) — Serde round-trip tests.