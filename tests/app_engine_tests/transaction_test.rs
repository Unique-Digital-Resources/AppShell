use app_shell::app_engine::{TransactionId, TransactionManager};

#[test]
fn new_manager_has_no_active_transaction() {
    let mgr = TransactionManager::new();
    assert!(!mgr.is_active());
    assert!(mgr.current().is_none());
}

#[test]
fn begin_creates_transaction() {
    let mut mgr = TransactionManager::new();
    let tx = mgr.begin();
    assert!(mgr.is_active());
    assert_eq!(mgr.current(), Some(tx));
}

#[test]
fn commit_clears_active_transaction() {
    let mut mgr = TransactionManager::new();
    let tx = mgr.begin();
    let committed = mgr.commit();
    assert_eq!(committed, Some(tx));
    assert!(!mgr.is_active());
}

#[test]
fn rollback_clears_active_transaction() {
    let mut mgr = TransactionManager::new();
    let tx = mgr.begin();
    let rolled = mgr.rollback();
    assert_eq!(rolled, Some(tx));
    assert!(!mgr.is_active());
}

#[test]
fn transaction_ids_are_unique() {
    let a = TransactionId::new();
    let b = TransactionId::new();
    assert_ne!(a, b);
}

#[test]
fn commit_when_inactive_returns_none() {
    let mut mgr = TransactionManager::new();
    assert!(mgr.commit().is_none());
}

#[test]
fn begin_after_commit_starts_new_transaction() {
    let mut mgr = TransactionManager::new();
    let tx1 = mgr.begin();
    mgr.commit();
    let tx2 = mgr.begin();
    assert_ne!(tx1, tx2);
}