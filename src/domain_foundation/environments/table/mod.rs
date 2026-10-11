//! # Table Environment
//!
//! Headless tabular data model. Provides rows, columns, cells and
//! ordering without imposing any grid UI, spreadsheet formula engine,
//! or rendering.
//!
//! ## What this environment owns
//!
//! - [`RowId`], [`ColumnId`], and composite [`CellId`] identity.
//! - [`Row`] and [`Column`] with optional names.
//! - [`CellValue`] for cell data (text, int, float, bool, none).
//! - [`Table`] container with stable ids and ordered rows/columns.
//! - [`TableOperation`]s as value-type intents with undoable outcomes.
//!
//! ## What this environment deliberately does *not* own
//!
//! - Spreadsheet formulas or calculation dependencies. A Domain Engine
//!   can define those using the Phase 3
//!   [`relations`](crate::domain_foundation::relations) system.
//! - Grid rendering, cell editors, headers, or freeze panes.
//! - Type-validated cell schemas (each cell accepts any [`CellValue`]).

pub mod cell;
pub mod column;
pub mod error;
pub mod operations;
pub mod row;
pub mod table;

pub use cell::{CellId, CellValue};
pub use column::{Column, ColumnId};
pub use error::TableError;
pub use operations::{TableOperation, TableOutcome};
pub use row::{Row, RowId};
pub use table::Table;