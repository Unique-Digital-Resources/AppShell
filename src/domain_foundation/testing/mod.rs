//! # Testing support
//!
//! Shared test helpers and fixtures for the Domain Foundation. This module
//! is part of the public API so that integration tests under `tests/` and
//! downstream Domain Engines can reuse the same fixtures.
//!
//! It contains no production behaviour. Nothing outside of test code should
//! depend on it.

pub mod fixtures;