//! Column identity and model.

use std::fmt;

/// Stable, opaque identifier for a column within a [`Table`](super::table::Table).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ColumnId(u64);

impl ColumnId {
    pub fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    pub fn as_raw(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ColumnId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "col{}", self.0)
    }
}

/// A column in a table, with an optional name.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    pub id: ColumnId,
    pub name: Option<String>,
}

impl Column {
    pub fn new(id: ColumnId, name: Option<String>) -> Self {
        Self { id, name }
    }
}