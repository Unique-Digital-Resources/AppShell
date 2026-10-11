//! Tests for the AppEngine composition root.

use app_shell::AppEngine;
use app_shell::app_engine::{Bootstrap, Lifecycle, RuntimeState, EngineRef};
use app_shell::app_engine::{
    Plugin, PluginContext, PluginError, PluginManifest, PluginState,
    Service, ServiceDefinition, ServiceError, ServiceState,
};
use std::sync::{Arc, Mutex};

#[test]
fn app_engine_is_app_runtime() {
    let engine: AppEngine = Bootstrap::create().unwrap();
    assert_eq!(engine.state(), RuntimeState::Created);
}

#[test]
fn app_engine_accessible_from_crate_root() {
    let _: AppEngine = Bootstrap::create().unwrap();
}

#[test]
fn app_engine_provides_all_systems() {
    let engine = Bootstrap::create().unwrap();
    let _ = engine.context_manager();
    let _ = engine.state_manager();
    let _ = engine.command_executor();
    let _ = engine.task_manager();
    let _ = engine.scheduler();
    let _ = engine.event_bus();
    let _ = engine.signal_bus();
    let _ = engine.history_store();
    let _ = engine.transaction_manager();
    let _ = engine.resource_manager();
    let _ = engine.service_manager();
    let _ = engine.plugin_manager();
    let _ = engine.config_store();
    let _ = engine.preferences();
}

#[test]
fn app_engine_lifecycle_integrates_subsystems() {
    struct StubService { started: Arc<Mutex<bool>> }
    impl Service for StubService {
        fn service_type(&self) -> &str { "stub" }
        fn initialize(&mut self, _engine: &EngineRef) -> Result<(), ServiceError> { Ok(()) }
        fn start(&mut self, _engine: &EngineRef) -> Result<(), ServiceError> {
            *self.started.lock().unwrap() = true; Ok(())
        }
        fn stop(&mut self) -> Result<(), ServiceError> {
            *self.started.lock().unwrap() = false; Ok(())
        }
        fn dispose(&mut self) -> Result<(), ServiceError> { Ok(()) }
    }

    struct StubPlugin { started: Arc<Mutex<bool>> }
    impl Plugin for StubPlugin {
        fn id(&self) -> &str { "stub" }
        fn initialize(&mut self, _ctx: &PluginContext, _engine: &EngineRef) -> Result<(), PluginError> { Ok(()) }
        fn start(&mut self, _engine: &EngineRef) -> Result<(), PluginError> {
            *self.started.lock().unwrap() = true; Ok(())
        }
        fn stop(&mut self) -> Result<(), PluginError> {
            *self.started.lock().unwrap() = false; Ok(())
        }
        fn dispose(&mut self) -> Result<(), PluginError> { Ok(()) }
    }

    let mut engine = Bootstrap::create().unwrap();
    let svc_started = Arc::new(Mutex::new(false));
    let plg_started = Arc::new(Mutex::new(false));

    engine.service_manager_mut().register(
        ServiceDefinition::new("stub", "Stub", "Stub service"),
        Box::new(StubService { started: svc_started.clone() }),
    ).unwrap();

    engine.plugin_manager_mut().register(
        PluginManifest::new("stub", "Stub"),
        Box::new(StubPlugin { started: plg_started.clone() }),
    ).unwrap();

    engine.initialize().unwrap();
    assert_eq!(
        engine.service_manager().state(engine.service_manager().list_ids()[0]),
        Some(ServiceState::Initialized)
    );
    assert_eq!(
        engine.plugin_manager().state(engine.plugin_manager().list_ids()[0]),
        Some(PluginState::Initialized)
    );

    engine.start().unwrap();
    assert!(*svc_started.lock().unwrap());
    assert!(*plg_started.lock().unwrap());

    engine.stop().unwrap();
    assert!(!*svc_started.lock().unwrap());
    assert!(!*plg_started.lock().unwrap());

    engine.dispose().unwrap();
    assert_eq!(
        engine.service_manager().state(engine.service_manager().list_ids()[0]),
        Some(ServiceState::Disposed)
    );
    assert_eq!(
        engine.plugin_manager().state(engine.plugin_manager().list_ids()[0]),
        Some(PluginState::Unloaded)
    );
}

#[test]
fn engine_error_converts_from_subsystem_errors() {
    use app_shell::app_engine::{
        CommandError, EngineError, PluginError, ResourceError, ServiceError, TaskError,
    };

    let _: EngineError = ServiceError::NotFound { id: 1 }.into();
    let _: EngineError = PluginError::NotFound { id: 1 }.into();
    let _: EngineError = CommandError::UnknownCommand { id: "x".into() }.into();
    let _: EngineError = TaskError::NotFound { id: 1 }.into();
    let _: EngineError = ResourceError::NotFound { id: 1 }.into();
}