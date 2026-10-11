use app_shell::app_engine::*;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// --- ConfigStore thread safety ---

#[test]
fn config_store_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ConfigStore>();
}

#[test]
fn config_store_concurrent_writes() {
    let store = Arc::new(ConfigStore::new(ConfigSchema::new()
        .with_property(ConfigPropertyDefinition::new("key", "Key"))));

    let mut handles = vec![];
    for i in 0..5 {
        let store_clone = Arc::clone(&store);
        handles.push(thread::spawn(move || {
            store_clone.set("key", format!("value_{}", i)).unwrap();
        }));
    }
    for h in handles {
        h.join().unwrap();
    }

    // The final value is one of the five (last writer wins)
    let value = store.get("key").unwrap();
    assert!(value.starts_with("value_"));
}

#[test]
fn config_store_concurrent_reads() {
    let store = Arc::new(ConfigStore::new(ConfigSchema::new()
        .with_property(ConfigPropertyDefinition::new("key", "Key").with_default("default"))));

    let mut handles = vec![];
    for _ in 0..10 {
        let store_clone = Arc::clone(&store);
        handles.push(thread::spawn(move || {
            let value = store_clone.get("key");
            assert_eq!(value, Some("default".to_string()));
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
}

// --- Preferences thread safety ---

#[test]
fn preferences_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Preferences>();
}

#[test]
fn preferences_concurrent_writes() {
    let prefs = Arc::new(Preferences::new());

    let mut handles = vec![];
    for i in 0..5 {
        let prefs_clone = Arc::clone(&prefs);
        handles.push(thread::spawn(move || {
            prefs_clone.set(format!("key_{}", i), "value");
        }));
    }
    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(prefs.len(), 5);
}

// --- ResourceManager thread safety ---

#[test]
fn resource_manager_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ResourceManager>();
}

struct StringLoader;
impl ResourceLoader for StringLoader {
    fn resource_type(&self) -> &str { "text" }
    fn load(&self, source: &str) -> Result<Box<dyn std::any::Any + Send>, ResourceError> {
        Ok(Box::new(format!("content:{}", source)))
    }
}

#[test]
fn resource_manager_concurrent_loads() {
    let mgr = Arc::new(ResourceManager::new());
    mgr.register_loader(Box::new(StringLoader));

    let mut handles = vec![];
    for i in 0..5 {
        let mgr_clone = Arc::clone(&mgr);
        handles.push(thread::spawn(move || {
            mgr_clone.load("text", &format!("file:///test_{}.txt", i)).unwrap()
        }));
    }

    let mut handles_vec = vec![];
    for h in handles {
        handles_vec.push(h.join().unwrap());
    }

    assert_eq!(mgr.resource_count(), 5);
}

// --- Scheduler thread safety ---

#[test]
fn scheduler_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Scheduler>();
}

#[test]
fn scheduler_concurrent_submits() {
    let scheduler = Arc::new(Scheduler::new());

    let mut handles = vec![];
    for _ in 0..5 {
        let scheduler_clone = Arc::clone(&scheduler);
        handles.push(thread::spawn(move || {
            scheduler_clone.submit(TaskId::new(), Schedule::Immediate, 0)
        }));
    }

    let mut job_ids = vec![];
    for h in handles {
        job_ids.push(h.join().unwrap());
    }

    // All jobs should be in the queue
    assert_eq!(scheduler.queue_len(), 5);

    // All job IDs are unique
    let mut sorted = job_ids.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), 5);
}

#[test]
fn scheduler_concurrent_tick_and_dispatch() {
    let scheduler = Arc::new(Scheduler::new());

    // Submit some jobs
    for _ in 0..10 {
        scheduler.submit(TaskId::new(), Schedule::Immediate, 0);
    }

    // Tick from one thread
    let scheduler_clone = Arc::clone(&scheduler);
    let tick_handle = thread::spawn(move || {
        scheduler_clone.tick();
    });
    tick_handle.join().unwrap();

    // Dispatch from another thread
    let scheduler_clone2 = Arc::clone(&scheduler);
    let dispatch_handle = thread::spawn(move || {
        let mut count = 0;
        while scheduler_clone2.dispatch().is_some() {
            count += 1;
        }
        count
    });

    let dispatched = dispatch_handle.join().unwrap();
    assert_eq!(dispatched, 10);
}

// --- Integration: all managers shared via Arc ---

#[test]
fn all_managers_shared_across_threads() {
    let config_store = Arc::new(ConfigStore::new(ConfigSchema::new()));
    let preferences = Arc::new(Preferences::new());
    let resource_manager = Arc::new(ResourceManager::new());
    let scheduler = Arc::new(Scheduler::new());

    let handles = vec![
        thread::spawn({
            let cs = Arc::clone(&config_store);
            move || { cs.set("thread_key", "thread_value").unwrap(); }
        }),
        thread::spawn({
            let p = Arc::clone(&preferences);
            move || { p.set("thread_pref", "pref_value"); }
        }),
        thread::spawn({
            let s = Arc::clone(&scheduler);
            move || { s.submit(TaskId::new(), Schedule::Immediate, 0); }
        }),
    ];

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(config_store.get("thread_key"), Some("thread_value".to_string()));
    assert_eq!(preferences.get("thread_pref"), Some("pref_value".to_string()));
    assert_eq!(scheduler.queue_len(), 1);
}