//! Tests for the batch utility.

use app_shell::domain_foundation::utilities::batch::*;

#[test]
fn apply_batch_increments_all() {
    let mut items = vec![1, 2, 3, 4, 5];
    let op = operation(|items: &mut [i32]| {
        for item in items.iter_mut() {
            *item += 10;
        }
    });

    apply_batch(&op, &mut items);
    assert_eq!(items, vec![11, 12, 13, 14, 15]);
}

#[test]
fn apply_batch_clears_all() {
    let mut items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let op = operation(|items: &mut [String]| {
        for item in items.iter_mut() {
            item.clear();
        }
    });

    apply_batch(&op, &mut items);
    assert!(items.iter().all(|s| s.is_empty()));
}

#[test]
fn batch_with_struct() {
    #[derive(Debug, PartialEq)]
    struct Item { value: i32 }

    let mut items = vec![
        Item { value: 1 },
        Item { value: 2 },
        Item { value: 3 },
    ];

    let op = operation(|items: &mut [Item]| {
        for item in items.iter_mut() {
            item.value *= 2;
        }
    });

    apply_batch(&op, &mut items);
    assert_eq!(items[0].value, 2);
    assert_eq!(items[1].value, 4);
    assert_eq!(items[2].value, 6);
}

#[test]
fn trait_based_batch() {
    struct Double;
    impl BatchOperation<i32> for Double {
        fn apply(&self, items: &mut [i32]) {
            for item in items.iter_mut() {
                *item *= 2;
            }
        }
    }

    let mut items = vec![1, 2, 3];
    apply_batch(&Double, &mut items);
    assert_eq!(items, vec![2, 4, 6]);
}