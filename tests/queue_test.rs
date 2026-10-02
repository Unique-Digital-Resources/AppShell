use app_shell::app_engine::{Job, JobQueue, Schedule, TaskId, Trigger};

fn make_job(priority: i32) -> Job {
    Job::new(TaskId::new(), Schedule::Immediate, Trigger::Manual, priority)
}

#[test]
fn new_queue_is_empty() {
    let q = JobQueue::new();
    assert!(q.is_empty());
    assert_eq!(q.len(), 0);
}

#[test]
fn enqueue_dequeue_single() {
    let mut q = JobQueue::new();
    let job = make_job(5);
    let id = job.id();
    q.enqueue(job);
    assert_eq!(q.len(), 1);
    let dequeued = q.dequeue().unwrap();
    assert_eq!(dequeued.id(), id);
    assert!(q.is_empty());
}

#[test]
fn dequeue_empty_returns_none() {
    let mut q = JobQueue::new();
    assert!(q.dequeue().is_none());
}

#[test]
fn priority_ordering_high_first() {
    let mut q = JobQueue::new();
    q.enqueue(make_job(1));
    q.enqueue(make_job(10));
    q.enqueue(make_job(5));

    assert_eq!(q.dequeue().unwrap().priority(), 10);
    assert_eq!(q.dequeue().unwrap().priority(), 5);
    assert_eq!(q.dequeue().unwrap().priority(), 1);
}

#[test]
fn peek_does_not_remove() {
    let mut q = JobQueue::new();
    q.enqueue(make_job(10));
    let peeked = q.peek().unwrap();
    let id = peeked.id();
    assert_eq!(q.len(), 1);
    let dequeued = q.dequeue().unwrap();
    assert_eq!(dequeued.id(), id);
}

#[test]
fn remove_by_id() {
    let mut q = JobQueue::new();
    let j1 = make_job(1);
    let j2 = make_job(2);
    let id1 = j1.id();
    q.enqueue(j1);
    q.enqueue(j2);
    assert!(q.contains(id1));
    let removed = q.remove(id1).unwrap();
    assert_eq!(removed.id(), id1);
    assert!(!q.contains(id1));
    assert_eq!(q.len(), 1);
}

#[test]
fn remove_missing_returns_none() {
    let mut q = JobQueue::new();
    let bogus_id = app_shell::app_engine::JobId::new();
    assert!(q.remove(bogus_id).is_none());
}

#[test]
fn clear_removes_all() {
    let mut q = JobQueue::new();
    q.enqueue(make_job(1));
    q.enqueue(make_job(2));
    q.clear();
    assert!(q.is_empty());
}