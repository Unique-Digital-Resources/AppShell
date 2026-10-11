//! Tests for the querying utility.

use app_shell::domain_foundation::utilities::filtering::{criteria, FilterCriteria};
use app_shell::domain_foundation::utilities::querying::*;

#[test]
fn query_evaluates_matching_items() {
    let items = vec![1, 2, 3, 4, 5, 6];
    let query = Query::new(
        "evens".to_string(),
        criteria(|x: &i32| *x % 2 == 0),
    );

    assert_eq!(query.name(), "evens");

    let result = evaluate(&query, &items);
    assert_eq!(result, vec![&2, &4, &6]);
}

#[test]
fn query_with_composed_criteria() {
    let items = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let query = Query::new(
        "range".to_string(),
        criteria(|x: &i32| *x > 2).and(criteria(|x| *x < 7)),
    );

    let result = evaluate(&query, &items);
    assert_eq!(result, vec![&3, &4, &5, &6]);
}

#[test]
fn query_matches_individual_items() {
    let query = Query::new(
        "positive".to_string(),
        criteria(|x: &i32| *x > 0),
    );

    assert!(query.matches(&5));
    assert!(!query.matches(&-1));
    assert!(!query.matches(&0));
}