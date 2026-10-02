//! Task lifecycle states.
//!
//! ```text
//! Pending → Running → {Completed, Cancelled, Failed}
//!                  ↕
//!               Paused
//! ```

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskState {
    Pending,
    Running,
    Paused,
    Cancelled,
    Completed,
    Failed,
}

impl TaskState {
    pub fn can_start(&self) -> bool {
        matches!(self, TaskState::Pending)
    }

    pub fn can_pause(&self) -> bool {
        matches!(self, TaskState::Running)
    }

    pub fn can_resume(&self) -> bool {
        matches!(self, TaskState::Paused)
    }

    pub fn can_cancel(&self) -> bool {
        matches!(
            self,
            TaskState::Pending | TaskState::Running | TaskState::Paused
        )
    }

    pub fn can_complete(&self) -> bool {
        matches!(self, TaskState::Running)
    }

    pub fn can_fail(&self) -> bool {
        matches!(self, TaskState::Running | TaskState::Paused)
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            TaskState::Completed | TaskState::Cancelled | TaskState::Failed
        )
    }

    pub fn is_active(&self) -> bool {
        matches!(self, TaskState::Running | TaskState::Paused)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            TaskState::Pending => "Pending",
            TaskState::Running => "Running",
            TaskState::Paused => "Paused",
            TaskState::Cancelled => "Cancelled",
            TaskState::Completed => "Completed",
            TaskState::Failed => "Failed",
        }
    }
}

impl fmt::Display for TaskState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}