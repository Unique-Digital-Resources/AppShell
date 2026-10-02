//! Tests for `ContextManager`.

use app_shell::app_engine::{ContextError, ContextManager, SessionId};

#[test]
fn new_manager_has_no_application_context() {
    let mgr = ContextManager::new();
    assert!(mgr.application_context().is_none());
    assert_eq!(mgr.session_count(), 0);
    assert_eq!(mgr.execution_count(), 0);
}

#[test]
fn create_application_context_succeeds() {
    let mut mgr = ContextManager::new();
    let id = mgr.create_application_context("app");
    let app = mgr.application_context().expect("application must exist");
    assert_eq!(app.application_id(), "app");
    assert_eq!(app.id(), id);
}

#[test]
fn create_session_requires_application_context() {
    let mut mgr = ContextManager::new();
    let result = mgr.create_session_context();
    assert!(matches!(result, Err(ContextError::NoApplicationContext)));
}

#[test]
fn create_session_under_application_succeeds() {
    let mut mgr = ContextManager::new();
    let app_id = mgr.create_application_context("app");
    let session_id = mgr.create_session_context().expect("session should be created");
    let session = mgr.session_context(session_id).expect("session must exist");
    assert_eq!(session.id(), session_id);
    assert_eq!(session.application_id(), app_id);
    assert_eq!(mgr.session_count(), 1);
}

#[test]
fn create_execution_requires_existing_session() {
    let mut mgr = ContextManager::new();
    mgr.create_application_context("app");
    let bogus_session_id = SessionId::new();
    let result = mgr.create_execution_context(bogus_session_id);
    assert!(matches!(result, Err(ContextError::SessionNotFound)));
}

#[test]
fn create_execution_under_session_succeeds() {
    let mut mgr = ContextManager::new();
    mgr.create_application_context("app");
    let session_id = mgr.create_session_context().unwrap();
    let exec_id = mgr
        .create_execution_context(session_id)
        .expect("execution should be created");
    let exec = mgr.execution_context(exec_id).expect("execution must exist");
    assert_eq!(exec.id(), exec_id);
    assert_eq!(exec.session_id(), session_id);
    assert_eq!(mgr.execution_count(), 1);
}

#[test]
fn hierarchy_application_session_execution_is_preserved() {
    let mut mgr = ContextManager::new();
    let app_id = mgr.create_application_context("app");
    let session_id = mgr.create_session_context().unwrap();
    let exec_id = mgr.create_execution_context(session_id).unwrap();

    let session = mgr.session_context(session_id).unwrap();
    let exec = mgr.execution_context(exec_id).unwrap();

    assert_eq!(session.application_id(), app_id);
    assert_eq!(exec.session_id(), session_id);
}

#[test]
fn multiple_sessions_under_same_application() {
    let mut mgr = ContextManager::new();
    mgr.create_application_context("app");
    let s1 = mgr.create_session_context().unwrap();
    let s2 = mgr.create_session_context().unwrap();
    let s3 = mgr.create_session_context().unwrap();
    assert_ne!(s1, s2);
    assert_ne!(s2, s3);
    assert_ne!(s1, s3);
    assert_eq!(mgr.session_count(), 3);
}

#[test]
fn multiple_executions_under_same_session() {
    let mut mgr = ContextManager::new();
    mgr.create_application_context("app");
    let session_id = mgr.create_session_context().unwrap();
    let e1 = mgr.create_execution_context(session_id).unwrap();
    let e2 = mgr.create_execution_context(session_id).unwrap();
    assert_ne!(e1, e2);
    assert_eq!(mgr.execution_count(), 2);
}

#[test]
fn executions_under_different_sessions_are_isolated() {
    let mut mgr = ContextManager::new();
    mgr.create_application_context("app");
    let s1 = mgr.create_session_context().unwrap();
    let s2 = mgr.create_session_context().unwrap();
    let e1 = mgr.create_execution_context(s1).unwrap();
    let e2 = mgr.create_execution_context(s2).unwrap();
    let exec1 = mgr.execution_context(e1).unwrap();
    let exec2 = mgr.execution_context(e2).unwrap();
    assert_ne!(exec1.session_id(), exec2.session_id());
}

#[test]
fn set_application_context_injects_external_context() {
    let mut mgr = ContextManager::new();
    let external = app_shell::app_engine::ApplicationContext::new("injected");
    let injected_id = external.id();
    let returned_id = mgr.set_application_context(external);
    assert_eq!(returned_id, injected_id);
    assert_eq!(mgr.application_context().unwrap().application_id(), "injected");
}