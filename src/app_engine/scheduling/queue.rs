//! `JobQueue` — stores jobs that are ready for execution.
//!
//! For Phase 5, this is a priority-ordered queue: higher priority first,
//! FIFO within the same priority level.

use crate::app_engine::scheduling::job::{Job, JobId};

pub struct JobQueue {
    jobs: Vec<Job>,
}

impl JobQueue {
    pub fn new() -> Self {
        Self { jobs: Vec::new() }
    }

    pub fn enqueue(&mut self, job: Job) {
        self.jobs.push(job);
        // Sort: higher priority first, then earlier creation (FIFO).
        self.jobs.sort_by(|a, b| {
            b.priority()
                .cmp(&a.priority())
                .then(a.created_at().cmp(&b.created_at()))
        });
    }

    pub fn dequeue(&mut self) -> Option<Job> {
        if self.jobs.is_empty() {
            None
        } else {
            Some(self.jobs.remove(0))
        }
    }

    pub fn peek(&self) -> Option<&Job> {
        self.jobs.first()
    }

    pub fn remove(&mut self, id: JobId) -> Option<Job> {
        self.jobs
            .iter()
            .position(|j| j.id() == id)
            .map(|i| self.jobs.remove(i))
    }

    pub fn contains(&self, id: JobId) -> bool {
        self.jobs.iter().any(|j| j.id() == id)
    }

    pub fn len(&self) -> usize {
        self.jobs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.jobs.is_empty()
    }

    pub fn clear(&mut self) {
        self.jobs.clear();
    }

    pub fn jobs(&self) -> &[Job] {
        &self.jobs
    }
}

impl Default for JobQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for JobQueue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JobQueue")
            .field("len", &self.jobs.len())
            .finish()
    }
}