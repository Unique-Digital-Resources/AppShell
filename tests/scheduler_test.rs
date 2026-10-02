use app_shell::app_engine::{
    JobState, Schedule, Scheduler, SchedulingError, TaskId, Trigger,
};
use std::time::Duration;

#[test]
fn new_scheduler_is_empty() {
    let s = Scheduler::new();
    assert_eq!(s.scheduled_count(), 0);
    assert_eq!(s.queue_len(), 0);
    assert!(!s.is_paused());
}

#[test]
fn submit_immediate_goes_to_queue() {
    let mut s = Scheduler::new();
    let task_id = TaskId::new();
    let _job_id = s.submit(task_id, Schedule::Immediate, 0);
    assert_eq!(s.queue_len(), 1);
    assert_eq!(s.scheduled_count(), 0);
}

#[test]
fn submit_delayed_goes_to_scheduled() {
    let mut s = Scheduler::new();
    let task_id = TaskId::new();
    let _job_id = s.submit(task_id, Schedule::Delayed(Duration::from_secs(60)), 0);
    assert_eq!(s.scheduled_count(), 1);
    assert_eq!(s.queue_len(), 0);
}

#[test]
fn tick_moves_eligible_to_queue() {
    let mut s = Scheduler::new();
    let task_id = TaskId::new();
    let _id = s.submit(task_id, Schedule::Delayed(Duration::from_millis(1)), 0);
    assert_eq!(s.scheduled_count(), 1);

    std::thread::sleep(Duration::from_millis(5));
    s.tick();
    assert_eq!(s.scheduled_count(), 0);
    assert_eq!(s.queue_len(), 1);
}

#[test]
fn tick_skips_when_paused() {
    let mut s = Scheduler::new();
    let _id = s.submit(
        TaskId::new(),
        Schedule::Delayed(Duration::from_millis(1)),
        0,
    );
    s.pause();
    assert!(s.is_paused());

    std::thread::sleep(Duration::from_millis(5));
    s.tick();
    assert_eq!(s.scheduled_count(), 1);
    assert_eq!(s.queue_len(), 0);

    s.resume();
    s.tick();
    assert_eq!(s.scheduled_count(), 0);
    assert_eq!(s.queue_len(), 1);
}

#[test]
fn dispatch_returns_queued_job() {
    let mut s = Scheduler::new();
    let task_id = TaskId::new();
    let job_id = s.submit(task_id, Schedule::Immediate, 5);

    let dispatched = s.dispatch().unwrap();
    assert_eq!(dispatched.id(), job_id);
    assert_eq!(dispatched.task_id(), task_id);
    assert_eq!(dispatched.state(), JobState::Dispatched);
    assert_eq!(s.queue_len(), 0);
}

#[test]
fn dispatch_empty_returns_none() {
    let mut s = Scheduler::new();
    assert!(s.dispatch().is_none());
}

#[test]
fn cancel_scheduled_job() {
    let mut s = Scheduler::new();
    let job_id = s.submit(
        TaskId::new(),
        Schedule::Delayed(Duration::from_secs(60)),
        0,
    );
    s.cancel(job_id).unwrap();
    assert_eq!(s.scheduled_count(), 0);
}

#[test]
fn cancel_queued_job() {
    let mut s = Scheduler::new();
    let job_id = s.submit(TaskId::new(), Schedule::Immediate, 0);
    assert_eq!(s.queue_len(), 1);
    s.cancel(job_id).unwrap();
    assert_eq!(s.queue_len(), 0);
}

#[test]
fn cancel_missing_returns_error() {
    let mut s = Scheduler::new();
    let bogus = app_shell::app_engine::JobId::new();
    let result = s.cancel(bogus);
    assert!(matches!(result, Err(SchedulingError::JobNotFound { .. })));
}

#[test]
fn priority_ordering_in_dispatch() {
    let mut s = Scheduler::new();
    let _low = s.submit(TaskId::new(), Schedule::Immediate, 1);
    let _high = s.submit(TaskId::new(), Schedule::Immediate, 10);
    let _mid = s.submit(TaskId::new(), Schedule::Immediate, 5);

    assert_eq!(s.dispatch().unwrap().priority(), 10);
    assert_eq!(s.dispatch().unwrap().priority(), 5);
    assert_eq!(s.dispatch().unwrap().priority(), 1);
}

#[test]
fn trigger_manual_always_satisfied() {
    let s = Scheduler::new();
    assert!(s.is_trigger_satisfied(&Trigger::Manual));
}

#[test]
fn trigger_event_not_satisfied_until_explicit() {
    let mut s = Scheduler::new();
    let trigger = Trigger::OnEvent("ready".to_string());
    assert!(!s.is_trigger_satisfied(&trigger));

    s.satisfy_trigger("ready");
    assert!(s.is_trigger_satisfied(&trigger));

    s.unsatisfy_trigger("ready");
    assert!(!s.is_trigger_satisfied(&trigger));
}

#[test]
fn triggered_job_becomes_eligible_after_satisfy() {
    let mut s = Scheduler::new();
    let trigger = Trigger::OnEvent("startup".to_string());
    let _job_id = s.submit_with_trigger(
        TaskId::new(),
        Schedule::Immediate,
        trigger,
        0,
    );
    // Not queued because trigger not satisfied
    assert_eq!(s.queue_len(), 0);
    assert_eq!(s.scheduled_count(), 1);

    s.satisfy_trigger("startup");
    s.tick();
    assert_eq!(s.scheduled_count(), 0);
    assert_eq!(s.queue_len(), 1);
}

#[test]
fn get_job_finds_in_scheduled() {
    let mut s = Scheduler::new();
    let job_id = s.submit(
        TaskId::new(),
        Schedule::Delayed(Duration::from_secs(60)),
        0,
    );
    assert!(s.get_job(job_id).is_some());
}

#[test]
fn get_job_finds_in_queue() {
    let mut s = Scheduler::new();
    let job_id = s.submit(TaskId::new(), Schedule::Immediate, 0);
    assert!(s.get_job(job_id).is_some());
}

#[test]
fn list_jobs_combines_scheduled_and_queued() {
    let mut s = Scheduler::new();
    s.submit(TaskId::new(), Schedule::Immediate, 0);
    s.submit(
        TaskId::new(),
        Schedule::Delayed(Duration::from_secs(60)),
        0,
    );
    let jobs = s.list_jobs();
    assert_eq!(jobs.len(), 2);
}