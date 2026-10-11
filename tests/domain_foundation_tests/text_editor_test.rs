//! Model and query tests for the TextEditor environment.

use app_shell::domain_foundation::environments::text_editor::{
    Document, Position, Range, TextEditorError,
};

#[test]
fn new_document_is_empty() {
    let doc = Document::new();
    assert!(doc.is_empty());
    assert_eq!(doc.len(), 0);
    assert_eq!(doc.text(), "");
}

#[test]
fn from_str_sets_text() {
    let doc = Document::from_str("hello");
    assert_eq!(doc.len(), 5);
    assert_eq!(doc.text(), "hello");
}

#[test]
fn char_at_returns_characters() {
    let doc = Document::from_str("hello");
    assert_eq!(doc.char_at(Position::new(0)), Some('h'));
    assert_eq!(doc.char_at(Position::new(4)), Some('o'));
    assert_eq!(doc.char_at(Position::new(5)), None); // end of document
}

#[test]
fn substring_returns_range() {
    let doc = Document::from_str("hello world");
    let range = Range::new(Position::new(0), Position::new(5));
    assert_eq!(doc.substring(range), Some("hello"));

    let range = Range::new(Position::new(6), Position::new(11));
    assert_eq!(doc.substring(range), Some("world"));
}

#[test]
fn insert_at_beginning() {
    let mut doc = Document::from_str("world");
    let range = doc.insert(Position::new(0), "hello ").unwrap();
    assert_eq!(range, Range::new(Position::new(0), Position::new(6)));
    assert_eq!(doc.text(), "hello world");
}

#[test]
fn insert_at_end() {
    let mut doc = Document::from_str("hello");
    doc.insert(Position::new(5), " world").unwrap();
    assert_eq!(doc.text(), "hello world");
}

#[test]
fn insert_in_middle() {
    let mut doc = Document::from_str("helo");
    doc.insert(Position::new(2), "l").unwrap();
    assert_eq!(doc.text(), "hello");
}

#[test]
fn insert_at_end_position_is_valid() {
    let mut doc = Document::from_str("abc");
    assert!(doc.insert(Position::new(3), "d").is_ok());
    assert_eq!(doc.text(), "abcd");
}

#[test]
fn insert_out_of_bounds_fails() {
    let mut doc = Document::from_str("abc");
    let err = doc.insert(Position::new(5), "x").unwrap_err();
    assert!(matches!(err, TextEditorError::PositionOutOfBounds { position: 5, length: 3 }));
}

#[test]
fn delete_range() {
    let mut doc = Document::from_str("hello world");
    let deleted = doc.delete(Range::new(Position::new(5), Position::new(11))).unwrap();
    assert_eq!(deleted, " world");
    assert_eq!(doc.text(), "hello");
}

#[test]
fn delete_empty_range_returns_empty_string() {
    let mut doc = Document::from_str("hello");
    let deleted = doc.delete(Range::at(Position::new(2))).unwrap();
    assert_eq!(deleted, "");
    assert_eq!(doc.text(), "hello");
}

#[test]
fn delete_invalid_range_fails() {
    let mut doc = Document::from_str("hello");
    let err = doc.delete(Range::new(Position::new(3), Position::new(1))).unwrap_err();
    assert!(matches!(err, TextEditorError::InvalidRange { .. }));
}

#[test]
fn delete_out_of_bounds_fails() {
    let mut doc = Document::from_str("abc");
    let err = doc.delete(Range::new(Position::new(0), Position::new(10))).unwrap_err();
    assert!(matches!(err, TextEditorError::PositionOutOfBounds { .. }));
}

#[test]
fn replace_range() {
    let mut doc = Document::from_str("hello world");
    let previous = doc.replace(
        Range::new(Position::new(0), Position::new(5)),
        "goodbye",
    ).unwrap();
    assert_eq!(previous, "hello");
    assert_eq!(doc.text(), "goodbye world");
}

#[test]
fn replace_with_empty_text_deletes() {
    let mut doc = Document::from_str("hello world");
    doc.replace(Range::new(Position::new(5), Position::new(11)), "").unwrap();
    assert_eq!(doc.text(), "hello");
}

#[test]
fn clear_empties_document() {
    let mut doc = Document::from_str("hello");
    let previous = doc.clear();
    assert_eq!(previous, "hello");
    assert!(doc.is_empty());
}

#[test]
fn range_len_and_is_empty() {
    let r = Range::new(Position::new(2), Position::new(5));
    assert_eq!(r.len(), 3);
    assert!(!r.is_empty());

    let r = Range::at(Position::new(3));
    assert_eq!(r.len(), 0);
    assert!(r.is_empty());
}

#[test]
fn range_contains() {
    let r = Range::new(Position::new(2), Position::new(5));
    assert!(r.contains(Position::new(2)));
    assert!(r.contains(Position::new(4)));
    assert!(!r.contains(Position::new(5))); // half-open: end not included
    assert!(!r.contains(Position::new(1)));
}

// --- UTF-8 / Unicode handling ---

#[test]
fn unicode_scalar_value_positions() {
    // "héllo" — 5 Unicode scalar values, but é is 2 bytes in UTF-8.
    let doc = Document::from_str("héllo");
    assert_eq!(doc.len(), 5);
    assert_eq!(doc.char_at(Position::new(1)), Some('é'));
}

#[test]
fn insert_with_multibyte() {
    let mut doc = Document::from_str("hlo");
    doc.insert(Position::new(1), "é").unwrap();
    assert_eq!(doc.text(), "hélo");
    assert_eq!(doc.len(), 4);
}

#[test]
fn delete_with_multibyte() {
    let mut doc = Document::from_str("héllo");
    let deleted = doc.delete(Range::new(Position::new(1), Position::new(2))).unwrap();
    assert_eq!(deleted, "é");
    assert_eq!(doc.text(), "hllo");
    assert_eq!(doc.len(), 4);
}

#[test]
fn emoji_handling() {
    // Each emoji is one Unicode scalar value.
    let doc = Document::from_str("a😀b");
    assert_eq!(doc.len(), 3);
    assert_eq!(doc.char_at(Position::new(1)), Some('😀'));

    let sub = doc.substring(Range::new(Position::new(0), Position::new(2)));
    assert_eq!(sub, Some("a😀"));
}

#[test]
fn substring_with_multibyte() {
    let doc = Document::from_str("héllo");
    assert_eq!(
        doc.substring(Range::new(Position::new(1), Position::new(3))),
        Some("él")
    );
}

#[test]
fn insert_at_each_position_in_unicode_text() {
    let mut doc = Document::from_str("é");
    // Insert before é
    doc.insert(Position::new(0), "X").unwrap();
    assert_eq!(doc.text(), "Xé");
    // Insert after é
    doc.insert(Position::new(2), "Y").unwrap();
    assert_eq!(doc.text(), "XéY");
    assert_eq!(doc.len(), 3);
}

#[test]
fn line_count_and_line_col() {
    let doc = Document::from_str("hello\nworld\nfoo");
    assert_eq!(doc.line_count(), 3);

    assert_eq!(doc.line_col_at(Position::new(0)), Some((0, 0)));
    assert_eq!(doc.line_col_at(Position::new(5)), Some((0, 5)));  // before \n
    assert_eq!(doc.line_col_at(Position::new(6)), Some((1, 0)));  // after \n
    assert_eq!(doc.line_col_at(Position::new(11)), Some((1, 5))); // before \n
    assert_eq!(doc.line_col_at(Position::new(12)), Some((2, 0))); // after \n
}

#[test]
fn find_returns_first_match() {
    let doc = Document::from_str("hello world hello");
    let range = doc.find("hello", Position::new(0)).unwrap();
    assert_eq!(range, Range::new(Position::new(0), Position::new(5)));
}

#[test]
fn find_from_position() {
    let doc = Document::from_str("hello world hello");
    let range = doc.find("hello", Position::new(6)).unwrap();
    assert_eq!(range, Range::new(Position::new(12), Position::new(17)));
}

#[test]
fn find_all_returns_all_matches() {
    let doc = Document::from_str("ababab");
    let matches = doc.find_all("ab");
    assert_eq!(matches.len(), 3);
    assert_eq!(matches[0], Range::new(Position::new(0), Position::new(2)));
    assert_eq!(matches[1], Range::new(Position::new(2), Position::new(4)));
    assert_eq!(matches[2], Range::new(Position::new(4), Position::new(6)));
}

#[test]
fn replace_all_replaces_and_counts() {
    let mut doc = Document::from_str("foo bar foo baz foo");
    let count = doc.replace_all("foo", "XXX");
    assert_eq!(count, 3);
    assert_eq!(doc.text(), "XXX bar XXX baz XXX");
}

#[test]
fn find_with_unicode() {
    let doc = Document::from_str("héllo world héllo");
    let matches = doc.find_all("héllo");
    assert_eq!(matches.len(), 2);
}

#[test]
fn document_lines() {
    let doc = Document::from_str("hello\nworld\nfoo");
    let lines: Vec<&str> = doc.lines().collect();
    assert_eq!(lines, vec!["hello", "world", "foo"]);
}

#[test]
fn document_byte_count() {
    let doc = Document::from_str("héllo"); // é is 2 bytes
    assert_eq!(doc.len(), 5);   // 5 chars
    assert_eq!(doc.byte_count(), 6); // h(1) + é(2) + l(1) + l(1) + o(1) = 6
}

#[test]
fn document_append() {
    let mut doc = Document::from_str("hello");
    let range = doc.append(" world");
    assert_eq!(range, Range::new(Position::new(5), Position::new(11)));
    assert_eq!(doc.text(), "hello world");
    assert_eq!(doc.len(), 11);
}

#[test]
fn range_overlaps() {
    let a = Range::new(Position::new(0), Position::new(5));
    let b = Range::new(Position::new(3), Position::new(8));
    let c = Range::new(Position::new(5), Position::new(10));

    assert!(a.overlaps(b));      // [0,5) and [3,8) overlap
    assert!(!a.overlaps(c));     // [0,5) and [5,10) don't (half-open)
    assert!(b.overlaps(c));      // [3,8) and [5,10) overlap
}

#[test]
fn range_intersect() {
    let a = Range::new(Position::new(0), Position::new(5));
    let b = Range::new(Position::new(3), Position::new(8));

    let inter = a.intersect(b).unwrap();
    assert_eq!(inter, Range::new(Position::new(3), Position::new(5)));

    let c = Range::new(Position::new(5), Position::new(10));
    assert!(a.intersect(c).is_none()); // no overlap (half-open)
}

#[test]
fn range_shift() {
    let r = Range::new(Position::new(2), Position::new(5));
    let shifted = r.shift(3);
    assert_eq!(shifted, Range::new(Position::new(5), Position::new(8)));
}

#[test]
fn range_ord() {
    let a = Range::new(Position::new(0), Position::new(5));
    let b = Range::new(Position::new(1), Position::new(3));
    let c = Range::new(Position::new(0), Position::new(5));

    assert!(a < b);  // start 0 < start 1
    assert!(a <= c); // equal
    assert!(a == c);
}

#[test]
fn range_from_tuple() {
    let r: Range = (Position::new(2), Position::new(5)).into();
    assert_eq!(r, Range::new(Position::new(2), Position::new(5)));
}

#[test]
fn position_add_sub() {
    let p = Position::new(5);
    assert_eq!(p + 3, Position::new(8));
    assert_eq!(p - 2, Position::new(3));
}

#[test]
fn document_into_iterator() {
    let doc = Document::from_str("héllo");
    let chars: Vec<char> = (&doc).into_iter().collect();
    assert_eq!(chars, vec!['h', 'é', 'l', 'l', 'o']);
}