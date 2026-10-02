use app_shell::app_engine::*;
use std::sync::{Arc, Mutex};
use std::any::Any;

struct StringLoader;
impl ResourceLoader for StringLoader {
    fn resource_type(&self) -> &str { "text" }
    fn load(&self, source: &str) -> Result<Box<dyn Any + Send>, ResourceError> {
        Ok(Box::new(format!("content:{}", source)))
    }
}

// --- RAII auto-release tests ---

#[test]
fn raii_handle_releases_on_drop() {
    let mgr = Arc::new(ResourceManager::new());
    mgr.set_self_ref(Arc::downgrade(&mgr));
    mgr.register_loader(Box::new(StringLoader));

    let id;
    {
        let handle = mgr.load("text", "file:///test.txt").unwrap();
        id = handle.resource_id();
        assert_eq!(mgr.active_count(id), 1);
        // handle drops here
    }
    // After drop, ref count should be 0
    assert_eq!(mgr.active_count(id), 0);
    // Resource still in cache
    assert!(mgr.exists(id));
}

#[test]
fn raii_handle_multiple_holders() {
    let mgr = Arc::new(ResourceManager::new());
    mgr.set_self_ref(Arc::downgrade(&mgr));
    mgr.register_loader(Box::new(StringLoader));

    let handle1 = mgr.load("text", "file:///multi.txt").unwrap();
    let id = handle1.resource_id();
    assert_eq!(mgr.active_count(id), 1);

    // Acquire a second handle
    mgr.acquire(&handle1).unwrap();
    assert_eq!(mgr.active_count(id), 2);

    // Drop first handle
    drop(handle1);
    assert_eq!(mgr.active_count(id), 1);

    // Manually release the second ref
    mgr.release_by_id(id).unwrap();
    assert_eq!(mgr.active_count(id), 0);
}

#[test]
fn raii_handle_after_release_resource_still_cached() {
    let mgr = Arc::new(ResourceManager::new());
    mgr.set_self_ref(Arc::downgrade(&mgr));
    mgr.register_loader(Box::new(StringLoader));

    let handle = mgr.load("text", "file:///cached.txt").unwrap();
    let id = handle.resource_id();
    drop(handle);

    assert_eq!(mgr.active_count(id), 0);
    assert!(mgr.exists(id));  // still in cache
    assert_eq!(mgr.resource_count(), 1);
}

#[test]
fn raii_handle_unload_after_auto_release() {
    let mgr = Arc::new(ResourceManager::new());
    mgr.set_self_ref(Arc::downgrade(&mgr));
    mgr.register_loader(Box::new(StringLoader));

    let id;
    {
        let handle = mgr.load("text", "file:///unload.txt").unwrap();
        id = handle.resource_id();
        // handle drops here → ref count = 0
    }

    // Now we can unload
    mgr.unload(id).unwrap();
    assert!(!mgr.exists(id));
    assert_eq!(mgr.resource_count(), 0);
}

#[test]
fn raii_handle_cannot_unload_while_in_use() {
    let mgr = Arc::new(ResourceManager::new());
    mgr.set_self_ref(Arc::downgrade(&mgr));
    mgr.register_loader(Box::new(StringLoader));

    let handle = mgr.load("text", "file:///inuse.txt").unwrap();
    let id = handle.resource_id();

    // Still in use (handle alive)
    let result = mgr.unload(id);
    assert!(matches!(result, Err(ResourceError::InUse { .. })));

    // Drop handle, then unload works
    drop(handle);
    mgr.unload(id).unwrap();
}

#[test]
fn raii_handle_with_resource_manager_via_arc() {
    let mgr = Arc::new(ResourceManager::new());
    mgr.set_self_ref(Arc::downgrade(&mgr));
    mgr.register_loader(Box::new(StringLoader));

    // Simulate usage from multiple "consumers"
    let mgr1 = Arc::clone(&mgr);
    let mgr2 = Arc::clone(&mgr);

    let handle1 = mgr1.load("text", "file:///shared.txt").unwrap();
    let id = handle1.resource_id();

    let handle2_handle = mgr2.acquire(&handle1);
    assert!(handle2_handle.is_ok());
    assert_eq!(mgr.active_count(id), 2);

    drop(handle1);
    assert_eq!(mgr.active_count(id), 1);

    mgr.release_by_id(id).unwrap();
    assert_eq!(mgr.active_count(id), 0);
}

// --- Integration: RAII with AppRuntime ---

#[test]
fn runtime_raii_handle_auto_releases() {
    let mut engine = Bootstrap::create().unwrap();
    engine.resource_manager().register_loader(Box::new(StringLoader));

    let id;
    {
        let handle = engine.resource_manager().load("text", "file:///runtime.txt").unwrap();
        id = handle.resource_id();
        assert_eq!(engine.resource_manager().active_count(id), 1);
        // handle drops here
    }

    // After scope exit, ref count should be 0
    assert_eq!(engine.resource_manager().active_count(id), 0);
    assert!(engine.resource_manager().exists(id));
}

#[test]
fn runtime_raii_multiple_loads_and_drops() {
    let mut engine = Bootstrap::create().unwrap();
    engine.resource_manager().register_loader(Box::new(StringLoader));

    let mut ids = vec![];

    // Load 5 resources
    for i in 0..5 {
        let handle = engine.resource_manager().load("text", &format!("file:///file_{}.txt", i)).unwrap();
        ids.push((handle.resource_id(), handle));
    }

    assert_eq!(engine.resource_manager().resource_count(), 5);

    // Drop all handles
    for (_, handle) in ids.drain(..) {
        drop(handle);
    }

    // All ref counts should be 0
    // Can't check individual IDs since we dropped the handles
    // but resource count should still be 5 (cached, not unloaded)
    assert_eq!(engine.resource_manager().resource_count(), 5);
}

// --- RAII handle is not Copy (can't accidentally double-drop) ---

#[test]
fn raii_handle_is_not_copy() {
    let mgr = Arc::new(ResourceManager::new());
    mgr.set_self_ref(Arc::downgrade(&mgr));
    mgr.register_loader(Box::new(StringLoader));

    let handle = mgr.load("text", "file:///nocopy.txt").unwrap();
    let id = handle.resource_id();

    // Move handle into a scope and drop
    drop(handle);

    // Ref count should be 0 (single release, not double)
    assert_eq!(mgr.active_count(id), 0);
}