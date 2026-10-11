//! Tests for `Bootstrap`.

use app_shell::app_engine::{
    AppRuntime, ApplicationContext, Bootstrap, Lifecycle, RuntimeState,
};

#[test]
fn bootstrap_create_returns_created_runtime() {
    let runtime = Bootstrap::create().expect("bootstrap should succeed");
    assert_eq!(runtime.state(), RuntimeState::Created);
}

#[test]
fn bootstrap_new_then_create_is_equivalent() {
    // Bootstrap::new() returns a value, but `create` is an associated function
    // (no `self`), so it is always invoked as `Bootstrap::create()`.
    let _bootstrap = Bootstrap::new();
    let runtime = Bootstrap::create().unwrap();
    assert_eq!(runtime.state(), RuntimeState::Created);
}

#[test]
fn bootstrap_with_custom_context() {
    let context = ApplicationContext::new("custom_app");
    let runtime = Bootstrap::create_with_context(context).unwrap();
    assert_eq!(runtime.context().application_id(), "custom_app");
    assert_eq!(runtime.state(), RuntimeState::Created);
}

#[test]
fn bootstrapped_runtime_can_complete_full_lifecycle() {
    let mut runtime: AppRuntime = Bootstrap::create().unwrap();
    runtime.initialize().unwrap();
    runtime.start().unwrap();
    runtime.run().unwrap();
    runtime.stop().unwrap();
    runtime.dispose().unwrap();
    assert_eq!(runtime.state(), RuntimeState::Disposed);
}

#[test]
fn bootstrap_default_works() {
    let _bootstrap = Bootstrap::default();
    let runtime = Bootstrap::create().unwrap();
    assert_eq!(runtime.state(), RuntimeState::Created);
}