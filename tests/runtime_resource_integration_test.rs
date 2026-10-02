use app_shell::app_engine::{
    Bootstrap, Lifecycle, ResourceLoader, ResourceManager,
};
use std::any::Any;

struct DummyLoader;
impl ResourceLoader for DummyLoader {
    fn resource_type(&self) -> &str {
        "test"
    }
    fn load(&self, source: &str) -> Result<Box<dyn Any + Send>, app_shell::app_engine::ResourceError> {
        Ok(Box::new(format!("data:{}", source)))
    }
}

#[test]
fn runtime_owns_resource_manager() {
    let runtime = Bootstrap::create().unwrap();
    let _: &ResourceManager = runtime.resource_manager();
}

#[test]
fn runtime_resource_manager_starts_empty() {
    let runtime = Bootstrap::create().unwrap();
    assert_eq!(runtime.resource_manager().resource_count(), 0);
    assert_eq!(runtime.resource_manager().loader_count(), 0);
}

#[test]
fn runtime_can_register_and_load_resource() {
    let mut runtime = Bootstrap::create().unwrap();

    runtime
        .resource_manager_mut()
        .register_loader(Box::new(DummyLoader));

    let handle = runtime
        .resource_manager_mut()
        .load("test", "file:///asset.dat")
        .unwrap();

    let data = runtime
        .resource_manager()
        .with_resource(handle.resource_id(), |r| {
            r.data::<String>().map(|s| s.clone())
        })
        .flatten();

    assert_eq!(data, Some("data:file:///asset.dat".to_string()));

    runtime.resource_manager_mut().release(handle).unwrap();
}

#[test]
fn resource_manager_independent_of_lifecycle() {
    let mut runtime = Bootstrap::create().unwrap();
    runtime.initialize().unwrap();
    runtime.start().unwrap();
    assert_eq!(runtime.resource_manager().resource_count(), 0);
    runtime.stop().unwrap();
    runtime.dispose().unwrap();
}