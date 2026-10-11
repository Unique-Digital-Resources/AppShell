//! Reusable test fixtures for the Domain Foundation.
//!
//! Phase 1 establishes only the file. Concrete fixtures are added as later
//! phases introduce environments, relations and utilities that benefit from
//! shared test data. Fixtures are added *when a real test needs them*, not
//! speculatively.
//!
//! ## Conventions
//!
//! - Fixtures live here only when at least two test modules benefit from
//!   sharing them. Single-use helpers stay in the test that uses them.
//! - Fixtures must not encode application-specific meaning. They describe
//!   neutral domain shapes (an object with an id and a transform, a
//!   timeline with N tracks, etc.) reusable across Domain Engines.
//! - Fixtures must not depend on UI or rendering code.

// Placeholder so the module compiles in Phase 1. Removed once the first
// real fixture is added in Phase 2 (Canvas2D).
#[allow(dead_code)]
pub(crate) const _PHASE1_PLACEHOLDER: () = ();