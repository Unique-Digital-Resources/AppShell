//! TextEditor operations: value-type intents for text changes.

use super::document::Document;
use super::error::TextEditorError;
use super::position::Position;
use super::range::Range;

/// A text editor operation.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum TextEditorOperation {
    /// Insert text at a position.
    Insert { position: Position, text: String },
    /// Delete text in a range.
    Delete { range: Range },
    /// Replace text in a range with new text.
    Replace { range: Range, text: String },
    /// Clear the entire document.
    Clear,

    /// Append text at the end of the document.
    Append { text: String },
    /// Delete the line containing position `pos`.
    DeleteLine { pos: Position },
    /// Replace the line containing position `pos` with `new_text`.
    ReplaceLine { pos: Position, new_text: String },
}

/// Outcome of a successful [`TextEditorOperation`].
///
/// Each variant carries enough state to construct the inverse for undo.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextEditorOutcome {
    /// The range of the inserted text.
    Inserted { range: Range },
    /// The deleted range and its content.
    Deleted { range: Range, text: String },
    /// The replaced range and the previous content.
    Replaced { range: Range, previous_text: String },
    /// The previous content of the entire document.
    Cleared { previous_text: String },

    /// Text appended. `range` is the inserted range.
    Appended { range: Range },
    /// Line deleted. `text` is the deleted content.
    LineDeleted { text: String },
    /// Line replaced. `previous_text` is the old line content.
    LineReplaced { previous_text: String },
}

impl TextEditorOperation {
    pub fn apply(self, doc: &mut Document) -> Result<TextEditorOutcome, TextEditorError> {
        match self {
            TextEditorOperation::Insert { position, text } => {
                let range = doc.insert(position, &text)?;
                Ok(TextEditorOutcome::Inserted { range })
            }
            TextEditorOperation::Delete { range } => {
                let text = doc.delete(range)?;
                Ok(TextEditorOutcome::Deleted { range, text })
            }
            TextEditorOperation::Replace { range, text } => {
                let previous_text = doc.replace(range, &text)?;
                Ok(TextEditorOutcome::Replaced { range, previous_text })
            }
            TextEditorOperation::Clear => {
                let previous_text = doc.clear();
                Ok(TextEditorOutcome::Cleared { previous_text })
            }

						TextEditorOperation::Append { text } => {
								let range = doc.append(&text);
								Ok(TextEditorOutcome::Appended { range })
						}
						TextEditorOperation::DeleteLine { pos } => {
								let text = doc.delete_line(pos)?;
								Ok(TextEditorOutcome::LineDeleted { text })
						}
						TextEditorOperation::ReplaceLine { pos, new_text } => {
								let previous_text = doc.replace_line(pos, &new_text)?;
								Ok(TextEditorOutcome::LineReplaced { previous_text })
						}
        }
    }
}