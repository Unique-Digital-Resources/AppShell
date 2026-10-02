//! Phase 6 acceptance criteria.

use app_shell::app_engine::{
    Bootstrap, Command, CommandContext, Event, EventBus, EventHandler, HandlerId, ExecutionId,
    Signal, SignalBus, SubscriptionId,
};
use std::sync::{Arc, Mutex};

// --- Closure wrapper for EventHandler ---

struct ClosureEventHandler<F: Fn(&Event) + Send + Sync + 'static> {
    f: F,
}

impl<F: Fn(&Event) + Send + Sync + 'static> EventHandler for ClosureEventHandler<F> {
    fn handle(&self, event: &Event) {
        (self.f)(event);
    }
}

fn event_handler<F: Fn(&Event) + Send + Sync + 'static>(f: F) -> Box<dyn EventHandler> {
    Box::new(ClosureEventHandler { f })
}

// --- Simple counting handler ---

struct CountingHandler {
    count: Arc<Mutex<usize>>,
}
impl EventHandler for CountingHandler {
    fn handle(&self, _event: &Event) {
        *self.count.lock().unwrap() += 1;
    }
}

/// 1. Represent an event
#[test]
fn acceptance_represent_event() {
    let e = Event::new("task.completed").with_payload("task #42".to_string());
    assert_eq!(e.event_type(), "task.completed");
    assert!(e.has_payload());
}

/// 2. Publish an event
/// 3. Subscribe handlers to events
/// 4. Dispatch events to multiple handlers
#[test]
fn acceptance_publish_and_dispatch() {
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
}

/// 5. Unsubscribe handlers
#[test]
fn acceptance_unsubscribe_handler() {
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

    bus.unsubscribe("task.completed", id);
    bus.publish(&Event::new("task.completed"));
    assert_eq!(*count.lock().unwrap(), 1);
}

/// 6. Represent lightweight signals
#[test]
fn acceptance_represent_signal() {
    let s = Signal::new("task.progress_changed").with_payload(0.75_f32);
    assert_eq!(s.signal_type(), "task.progress_changed");
    assert_eq!(s.payload::<f32>(), Some(&0.75));
}

/// 7. Emit signals
/// 8. Subscribe to signals
#[test]
fn acceptance_emit_and_subscribe_signal() {
    let bus = SignalBus::new();
    let received = Arc::new(Mutex::new(vec![]));

    let r = received.clone();
    bus.subscribe(
        "state.changed",
        Box::new(move |signal: &Signal| {
            r.lock().unwrap().push(signal.signal_type().to_string());
        }),
    );

    bus.emit(&Signal::new("state.changed"));
    bus.emit(&Signal::new("state.changed"));

    assert_eq!(received.lock().unwrap().len(), 2);
}

/// 9. Manage signal subscription lifetime
#[test]
fn acceptance_manage_subscription_lifetime() {
    let bus = SignalBus::new();
    let received = Arc::new(Mutex::new(0usize));

    let r = received.clone();
    let id: SubscriptionId = bus.subscribe(
        "state.changed",
        Box::new(move |_signal: &Signal| {
            *r.lock().unwrap() += 1;
        }),
    );

    bus.emit(&Signal::new("state.changed"));
    assert_eq!(*received.lock().unwrap(), 1);

    assert!(bus.unsubscribe("state.changed", id));
    bus.emit(&Signal::new("state.changed"));
    assert_eq!(*received.lock().unwrap(), 1);
}

/// 10. Allow multiple independent consumers
#[test]
fn acceptance_multiple_independent_consumers() {
    let mut ebus = EventBus::new();
    let sbus = SignalBus::new();

    let e_count = Arc::new(Mutex::new(0usize));
    let s_count = Arc::new(Mutex::new(0usize));

    ebus.subscribe(
        "task.completed",
        Box::new(CountingHandler {
            count: e_count.clone(),
        }),
    );

    let sc = s_count.clone();
    sbus.subscribe(
        "state.changed",
        Box::new(move |_signal: &Signal| {
            *sc.lock().unwrap() += 1;
        }),
    );

    ebus.publish(&Event::new("task.completed"));
    sbus.emit(&Signal::new("state.changed"));

    assert_eq!(*e_count.lock().unwrap(), 1);
    assert_eq!(*s_count.lock().unwrap(), 1);
}

/// 11. Keep producers unaware of consumers
#[test]
fn acceptance_producer_unaware_of_consumers() {
    let mut bus = EventBus::new();
    let received = Arc::new(Mutex::new(false));

    // Consumer subscribes
    let r = received.clone();
    bus.subscribe(
        "task.completed",
        event_handler(move |_event: &Event| {
            *r.lock().unwrap() = true;
        }),
    );

    // Producer publishes without any reference to the consumer
    bus.publish(&Event::new("task.completed"));
    assert!(*received.lock().unwrap());
}

/// 12. Allow Task/Scheduler/State/Plugin systems to publish/consume
#[test]
fn acceptance_systems_can_communicate() {
    let mut runtime = Bootstrap::create().unwrap();
    let received = Arc::new(Mutex::new(vec![]));

    let r = received.clone();
    runtime.event_bus_mut().subscribe(
        "task.completed",
        event_handler(move |event: &Event| {
            r.lock().unwrap().push(event.event_type().to_string());
        }),
    );

    // Simulate a system publishing
    runtime
        .event_bus()
        .publish(&Event::new("task.completed").with_source("TaskExecutor"));
    assert_eq!(received.lock().unwrap().len(), 1);
}

/// 13. Allow Domain Engine / UI to consume App Engine events
#[test]
fn acceptance_external_systems_consume_events() {
    let mut runtime = Bootstrap::create().unwrap();
    let ui_received = Arc::new(Mutex::new(false));
    let domain_received = Arc::new(Mutex::new(false));

    // "UI" subscribes
    let u = ui_received.clone();
    runtime.event_bus_mut().subscribe(
        "task.completed",
        event_handler(move |_event: &Event| {
            *u.lock().unwrap() = true;
        }),
    );

    // "Domain Engine" subscribes
    let d = domain_received.clone();
    runtime.event_bus_mut().subscribe(
        "task.completed",
        event_handler(move |_event: &Event| {
            *d.lock().unwrap() = true;
        }),
    );

    // App Engine publishes
    runtime.event_bus().publish(&Event::new("task.completed"));

    assert!(*ui_received.lock().unwrap());
    assert!(*domain_received.lock().unwrap());
}

/// 14. Keep commands distinct from events
#[test]
fn acceptance_commands_distinct_from_events() {
    // A Command is an intent to do something
    let cmd = Command::with_empty_input(
        "app.quit",
        CommandContext::new(ExecutionId::new()),
    );

    // An Event is a notification that something happened
    let event = Event::new("command.executed").with_payload("app.quit".to_string());

    assert_eq!(cmd.definition_id().as_str(), "app.quit");
    assert_eq!(event.event_type(), "command.executed");
    // The command causes action; the event announces the result.
}

/// 15. Keep direct calls available where a result is required
#[test]
fn acceptance_direct_calls_still_available() {
    let runtime = Bootstrap::create().unwrap();

    // Direct call — need a result
    let _state = runtime.state();
    let _task_count = runtime.task_manager().count();
    let _scheduler_jobs = runtime.scheduler().total_job_count();

    // Event — no result needed, just announce
    runtime.event_bus().publish(&Event::new("runtime.initialized"));
}