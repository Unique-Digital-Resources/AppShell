use app_shell::app_engine::{Resource, ResourceCache, ResourceId, ResourceState};

fn make_resource() -> Resource {
    Resource::new("image", "file:///test.png").with_data("data".to_string())
}

#[test]
fn new_cache_is_empty() {
    let c = ResourceCache::new();
    assert!(c.is_empty());
    assert_eq!(c.len(), 0);
}

#[test]
fn insert_and_get() {
    let mut c = ResourceCache::new();
    let r = make_resource();
    let id = r.id();
    c.insert(r);
    assert_eq!(c.len(), 1);
    assert!(c.contains(id));
    assert!(c.get(id).is_some());
    assert_eq!(c.get(id).unwrap().resource_type(), "image");
}

#[test]
fn get_missing_returns_none() {
    let c = ResourceCache::new();
    assert!(c.get(ResourceId::new()).is_none());
}

#[test]
fn remove() {
    let mut c = ResourceCache::new();
    let r = make_resource();
    let id = r.id();
    c.insert(r);
    let removed = c.remove(id);
    assert!(removed.is_some());
    assert!(!c.contains(id));
    assert_eq!(c.len(), 0);
}

#[test]
fn clear() {
    let mut c = ResourceCache::new();
    c.insert(make_resource());
    c.insert(make_resource());
    c.clear();
    assert!(c.is_empty());
}

#[test]
fn get_mut_returns_mutable_reference() {
    let mut c = ResourceCache::new();
    let r = make_resource();
    let id = r.id();
    c.insert(r);

    let resource = c.get_mut(id).unwrap();
    assert_eq!(resource.resource_type(), "image");
    assert_eq!(resource.state(), ResourceState::Loaded);
}

#[test]
fn ids_returns_all() {
    let mut c = ResourceCache::new();
    let r1 = make_resource();
    let r2 = make_resource();
    let id1 = r1.id();
    let id2 = r2.id();
    c.insert(r1);
    c.insert(r2);
    let ids = c.ids();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));
}