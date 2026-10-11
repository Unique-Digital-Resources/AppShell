//! Position: a character offset into a [`Document`](super::document::Document).

use std::fmt;
use std::ops::{Add, Sub};

/// A character position in a text document.
///
/// `Position(0)` is before the first character. `Position(n)` is after
/// the `n`-th Unicode scalar value. Valid positions are `0..=document.len()`.
///
/// Positions are **not** byte offsets. A multibyte character (é, emoji)
/// occupies exactly one position.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Position(u32);

impl Position {
    pub const fn new(pos: u32) -> Self {
        Self(pos)
    }

    pub const fn zero() -> Self {
        Self(0)
    }

    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pos{}", self.0)
    }
}

impl Add<u32> for Position {
    type Output = Position;
    fn add(self, rhs: u32) -> Self {
        Position(self.0 + rhs)
    }
}

impl Sub<u32> for Position {
    type Output = Position;
    fn sub(self, rhs: u32) -> Self {
        Position(self.0 - rhs)
    }
}