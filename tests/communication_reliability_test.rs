use app_shell::app_engine::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// --- Event priority tests ---

struct CountingHandler {
    order: Arc<Mutex<Vec<usize>>>,
    tag: usize,
}

impl CountingHandler {
    fn new(order: Arc<Mutex<Vec<usize>>>, tag: usize) -> Self {
        Self { order, tag }
    }
}

impl EventHandler for CountingHandler {
    fn handle(&self, _event: &Event) {
        self.order.lock().unwrap().push(self.tag);
    }
}

#[test]
fn high_priority_delivered_first() {
    let mut bus = EventBus::new();
    let order = Arc::new(Mutex::new(vec![]));

    bus.subscribe_with_priority(
        "test.priority",
        Box::new(CountingHandler::new(order.clone(), 1)), // Normal
        EventPriority::Normal,
    );
    bus.subscribe_with_priority(
        "test.priority",
        Box::new(CountingHandler::new(order.clone(), 0)), // High
        EventPriority::High,
    );
    bus.subscribe_with_priority(
        "test.priority",
        Box::new(CountingHandler::new(order.clone(), 2)), // Low
        EventPriority::Low,
    );

    bus.publish(&Event::new("test.priority"));

    let order = order.lock().unwrap();
    assert_eq!(*order, vec![0, 1, 2]); // High, Normal, Low
}

#[test]
fn normal_priority_is_default() {
    let priority = EventPriority::default();
    assert_eq!(priority, EventPriority::Normal);
}

#[test]
fn priority_ordering() {
    assert!(EventPriority::High > EventPriority::Normal);
    assert!(EventPriority::Normal > EventPriority::Low);
}

// --- Handler error isolation tests ---

struct PanickingHandler;
impl EventHandler for PanickingHandler {
    fn handle(&self, _event: &Event) {
        panic!("handler crashed!");
    }
}

struct SurvivingHandler {
    received: Arc<Mutex<bool>>,
}

impl EventHandler for SurvivingHandler {
    fn handle(&self, _event: &Event) {
        *self.received.lock().unwrap() = true;
    }
}

#[test]
fn panicking_handler_doesnt_crash_engine() {
    let mut bus = EventBus::new();
    let received = Arc::new(Mutex::new(false));

    bus.subscribe(
        "test.isolation",
        Box::new(PanickingHandler),
    );
    bus.subscribe(
        "test.isolation",
        Box::new(SurvivingHandler {
            received: received.clone(),
        }),
    );

    // This should not panic
    bus.publish(&Event::new("test.isolation"));

    // The surviving handler should still have been called
    assert!(*received.lock().unwrap());
}

#[test]
fn panicking_handler_in_high_priority_doesnt_block_others() {
    let mut bus = EventBus::new();
    let received = Arc::new(Mutex::new(false));

    bus.subscribe_with_priority(
        "test.isolation2",
        Box::new(PanickingHandler),
        EventPriority::High,
    );
    bus.subscribe_with_priority(
        "test.isolation2",
        Box::new(SurvivingHandler {
            received: received.clone(),
        }),
        EventPriority::Normal,
    );

    bus.publish(&Event::new("test.isolation2"));

    assert!(*received.lock().unwrap());
}

// --- Config change notification tests ---

#[test]
fn config_store_notifies_on_set() {
    let schema = ConfigSchema::new()
        .with_property(ConfigPropertyDefinition::new("key", "Key").with_default("default"));

    let backend = Box::new(app_shell::app_engine::MemoryStorageBackend::new());
    let mut store = ConfigStore::with_backend(schema, backend);

    let received = Arc::new(Mutex::new(None::<(String, Option<String>, Option<String>)>));
    let r = received.clone();
    store.on_changed(move |key, old, new| {
        *r.lock().unwrap() = Some((
            key.to_string(),
            old.map(|s| s.to_string()),
            new.map(|s| s.to_string()),
        ));
    });

    store.set("key", "new_value").unwrap();

    let received = received.lock().unwrap();
    let (key, old, new) = received.as_ref().unwrap();
    assert_eq!(key, "key");
    assert_eq!(old.as_deref(), Some("default")); // old value was the default
    assert_eq!(new.as_deref(), Some("new_value"));
}

#[test]
fn config_store_notifies_on_remove() {
    let schema = ConfigSchema::new()
        .with_property(ConfigPropertyDefinition::new("key", "Key").with_default("default"));

    let mut store = ConfigStore::new(schema);

    let received = Arc::new(Mutex::new(None::<(String, Option<String>, Option<String>)>));
    let r = received.clone();
    store.on_changed(move |key, old, new| {
        *r.lock().unwrap() = Some((
            key.to_string(),
            old.map(|s| s.to_string()),
            new.map(|s| s.to_string()),
        ));
    });

    store.remove("key");

    let received = received.lock().unwrap();
    let (key, old, new) = received.as_ref().unwrap();
    assert_eq!(key, "key");
    assert_eq!(old.as_deref(), Some("default"));
    assert_eq!(*new, None); // removed
}

#[test]
fn config_store_no_notification_without_callback() {
    let schema = ConfigSchema::new()
        .with_property(ConfigPropertyDefinition::new("key", "Key").with_default("default"));

    let mut store = ConfigStore::new(schema);
    // No callback set — should not crash
    store.set("key", "new").unwrap();
    store.remove("key");
}

#[test]
fn config_store_notifies_on_save() {
    let schema = ConfigSchema::new()
        .with_property(ConfigPropertyDefinition::new("a", "A").with_default("1"))
        .with_property(ConfigPropertyDefinition::new("b", "B").with_default("2"));

    let mut store = ConfigStore::new(schema);

    let changes = Arc::new(Mutex::new(vec![]));
    let c = changes.clone();
    store.on_changed(move |key, old, new| {
        c.lock().unwrap().push((
            key.to_string(),
            old.map(|s| s.to_string()),
            new.map(|s| s.to_string()),
        ));
    });

    let mut new_config = Config::new();
    new_config.set("a", "changed");
    new_config.set("b", "2"); // same as default
    new_config.set("c", "new_key");
    store.save(new_config).unwrap();

    let changes = changes.lock().unwrap();
    // "a" changed, "b" didn't change, "c" is new
    assert_eq!(changes.len(), 2);
    assert!(changes.iter().any(|(k, _, _)| k == "a"));
    assert!(changes.iter().any(|(k, _, _)| k == "c"));
}

// --- Retry policy tests ---

#[test]
fn retry_policy_none_no_retry() {
    let policy = RetryPolicy::none();
    assert!(!policy.should_retry());
    assert_eq!(policy.max_attempts(), 1);
    assert_eq!(policy.delay_for_attempt(0), None);
}

#[test]
fn retry_policy_fixed() {
    let policy = RetryPolicy::fixed(3, Duration::from_secs(5));
    assert!(policy.should_retry());
    assert_eq!(policy.max_attempts(), 4); // 3 retries + 1 initial
    assert_eq!(policy.delay_for_attempt(0), Some(Duration::from_secs(5)));
    assert_eq!(policy.delay_for_attempt(2), Some(Duration::from_secs(5)));
    assert_eq!(policy.delay_for_attempt(3), None); // exceeded
}

#[test]
fn retry_policy_exponential() {
    let policy = RetryPolicy::exponential(3, Duration::from_millis(100), 2.0);
    assert!(policy.should_retry());
    assert_eq!(policy.max_attempts(), 4);

    let d0 = policy.delay_for_attempt(0).unwrap();
    assert_eq!(d0, Duration::from_millis(100));

    let d1 = policy.delay_for_attempt(1).unwrap();
    assert_eq!(d1, Duration::from_millis(200));

    let d2 = policy.delay_for_attempt(2).unwrap();
    assert_eq!(d2, Duration::from_millis(400));

    assert_eq!(policy.delay_for_attempt(3), None); // exceeded
}

#[test]
fn retry_policy_display() {
    let policy = RetryPolicy::fixed(2, Duration::from_secs(1));
    assert!(policy.to_string().contains("Fixed"));
}

// --- Integration: retry with scheduler ---

#[test]
fn scheduler_submit_with_retry() {
    let mut scheduler = Scheduler::new();
    let task_id = TaskId::new();

    let _job_id = scheduler.submit_with_retry(
        task_id,
        Schedule::Immediate,
        0,
        RetryPolicy::fixed(3, Duration::from_secs(1)),
    );

    // The initial job is submitted
    assert_eq!(scheduler.queue_len(), 1);
}