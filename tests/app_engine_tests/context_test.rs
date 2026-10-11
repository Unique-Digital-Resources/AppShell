//! Tests for `Context` and `ApplicationContext`.

use app_shell::app_engine::{ApplicationContext, Context};

#[test]
fn base_context_constructs() {
    let _c = Context::new();
    let _default: Context = Context::default();
}

#[test]
fn application_context_stores_id() {
    let ctx = ApplicationContext::new("my_app");
    assert_eq!(ctx.application_id(), "my_app");
}

#[test]
fn application_context_instance_ids_are_unique() {
    let a = ApplicationContext::new("app");
    let b = ApplicationContext::new("app");
    assert_ne!(a.instance_id(), b.instance_id());
}

#[test]
fn application_context_equality_is_value_based() {
    let a = ApplicationContext::new("app");
    let b = ApplicationContext::new("app");
    // instance_ids differ → not equal
    assert_ne!(a, b);

    let c = a.clone();
    assert_eq!(a, c);
}