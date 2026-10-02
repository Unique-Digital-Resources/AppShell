//! `Job` — a scheduled execution request.
//!
//! A `Task` represents the work. A `Job` represents one particular request to
//! schedule that work. For recurring tasks, each occurrence is a separate job.

//! `Job` — a scheduled execution request.

use std::time::Instant;

use crate::app_engine::scheduling::schedule::Schedule;
use crate::app_engine::scheduling::trigger::Trigger;
use crate::app_engine::scheduling::retry_policy::RetryPolicy;
use crate::app_engine::tasks::task::TaskId;
use std::sync::atomic::{AtomicU64, Ordering};

static JOB_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct JobId(u64);

impl JobId {
    pub fn new() -> Self {
        Self(JOB_ID_COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Default for JobId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JobState {
    Scheduled,
    Waiting,
    Queued,
    Dispatched,
    Executing,
    Completed,
    Cancelled,
    Expired,
}

impl JobState {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            JobState::Completed | JobState::Cancelled | JobState::Expired
        )
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            JobState::Scheduled => "Scheduled",
            JobState::Waiting => "Waiting",
            JobState::Queued => "Queued",
            JobState::Dispatched => "Dispatched",
            JobState::Executing => "Executing",
            JobState::Completed => "Completed",
            JobState::Cancelled => "Cancelled",
            JobState::Expired => "Expired",
        }
    }
}

/// A scheduled execution request.
#[derive(Debug, Clone)]
pub struct Job {
    id: JobId,
    task_id: TaskId,
    schedule: Schedule,
    trigger: Trigger,
    priority: i32,
    state: JobState,
    created_at: Instant,
    last_executed_at: Option<Instant>,
    retry_policy: RetryPolicy,
    retry_count: u32,
}

impl Job {
    pub fn new(
        task_id: TaskId,
        schedule: Schedule,
        trigger: Trigger,
        priority: i32,
    ) -> Self {
        Self {
            id: JobId::new(),
            task_id,
            schedule,
            trigger,
            priority,
            state: JobState::Scheduled,
            created_at: Instant::now(),
            last_executed_at: None,
            retry_policy: RetryPolicy::None,
            retry_count: 0,
        }
    }

    /// Create a job with a retry policy.
    pub fn new_with_retry(
        task_id: TaskId,
        schedule: Schedule,
        priority: i32,
        retry_policy: RetryPolicy,
    ) -> Self {
        Self {
            id: JobId::new(),
            task_id,
            schedule,
            trigger: Trigger::Manual,
            priority,
            state: JobState::Scheduled,
            created_at: Instant::now(),
            last_executed_at: None,
            retry_policy,
            retry_count: 0,
        }
    }

    pub fn id(&self) -> JobId { self.id }
    pub fn task_id(&self) -> TaskId { self.task_id }
    pub fn schedule(&self) -> &Schedule { &self.schedule }
    pub fn trigger(&self) -> &Trigger { &self.trigger }
    pub fn priority(&self) -> i32 { self.priority }
    pub fn state(&self) -> JobState { self.state }
    pub fn created_at(&self) -> Instant { self.created_at }
    pub fn last_executed_at(&self) -> Option<Instant> { self.last_executed_at }
    pub fn retry_policy(&self) -> &RetryPolicy { &self.retry_policy }
    pub fn retry_count(&self) -> u32 { self.retry_count }

    pub fn is_schedule_eligible(&self, now: Instant) -> bool {
        self.schedule.is_eligible(self.created_at, now)
    }

    pub fn is_recurring(&self) -> bool {
        self.schedule.is_recurring()
    }

    pub(crate) fn set_state(&mut self, state: JobState) {
        self.state = state;
    }

    pub(crate) fn set_last_executed(&mut self, at: Instant) {
        self.last_executed_at = Some(at);
    }
}