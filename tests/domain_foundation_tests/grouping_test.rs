//! Tests for the grouping utility.

use app_shell::domain_foundation::utilities::grouping::*;
use std::collections::HashMap;

#[test]
fn group_by_closure() {
    let items = vec![1, 2, 3, 4, 5, 6];
    let groups: HashMap<bool, Vec<&i32>> = group_by(&items, |x| *x % 2 == 0);

    let evens = groups.get(&true).unwrap();
    let odds = groups.get(&false).unwrap();
    assert_eq!(evens.len(), 3);
    assert_eq!(odds.len(), 3);
}

#[test]
fn group_by_string_key() {
    #[derive(Debug)]
    struct Item { category: String, value: i32 }

    let items = vec![
        Item { category: "A".to_string(), value: 1 },
        Item { category: "B".to_string(), value: 2 },
        Item { category: "A".to_string(), value: 3 },
    ];

    let groups = group_by(&items, |item: &Item| item.category.clone());
    assert_eq!(groups.get("A").unwrap().len(), 2);
    assert_eq!(groups.get("B").unwrap().len(), 1);
}

#[test]
fn group_by_trait() {
    struct ByParity;

    impl GroupCriteria<i32> for ByParity {
        type Key = bool;
        fn group_key(&self, item: &i32) -> bool {
            *item % 2 == 0
        }
    }

    let items = vec![1, 2, 3, 4, 5];
    let groups = group(&items, &ByParity);
    assert_eq!(groups.get(&true).unwrap().len(), 2);  // 2, 4
    assert_eq!(groups.get(&false).unwrap().len(), 3); // 1, 3, 5
}