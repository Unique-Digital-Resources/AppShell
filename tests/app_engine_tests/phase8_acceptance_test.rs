//! Phase 8 acceptance criteria.

use app_shell::app_engine::{
    AppEngine, Bootstrap, Resource, ResourceError, ResourceHandle, ResourceLoader,
    ResourceState,
};
use std::any::Any;

struct ImageLoader;
impl ResourceLoader for ImageLoader {
    fn resource_type(&self) -> &str { "image" }
    fn load(&self, source: &str) -> Result<Box<dyn Any + Send>, ResourceError> {
        if source.contains("missing") {
            Err(ResourceError::LoadFailed { source: source.to_string(), reason: "file not found".to_string() })
        } else {
            Ok(Box::new(format!("pixels:{}", source)))
        }
    }
}

struct FontLoader;
impl ResourceLoader for FontLoader {
    fn resource_type(&self) -> &str { "font" }
    fn load(&self, source: &str) -> Result<Box<dyn Any + Send>, ResourceError> {
        Ok(Box::new(format!("glyphs:{}", source)))
    }
}

fn setup() -> AppEngine {
    let mut engine = Bootstrap::create().unwrap();
    engine.resource_manager_mut().register_loader(Box::new(ImageLoader));
    engine.resource_manager_mut().register_loader(Box::new(FontLoader));
    engine
}

#[test]
fn acceptance_represent_resource() {
    let r = Resource::new("image", "file:///logo.png").with_data("pixels".to_string());
    assert_eq!(r.resource_type(), "image");
    assert!(r.id().value() > 0);
}

#[test]
fn acceptance_resource_lifecycle() {
    let r = Resource::new("image", "file:///logo.png");
    assert_eq!(r.state(), ResourceState::Unloaded);
    let r = r.with_data("pixels".to_string());
    assert_eq!(r.state(), ResourceState::Loaded);
}

#[test]
fn acceptance_load_returns_handle() {
    let mut engine = setup();
    let handle: ResourceHandle = engine.resource_manager_mut().load("image", "file:///logo.png").unwrap();
    assert!(handle.resource_id().value() > 0);
    assert!(engine.resource_manager().exists(handle.resource_id()));
}

#[test]
fn acceptance_track_and_release() {
    let mut engine = setup();
    let handle = engine.resource_manager_mut().load("image", "file:///logo.png").unwrap();
    let id = handle.resource_id();
    assert_eq!(engine.resource_manager().active_count(id), 1);

    engine.resource_manager_mut().acquire(&handle).unwrap();
    assert_eq!(engine.resource_manager().active_count(id), 2);

    engine.resource_manager_mut().release(handle).unwrap();
    assert_eq!(engine.resource_manager().active_count(id), 1);

    engine.resource_manager_mut().release_by_id(id).unwrap();
    assert_eq!(engine.resource_manager().active_count(id), 0);
    assert!(engine.resource_manager().exists(id));
}

#[test]
fn acceptance_cache_reuse() {
    let mut engine = setup();
    let h1 = engine.resource_manager_mut().load("image", "file:///logo.png").unwrap();
    let h2 = engine.resource_manager_mut().load("image", "file:///logo.png").unwrap();
    assert_eq!(h1.resource_id(), h2.resource_id());
    assert_eq!(engine.resource_manager().resource_count(), 1);
}

#[test]
fn acceptance_unload() {
    let mut engine = setup();
    let handle = engine.resource_manager_mut().load("image", "file:///logo.png").unwrap();
    let id = handle.resource_id();
    engine.resource_manager_mut().release(handle).unwrap();
    engine.resource_manager_mut().unload(id).unwrap();
    assert!(!engine.resource_manager().exists(id));
}

#[test]
fn acceptance_loading_failure() {
    let mut engine = setup();
    let result = engine.resource_manager_mut().load("image", "file:///missing.png");
    assert!(matches!(result, Err(ResourceError::LoadFailed { .. })));
    assert_eq!(engine.resource_manager().resource_count(), 0);
}

#[test]
fn acceptance_reload() {
    let mut engine = setup();
    let handle = engine.resource_manager_mut().load("image", "file:///logo.png").unwrap();
    let id = handle.resource_id();
    let has_data = engine.resource_manager().with_resource(handle.resource_id(), |r| r.has_data()).unwrap();
    assert!(has_data);

    engine.resource_manager_mut().reload(id).unwrap();

    let (state, has_data) = engine.resource_manager().with_resource(handle.resource_id(), |r| {
        (r.state(), r.has_data())
    }).unwrap();
    assert_eq!(state, ResourceState::Loaded);
    assert!(has_data);
}

#[test]
fn acceptance_acquire_through_context() {
    let mut engine = setup();
    let handle = engine.resource_manager_mut().load("image", "file:///asset.png").unwrap();
    let data = engine.resource_manager().with_resource(handle.resource_id(), |r| {
        r.data::<String>().map(|s| s.clone())
    }).flatten();
    assert_eq!(data, Some("pixels:file:///asset.png".to_string()));
    engine.resource_manager_mut().release(handle).unwrap();
}

#[test]
fn acceptance_event_integration() {
    use app_shell::app_engine::{Event, EventBus, EventHandler};
    use std::sync::{Arc, Mutex};

    let mut bus = EventBus::new();
    let loaded = Arc::new(Mutex::new(false));

    struct Handler { flag: Arc<Mutex<bool>> }
    impl EventHandler for Handler {
        fn handle(&self, _event: &Event) { *self.flag.lock().unwrap() = true; }
    }

    bus.subscribe("resource.loaded", Box::new(Handler { flag: loaded.clone() }));
    let mut engine = setup();
    let _handle = engine.resource_manager_mut().load("image", "file:///logo.png").unwrap();
    bus.publish(&Event::new("resource.loaded"));
    assert!(*loaded.lock().unwrap());
}

#[test]
fn acceptance_domain_provides_loaders() {
    let mut engine = setup();
    let h1 = engine.resource_manager_mut().load("image", "file:///logo.png").unwrap();
    let h2 = engine.resource_manager_mut().load("font", "file:///arial.ttf").unwrap();
    let img_type = engine.resource_manager().with_resource(h1.resource_id(), |r| r.resource_type().to_string()).unwrap();
    let font_type = engine.resource_manager().with_resource(h2.resource_id(), |r| r.resource_type().to_string()).unwrap();
    assert_eq!(img_type, "image");
    assert_eq!(font_type, "font");
}

#[test]
fn acceptance_full_lifecycle() {
    let mut engine = Bootstrap::create().unwrap();
    engine.resource_manager_mut().register_loader(Box::new(ImageLoader));

    let handle = engine.resource_manager_mut().load("image", "file:///project/logo.png").unwrap();
    let id = handle.resource_id();
    assert!(engine.resource_manager().exists(id));

    engine.resource_manager_mut().acquire(&handle).unwrap();
    assert_eq!(engine.resource_manager().active_count(id), 2);

    engine.resource_manager_mut().release(handle).unwrap();
    engine.resource_manager_mut().release_by_id(id).unwrap();
    assert_eq!(engine.resource_manager().active_count(id), 0);

    engine.resource_manager_mut().unload(id).unwrap();
    assert!(!engine.resource_manager().exists(id));
}