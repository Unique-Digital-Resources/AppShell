# Testing conventions

*Read when: writing or running tests, adding fixtures, or checking regressions.
Stateless — current test counts live in `current_state/CHECKPOINT.md`.*

## Suite layout

- **Integration suites** (external crates): `tests/app_engine_tests/` and
  `tests/domain_foundation_tests/`. Each is one Rust binary driven by
  `tests/<suite>/main.rs` declaring one `mod <name>_test;` per sibling `.rs` file.
- **In-crate unit tests**: `#[cfg(test)] mod …` inside source files (lib target).
- **Benches**: `benches/domain_foundation_bench.rs` (criterion, `harness = false`).

## Rules

1. Integration tests use the **public crate API only**. They must not need
   `pub(crate)` access.
2. **One file per feature area** (`canvas_2d_test.rs`, `layers_operations_test.rs`, …)
   registered in that suite's `main.rs`. Suite/filenames are flattened: `mod` names
   come from file names.
3. Shared fixtures live in `src/domain_foundation/testing/fixtures.rs` **only when ≥2
   suites reuse the same shape**; otherwise build fixtures locally.
4. Every accepted change must leave **0 regressions** — run the full suite
   (`cargo test`), not only the new file.
5. Invalid / unreachable states that the public API cannot produce are tested
   **in-crate** through `pub(crate)` mutation paths (see pattern below), keeping the
   public API enforcement honest.

## Verification commands (canonical)

```bash
cargo test --test domain_foundation_tests -- <name>   # one area
cargo test                                            # full suite
cargo check --features serde                          # serde path compiles
cargo test --features serde                           # serde-gated tests run
```

## In-crate invalid-state fixture pattern (docking precedented)

When public creation preconditions make a broken state impossible (which is good),
fixtures for the *imported / restored* equivalents use:

- `pub(crate)` mutators from a `#[cfg(test)] mod …` in the same crate:
  `put_*`, `take_*`, `get_mut`, registry/assignment mutators;
- test-only bypass constructors: `#[cfg(test)] pub(crate) fn from_weights_unchecked(...)`;
- inside the environment module tree, `#[cfg(test)]` submodules may also reach
  private fields of their *own* file's types (module-descendant visibility).
- Record the strategy in the test module docs so the next agent knows it is the
  sanctioned route (see `docking/validation.rs` `validation_tests`).

## Serde-gated tests

- Tests needing serde must be `#[cfg(all(test, feature = "serde"))]` (in-crate) or
  `#[cfg(feature = "serde")]` at the `mod` declaration in a suite's `main.rs`
  (see `tests/domain_foundation_tests/serde_roundtrip_test.rs`).
- Serialization round-trip convention: build model → `serde_json::to_string` →
  `from_str` → `assert_eq!(original, restored)`.
- Imported-state validation: feed structurally well-formed JSON that still violates
  invariants and assert the exact error variant (docking `validation_tests::serde_import`
  precedent). Note: JSON cannot carry `f64::NaN` — pick representable invalid values.

## Feature-gating hygiene

`cargo check --features serde` / `cargo test --features serde` must stay green; new
`#[cfg(test)]`-only helpers must not leak into non-test builds (`cfg(test)` on the
item, not the module).
