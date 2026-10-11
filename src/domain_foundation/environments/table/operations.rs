//! Table operations: value-type intents for tabular changes.

use super::cell::{CellId, CellValue};
use super::column::{Column, ColumnId};
use super::error::TableError;
use super::row::{Row, RowId};
use super::table::Table;

/// A table operation.
#[derive(Debug, Clone, PartialEq)]
pub enum TableOperation {
    InsertRow { name: Option<String> },
    InsertColumn { name: Option<String> },
    RemoveRow { id: RowId },
    RemoveColumn { id: ColumnId },
    SetCellValue { row: RowId, column: ColumnId, value: CellValue },
    ClearCell { row: RowId, column: ColumnId },
    MoveRow { id: RowId, new_index: usize },
    MoveColumn { id: ColumnId, new_index: usize },
    RenameRow { id: RowId, name: Option<String> },
    RenameColumn { id: ColumnId, name: Option<String> },
    /// Merge a rectangular region from `from` (top-left) to `to` (bottom-right).
    MergeCells { from: CellId, to: CellId },
    /// Unmerge all cells that were merged into `master`.
    UnmergeCells { master: CellId },
    /// Remove all rows, columns, and cells.
    ClearTable,
}

/// Outcome of a successful [`TableOperation`].
#[derive(Debug, Clone, PartialEq)]
pub enum TableOutcome {
    RowInserted { id: RowId },
    ColumnInserted { id: ColumnId },
    RowRemoved { row: Row, cells: Vec<(CellId, CellValue)> },
    ColumnRemoved { column: Column, cells: Vec<(CellId, CellValue)> },
    CellValueSet { row: RowId, column: ColumnId, previous: CellValue },
    CellCleared { row: RowId, column: ColumnId, previous: CellValue },
    RowMoved { id: RowId, previous_index: usize },
    ColumnMoved { id: ColumnId, previous_index: usize },
    RowRenamed { id: RowId, previous_name: Option<String> },
    ColumnRenamed { id: ColumnId, previous_name: Option<String> },
    CellsMerged,
    CellsUnmerged,
    TableCleared,
}

impl TableOperation {
    pub fn apply(self, table: &mut Table) -> Result<TableOutcome, TableError> {
        match self {
            TableOperation::InsertRow { name } => {
                let id = table.insert_row(name);
                Ok(TableOutcome::RowInserted { id })
            }
            TableOperation::InsertColumn { name } => {
                let id = table.insert_column(name);
                Ok(TableOutcome::ColumnInserted { id })
            }
            TableOperation::RemoveRow { id } => {
                let (row, cells) = table.remove_row(id)?;
                Ok(TableOutcome::RowRemoved { row, cells })
            }
            TableOperation::RemoveColumn { id } => {
                let (column, cells) = table.remove_column(id)?;
                Ok(TableOutcome::ColumnRemoved { column, cells })
            }
            TableOperation::SetCellValue { row, column, value } => {
                let previous = table.set_cell_value(row, column, value)?;
                Ok(TableOutcome::CellValueSet { row, column, previous })
            }
            TableOperation::ClearCell { row, column } => {
                let previous = table.clear_cell(row, column)?;
                Ok(TableOutcome::CellCleared { row, column, previous })
            }
            TableOperation::MoveRow { id, new_index } => {
                let previous_index = table.move_row(id, new_index)?;
                Ok(TableOutcome::RowMoved { id, previous_index })
            }
            TableOperation::MoveColumn { id, new_index } => {
                let previous_index = table.move_column(id, new_index)?;
                Ok(TableOutcome::ColumnMoved { id, previous_index })
            }
            TableOperation::RenameRow { id, name } => {
                let previous_name = table.rename_row(id, name)?;
                Ok(TableOutcome::RowRenamed { id, previous_name })
            }
            TableOperation::RenameColumn { id, name } => {
                let previous_name = table.rename_column(id, name)?;
                Ok(TableOutcome::ColumnRenamed { id, previous_name })
            }
            TableOperation::MergeCells { from, to } => {
                table.merge_cells(from, to)?;
                Ok(TableOutcome::CellsMerged)
            }
            TableOperation::UnmergeCells { master } => {
                table.unmerge_cells(master)?;
                Ok(TableOutcome::CellsUnmerged)
            }
            TableOperation::ClearTable => {
                table.clear();
                Ok(TableOutcome::TableCleared)
            }
        }
    }
}