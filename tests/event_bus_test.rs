use app_shell::app_engine::{Event, EventBus, EventHandler, HandlerId};
use std::sync::{Arc, Mutex};

struct CountingHandler {
    count: Arc<Mutex<usize>>,
}

impl EventHandler for CountingHandler {
    fn handle(&self, _event: &Event) {
        *self.count.lock().unwrap() += 1;
    }
}

#[test]
fn publish_dispatches_to_subscribers() {
    let mut bus = EventBus::new();
    let count = Arc::new(Mutex::new(0usize));

    bus.subscribe(
        "task.completed",
        Box::new(CountingHandler {
            count: count.clone(),
        }),
    );

    let event = Event::new("task.completed");
    bus.publish(&event);
    bus.publish(&event);
    bus.publish(&event);

    assert_eq!(*count.lock().unwrap(), 3);
}

#[test]
fn only_matching_handlers_receive_events() {
    let mut bus = EventBus::new();
    let count = Arc::new(Mutex::new(0usize));

    bus.subscribe(
        "task.completed",
        Box::new(CountingHandler {
            count: count.clone(),
        }),
    );
    bus.subscribe(
        "task.failed",
        Box::new(CountingHandler {
            count: count.clone(),
        }),
    );

    bus.publish(&Event::new("task.completed"));
    assert_eq!(*count.lock().unwrap(), 1);

    bus.publish(&Event::new("task.failed"));
    assert_eq!(*count.lock().unwrap(), 2);

    bus.publish(&Event::new("task.started"));
    assert_eq!(*count.lock().unwrap(), 2); // no handlers for "task.started"
}

#[test]
fn unsubscribe_stops_delivery() {
    let mut bus = EventBus::new();
    let count = Arc::new(Mutex::new(0usize));

    let id: HandlerId = bus.subscribe(
        "task.completed",
        Box::new(CountingHandler {
            count: count.clone(),
        }),
    );

    bus.publish(&Event::new("task.completed"));
    assert_eq!(*count.lock().unwrap(), 1);

    assert!(bus.unsubscribe("task.completed", id));
    bus.publish(&Event::new("task.completed"));
    assert_eq!(*count.lock().unwrap(), 1); // unchanged
}

#[test]
fn multiple_handlers_same_type() {
    let mut bus = EventBus::new();
    let count_a = Arc::new(Mutex::new(0usize));
    let count_b = Arc::new(Mutex::new(0usize));

    bus.subscribe(
        "task.completed",
        Box::new(CountingHandler {
            count: count_a.clone(),
        }),
    );
    bus.subscribe(
        "task.completed",
        Box::new(CountingHandler {
            count: count_b.clone(),
        }),
    );

    bus.publish(&Event::new("task.completed"));
    assert_eq!(*count_a.lock().unwrap(), 1);
    assert_eq!(*count_b.lock().unwrap(), 1);
    assert_eq!(bus.handler_count("task.completed"), 2);
}

#[test]
fn event_payload_reaches_handler() {
    struct CapturingHandler {
        captured: Arc<Mutex<Option<i32>>>,
    }
    impl EventHandler for CapturingHandler {
        fn handle(&self, event: &Event) {
            if let Some(v) = event.payload::<i32>() {
                *self.captured.lock().unwrap() = Some(*v);
            }
        }
    }

    let mut bus = EventBus::new();
    let captured = Arc::new(Mutex::new(None));
    bus.subscribe(
        "task.completed",
        Box::new(CapturingHandler {
            captured: captured.clone(),
        }),
    );

    let event = Event::new("task.completed").with_payload(99_i32);
    bus.publish(&event);

    assert_eq!(*captured.lock().unwrap(), Some(99));
}

#[test]
fn producer_unaware_of_consumers() {
    // The producer just creates an Event and publishes it.
    // It has no reference to any handler.
    let mut bus = EventBus::new();
    let count = Arc::new(Mutex::new(0usize));

    // Simulate a "UI" subscribing
    bus.subscribe(
        "task.completed",
        Box::new(CountingHandler {
            count: count.clone(),
        }),
    );

    // Simulate "TaskExecutor" publishing — no reference to the handler
    fn produce_event() -> Event {
        Event::new("task.completed").with_source("TaskExecutor")
    }

    let event = produce_event();
    bus.publish(&event);

    assert_eq!(*count.lock().unwrap(), 1);
}