//! Tests for `SessionContext`.

use app_shell::app_engine::{ApplicationId, SessionContext};

#[test]
fn session_context_has_unique_id() {
    let app_id = ApplicationId::new();
    let s1 = SessionContext::new(app_id);
    let s2 = SessionContext::new(app_id);
    assert_ne!(s1.id(), s2.id());
}

#[test]
fn session_context_stores_parent_application() {
    let app_id = ApplicationId::new();
    let session = SessionContext::new(app_id);
    assert_eq!(session.application_id(), app_id);
}

#[test]
fn session_context_id_is_copyable() {
    let app_id = ApplicationId::new();
    let session = SessionContext::new(app_id);
    let id = session.id();
    let copied = id;
    assert_eq!(id, copied);
    assert!(id.value() > 0);
    assert_eq!(copied.value(), id.value());
}

#[test]
fn session_context_equality_is_value_based() {
    let app_id = ApplicationId::new();
    let s = SessionContext::new(app_id);
    let cloned = s.clone();
    assert_eq!(s, cloned);
}

#[test]
fn different_sessions_under_same_app_are_distinct() {
    let app_id = ApplicationId::new();
    let s1 = SessionContext::new(app_id);
    let s2 = SessionContext::new(app_id);
    assert_ne!(s1, s2);
}