//! # TextEditor Environment
//!
//! Headless text editing model. Provides a [`Document`] with
//! [`Position`] and [`Range`] semantics, plus insert, delete, replace
//! and clear operations — without any editor widget, rendering, or
//! syntax parsing.
//!
//! ## Indexing convention: Unicode scalar values
//!
//! Positions are **character offsets**, not UTF-8 byte offsets. The
//! document's `len()` returns the count of Unicode scalar values (code
//! points), matching Rust's `str::chars()`. Multibyte characters (é,
//! emoji, CJK) each occupy exactly one position.
//!
//! ```text
//! Document: "héllo"
//! Positions: 0  1  2  3  4  5
//!            h  é  l  l  o
//! ```
//!
//! Position 0 is before the first character. Position `len()` is after
//! the last character. Valid positions are `0..=len()`.
//!
//! ## What this environment owns
//!
//! - [`Document`] text storage and query.
//! - [`Position`] and [`Range`] for addressing text spans.
//! - [`TextEditorOperation`]s as value-type intents with undoable outcomes.
//!
//! ## What this environment deliberately does *not* own
//!
//! - Parsing, syntax highlighting, or language server features.
//! - Editor widget, cursor rendering, or input handling.
//! - File I/O or persistence.

pub mod document;
pub mod error;
pub mod operations;
pub mod position;
pub mod range;

pub use document::Document;
pub use error::TextEditorError;
pub use operations::{TextEditorOperation, TextEditorOutcome};
pub use position::Position;
pub use range::Range;