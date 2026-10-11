//! TextEditor-specific errors.

use std::fmt;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextEditorError {
    /// A position was outside the valid range `0..=len()`.
    PositionOutOfBounds { position: u32, length: u32 },
    /// A range had `start > end`.
    InvalidRange { start: u32, end: u32 },
}

impl fmt::Display for TextEditorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextEditorError::PositionOutOfBounds { position, length } => {
                write!(f, "position {position} out of bounds (document length: {length})")
            }
            TextEditorError::InvalidRange { start, end } => {
                write!(f, "invalid range: start {start} > end {end}")
            }
        }
    }
}

impl std::error::Error for TextEditorError {}