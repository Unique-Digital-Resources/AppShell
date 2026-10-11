//! # Validation
//!
//! Optional reusable validation rules and results.

pub mod result;
pub mod rule;

pub use result::ValidationResult;
pub use rule::{rule, validate_all, validate_with_rules, FnRule, ValidationRule};