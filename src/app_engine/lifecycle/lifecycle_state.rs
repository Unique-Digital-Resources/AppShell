//! Generic lifecycle state used by Runtime, Tasks, Plugins, etc.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LifecycleState {
    Created,
    Initializing,
    Initialized,
    Starting,
    Running,
    Stopping,
    Stopped,
    Disposing,
    Disposed,
}

impl LifecycleState {
    /// Whether `initialize()` can be called from this state.
    pub fn can_initialize(&self) -> bool {
        matches!(self, LifecycleState::Created)
    }

    /// Whether `start()` can be called from this state.
    pub fn can_start(&self) -> bool {
        matches!(self, LifecycleState::Initialized)
    }

    /// Whether `run()` (an operation while Running) can be called from this state.
    pub fn can_run(&self) -> bool {
        matches!(self, LifecycleState::Running)
    }

    /// Whether `stop()` can be called from this state.
    pub fn can_stop(&self) -> bool {
        matches!(self, LifecycleState::Running)
    }

    /// Whether `dispose()` can be called from this state.
    pub fn can_dispose(&self) -> bool {
        matches!(self, LifecycleState::Stopped)
    }

    /// Terminal state — no further transitions allowed.
    pub fn is_terminal(&self) -> bool {
        matches!(self, LifecycleState::Disposed)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            LifecycleState::Created => "Created",
            LifecycleState::Initializing => "Initializing",
            LifecycleState::Initialized => "Initialized",
            LifecycleState::Starting => "Starting",
            LifecycleState::Running => "Running",
            LifecycleState::Stopping => "Stopping",
            LifecycleState::Stopped => "Stopped",
            LifecycleState::Disposing => "Disposing",
            LifecycleState::Disposed => "Disposed",
        }
    }
}

impl fmt::Display for LifecycleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}