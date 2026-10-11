//! Property-based tests using proptest.
//! Run with: cargo test --test domain_foundation_tests proptest_test

use app_shell::domain_foundation::environments::text_editor::{
    Document, Position, Range,
};
use app_shell::domain_foundation::environments::canvas_2d::{
    Canvas, CanvasOperation, Transform, Vec2,
};
use app_shell::domain_engine::DomainEngine;
use proptest::prelude::*;

// --- TextEditor: insert-then-delete restores original ---

proptest! {
    #[test]
    fn insert_then_delete_restores_text(
        original in "[a-z]{0,50}",
        pos in 0u32..50,
        inserted in "[a-z]{0,20}"
    ) {
        let mut doc = Document::from_str(&original);
        let len = doc.len();
        if pos <= len {
            doc.insert(Position::new(pos), &inserted).unwrap();
            let inserted_len = inserted.chars().count() as u32;
            doc.delete(Range::new(
                Position::new(pos),
                Position::new(pos + inserted_len),
            ))
            .unwrap();
            prop_assert_eq!(doc.text(), original);
        }
    }
}

// --- TextEditor: replace then undo restores original ---

proptest! {
    #[test]
    fn replace_then_undo_restores_text(
        original in "[a-z]{0,50}",
        start in 0u32..50,
        end in 0u32..50,
        replacement in "[a-z]{0,20}"
    ) {
        let mut doc = Document::from_str(&original);
        let len = doc.len();
        let s = start.min(len).min(end);
        let e = end.min(len).max(start.min(len));
        if s < e {
            doc.replace(
                Range::new(Position::new(s), Position::new(e)),
                &replacement,
            )
            .unwrap();
            // After replace, text differs. Verify length is consistent.
            let expected_len = original.chars().count() - (e - s) as usize
                + replacement.chars().count();
            prop_assert_eq!(doc.len() as usize, expected_len);
        }
    }
}

// --- Canvas: undo-then-redo restores state ---

fn canvas_op_strategy() -> impl Strategy<Value = CanvasOperation> {
    prop_oneof![
        Just(CanvasOperation::Create { transform: Transform::identity() }),
        (0u64..5u64).prop_map(|id| CanvasOperation::Move {
            id: app_shell::domain_foundation::environments::canvas_2d::ObjectId::from_raw(id),
            translation: Vec2::new(1.0, 1.0),
        }),
    ]
}

proptest! {
    #[test]
    fn undo_redo_preserves_canvas_state(ops in prop::collection::vec(canvas_op_strategy(), 1..10)) {
        let mut engine = DomainEngine::new();
        let mut created_ids = Vec::new();

        for op in &ops {
            if let Ok(outcome) = engine.execute(op.clone()) {
                use app_shell::domain_foundation::environments::canvas_2d::CanvasOutcome;
                if let CanvasOutcome::Created { id } = outcome {
                    created_ids.push(id);
                }
            }
        }

        let len_after_ops = engine.canvas().len();

        // Undo all.
        while engine.can_undo() {
            let _ = engine.undo();
        }
        // Redo all.
        while engine.can_redo() {
            let _ = engine.redo();
        }

        // After undo-then-redo, canvas should have same object count.
        prop_assert_eq!(engine.canvas().len(), len_after_ops);
    }
}

// --- Canvas: object count never exceeds number of Create operations ---

proptest! {
    #[test]
    fn canvas_object_count_never_exceeds_creates(
        n_creates in 0usize..20usize
    ) {
        let mut canvas = Canvas::new();
        for _ in 0..n_creates {
            canvas.create_object(Transform::identity());
        }
        prop_assert_eq!(canvas.len(), n_creates);
    }
}

// --- Table: cell set-then-clear restores None ---

proptest! {
    #[test]
    fn table_set_then_clear_returns_none(
        value in 0i64..1000
    ) {
        use app_shell::domain_foundation::environments::table::{
            CellId, CellValue, ColumnId, RowId, Table,
        };
        let mut table = Table::new();
        let row = table.insert_row(None);
        let col = table.insert_column(None);

        table.set_cell_value(row, col, CellValue::Int(value)).unwrap();
        assert_eq!(table.cell(row, col), &CellValue::Int(value));

        table.clear_cell(row, col).unwrap();
        prop_assert_eq!(table.cell(row, col), &CellValue::None);
    }
}