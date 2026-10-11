use app_shell::app_engine::PluginContext;

#[test]
fn context_stores_plugin_id() {
    let ctx = PluginContext::new("image-plugin");
    assert_eq!(ctx.plugin_id(), "image-plugin");
}

#[test]
fn context_with_metadata() {
    let ctx = PluginContext::new("image-plugin")
        .with_metadata("engine_version", "1.0");
    assert_eq!(ctx.metadata().get("engine_version"), Some(&"1.0".to_string()));
}

#[test]
fn context_is_cloneable() {
    let ctx = PluginContext::new("test").with_metadata("k", "v");
    let cloned = ctx.clone();
    assert_eq!(ctx.plugin_id(), cloned.plugin_id());
}