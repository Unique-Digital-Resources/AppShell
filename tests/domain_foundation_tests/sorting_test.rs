//! Tests for the sorting utility.

use app_shell::domain_foundation::utilities::sorting::*;

#[test]
fn sort_ascending() {
    let mut items = vec![3, 1, 4, 1, 5, 9, 2, 6];
    sort(&mut items, &by(|a: &i32, b: &i32| a.cmp(b)));
    assert_eq!(items, vec![1, 1, 2, 3, 4, 5, 6, 9]);
}

#[test]
fn sort_descending() {
    let mut items = vec![3, 1, 4, 1, 5, 9, 2, 6];
    sort(&mut items, &by(|a: &i32, b: &i32| b.cmp(a)));
    assert_eq!(items, vec![9, 6, 5, 4, 3, 2, 1, 1]);
}

#[test]
fn sorted_returns_references() {
    let items = vec![3, 1, 4, 1, 5];
    let result = sorted(&items, &by(|a: &i32, b: &i32| a.cmp(b)));
    assert_eq!(result, vec![&1, &1, &3, &4, &5]);
    // Original unchanged.
    assert_eq!(items, vec![3, 1, 4, 1, 5]);
}

#[test]
fn sort_struct_by_field() {
    #[derive(Debug, PartialEq)]
    struct Item { name: String, priority: i32 }

    let mut items = vec![
        Item { name: "C".to_string(), priority: 3 },
        Item { name: "A".to_string(), priority: 1 },
        Item { name: "B".to_string(), priority: 2 },
    ];

    sort(&mut items, &by(|a: &Item, b: &Item| a.priority.cmp(&b.priority)));
    assert_eq!(items[0].name, "A");
    assert_eq!(items[1].name, "B");
    assert_eq!(items[2].name, "C");
}