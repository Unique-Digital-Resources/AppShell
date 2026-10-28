//! `Scheduler` — the main coordinator of scheduled execution.
//!
//! ```text
//! Task + Schedule + Trigger
//!         ↓
//!     Scheduler
//!         ↓
//!       Job
//!         ↓
//!      Queue
//!         ↓
//!     (handed to TaskExecutor by the caller)
//! ```
//!
//! The scheduler does **not** execute tasks. It determines *when* jobs become
//! eligible and hands them off via `dispatch()`.

//! `Scheduler` — the main coordinator of scheduled execution.
//!
//! Phase 23: now thread-safe using Mutex/RwLock. All methods take &self.

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, RwLock};
use std::time::Instant;

use crate::app_engine::errors::scheduling_error::SchedulingError;
use crate::app_engine::scheduling::job::{Job, JobId, JobState};
use crate::app_engine::scheduling::queue::JobQueue;
use crate::app_engine::scheduling::schedule::Schedule;
use crate::app_engine::scheduling::trigger::Trigger;
use crate::app_engine::tasks::task::TaskId;
use crate::app_engine::scheduling::retry_policy::RetryPolicy;

pub struct Scheduler {
    scheduled: Mutex<HashMap<JobId, Job>>,
    queue: Mutex<JobQueue>,
    satisfied_triggers: RwLock<HashSet<String>>,
    paused: RwLock<bool>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            scheduled: Mutex::new(HashMap::new()),
            queue: Mutex::new(JobQueue::new()),
            satisfied_triggers: RwLock::new(HashSet::new()),
            paused: RwLock::new(false),
        }
    }

    pub fn submit(
        &self,
        task_id: TaskId,
        schedule: Schedule,
        priority: i32,
    ) -> JobId {
        self.submit_with_trigger(task_id, schedule, Trigger::Manual, priority)
    }

    pub fn submit_with_trigger(
        &self,
        task_id: TaskId,
        schedule: Schedule,
        trigger: Trigger,
        priority: i32,
    ) -> JobId {
        let job = Job::new(task_id, schedule, trigger, priority);
        let id = job.id();

        let now = Instant::now();
        let schedule_eligible = job.is_schedule_eligible(now);
        let trigger_satisfied = self.is_trigger_satisfied(job.trigger());

        if schedule_eligible && trigger_satisfied {
            let mut job = job;
            job.set_state(JobState::Queued);
            self.queue.lock().unwrap().enqueue(job);
        } else {
            self.scheduled.lock().unwrap().insert(id, job);
        }

        id
    }

    pub fn satisfy_trigger(&self, name: &str) {
        self.satisfied_triggers.write().unwrap().insert(name.to_string());
    }

    pub fn unsatisfy_trigger(&self, name: &str) {
        self.satisfied_triggers.write().unwrap().remove(name);
    }

    pub fn is_trigger_satisfied(&self, trigger: &Trigger) -> bool {
        match trigger {
            Trigger::Manual => true,
            Trigger::OnEvent(n)
            | Trigger::OnSignal(n)
            | Trigger::OnCondition(n) => {
                self.satisfied_triggers.read().unwrap().contains(n)
            }
        }
    }

    pub fn tick(&self) {
        if *self.paused.read().unwrap() {
            return;
        }

        let now = Instant::now();
        let eligible_ids: Vec<JobId> = {
            let scheduled = self.scheduled.lock().unwrap();
            scheduled.iter()
                .filter(|(_, job)| {
                    job.is_schedule_eligible(now) && self.is_trigger_satisfied(job.trigger())
                })
                .map(|(&id, _)| id)
                .collect()
        };

        for id in eligible_ids {
            if let Some(mut job) = self.scheduled.lock().unwrap().remove(&id) {
                job.set_state(JobState::Queued);
                self.queue.lock().unwrap().enqueue(job);
            }
        }
    }

    pub fn dispatch(&self) -> Option<Job> {
        if let Some(mut job) = self.queue.lock().unwrap().dequeue() {
            job.set_state(JobState::Dispatched);
            Some(job)
        } else {
            None
        }
    }

    pub fn cancel(&self, job_id: JobId) -> Result<(), SchedulingError> {
        if let Some(mut job) = self.scheduled.lock().unwrap().remove(&job_id) {
            job.set_state(JobState::Cancelled);
            return Ok(());
        }

        if let Some(mut job) = self.queue.lock().unwrap().remove(job_id) {
            job.set_state(JobState::Cancelled);
            return Ok(());
        }

        Err(SchedulingError::JobNotFound { id: job_id.value() })
    }

    pub fn pause(&self) {
        *self.paused.write().unwrap() = true;
    }

    pub fn resume(&self) {
        *self.paused.write().unwrap() = false;
    }

    pub fn is_paused(&self) -> bool {
        *self.paused.read().unwrap()
    }

    pub fn get_job(&self, id: JobId) -> Option<Job> {
        // Can't return &Job through Mutex — clone if Job: Clone
        // Job contains Schedule (Clone), Trigger (Clone), Instant (Copy)
        // So Job should be Clone... let me check.
        // Actually, Job derives Clone in job.rs. So we can clone.
        self.scheduled.lock().unwrap().get(&id).cloned()
            .or_else(|| {
                self.queue.lock().unwrap().jobs().iter().find(|j| j.id() == id).cloned()
            })
    }

    pub fn list_jobs(&self) -> Vec<Job> {
        let scheduled = self.scheduled.lock().unwrap();
        let queue = self.queue.lock().unwrap();
        scheduled.values().cloned()
            .chain(queue.jobs().iter().cloned())
            .collect()
    }

    pub fn list_scheduled(&self) -> Vec<Job> {
        self.scheduled.lock().unwrap().values().cloned().collect()
    }

    pub fn list_queued(&self) -> Vec<Job> {
        self.queue.lock().unwrap().jobs().to_vec()
    }

    pub fn queue_len(&self) -> usize {
        self.queue.lock().unwrap().len()
    }

    pub fn scheduled_count(&self) -> usize {
        self.scheduled.lock().unwrap().len()
    }

    pub fn total_job_count(&self) -> usize {
        self.scheduled.lock().unwrap().len() + self.queue.lock().unwrap().len()
    }

    pub fn queue(&self) -> std::sync::MutexGuard<'_, JobQueue> {
        self.queue.lock().unwrap()
    }

    // ----- Retry support (from Phase 19/27) -----

    /// Submit a task with a retry policy.
    /// The policy is stored on the job and checked by `handle_task_failure()`.
    pub fn submit_with_retry(
        &self,
        task_id: TaskId,
        schedule: Schedule,
        priority: i32,
        retry_policy: RetryPolicy,
    ) -> JobId {
        let job = Job::new_with_retry(task_id, schedule, priority, retry_policy);
        let id = job.id();

        let now = Instant::now();
        let schedule_eligible = job.is_schedule_eligible(now);
        let trigger_satisfied = self.is_trigger_satisfied(job.trigger());

        if schedule_eligible && trigger_satisfied {
            let mut job = job;
            job.set_state(JobState::Queued);
            self.queue.lock().unwrap().enqueue(job);
        } else {
            self.scheduled.lock().unwrap().insert(id, job);
        }

        id
    }

    /// Called when a dispatched task fails — checks retry policy and
    /// creates a new delayed job if retries remain.
    /// Takes the dispatched Job directly (it's no longer in the scheduler).
    pub fn handle_task_failure(
        &self,
        job: &Job,
    ) -> Option<JobId> {
        if !job.retry_policy().should_retry() {
            return None;
        }

        let next_attempt = job.retry_count();

        if let Some(delay) = job.retry_policy().delay_for_attempt(next_attempt) {
            let new_job_id = self.submit(
                job.task_id(),
                Schedule::Delayed(delay),
                job.priority(),
            );
            Some(new_job_id)
        } else {
            None
        }
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Scheduler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scheduler")
            .field("scheduled", &self.scheduled.lock().unwrap().len())
            .field("queued", &self.queue.lock().unwrap().len())
            .field("paused", &*self.paused.read().unwrap())
            .field("satisfied_triggers", &self.satisfied_triggers.read().unwrap().len())
            .finish()
    }
}