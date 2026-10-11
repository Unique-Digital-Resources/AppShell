//! Table-specific errors.

use super::column::ColumnId;
use super::row::RowId;
use std::fmt;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableError {
    RowNotFound { id: RowId },
    ColumnNotFound { id: ColumnId },
    InvalidRowIndex { index: usize, row_count: usize },
    InvalidColumnIndex { index: usize, column_count: usize },
}

impl fmt::Display for TableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TableError::RowNotFound { id } => write!(f, "row {id} not found"),
            TableError::ColumnNotFound { id } => write!(f, "column {id} not found"),
            TableError::InvalidRowIndex { index, row_count } => {
                write!(f, "row index {index} out of range (have {row_count} rows)")
            }
            TableError::InvalidColumnIndex { index, column_count } => {
                write!(f, "column index {index} out of range (have {column_count} columns)")
            }
        }
    }
}

impl std::error::Error for TableError {}