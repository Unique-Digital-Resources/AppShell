//! Model and query tests for the Table environment.

use app_shell::domain_foundation::environments::table::{
    CellId, CellValue, ColumnId, RowId, Table, TableError,
};

#[test]
fn new_table_is_empty() {
    let table = Table::new();
    assert!(table.is_empty());
    assert_eq!(table.row_count(), 0);
    assert_eq!(table.column_count(), 0);
    assert_eq!(table.cell_count(), 0);
}

#[test]
fn insert_row_and_column() {
    let mut table = Table::new();
    let row = table.insert_row(Some("Header".to_string()));
    let col = table.insert_column(Some("Name".to_string()));

    assert_eq!(table.row_count(), 1);
    assert_eq!(table.column_count(), 1);
    assert!(table.contains_row(row));
    assert!(table.contains_column(col));
    assert_eq!(table.row(row).unwrap().name, Some("Header".to_string()));
    assert_eq!(table.column(col).unwrap().name, Some("Name".to_string()));
}

#[test]
fn rows_and_columns_are_ordered() {
    let mut table = Table::new();
    let r1 = table.insert_row(None);
    let r2 = table.insert_row(None);
    let r3 = table.insert_row(None);
    let c1 = table.insert_column(None);
    let c2 = table.insert_column(None);

    assert_eq!(table.row_ids(), &[r1, r2, r3]);
    assert_eq!(table.column_ids(), &[c1, c2]);
}

#[test]
fn set_and_get_cell_value() {
    let mut table = Table::new();
    let row = table.insert_row(None);
    let col = table.insert_column(None);

    assert_eq!(table.cell(row, col), &CellValue::None);

    table
        .set_cell_value(row, col, CellValue::Text("hello".to_string()))
        .unwrap();
    assert_eq!(table.cell(row, col), &CellValue::Text("hello".to_string()));
    assert_eq!(table.cell_count(), 1);
}

#[test]
fn set_cell_value_returns_previous() {
    let mut table = Table::new();
    let row = table.insert_row(None);
    let col = table.insert_column(None);

    table
        .set_cell_value(row, col, CellValue::Int(42))
        .unwrap();
    let previous = table
        .set_cell_value(row, col, CellValue::Float(3.14))
        .unwrap();
    assert_eq!(previous, CellValue::Int(42));
}

#[test]
fn clear_cell_returns_previous() {
    let mut table = Table::new();
    let row = table.insert_row(None);
    let col = table.insert_column(None);

    table
        .set_cell_value(row, col, CellValue::Bool(true))
        .unwrap();
    let previous = table.clear_cell(row, col).unwrap();
    assert_eq!(previous, CellValue::Bool(true));
    assert_eq!(table.cell(row, col), &CellValue::None);
    assert_eq!(table.cell_count(), 0);
}

#[test]
fn remove_row_cascades_to_cells() {
    let mut table = Table::new();
    let r1 = table.insert_row(None);
    let r2 = table.insert_row(None);
    let c1 = table.insert_column(None);
    let c2 = table.insert_column(None);

    table.set_cell_value(r1, c1, CellValue::Int(1)).unwrap();
    table.set_cell_value(r1, c2, CellValue::Int(2)).unwrap();
    table.set_cell_value(r2, c1, CellValue::Int(3)).unwrap();

    assert_eq!(table.cell_count(), 3);

    let (row, cells) = table.remove_row(r1).unwrap();
    assert_eq!(row.id, r1);
    assert_eq!(cells.len(), 2);
    assert_eq!(table.row_count(), 1);
    assert_eq!(table.cell_count(), 1); // only r2/c1 remains
    assert_eq!(table.cell(r2, c1), &CellValue::Int(3));
}

#[test]
fn remove_column_cascades_to_cells() {
    let mut table = Table::new();
    let row = table.insert_row(None);
    let c1 = table.insert_column(None);
    let c2 = table.insert_column(None);

    table.set_cell_value(row, c1, CellValue::Int(1)).unwrap();
    table.set_cell_value(row, c2, CellValue::Int(2)).unwrap();

    let (col, cells) = table.remove_column(c1).unwrap();
    assert_eq!(col.id, c1);
    assert_eq!(cells.len(), 1);
    assert_eq!(table.column_count(), 1);
    assert_eq!(table.cell_count(), 1);
    assert_eq!(table.cell(row, c2), &CellValue::Int(2));
}

#[test]
fn move_row_changes_order() {
    let mut table = Table::new();
    let a = table.insert_row(None);
    let b = table.insert_row(None);
    let c = table.insert_row(None);

    let prev = table.move_row(c, 0).unwrap();
    assert_eq!(prev, 2);
    assert_eq!(table.row_ids(), &[c, a, b]);
}

#[test]
fn move_column_changes_order() {
    let mut table = Table::new();
    let a = table.insert_column(None);
    let b = table.insert_column(None);
    let c = table.insert_column(None);

    let prev = table.move_column(a, 2).unwrap();
    assert_eq!(prev, 0);
    assert_eq!(table.column_ids(), &[b, c, a]);
}

#[test]
fn move_row_out_of_range_fails() {
    let mut table = Table::new();
    let a = table.insert_row(None);
    let _ = table.insert_row(None);

    let err = table.move_row(a, 5).unwrap_err();
    assert!(matches!(err, TableError::InvalidRowIndex { .. }));
}

#[test]
fn set_cell_on_missing_row_fails() {
    let mut table = Table::new();
    let col = table.insert_column(None);
    let fake_row = RowId::from_raw(999);

    let err = table
        .set_cell_value(fake_row, col, CellValue::Int(1))
        .unwrap_err();
    assert!(matches!(err, TableError::RowNotFound { .. }));
}

#[test]
fn set_cell_on_missing_column_fails() {
    let mut table = Table::new();
    let row = table.insert_row(None);
    let fake_col = ColumnId::from_raw(999);

    let err = table
        .set_cell_value(row, fake_col, CellValue::Int(1))
        .unwrap_err();
    assert!(matches!(err, TableError::ColumnNotFound { .. }));
}

#[test]
fn restore_row_brings_back_cells() {
    let mut table = Table::new();
    let row = table.insert_row(None);
    let col = table.insert_column(None);
    table
        .set_cell_value(row, col, CellValue::Text("data".to_string()))
        .unwrap();

    let (removed_row, removed_cells) = table.remove_row(row).unwrap();
    assert_eq!(table.cell_count(), 0);

    table.restore_row(removed_row, removed_cells);
    assert!(table.contains_row(row));
    assert_eq!(
        table.cell(row, col),
        &CellValue::Text("data".to_string())
    );
}

#[test]
fn restore_column_brings_back_cells() {
    let mut table = Table::new();
    let row = table.insert_row(None);
    let col = table.insert_column(None);
    table
        .set_cell_value(row, col, CellValue::Int(99))
        .unwrap();

    let (removed_col, removed_cells) = table.remove_column(col).unwrap();
    assert_eq!(table.cell_count(), 0);

    table.restore_column(removed_col, removed_cells);
    assert!(table.contains_column(col));
    assert_eq!(table.cell(row, col), &CellValue::Int(99));
}

#[test]
fn rename_row_and_column() {
    let mut table = Table::new();
    let row = table.insert_row(Some("Old".to_string()));
    let col = table.insert_column(Some("Old".to_string()));

    let prev = table.rename_row(row, Some("New".to_string())).unwrap();
    assert_eq!(prev, Some("Old".to_string()));
    assert_eq!(table.row(row).unwrap().name, Some("New".to_string()));

    let prev = table
        .rename_column(col, Some("New".to_string()))
        .unwrap();
    assert_eq!(prev, Some("Old".to_string()));
    assert_eq!(table.column(col).unwrap().name, Some("New".to_string()));
}

#[test]
fn merge_cells_marks_non_master_as_merged() {
    let mut table = Table::new();
    let r1 = table.insert_row(None);
    let r2 = table.insert_row(None);
    let c1 = table.insert_column(None);
    let c2 = table.insert_column(None);

    let from = CellId { row: r1, column: c1 };
    let to = CellId { row: r2, column: c2 };
    table.merge_cells(from, to).unwrap();

    assert!(!table.is_merged(from)); // master is not merged
    assert!(table.is_merged(CellId { row: r1, column: c2 }));
    assert!(table.is_merged(CellId { row: r2, column: c1 }));
    assert!(table.is_merged(CellId { row: r2, column: c2 }));

    assert_eq!(table.master_cell(CellId { row: r2, column: c2 }), from);
}

#[test]
fn unmerge_cells_frees_merged() {
    let mut table = Table::new();
    let r1 = table.insert_row(None);
    let r2 = table.insert_row(None);
    let c1 = table.insert_column(None);
    let c2 = table.insert_column(None);

    let from = CellId { row: r1, column: c1 };
    let to = CellId { row: r2, column: c2 };
    table.merge_cells(from, to).unwrap();
    assert!(table.is_merged(to));

    table.unmerge_cells(from).unwrap();
    assert!(!table.is_merged(to));
}
#[test]
fn table_clear_empties() {
    let mut table = Table::new();
    table.insert_row(None);
    table.insert_column(None);
    assert_eq!(table.row_count(), 1);
    
    table.clear();
    assert_eq!(table.row_count(), 0);
    assert_eq!(table.column_count(), 0);
    assert_eq!(table.cell_count(), 0);
}

#[test]
fn table_partial_eq_works() {
    let a = Table::new();
    let b = Table::new();
    assert_eq!(a, b);
}

#[test]
fn swap_rows_and_columns() {
    let mut table = Table::new();
    let r1 = table.insert_row(Some("A".to_string()));
    let r2 = table.insert_row(Some("B".to_string()));
    let r3 = table.insert_row(Some("C".to_string()));

    // Swap rows 0 and 2.
    table.swap_rows(0, 2);
    assert_eq!(table.row_ids(), &[r3, r2, r1]);

    // Swap back.
    table.swap_rows(0, 2);
    assert_eq!(table.row_ids(), &[r1, r2, r3]);

    // Swap same index — no-op.
    table.swap_rows(1, 1);
    assert_eq!(table.row_ids(), &[r1, r2, r3]);

    let c1 = table.insert_column(Some("X".to_string()));
    let c2 = table.insert_column(Some("Y".to_string()));

    table.swap_columns(0, 1);
    assert_eq!(table.column_ids(), &[c2, c1]);
    table.swap_columns(0, 1);
    assert_eq!(table.column_ids(), &[c1, c2]);
}

#[test]
fn table_into_iterator() {
    let mut table = Table::new();
    let row = table.insert_row(None);
    let col = table.insert_column(None);
    table.set_cell_value(row, col, CellValue::Int(42)).unwrap();

    let entries: Vec<_> = (&table).into_iter().collect();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].1, &CellValue::Int(42));
}