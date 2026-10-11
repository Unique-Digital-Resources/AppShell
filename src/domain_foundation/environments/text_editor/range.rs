//! Range: a half-open `[start, end)` span of text.

use super::position::Position;

/// A half-open range of character positions `[start, end)`.
///
/// `start` must be `<= end`. An empty range (`start == end`) represents
/// a cursor position with no selected text.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub const fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// A zero-length range at a single position.
    pub const fn at(pos: Position) -> Self {
        Self { start: pos, end: pos }
    }

    /// Number of characters in the range.
    pub fn len(&self) -> u32 {
        self.end.as_u32() - self.start.as_u32()
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Whether `pos` falls within `[start, end)`.
    pub fn contains(&self, pos: Position) -> bool {
        pos >= self.start && pos < self.end
    }

    /// Whether this range overlaps with `other`.
    /// Two ranges overlap if they share at least one position.
    pub fn overlaps(&self, other: Range) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// The intersection of this range and `other`, or `None` if they
    /// don't overlap.
    pub fn intersect(&self, other: Range) -> Option<Range> {
        if !self.overlaps(other) {
            return None;
        }
        let start = std::cmp::max(self.start, other.start);
        let end = std::cmp::min(self.end, other.end);
        if start >= end {
            return None;
        }
        Some(Range::new(start, end))
    }

    /// Shift the range by `n` positions. Both start and end move forward.
    pub fn shift(self, n: u32) -> Range {
        Range::new(
            Position::new(self.start.as_u32() + n),
            Position::new(self.end.as_u32() + n),
        )
    }
}

impl From<(Position, Position)> for Range {
    fn from((start, end): (Position, Position)) -> Self {
        Range::new(start, end)
    }
}