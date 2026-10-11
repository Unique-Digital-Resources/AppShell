//! Tests for the validation utility.

use app_shell::domain_foundation::utilities::validation::*;

#[test]
fn valid_result() {
    let r = ValidationResult::valid();
    assert!(r.is_valid());
    assert!(!r.is_invalid());
}

#[test]
fn invalid_result() {
    let r = ValidationResult::invalid("too small");
    assert!(r.is_invalid());
    assert!(!r.is_valid());
    match &r {
        ValidationResult::Invalid { reason } => assert_eq!(reason, "too small"),
        _ => unreachable!(),
    }
}

#[test]
fn rule_from_closure() {
    let r = rule(|x: &i32| {
        if *x >= 0 { ValidationResult::valid() }
        else { ValidationResult::invalid("negative") }
    });

    assert!(r.validate(&5).is_valid());
    assert!(r.validate(&-1).is_invalid());
}

#[test]
fn validate_all_returns_only_invalid() {
    let items = vec![5, -3, 0, -1, 10];
    let r = rule(|x: &i32| {
        if *x >= 0 { ValidationResult::valid() }
        else { ValidationResult::invalid("negative") }
    });

    let invalids = validate_all(&items, &r);
    assert_eq!(invalids.len(), 2);
    assert_eq!(*invalids[0].0, -3);
    assert_eq!(*invalids[1].0, -1);
}

#[test]
fn validate_all_empty_when_all_valid() {
    let items = vec![1, 2, 3];
    let r = rule(|_: &i32| ValidationResult::valid());
    let invalids = validate_all(&items, &r);
    assert!(invalids.is_empty());
}