//! Operation tests for the TextEditor environment.

use app_shell::domain_foundation::environments::text_editor::{
    Document, Position, Range, TextEditorError, TextEditorOperation, TextEditorOutcome,
};

#[test]
fn insert_operation() {
    let mut doc = Document::from_str("hello");

    let outcome = TextEditorOperation::Insert {
        position: Position::new(5),
        text: " world".to_string(),
    }
    .apply(&mut doc)
    .unwrap();

    match outcome {
        TextEditorOutcome::Inserted { range } => {
            assert_eq!(range, Range::new(Position::new(5), Position::new(11)));
        }
        other => panic!("expected Inserted, got {other:?}"),
    }
    assert_eq!(doc.text(), "hello world");
}

#[test]
fn delete_operation_returns_deleted_text() {
    let mut doc = Document::from_str("hello world");

    let outcome = TextEditorOperation::Delete {
        range: Range::new(Position::new(0), Position::new(6)),
    }
    .apply(&mut doc)
    .unwrap();

    match outcome {
        TextEditorOutcome::Deleted { range, text } => {
            assert_eq!(range, Range::new(Position::new(0), Position::new(6)));
            assert_eq!(text, "hello ");
        }
        other => panic!("expected Deleted, got {other:?}"),
    }
    assert_eq!(doc.text(), "world");
}

#[test]
fn replace_operation_returns_previous() {
    let mut doc = Document::from_str("hello world");

    let outcome = TextEditorOperation::Replace {
        range: Range::new(Position::new(0), Position::new(5)),
        text: "goodbye".to_string(),
    }
    .apply(&mut doc)
    .unwrap();

    match outcome {
        TextEditorOutcome::Replaced { range, previous_text } => {
            assert_eq!(range, Range::new(Position::new(0), Position::new(5)));
            assert_eq!(previous_text, "hello");
        }
        other => panic!("expected Replaced, got {other:?}"),
    }
    assert_eq!(doc.text(), "goodbye world");
}

#[test]
fn clear_operation_returns_previous() {
    let mut doc = Document::from_str("hello");

    let outcome = TextEditorOperation::Clear
        .apply(&mut doc)
        .unwrap();

    match outcome {
        TextEditorOutcome::Cleared { previous_text } => {
            assert_eq!(previous_text, "hello");
        }
        other => panic!("expected Cleared, got {other:?}"),
    }
    assert!(doc.is_empty());
}

#[test]
fn insert_out_of_bounds_fails() {
    let mut doc = Document::from_str("abc");
    let err = TextEditorOperation::Insert {
        position: Position::new(10),
        text: "x".to_string(),
    }
    .apply(&mut doc)
    .unwrap_err();
    assert!(matches!(err, TextEditorError::PositionOutOfBounds { .. }));
}

#[test]
fn delete_invalid_range_fails() {
    let mut doc = Document::from_str("abc");
    let err = TextEditorOperation::Delete {
        range: Range::new(Position::new(2), Position::new(1)),
    }
    .apply(&mut doc)
    .unwrap_err();
    assert!(matches!(err, TextEditorError::InvalidRange { .. }));
}

#[test]
fn unicode_insert_operation() {
    let mut doc = Document::from_str("hlo");

    let outcome = TextEditorOperation::Insert {
        position: Position::new(1),
        text: "é".to_string(),
    }
    .apply(&mut doc)
    .unwrap();

    match outcome {
        TextEditorOutcome::Inserted { range } => {
            assert_eq!(range, Range::new(Position::new(1), Position::new(2)));
        }
        other => panic!("expected Inserted, got {other:?}"),
    }
    assert_eq!(doc.text(), "hélo");
    assert_eq!(doc.len(), 4);
}

#[test]
fn unicode_delete_operation() {
    let mut doc = Document::from_str("héllo");

    let outcome = TextEditorOperation::Delete {
        range: Range::new(Position::new(1), Position::new(2)),
    }
    .apply(&mut doc)
    .unwrap();

    match outcome {
        TextEditorOutcome::Deleted { text, .. } => {
            assert_eq!(text, "é");
        }
        other => panic!("expected Deleted, got {other:?}"),
    }
    assert_eq!(doc.text(), "hllo");
}

#[test]
fn emoji_replace_operation() {
    let mut doc = Document::from_str("a😀b");

    let outcome = TextEditorOperation::Replace {
        range: Range::new(Position::new(1), Position::new(2)),
        text: "X".to_string(),
    }
    .apply(&mut doc)
    .unwrap();

    match outcome {
        TextEditorOutcome::Replaced { previous_text, .. } => {
            assert_eq!(previous_text, "😀");
        }
        other => panic!("expected Replaced, got {other:?}"),
    }
    assert_eq!(doc.text(), "aXb");
}

#[test]
fn sequence_of_operations_composes() {
    let mut doc = Document::new();

    // Insert "hello".
    TextEditorOperation::Insert {
        position: Position::new(0),
        text: "hello".to_string(),
    }
    .apply(&mut doc)
    .unwrap();
    assert_eq!(doc.text(), "hello");

    // Append " world".
    TextEditorOperation::Insert {
        position: Position::new(5),
        text: " world".to_string(),
    }
    .apply(&mut doc)
    .unwrap();
    assert_eq!(doc.text(), "hello world");

    // Replace "hello" with "goodbye".
    TextEditorOperation::Replace {
        range: Range::new(Position::new(0), Position::new(5)),
        text: "goodbye".to_string(),
    }
    .apply(&mut doc)
    .unwrap();
    assert_eq!(doc.text(), "goodbye world");

    // Delete " world".
    TextEditorOperation::Delete {
        range: Range::new(Position::new(7), Position::new(13)),
    }
    .apply(&mut doc)
    .unwrap();
    assert_eq!(doc.text(), "goodbye");

    // Clear.
    TextEditorOperation::Clear
        .apply(&mut doc)
        .unwrap();
    assert!(doc.is_empty());
}

#[test]
fn append_operation() {
    let mut doc = Document::from_str("hello");

    let outcome = TextEditorOperation::Append {
        text: " world".to_string(),
    }
    .apply(&mut doc)
    .unwrap();

    match outcome {
        TextEditorOutcome::Appended { range } => {
            assert_eq!(range, Range::new(Position::new(5), Position::new(11)));
        }
        other => panic!("expected Appended, got {other:?}"),
    }
    assert_eq!(doc.text(), "hello world");
}

#[test]
fn delete_line_operation() {
    let mut doc = Document::from_str("line1\nline2\nline3");

    // Delete the line containing position 6 (line2).
    let outcome = TextEditorOperation::DeleteLine {
        pos: Position::new(6),
    }
    .apply(&mut doc)
    .unwrap();

    match outcome {
        TextEditorOutcome::LineDeleted { text } => {
            assert_eq!(text, "line2\n");
        }
        other => panic!("expected LineDeleted, got {other:?}"),
    }
    assert_eq!(doc.text(), "line1\nline3");
}

#[test]
fn replace_line_operation() {
    let mut doc = Document::from_str("line1\nline2\nline3");

    // Replace the line containing position 6 (line2).
    let outcome = TextEditorOperation::ReplaceLine {
        pos: Position::new(6),
        new_text: "replaced".to_string(),
    }
    .apply(&mut doc)
    .unwrap();

    match outcome {
        TextEditorOutcome::LineReplaced { previous_text } => {
            assert_eq!(previous_text, "line2");
        }
        other => panic!("expected LineReplaced, got {other:?}"),
    }
    assert_eq!(doc.text(), "line1\nreplaced\nline3");
}