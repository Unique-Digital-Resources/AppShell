//! Operation tests for the Table environment.

use app_shell::domain_foundation::environments::table::{
    CellId, CellValue, ColumnId, RowId, Table, TableError, TableOperation, TableOutcome,
};

fn setup_table() -> (Table, RowId, ColumnId) {
    let mut table = Table::new();
    let row = table.insert_row(Some("R1".to_string()));
    let col = table.insert_column(Some("C1".to_string()));
    (table, row, col)
}

#[test]
fn insert_row_operation() {
    let mut table = Table::new();
    let outcome = TableOperation::InsertRow {
        name: Some("Header".to_string()),
    }
    .apply(&mut table)
    .unwrap();

    let id = match outcome {
        TableOutcome::RowInserted { id } => id,
        other => panic!("expected RowInserted, got {other:?}"),
    };
    assert!(table.contains_row(id));
    assert_eq!(table.row(id).unwrap().name, Some("Header".to_string()));
}

#[test]
fn insert_column_operation() {
    let mut table = Table::new();
    let outcome = TableOperation::InsertColumn {
        name: Some("Name".to_string()),
    }
    .apply(&mut table)
    .unwrap();

    let id = match outcome {
        TableOutcome::ColumnInserted { id } => id,
        other => panic!("expected ColumnInserted, got {other:?}"),
    };
    assert!(table.contains_column(id));
}

#[test]
fn remove_row_operation_returns_cells() {
    let (mut table, row, col) = setup_table();
    table
        .set_cell_value(row, col, CellValue::Int(42))
        .unwrap();

    let outcome = TableOperation::RemoveRow { id: row }
        .apply(&mut table)
        .unwrap();

    match outcome {
        TableOutcome::RowRemoved { row: r, cells } => {
            assert_eq!(r.id, row);
            assert_eq!(cells.len(), 1);
        }
        other => panic!("expected RowRemoved, got {other:?}"),
    }
    assert!(!table.contains_row(row));
    assert_eq!(table.cell_count(), 0);
}

#[test]
fn remove_column_operation_returns_cells() {
    let (mut table, row, col) = setup_table();
    table
        .set_cell_value(row, col, CellValue::Text("x".to_string()))
        .unwrap();

    let outcome = TableOperation::RemoveColumn { id: col }
        .apply(&mut table)
        .unwrap();

    match outcome {
        TableOutcome::ColumnRemoved { column, cells } => {
            assert_eq!(column.id, col);
            assert_eq!(cells.len(), 1);
        }
        other => panic!("expected ColumnRemoved, got {other:?}"),
    }
    assert!(!table.contains_column(col));
}

#[test]
fn set_cell_value_operation() {
    let (mut table, row, col) = setup_table();

    let outcome = TableOperation::SetCellValue {
        row,
        column: col,
        value: CellValue::Float(3.14),
    }
    .apply(&mut table)
    .unwrap();

    match outcome {
        TableOutcome::CellValueSet { previous, .. } => {
            assert_eq!(previous, CellValue::None);
        }
        other => panic!("expected CellValueSet, got {other:?}"),
    }
    assert_eq!(table.cell(row, col), &CellValue::Float(3.14));
}

#[test]
fn clear_cell_operation() {
    let (mut table, row, col) = setup_table();
    table
        .set_cell_value(row, col, CellValue::Bool(true))
        .unwrap();

    let outcome = TableOperation::ClearCell { row, column: col }
        .apply(&mut table)
        .unwrap();

    match outcome {
        TableOutcome::CellCleared { previous, .. } => {
            assert_eq!(previous, CellValue::Bool(true));
        }
        other => panic!("expected CellCleared, got {other:?}"),
    }
    assert_eq!(table.cell(row, col), &CellValue::None);
}

#[test]
fn move_row_operation() {
    let mut table = Table::new();
    let a = table.insert_row(None);
    let _b = table.insert_row(None);
    let c = table.insert_row(None);

    let outcome = TableOperation::MoveRow { id: c, new_index: 0 }
        .apply(&mut table)
        .unwrap();

    match outcome {
        TableOutcome::RowMoved { previous_index, .. } => {
            assert_eq!(previous_index, 2);
        }
        other => panic!("expected RowMoved, got {other:?}"),
    }
    assert_eq!(table.row_ids(), &[c, a, _b]);
}

#[test]
fn move_column_operation() {
    let mut table = Table::new();
    let a = table.insert_column(None);
    let b = table.insert_column(None);

    let outcome = TableOperation::MoveColumn { id: b, new_index: 0 }
        .apply(&mut table)
        .unwrap();

    match outcome {
        TableOutcome::ColumnMoved { previous_index, .. } => {
            assert_eq!(previous_index, 1);
        }
        other => panic!("expected ColumnMoved, got {other:?}"),
    }
    assert_eq!(table.column_ids(), &[b, a]);
}

#[test]
fn operations_on_missing_row_fail() {
    let mut table = Table::new();
    let col = table.insert_column(None);
    let fake = RowId::from_raw(999);

    let err = TableOperation::RemoveRow { id: fake }
        .apply(&mut table)
        .unwrap_err();
    assert!(matches!(err, TableError::RowNotFound { .. }));

    let err = TableOperation::SetCellValue {
        row: fake,
        column: col,
        value: CellValue::Int(1),
    }
    .apply(&mut table)
    .unwrap_err();
    assert!(matches!(err, TableError::RowNotFound { .. }));
}

#[test]
fn operations_on_missing_column_fail() {
    let mut table = Table::new();
    let row = table.insert_row(None);
    let fake = ColumnId::from_raw(999);

    let err = TableOperation::RemoveColumn { id: fake }
        .apply(&mut table)
        .unwrap_err();
    assert!(matches!(err, TableError::ColumnNotFound { .. }));
}

#[test]
fn sequence_of_operations_composes() {
    let mut table = Table::new();

    // Insert rows and columns.
    let outcome = TableOperation::InsertRow {
        name: Some("A".to_string()),
    }
    .apply(&mut table)
    .unwrap();
    let row_a = match outcome {
        TableOutcome::RowInserted { id } => id,
        _ => unreachable!(),
    };

    let outcome = TableOperation::InsertRow {
        name: Some("B".to_string()),
    }
    .apply(&mut table)
    .unwrap();
    let row_b = match outcome {
        TableOutcome::RowInserted { id } => id,
        _ => unreachable!(),
    };

    let outcome = TableOperation::InsertColumn {
        name: Some("Value".to_string()),
    }
    .apply(&mut table)
    .unwrap();
    let col = match outcome {
        TableOutcome::ColumnInserted { id } => id,
        _ => unreachable!(),
    };

    // Set cell values.
    TableOperation::SetCellValue {
        row: row_a,
        column: col,
        value: CellValue::Int(10),
    }
    .apply(&mut table)
    .unwrap();
    TableOperation::SetCellValue {
        row: row_b,
        column: col,
        value: CellValue::Int(20),
    }
    .apply(&mut table)
    .unwrap();

    // Move row B to top.
    TableOperation::MoveRow {
        id: row_b,
        new_index: 0,
    }
    .apply(&mut table)
    .unwrap();

    // Verify.
    assert_eq!(table.row_count(), 2);
    assert_eq!(table.column_count(), 1);
    assert_eq!(table.cell_count(), 2);
    assert_eq!(table.row_ids(), &[row_b, row_a]);
    assert_eq!(table.cell(row_a, col), &CellValue::Int(10));
    assert_eq!(table.cell(row_b, col), &CellValue::Int(20));
}

#[test]
fn merge_cells_operation() {
    let mut table = Table::new();
    let r1 = table.insert_row(None);
    let r2 = table.insert_row(None);
    let c1 = table.insert_column(None);
    let c2 = table.insert_column(None);

    let outcome = TableOperation::MergeCells {
        from: CellId { row: r1, column: c1 },
        to: CellId { row: r2, column: c2 },
    }
    .apply(&mut table)
    .unwrap();
    assert!(matches!(outcome, TableOutcome::CellsMerged));
    assert!(table.is_merged(CellId { row: r2, column: c2 }));
}

#[test]
fn unmerge_cells_operation() {
    let mut table = Table::new();
    let r1 = table.insert_row(None);
    let r2 = table.insert_row(None);
    let c1 = table.insert_column(None);
    let c2 = table.insert_column(None);

    let from = CellId { row: r1, column: c1 };
    let to = CellId { row: r2, column: c2 };
    table.merge_cells(from, to).unwrap();

    let outcome = TableOperation::UnmergeCells { master: from }
        .apply(&mut table)
        .unwrap();
    assert!(matches!(outcome, TableOutcome::CellsUnmerged));
    assert!(!table.is_merged(to));
}

#[test]
fn clear_table_operation() {
    let mut table = Table::new();
    table.insert_row(None);
    table.insert_column(None);
    table.set_cell_value(
        table.row_ids()[0],
        table.column_ids()[0],
        CellValue::Int(42),
    ).unwrap();

    let outcome = TableOperation::ClearTable
        .apply(&mut table)
        .unwrap();
    assert!(matches!(outcome, TableOutcome::TableCleared));
    assert!(table.is_empty());
    assert_eq!(table.row_count(), 0);
    assert_eq!(table.column_count(), 0);
}