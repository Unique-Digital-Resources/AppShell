//! Table model: owns rows, columns, cells, and ordering.

use super::cell::{CellId, CellValue};
use super::column::{Column, ColumnId};
use super::error::TableError;
use super::row::{Row, RowId};
use std::collections::HashMap;

/// Headless tabular data model.
///
/// Owns a set of [`Row`]s and [`Column`]s, plus [`CellValue`]s keyed
/// by composite [`CellId`]. Row and column ordering are tracked
/// separately from the HashMap storage so iteration order is stable.
///
/// The table does not enforce cell type schemas, formula semantics,
/// or rendering. Domain Engines layer those externally.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Table {
    rows: HashMap<RowId, Row>,
    columns: HashMap<ColumnId, Column>,
    row_order: Vec<RowId>,
    column_order: Vec<ColumnId>,
    cells: HashMap<CellId, CellValue>,
		merged_cells: HashMap<CellId, CellId>, 
    next_row_id: u64,
    next_column_id: u64,
}

impl Table {
    pub fn new() -> Self {
        Self::default()
    }

    // --- Counts ---

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty() && self.columns.is_empty()
    }

    // --- Row access ---

    pub fn contains_row(&self, id: RowId) -> bool {
        self.rows.contains_key(&id)
    }

    pub fn row(&self, id: RowId) -> Option<&Row> {
        self.rows.get(&id)
    }

    /// Rows in order.
    pub fn rows(&self) -> impl Iterator<Item = &Row> {
        self.row_order.iter().filter_map(|id| self.rows.get(id))
    }

    /// Ordered row ids.
    pub fn row_ids(&self) -> &[RowId] {
        &self.row_order
    }

    // --- Column access ---

    pub fn contains_column(&self, id: ColumnId) -> bool {
        self.columns.contains_key(&id)
    }

    pub fn column(&self, id: ColumnId) -> Option<&Column> {
        self.columns.get(&id)
    }

    /// Columns in order.
    pub fn columns(&self) -> impl Iterator<Item = &Column> {
        self.column_order
            .iter()
            .filter_map(|id| self.columns.get(id))
    }

    /// Ordered column ids.
    pub fn column_ids(&self) -> &[ColumnId] {
        &self.column_order
    }

    // --- Cell access ---

    /// Get a cell's value. Returns `CellValue::None` if the cell is unset
    /// or the row/column doesn't exist.
    pub fn cell(&self, row: RowId, column: ColumnId) -> &CellValue {
        let id = CellId { row, column };
        self.cells.get(&id).unwrap_or(&CellValue::None)
    }

    pub fn cell_by_id(&self, id: CellId) -> &CellValue {
        self.cells.get(&id).unwrap_or(&CellValue::None)
    }

    // --- Row management ---

    pub fn insert_row(&mut self, name: Option<String>) -> RowId {
        let id = RowId::from_raw(self.next_row_id);
        self.next_row_id += 1;
        self.rows.insert(id, Row::new(id, name));
        self.row_order.push(id);
        id
    }

    /// Restore a row with its original id and cells (for undo).
    pub fn restore_row(&mut self, row: Row, cells: Vec<(CellId, CellValue)>) -> RowId {
        let id = row.id;
        if id.as_raw() >= self.next_row_id {
            self.next_row_id = id.as_raw() + 1;
        }
        if !self.row_order.contains(&id) {
            self.row_order.push(id);
        }
        self.rows.insert(id, row);
        for (cell_id, value) in cells {
            self.cells.insert(cell_id, value);
        }
        id
    }

    pub fn remove_row(&mut self, id: RowId) -> Result<(Row, Vec<(CellId, CellValue)>), TableError> {
        let row = self.rows.remove(&id).ok_or(TableError::RowNotFound { id })?;
        self.row_order.retain(|r| *r != id);

        // Collect and remove all cells in this row.
        let cell_ids: Vec<CellId> = self
            .cells
            .keys()
            .filter(|c| c.row == id)
            .copied()
            .collect();
        let mut removed_cells = Vec::new();
        for cid in cell_ids {
            if let Some(v) = self.cells.remove(&cid) {
                removed_cells.push((cid, v));
            }
        }
        Ok((row, removed_cells))
    }

    pub fn rename_row(&mut self, id: RowId, name: Option<String>) -> Result<Option<String>, TableError> {
        let row = self.rows.get_mut(&id).ok_or(TableError::RowNotFound { id })?;
        let previous = row.name.clone();
        row.name = name;
        Ok(previous)
    }

    pub fn move_row(&mut self, id: RowId, new_index: usize) -> Result<usize, TableError> {
        if !self.rows.contains_key(&id) {
            return Err(TableError::RowNotFound { id });
        }
        let current = self
            .row_order
            .iter()
            .position(|r| *r == id)
            .ok_or(TableError::RowNotFound { id })?;
        if new_index >= self.row_order.len() {
            return Err(TableError::InvalidRowIndex {
                index: new_index,
                row_count: self.row_order.len(),
            });
        }
        if current == new_index {
            return Ok(current);
        }
        self.row_order.remove(current);
        self.row_order.insert(new_index, id);
        Ok(current)
    }

    // --- Column management ---

    pub fn insert_column(&mut self, name: Option<String>) -> ColumnId {
        let id = ColumnId::from_raw(self.next_column_id);
        self.next_column_id += 1;
        self.columns.insert(id, Column::new(id, name));
        self.column_order.push(id);
        id
    }

    /// Restore a column with its original id and cells (for undo).
    pub fn restore_column(
        &mut self,
        column: Column,
        cells: Vec<(CellId, CellValue)>,
    ) -> ColumnId {
        let id = column.id;
        if id.as_raw() >= self.next_column_id {
            self.next_column_id = id.as_raw() + 1;
        }
        if !self.column_order.contains(&id) {
            self.column_order.push(id);
        }
        self.columns.insert(id, column);
        for (cell_id, value) in cells {
            self.cells.insert(cell_id, value);
        }
        id
    }

    pub fn remove_column(
        &mut self,
        id: ColumnId,
    ) -> Result<(Column, Vec<(CellId, CellValue)>), TableError> {
        let column = self
            .columns
            .remove(&id)
            .ok_or(TableError::ColumnNotFound { id })?;
        self.column_order.retain(|c| *c != id);

        let cell_ids: Vec<CellId> = self
            .cells
            .keys()
            .filter(|c| c.column == id)
            .copied()
            .collect();
        let mut removed_cells = Vec::new();
        for cid in cell_ids {
            if let Some(v) = self.cells.remove(&cid) {
                removed_cells.push((cid, v));
            }
        }
        Ok((column, removed_cells))
    }

    pub fn rename_column(
        &mut self,
        id: ColumnId,
        name: Option<String>,
    ) -> Result<Option<String>, TableError> {
        let col = self
            .columns
            .get_mut(&id)
            .ok_or(TableError::ColumnNotFound { id })?;
        let previous = col.name.clone();
        col.name = name;
        Ok(previous)
    }

    pub fn move_column(&mut self, id: ColumnId, new_index: usize) -> Result<usize, TableError> {
        if !self.columns.contains_key(&id) {
            return Err(TableError::ColumnNotFound { id });
        }
        let current = self
            .column_order
            .iter()
            .position(|c| *c == id)
            .ok_or(TableError::ColumnNotFound { id })?;
        if new_index >= self.column_order.len() {
            return Err(TableError::InvalidColumnIndex {
                index: new_index,
                column_count: self.column_order.len(),
            });
        }
        if current == new_index {
            return Ok(current);
        }
        self.column_order.remove(current);
        self.column_order.insert(new_index, id);
        Ok(current)
    }

    // --- Cell management ---

    /// Set a cell's value. Returns the previous value.
    pub fn set_cell_value(
        &mut self,
        row: RowId,
        column: ColumnId,
        value: CellValue,
    ) -> Result<CellValue, TableError> {
        if !self.rows.contains_key(&row) {
            return Err(TableError::RowNotFound { id: row });
        }
        if !self.columns.contains_key(&column) {
            return Err(TableError::ColumnNotFound { id: column });
        }
        let id = CellId { row, column };
        let previous = self.cells.remove(&id).unwrap_or(CellValue::None);
        self.cells.insert(id, value);
        Ok(previous)
    }

    /// Clear a cell's value (set to None). Returns the previous value.
    pub fn clear_cell(&mut self, row: RowId, column: ColumnId) -> Result<CellValue, TableError> {
        if !self.rows.contains_key(&row) {
            return Err(TableError::RowNotFound { id: row });
        }
        if !self.columns.contains_key(&column) {
            return Err(TableError::ColumnNotFound { id: column });
        }
        let id = CellId { row, column };
        let previous = self.cells.remove(&id).unwrap_or(CellValue::None);
        Ok(previous)
    }

    // --- Gap 18: Merged cells ---

    /// Merge a rectangular region from `from` (top-left) to `to` (bottom-right).
    /// The top-left cell becomes the master; all others are marked as merged.
    pub fn merge_cells(&mut self, from: CellId, to: CellId) -> Result<(), TableError> {
        for cell_id in &[from, to] {
            if !self.rows.contains_key(&cell_id.row) {
                return Err(TableError::RowNotFound { id: cell_id.row });
            }
            if !self.columns.contains_key(&cell_id.column) {
                return Err(TableError::ColumnNotFound { id: cell_id.column });
            }
        }

        let row_start = self.row_order.iter().position(|r| *r == from.row)
            .ok_or(TableError::RowNotFound { id: from.row })?;
        let row_end = self.row_order.iter().position(|r| *r == to.row)
            .ok_or(TableError::RowNotFound { id: to.row })?;
        let col_start = self.column_order.iter().position(|c| *c == from.column)
            .ok_or(TableError::ColumnNotFound { id: from.column })?;
        let col_end = self.column_order.iter().position(|c| *c == to.column)
            .ok_or(TableError::ColumnNotFound { id: to.column })?;

        let (rs, re_) = if row_start <= row_end { (row_start, row_end) } else { (row_end, row_start) };
        let (cs, ce_) = if col_start <= col_end { (col_start, col_end) } else { (col_end, col_start) };

        let master = CellId {
            row: self.row_order[rs],
            column: self.column_order[cs],
        };

        for ri in rs..=re_ {
            for ci in cs..=ce_ {
                let cell_id = CellId {
                    row: self.row_order[ri],
                    column: self.column_order[ci],
                };
                if cell_id != master {
                    self.merged_cells.insert(cell_id, master);
                }
            }
        }
        Ok(())
    }

    /// Unmerge all cells that were merged into `master`.
    pub fn unmerge_cells(&mut self, master: CellId) -> Result<(), TableError> {
        self.merged_cells.retain(|_, m| *m != master);
        Ok(())
    }

    /// Whether a cell is merged into another.
    pub fn is_merged(&self, id: CellId) -> bool {
        self.merged_cells.contains_key(&id)
    }

    /// The master cell for a merged cell, or the cell itself if not merged.
    pub fn master_cell(&self, id: CellId) -> CellId {
        self.merged_cells.get(&id).copied().unwrap_or(id)
    }

    /// Remove all rows, columns, cells, and merged cells.
    /// Id counters are not reset.
    pub fn clear(&mut self) {
        self.rows.clear();
        self.columns.clear();
        self.row_order.clear();
        self.column_order.clear();
        self.cells.clear();
        self.merged_cells.clear();
    }

    /// Swap two rows by position. Does nothing if indices are equal.
    pub fn swap_rows(&mut self, a: usize, b: usize) {
        if a != b && a < self.row_order.len() && b < self.row_order.len() {
            self.row_order.swap(a, b);
        }
    }

    /// Swap two columns by position. Does nothing if indices are equal.
    pub fn swap_columns(&mut self, a: usize, b: usize) {
        if a != b && a < self.column_order.len() && b < self.column_order.len() {
            self.column_order.swap(a, b);
        }
    }

}

impl<'a> IntoIterator for &'a Table {
    type Item = (&'a CellId, &'a CellValue);
    type IntoIter = std::collections::hash_map::Iter<'a, CellId, CellValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.iter()
    }
}