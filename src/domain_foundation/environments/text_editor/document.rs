//! Document model: owns text, provides position-based queries and edits.
//!
//! `char_count` is cached to make `len()` O(1). All mutating methods
//! keep the cache in sync.

use super::error::TextEditorError;
use super::position::Position;
use super::range::Range;

/// Headless text document.
///
/// Stores text as a Rust `String` (UTF-8). All positions are Unicode
/// scalar value offsets (character indices), not byte offsets.
/// `char_count` is cached for O(1) `len()`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Document {
    text: String,
    char_count: u32,
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_str(s: &str) -> Self {
        Self {
            char_count: s.chars().count() as u32,
            text: s.to_string(),
        }
    }

    /// The full text content.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Number of Unicode scalar values (characters) in the document.
    /// O(1) — uses cached count.
    pub fn len(&self) -> u32 {
        self.char_count
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Number of bytes in the UTF-8 representation.
    pub fn byte_count(&self) -> usize {
        self.text.len()
    }

    /// Get the character at `pos`, or `None` if out of bounds.
    pub fn char_at(&self, pos: Position) -> Option<char> {
        self.text.chars().nth(pos.as_u32() as usize)
    }

    /// Get a substring for `range`, or `None` if the range is invalid.
    pub fn substring(&self, range: Range) -> Option<&str> {
        if range.start > range.end {
            return None;
        }
        let start = char_to_byte(&self.text, range.start.as_u32() as usize)?;
        let end = char_to_byte(&self.text, range.end.as_u32() as usize)?;
        Some(&self.text[start..end])
    }

    /// Iterate over lines (split by `\n`). Lines do not include the
    /// trailing newline character.
    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.text.lines()
    }

    /// Number of lines in the document.
    pub fn line_count(&self) -> u32 {
        self.text.lines().count() as u32
    }

    /// Convert a character position to (line, column), both 0-indexed.
    pub fn line_col_at(&self, pos: Position) -> Option<(u32, u32)> {
        let pos_usize = pos.as_u32() as usize;
        if pos_usize > self.char_count as usize {
            return None;
        }

        let mut line = 0u32;
        let mut col = 0u32;
        for (i, ch) in self.text.chars().enumerate() {
            if i == pos_usize {
                return Some((line, col));
            }
            if ch == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }
        Some((line, col))
    }

    /// Insert `text` at `pos`. Returns the range of the inserted text.
    pub fn insert(&mut self, pos: Position, text: &str) -> Result<Range, TextEditorError> {
        let pos_u32 = pos.as_u32();
        if pos_u32 > self.char_count {
            return Err(TextEditorError::PositionOutOfBounds {
                position: pos_u32,
                length: self.char_count,
            });
        }

        let byte_pos = char_to_byte(&self.text, pos_u32 as usize)
            .expect("position validated above");

        let inserted_len = text.chars().count() as u32;
        self.text.insert_str(byte_pos, text);
        self.char_count += inserted_len;

        Ok(Range::new(pos, Position::new(pos_u32 + inserted_len)))
    }

    /// Convenience: insert text at the end of the document.
    pub fn append(&mut self, text: &str) -> Range {
        let pos = Position::new(self.char_count);
        self.insert(pos, text).expect("append at end is always valid")
    }

    /// Delete the text in `range`. Returns the deleted text.
    pub fn delete(&mut self, range: Range) -> Result<String, TextEditorError> {
        if range.start > range.end {
            return Err(TextEditorError::InvalidRange {
                start: range.start.as_u32(),
                end: range.end.as_u32(),
            });
        }

        if range.end.as_u32() > self.char_count {
            return Err(TextEditorError::PositionOutOfBounds {
                position: range.end.as_u32(),
                length: self.char_count,
            });
        }

        let start_byte = char_to_byte(&self.text, range.start.as_u32() as usize)
            .expect("position validated above");
        let end_byte = char_to_byte(&self.text, range.end.as_u32() as usize)
            .expect("position validated above");

        let deleted = self.text[start_byte..end_byte].to_string();
        self.text.drain(start_byte..end_byte);
        self.char_count -= range.len();

        Ok(deleted)
    }

    /// Replace the text in `range` with `text`. Returns the previous text.
    pub fn replace(&mut self, range: Range, text: &str) -> Result<String, TextEditorError> {
        let previous = self.delete(range)?;
        self.insert(range.start, text)?;
        Ok(previous)
    }

    /// Clear all text. Returns the previous content.
    pub fn clear(&mut self) -> String {
        self.char_count = 0;
        std::mem::take(&mut self.text)
    }

    // --- Search / replace ---

    /// Find the first occurrence of `needle` starting from `from`.
    pub fn find(&self, needle: &str, from: Position) -> Option<Range> {
        let from_usize = from.as_u32() as usize;
        let byte_start = char_to_byte(&self.text, from_usize)?;

        if needle.is_empty() {
            return Some(Range::at(from));
        }

        let found = self.text[byte_start..].find(needle)?;
        let match_byte_start = byte_start + found;
        let match_byte_end = match_byte_start + needle.len();

        let char_start = self.text[..match_byte_start].chars().count() as u32;
        let char_end = self.text[..match_byte_end].chars().count() as u32;

        Some(Range::new(
            Position::new(char_start),
            Position::new(char_end),
        ))
    }

    /// Find all occurrences of `needle`.
    pub fn find_all(&self, needle: &str) -> Vec<Range> {
        if needle.is_empty() {
            return Vec::new();
        }
        let mut results = Vec::new();
        let mut from = Position::new(0);
        while let Some(range) = self.find(needle, from) {
            results.push(range);
            from = range.end;
        }
        results
    }

    /// Replace all occurrences of `needle` with `replacement`.
    /// Returns the number of replacements made.
    pub fn replace_all(&mut self, needle: &str, replacement: &str) -> usize {
        if needle.is_empty() {
            return 0;
        }
        let count = self.find_all(needle).len();
        self.text = self.text.replace(needle, replacement);
        self.char_count = self.text.chars().count() as u32;
        count
    }

    /// Delete the line containing position `pos`. Returns the deleted text
    /// including the trailing newline (if any).
    pub fn delete_line(&mut self, pos: Position) -> Result<String, TextEditorError> {
        self.line_col_at(pos)
            .ok_or(TextEditorError::PositionOutOfBounds {
                position: pos.as_u32(),
                length: self.char_count,
            })?;

        // Find the start of the line (after previous \n or 0).
        let mut line_start = 0u32;
        let mut char_idx = 0u32;
        for ch in self.text.chars() {
            if char_idx == pos.as_u32() {
                break;
            }
            if ch == '\n' {
                line_start = char_idx + 1;
            }
            char_idx += 1;
        }

        // Find the end of the line (next \n or end of text).
        let mut line_end = self.char_count;
        let mut idx = 0u32;
        for ch in self.text.chars() {
            if idx >= line_start && ch == '\n' {
                line_end = idx + 1; // include the newline
                break;
            }
            idx += 1;
        }
        // If no newline found after line_start, line_end = char_count (last line, no trailing \n).

        // If line_start > line_end (shouldn't happen), clamp.
        if line_start > line_end {
            line_start = line_end;
        }

        let range = Range::new(
            Position::new(line_start),
            Position::new(line_end),
        );
        self.delete(range)
    }

    /// Replace the line containing position `pos` with `new_text`.
    /// Returns the previous line content (including trailing newline if any).
    pub fn replace_line(&mut self, pos: Position, new_text: &str) -> Result<String, TextEditorError> {
        self.line_col_at(pos)
            .ok_or(TextEditorError::PositionOutOfBounds {
                position: pos.as_u32(),
                length: self.char_count,
            })?;

        // Find the start of the line.
        let mut line_start = 0u32;
        let mut char_idx = 0u32;
        for ch in self.text.chars() {
            if char_idx == pos.as_u32() {
                break;
            }
            if ch == '\n' {
                line_start = char_idx + 1;
            }
            char_idx += 1;
        }

        // Find the end of the line (next \n or end).
        let mut line_end = self.char_count;
        let mut idx = 0u32;
        for ch in self.text.chars() {
            if idx >= line_start && ch == '\n' {
                line_end = idx; // exclude the newline
                break;
            }
            idx += 1;
        }

        if line_start > line_end {
            line_start = line_end;
        }

        let range = Range::new(
            Position::new(line_start),
            Position::new(line_end),
        );
        self.replace(range, new_text)
    }

}

/// Convert a character position to a UTF-8 byte offset.
fn char_to_byte(text: &str, char_pos: usize) -> Option<usize> {
    let char_count = text.chars().count();
    if char_pos > char_count {
        return None;
    }
    text.char_indices()
        .nth(char_pos)
        .map(|(i, _)| i)
        .or_else(|| Some(text.len()))
}

impl<'a> IntoIterator for &'a Document {
    type Item = char;
    type IntoIter = std::str::Chars<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.text.chars()
    }
}