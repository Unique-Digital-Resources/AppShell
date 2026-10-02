use app_shell::app_engine::{
    Bootstrap, Event, EventBus, EventHandler, Lifecycle, Signal, SignalBus,
};
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
fn runtime_owns_event_bus() {
    let runtime = Bootstrap::create().unwrap();
    let _: &EventBus = runtime.event_bus();
}

#[test]
fn runtime_owns_signal_bus() {
    let runtime = Bootstrap::create().unwrap();
    let _: &SignalBus = runtime.signal_bus();
}

#[test]
fn runtime_event_bus_starts_empty() {
    let runtime = Bootstrap::create().unwrap();
    assert_eq!(runtime.event_bus().total_handlers(), 0);
}

#[test]
fn runtime_signal_bus_starts_empty() {
    let runtime = Bootstrap::create().unwrap();
    assert_eq!(runtime.signal_bus().total_subscribers(), 0);
}

#[test]
fn runtime_can_publish_and_subscribe_events() {
    let mut runtime = Bootstrap::create().unwrap();
    let count = Arc::new(Mutex::new(0usize));

    runtime.event_bus_mut().subscribe(
        "task.completed",
        Box::new(CountingHandler {
            count: count.clone(),
        }),
    );

    let event = Event::new("task.completed");
    runtime.event_bus().publish(&event);
    runtime.event_bus().publish(&event);

    assert_eq!(*count.lock().unwrap(), 2);
}

#[test]
fn runtime_can_emit_and_subscribe_signals() {
    let runtime = Bootstrap::create().unwrap();
    let received = Arc::new(Mutex::new(0usize));

    let r = received.clone();
    runtime.signal_bus().subscribe(
        "state.changed",
        Box::new(move |_signal: &Signal| {
            *r.lock().unwrap() += 1;
        }),
    );

    let signal = Signal::new("state.changed");
    runtime.signal_bus().emit(&signal);
    runtime.signal_bus().emit(&signal);

    assert_eq!(*received.lock().unwrap(), 2);
}

#[test]
fn communication_independent_of_lifecycle() {
    let mut runtime = Bootstrap::create().unwrap();
    runtime.initialize().unwrap();
    runtime.start().unwrap();
    assert_eq!(runtime.event_bus().total_handlers(), 0);
    assert_eq!(runtime.signal_bus().total_subscribers(), 0);
    runtime.stop().unwrap();
    runtime.dispose().unwrap();
}