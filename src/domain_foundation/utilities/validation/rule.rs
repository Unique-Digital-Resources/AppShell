//! Validation rules.

use super::result::ValidationResult;

/// A rule that validates an item.
pub trait ValidationRule<T> {
    fn validate(&self, item: &T) -> ValidationResult;
}

/// Closure-based validation rule.
pub struct FnRule<F> {
    f: F,
}

impl<T, F: Fn(&T) -> ValidationResult> ValidationRule<T> for FnRule<F> {
    fn validate(&self, item: &T) -> ValidationResult {
        (self.f)(item)
    }
}

/// Create a validation rule from a closure.
pub fn rule<T, F: Fn(&T) -> ValidationResult>(f: F) -> FnRule<F> {
    FnRule { f }
}

/// Validate all items and return the invalid ones with their results.
pub fn validate_all<'a, T, R: ValidationRule<T>>(
    items: &'a [T],
    rule: &R,
) -> Vec<(&'a T, ValidationResult)> {
    items
        .iter()
        .filter_map(|item| {
            let result = rule.validate(item);
            if result.is_invalid() {
                Some((item, result))
            } else {
                None
            }
        })
        .collect()
}

/// Validate an item against multiple rules. Returns all failures.
pub fn validate_with_rules<'a, T, R, I>(
    item: &T,
    rules: I,
) -> Vec<(&'a str, ValidationResult)>
where
    R: ValidationRule<T> + 'a,
    I: IntoIterator<Item = (&'a str, &'a R)>,
{
    rules
        .into_iter()
        .filter_map(|(name, rule)| {
            let result = rule.validate(item);
            if result.is_invalid() {
                Some((name, result))
            } else {
                None
            }
        })
        .collect()
}