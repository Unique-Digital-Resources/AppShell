use app_shell::app_engine::{
    Bootstrap, Lifecycle, ServiceDefinition, ServiceError, ServiceManager, ServiceState,
    Service, EngineRef,
};
use std::sync::{Arc, Mutex};

struct CounterService {
    started: Arc<Mutex<bool>>,
}

impl Service for CounterService {
    fn service_type(&self) -> &str { "counter" }
    fn initialize(&mut self, _engine: &EngineRef) -> Result<(), ServiceError> { Ok(()) }
    fn start(&mut self, _engine: &EngineRef) -> Result<(), ServiceError> {
        *self.started.lock().unwrap() = true;
        Ok(())
    }
    fn stop(&mut self) -> Result<(), ServiceError> {
        *self.started.lock().unwrap() = false;
        Ok(())
    }
    fn dispose(&mut self) -> Result<(), ServiceError> { Ok(()) }
}

#[test]
fn runtime_owns_service_manager() {
    let runtime = Bootstrap::create().unwrap();
    let _: &ServiceManager = runtime.service_manager();
}

#[test]
fn runtime_service_manager_starts_empty() {
    let runtime = Bootstrap::create().unwrap();
    assert_eq!(runtime.service_manager().count(), 0);
}

#[test]
fn runtime_can_register_and_start_service() {
    let mut runtime = Bootstrap::create().unwrap();
    let started = Arc::new(Mutex::new(false));

    let id = runtime.service_manager_mut().register(
        ServiceDefinition::new("counter", "Counter", "Counting service"),
        Box::new(CounterService { started: started.clone() }),
    ).unwrap();

    // Initialize and start through the runtime lifecycle (which constructs EngineRef internally)
    runtime.initialize().unwrap();
    runtime.start().unwrap();
    assert_eq!(runtime.service_manager().state(id), Some(ServiceState::Running));
    assert!(*started.lock().unwrap());

    runtime.stop().unwrap();
    assert!(!*started.lock().unwrap());

    runtime.dispose().unwrap();
}

#[test]
fn service_manager_independent_of_lifecycle() {
    let mut runtime = Bootstrap::create().unwrap();
    runtime.initialize().unwrap();
    runtime.start().unwrap();
    assert_eq!(runtime.service_manager().count(), 0);
    runtime.stop().unwrap();
    runtime.dispose().unwrap();
}