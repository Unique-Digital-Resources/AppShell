//! Cell identity and value types.

use super::column::ColumnId;
use super::row::RowId;

/// Composite identifier for a cell: a row × column intersection.
///
/// Carries both [`RowId`] and [`ColumnId`] so the table can locate
/// the cell's row and column without a separate index.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellId {
    pub row: RowId,
    pub column: ColumnId,
}

impl CellId {
    pub fn new(row: RowId, column: ColumnId) -> Self {
        Self { row, column }
    }
}

/// A value stored in a cell.
///
/// The foundation supports common scalar types. Domain Engines can
/// define cell calculation dependencies using the Phase 3
/// [`relations`](crate::domain_foundation::relations) system.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum CellValue {
    /// Empty / unset cell.
    None,
    Text(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl Default for CellValue {
    fn default() -> Self {
        CellValue::None
    }
}

impl CellValue {
    pub fn is_none(&self) -> bool {
        matches!(self, CellValue::None)
    }

    pub fn is_some(&self) -> bool {
        !self.is_none()
    }
}