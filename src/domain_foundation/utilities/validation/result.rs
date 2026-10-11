//! Validation results.

/// The outcome of validating an item.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationResult {
    /// The item is valid.
    Valid,
    /// The item is invalid, with a human-readable reason.
    Invalid { reason: String },
}

impl ValidationResult {
    pub fn valid() -> Self {
        ValidationResult::Valid
    }

    pub fn invalid(reason: &str) -> Self {
        ValidationResult::Invalid { reason: reason.to_string() }
    }

    pub fn is_valid(&self) -> bool {
        matches!(self, ValidationResult::Valid)
    }

    pub fn is_invalid(&self) -> bool {
        !self.is_valid()
    }
}