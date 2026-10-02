use app_shell::app_engine::*;
use std::time::Duration;

// --- Job with retry policy tests ---

#[test]
fn job_new_with_retry_stores_policy() {
    let job = Job::new_with_retry(
        TaskId::new(),
        Schedule::Immediate,
        0,
        RetryPolicy::fixed(3, Duration::from_secs(1)),
    );
    assert!(job.retry_policy().should_retry());
    assert_eq!(job.retry_count(), 0);
}

#[test]
fn job_new_without_retry_has_none_policy() {
    let job = Job::new(
        TaskId::new(),
        Schedule::Immediate,
        Trigger::Manual,
        0,
    );
    assert!(!job.retry_policy().should_retry());
    assert_eq!(job.retry_count(), 0);
}

// --- Scheduler::submit_with_retry tests ---

#[test]
fn submit_with_retry_stores_policy_on_job() {
    let scheduler = Scheduler::new();
    let task_id = TaskId::new();

    let job_id = scheduler.submit_with_retry(
        task_id,
        Schedule::Immediate,
        0,
        RetryPolicy::fixed(3, Duration::from_secs(1)),
    );

    assert_eq!(scheduler.queue_len(), 1);

    let job = scheduler.get_job(job_id).unwrap();
    assert!(job.retry_policy().should_retry());
    assert_eq!(job.retry_count(), 0);
}

#[test]
fn submit_with_retry_none_policy_works() {
    let scheduler = Scheduler::new();
    let task_id = TaskId::new();

    let job_id = scheduler.submit_with_retry(
        task_id,
        Schedule::Immediate,
        0,
        RetryPolicy::None,
    );

    let job = scheduler.get_job(job_id).unwrap();
    assert!(!job.retry_policy().should_retry());
}

#[test]
fn submit_with_retry_delayed_goes_to_scheduled() {
    let scheduler = Scheduler::new();
    let task_id = TaskId::new();

    let job_id = scheduler.submit_with_retry(
        task_id,
        Schedule::Delayed(Duration::from_secs(60)),
        0,
        RetryPolicy::fixed(2, Duration::from_secs(1)),
    );

    assert_eq!(scheduler.queue_len(), 0);
    assert_eq!(scheduler.scheduled_count(), 1);

    let job = scheduler.get_job(job_id).unwrap();
    assert!(job.retry_policy().should_retry());
}

// --- Scheduler::handle_task_failure tests ---

#[test]
fn handle_task_failure_creates_retry_job() {
    let scheduler = Scheduler::new();
    let task_id = TaskId::new();

    let job_id = scheduler.submit_with_retry(
        task_id,
        Schedule::Immediate,
        0,
        RetryPolicy::fixed(3, Duration::from_millis(1)),
    );

    let job = scheduler.dispatch().unwrap();
    assert_eq!(job.id(), job_id);

    let retry_job_id = scheduler.handle_task_failure(&job);
    assert!(retry_job_id.is_some());

    assert_eq!(scheduler.scheduled_count(), 1);
    assert_ne!(retry_job_id.unwrap(), job_id);
}

#[test]
fn handle_task_failure_with_none_policy_returns_none() {
    let scheduler = Scheduler::new();
    let task_id = TaskId::new();

    let job_id = scheduler.submit_with_retry(
        task_id,
        Schedule::Immediate,
        0,
        RetryPolicy::None,
    );

    let job = scheduler.dispatch().unwrap();
    assert_eq!(job.id(), job_id);
    let result = scheduler.handle_task_failure(&job);
    assert!(result.is_none());
}

#[test]
fn handle_task_failure_respects_max_attempts() {
    let scheduler = Scheduler::new();
    let task_id = TaskId::new();

    let policy = RetryPolicy::fixed(1, Duration::from_millis(1));
    let job_id = scheduler.submit_with_retry(
        task_id,
        Schedule::Immediate,
        0,
        policy,
    );

    let job = scheduler.dispatch().unwrap();
    assert_eq!(job.id(), job_id);

    let retry1 = scheduler.handle_task_failure(&job);
    assert!(retry1.is_some());

    // The retry job is created via submit() with default RetryPolicy::None
    // So a second failure on the retry won't retry again
    std::thread::sleep(Duration::from_millis(5));
    scheduler.tick();
    let retry_job = scheduler.dispatch().unwrap();
    let retry2 = scheduler.handle_task_failure(&retry_job);
    assert!(retry2.is_none()); // None policy on retry job
}

#[test]
fn handle_task_failure_exponential_delay() {
    let scheduler = Scheduler::new();
    let task_id = TaskId::new();

    let policy = RetryPolicy::exponential(5, Duration::from_millis(100), 2.0);
    let job_id = scheduler.submit_with_retry(
        task_id,
        Schedule::Immediate,
        0,
        policy,
    );

    let job = scheduler.dispatch().unwrap();
    assert_eq!(job.id(), job_id);

    let retry_job_id = scheduler.handle_task_failure(&job);
    assert!(retry_job_id.is_some());

    assert_eq!(scheduler.scheduled_count(), 1);

    std::thread::sleep(Duration::from_millis(150));

    scheduler.tick();
    assert_eq!(scheduler.queue_len(), 1);
}

#[test]
fn handle_task_failure_fixed_delay() {
    let scheduler = Scheduler::new();
    let task_id = TaskId::new();

    let policy = RetryPolicy::fixed(3, Duration::from_millis(10));
    let job_id = scheduler.submit_with_retry(
        task_id,
        Schedule::Immediate,
        0,
        policy,
    );

    let job = scheduler.dispatch().unwrap();
    assert_eq!(job.id(), job_id);

    let retry_job_id = scheduler.handle_task_failure(&job);
    assert!(retry_job_id.is_some());

    std::thread::sleep(Duration::from_millis(20));

    scheduler.tick();
    assert_eq!(scheduler.queue_len(), 1);
}

// --- Integration: full retry flow ---

#[test]
fn full_retry_flow() {
    let scheduler = Scheduler::new();
    let task_id = TaskId::new();

    let job_id = scheduler.submit_with_retry(
        task_id,
        Schedule::Immediate,
        0,
        RetryPolicy::fixed(3, Duration::from_millis(1)),
    );

    let job = scheduler.get_job(job_id).unwrap();
    assert!(job.retry_policy().should_retry());

    let dispatched = scheduler.dispatch().unwrap();
    assert_eq!(dispatched.id(), job_id);

    let retry_id = scheduler.handle_task_failure(&dispatched);
    assert!(retry_id.is_some());
    assert_ne!(retry_id.unwrap(), job_id);

    assert_eq!(scheduler.scheduled_count(), 1);

    std::thread::sleep(Duration::from_millis(5));
    scheduler.tick();
    assert_eq!(scheduler.queue_len(), 1);
}

// --- Retry job properties ---

#[test]
fn job_retried_has_new_id() {
    let scheduler = Scheduler::new();
    let task_id = TaskId::new();

    let job_id = scheduler.submit_with_retry(
        task_id,
        Schedule::Immediate,
        0,
        RetryPolicy::fixed(2, Duration::from_millis(1)),
    );

    let job = scheduler.dispatch().unwrap();
    let retry = scheduler.handle_task_failure(&job).unwrap();
    assert_ne!(job.id(), retry);

    let retry_job = scheduler.get_job(retry).unwrap();
    assert!(!retry_job.retry_policy().should_retry());
}

#[test]
fn retry_priority_preserved() {
    let scheduler = Scheduler::new();
    let task_id = TaskId::new();

    let job_id = scheduler.submit_with_retry(
        task_id,
        Schedule::Immediate,
        7,
        RetryPolicy::fixed(2, Duration::from_millis(1)),
    );

    let original_job = scheduler.get_job(job_id).unwrap();
    assert_eq!(original_job.priority(), 7);

    let job = scheduler.dispatch().unwrap();
    let retry_id = scheduler.handle_task_failure(&job).unwrap();
    let retry_job = scheduler.get_job(retry_id).unwrap();
    assert_eq!(retry_job.priority(), 7);
}