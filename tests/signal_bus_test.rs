use app_shell::app_engine::{Signal, SignalBus, SubscriptionId};
use std::sync::{Arc, Mutex};

#[test]
fn emit_delivers_to_subscribers() {
    let bus = SignalBus::new();
    let received = Arc::new(Mutex::new(vec![]));

    let r = received.clone();
    bus.subscribe(
        "task.progress_changed",
        Box::new(move |signal: &Signal| {
            r.lock().unwrap().push(signal.signal_type().to_string());
        }),
    );

    let s = Signal::new("task.progress_changed");
    bus.emit(&s);
    bus.emit(&s);

    let received = received.lock().unwrap();
    assert_eq!(received.len(), 2);
    assert_eq!(received[0], "task.progress_changed");
}

#[test]
fn only_matching_signal_types_delivered() {
    let bus = SignalBus::new();
    let received = Arc::new(Mutex::new(0usize));

    let r = received.clone();
    bus.subscribe(
        "task.progress_changed",
        Box::new(move |_signal: &Signal| {
            *r.lock().unwrap() += 1;
        }),
    );

    bus.emit(&Signal::new("task.progress_changed"));
    bus.emit(&Signal::new("state.changed")); // different type
    bus.emit(&Signal::new("task.progress_changed"));

    assert_eq!(*received.lock().unwrap(), 2);
}

#[test]
fn unsubscribe_stops_delivery() {
    let bus = SignalBus::new();
    let received = Arc::new(Mutex::new(0usize));

    let r = received.clone();
    let id: SubscriptionId = bus.subscribe(
        "task.progress_changed",
        Box::new(move |_signal: &Signal| {
            *r.lock().unwrap() += 1;
        }),
    );

    bus.emit(&Signal::new("task.progress_changed"));
    assert_eq!(*received.lock().unwrap(), 1);

    assert!(bus.unsubscribe("task.progress_changed", id));
    bus.emit(&Signal::new("task.progress_changed"));
    assert_eq!(*received.lock().unwrap(), 1); // unchanged
}

#[test]
fn multiple_subscribers_same_type() {
    let bus = SignalBus::new();
    let a = Arc::new(Mutex::new(0usize));
    let b = Arc::new(Mutex::new(0usize));

    let ra = a.clone();
    bus.subscribe(
        "state.changed",
        Box::new(move |_signal: &Signal| {
            *ra.lock().unwrap() += 1;
        }),
    );
    let rb = b.clone();
    bus.subscribe(
        "state.changed",
        Box::new(move |_signal: &Signal| {
            *rb.lock().unwrap() += 1;
        }),
    );

    bus.emit(&Signal::new("state.changed"));
    assert_eq!(*a.lock().unwrap(), 1);
    assert_eq!(*b.lock().unwrap(), 1);
    assert_eq!(bus.subscriber_count("state.changed"), 2);
}

#[test]
fn signal_payload_reaches_subscriber() {
    let bus = SignalBus::new();
    let captured = Arc::new(Mutex::new(None::<String>));

    let c = captured.clone();
    bus.subscribe(
        "task.progress_changed",
        Box::new(move |signal: &Signal| {
            if let Some(v) = signal.payload::<f32>() {
                *c.lock().unwrap() = Some(format!("{:.0}%", v * 100.0));
            }
        }),
    );

    let s = Signal::new("task.progress_changed").with_payload(0.5_f32);
    bus.emit(&s);

    assert_eq!(*captured.lock().unwrap(), Some("50%".to_string()));
}

#[test]
fn producer_unaware_of_consumers() {
    let bus = SignalBus::new();
    let received = Arc::new(Mutex::new(false));

    // Simulate "UI" subscribing
    let r = received.clone();
    bus.subscribe(
        "task.progress_changed",
        Box::new(move |_signal: &Signal| {
            *r.lock().unwrap() = true;
        }),
    );

    // Simulate "TaskExecutor" emitting — no reference to the subscriber
    let s = Signal::new("task.progress_changed").with_source("TaskExecutor");
    bus.emit(&s);

    assert!(*received.lock().unwrap());
}