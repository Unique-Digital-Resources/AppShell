//! Tests for the filtering utility.

use app_shell::domain_foundation::utilities::filtering::*;

#[test]
fn filter_returns_matching_items() {
    let items = vec![1, 2, 3, 4, 5];
    let result = filter(&items, &criteria(|x: &i32| *x > 3));
    assert_eq!(result, vec![&4, &5]);
}

#[test]
fn count_matching() {
    let items = vec![1, 2, 3, 4, 5];
    assert_eq!(count(&items, &criteria(|x: &i32| *x > 3)), 2);
}

#[test]
fn any_and_all() {
    let items = vec![1, 2, 3];
    assert!(any(&items, &criteria(|x: &i32| *x == 2)));
    assert!(!all(&items, &criteria(|x: &i32| *x == 2)));
    assert!(all(&items, &criteria(|x: &i32| *x > 0)));
}

#[test]
fn and_composition() {
    let items = vec![1, 3, 5, 7, 9];
    let c = criteria(|x: &i32| *x > 2).and(criteria(|x| *x < 8));
    let result = filter(&items, &c);
    assert_eq!(result, vec![&3, &5, &7]);
}

#[test]
fn or_composition() {
    let items = vec![1, 2, 3, 4, 5];
    let c = criteria(|x: &i32| *x == 1).or(criteria(|x| *x == 5));
    let result = filter(&items, &c);
    assert_eq!(result, vec![&1, &5]);
}

#[test]
fn not_composition() {
    let items = vec![1, 2, 3, 4, 5];
    let c = criteria(|x: &i32| *x > 3).not();
    let result = filter(&items, &c);
    assert_eq!(result, vec![&1, &2, &3]);
}

#[test]
fn filter_with_struct_type() {
    #[derive(Debug)]
    struct Item { category: String, value: i32 }

    let items = vec![
        Item { category: "A".to_string(), value: 10 },
        Item { category: "B".to_string(), value: 20 },
        Item { category: "A".to_string(), value: 30 },
    ];

    let c = criteria(|item: &Item| item.category == "A");
    let result = filter(&items, &c);
    assert_eq!(result.len(), 2);
}