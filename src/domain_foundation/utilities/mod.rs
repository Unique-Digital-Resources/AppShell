//! # Utilities
//!
//! Optional reusable capabilities that a Domain Engine explicitly chooses
//! to compose. Utilities are **not** a mandatory foundation underneath
//! every environment.
//!
//! ```text
//!              Filtering
//!             /    |     \
//!            ▼     ▼      ▼
//!       Canvas  Timeline  Table
//! ```
//!
//! The environments do not need to know that filtering exists.
//!
//! ## Orthogonality
//!
//! Utilities must not create mandatory dependencies such as
//! `Canvas2D → Utilities`. They operate over domain data supplied to them;
//! they do not own the meaning of that data.
//!
//! ## Available utilities
//!
//! - [`filtering`] — evaluate objects against composable criteria.
//! - [`querying`] — named, reusable queries built on filtering.
//! - [`tagging`] — tags and tag collections.
//! - [`sorting`] — ordering by developer-defined criteria.
//! - [`grouping`] — grouping objects by key.
//! - [`validation`] — reusable rules and results.
//! - [`batch`] — multi-object operations.

pub mod batch;
pub mod filtering;
pub mod grouping;
pub mod querying;
pub mod sorting;
pub mod tagging;
pub mod validation;