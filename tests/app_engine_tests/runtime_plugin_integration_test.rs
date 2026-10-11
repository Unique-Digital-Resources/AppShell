use app_shell::app_engine::{
    Bootstrap, Lifecycle, Plugin, PluginContext, PluginError, PluginManager,
    PluginManifest, PluginState, EngineRef,
};

struct StubPlugin { id: String }
impl Plugin for StubPlugin {
    fn id(&self) -> &str { &self.id }
    fn initialize(&mut self, _ctx: &PluginContext, _engine: &EngineRef) -> Result<(), PluginError> { Ok(()) }
    fn start(&mut self, _engine: &EngineRef) -> Result<(), PluginError> { Ok(()) }
    fn stop(&mut self) -> Result<(), PluginError> { Ok(()) }
    fn dispose(&mut self) -> Result<(), PluginError> { Ok(()) }
}

#[test]
fn runtime_owns_plugin_manager() {
    let runtime = Bootstrap::create().unwrap();
    let _: &PluginManager = runtime.plugin_manager();
}

#[test]
fn runtime_plugin_manager_starts_empty() {
    let runtime = Bootstrap::create().unwrap();
    assert_eq!(runtime.plugin_manager().count(), 0);
}

#[test]
fn runtime_can_register_and_lifecycle_plugin() {
    let mut runtime = Bootstrap::create().unwrap();

    let id = runtime.plugin_manager_mut().register(
        PluginManifest::new("stub", "Stub").with_description("Stub plugin"),
        Box::new(StubPlugin { id: "stub".to_string() }),
    ).unwrap();

    // Runtime lifecycle constructs EngineRef internally
    runtime.initialize().unwrap();
    assert_eq!(runtime.plugin_manager().state(id), Some(PluginState::Initialized));

    runtime.start().unwrap();
    assert_eq!(runtime.plugin_manager().state(id), Some(PluginState::Started));

    runtime.stop().unwrap();
    assert_eq!(runtime.plugin_manager().state(id), Some(PluginState::Stopped));

    runtime.dispose().unwrap();
    assert_eq!(runtime.plugin_manager().state(id), Some(PluginState::Unloaded));
}

#[test]
fn plugin_manager_independent_of_lifecycle() {
    let mut runtime = Bootstrap::create().unwrap();
    runtime.initialize().unwrap();
    runtime.start().unwrap();
    assert_eq!(runtime.plugin_manager().count(), 0);
    runtime.stop().unwrap();
    runtime.dispose().unwrap();
}