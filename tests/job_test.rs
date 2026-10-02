use app_shell::app_engine::{
    Job, JobId, JobState, Schedule, TaskId, Trigger,
};
use std::time::Duration;

#[test]
fn job_id_is_unique() {
    let a = JobId::new();
    let b = JobId::new();
    assert_ne!(a, b);
}

#[test]
fn job_starts_in_scheduled_state() {
    let job = Job::new(TaskId::new(), Schedule::Immediate, Trigger::Manual, 0);
    assert_eq!(job.state(), JobState::Scheduled);
    assert_eq!(job.priority(), 0);
    assert!(job.last_executed_at().is_none());
}

#[test]
fn job_stores_task_id_schedule_trigger() {
    let task_id = TaskId::new();
    let job = Job::new(
        task_id,
        Schedule::Delayed(Duration::from_secs(5)),
        Trigger::OnEvent("ready".to_string()),
        10,
    );
    assert_eq!(job.task_id(), task_id);
    assert!(matches!(job.schedule(), Schedule::Delayed(_)));
    assert_eq!(job.trigger(), &Trigger::OnEvent("ready".to_string()));
    assert_eq!(job.priority(), 10);
}

#[test]
fn job_is_recurring_when_interval() {
    let job = Job::new(
        TaskId::new(),
        Schedule::Interval(Duration::from_secs(60)),
        Trigger::Manual,
        0,
    );
    assert!(job.is_recurring());
}

#[test]
fn job_schedule_eligible_immediate() {
    let job = Job::new(TaskId::new(), Schedule::Immediate, Trigger::Manual, 0);
    assert!(job.is_schedule_eligible(std::time::Instant::now()));
}

#[test]
fn job_state_is_terminal() {
    assert!(JobState::Completed.is_terminal());
    assert!(JobState::Cancelled.is_terminal());
    assert!(JobState::Expired.is_terminal());
    assert!(!JobState::Scheduled.is_terminal());
    assert!(!JobState::Queued.is_terminal());
}