use app_shell::app_engine::Schedule;
use std::time::{Duration, Instant};

#[test]
fn immediate_is_always_eligible() {
    let created = Instant::now();
    let now = created;
    assert!(Schedule::Immediate.is_eligible(created, now));
}

#[test]
fn delayed_not_eligible_before_delay() {
    let created = Instant::now();
    let now = created;
    assert!(!Schedule::Delayed(Duration::from_secs(10)).is_eligible(created, now));
}

#[test]
fn delayed_eligible_after_delay() {
    let created = Instant::now();
    let now = created + Duration::from_millis(10);
    assert!(Schedule::Delayed(Duration::from_millis(5)).is_eligible(created, now));
}

#[test]
fn at_time_eligible_after_target() {
    let target = Instant::now() + Duration::from_millis(5);
    let created = Instant::now();
    let now = target + Duration::from_millis(1);
    assert!(Schedule::AtTime(target).is_eligible(created, now));
}

#[test]
fn interval_is_recurring() {
    assert!(Schedule::Interval(Duration::from_secs(60)).is_recurring());
    assert!(!Schedule::Immediate.is_recurring());
}

#[test]
fn interval_next_occurrence() {
    let last = Instant::now();
    let next = Schedule::Interval(Duration::from_secs(60)).next_occurrence(last);
    assert!(next.is_some());
}

#[test]
fn non_recurring_has_no_next_occurrence() {
    let last = Instant::now();
    assert!(Schedule::Immediate.next_occurrence(last).is_none());
    assert!(Schedule::Delayed(Duration::from_secs(1)).next_occurrence(last).is_none());
}