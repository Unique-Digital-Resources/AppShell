//! Foundational App Engine error type.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineError {
    InvalidLifecycleTransition { from: String, to: String },
    AlreadyInitialized,
    NotInitialized,
    AlreadyRunning,
    AlreadyStopped,
    AlreadyDisposed,
    InitializationFailed(String),
    StartupFailed(String),
    ShutdownFailed(String),
    /// A subsystem (service, plugin, etc.) reported a failure.
    SubsystemFailure { subsystem: String, reason: String },
}

impl EngineError {
    pub fn invalid_transition(from: &str, to: &str) -> Self {
        Self::InvalidLifecycleTransition {
            from: from.to_string(),
            to: to.to_string(),
        }
    }

    pub fn subsystem(subsystem: &str, reason: impl Into<String>) -> Self {
        Self::SubsystemFailure {
            subsystem: subsystem.to_string(),
            reason: reason.into(),
        }
    }
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::InvalidLifecycleTransition { from, to } => {
                write!(f, "Invalid lifecycle transition: {} -> {}", from, to)
            }
            EngineError::AlreadyInitialized => write!(f, "Engine is already initialized"),
            EngineError::NotInitialized => write!(f, "Engine is not initialized"),
            EngineError::AlreadyRunning => write!(f, "Engine is already running"),
            EngineError::AlreadyStopped => write!(f, "Engine is already stopped"),
            EngineError::AlreadyDisposed => write!(f, "Engine is already disposed"),
            EngineError::InitializationFailed(msg) => {
                write!(f, "Initialization failed: {}", msg)
            }
            EngineError::StartupFailed(msg) => write!(f, "Startup failed: {}", msg),
            EngineError::ShutdownFailed(msg) => write!(f, "Shutdown failed: {}", msg),
            EngineError::SubsystemFailure { subsystem, reason } => {
                write!(f, "{} subsystem failure: {}", subsystem, reason)
            }
        }
    }
}

impl std::error::Error for EngineError {}

// ----- Error consolidation: From impls for subsystem errors -----

impl From<crate::app_engine::errors::service_error::ServiceError> for EngineError {
    fn from(e: crate::app_engine::errors::service_error::ServiceError) -> Self {
        EngineError::SubsystemFailure {
            subsystem: "service".to_string(),
            reason: e.to_string(),
        }
    }
}

impl From<crate::app_engine::errors::plugin_error::PluginError> for EngineError {
    fn from(e: crate::app_engine::errors::plugin_error::PluginError) -> Self {
        EngineError::SubsystemFailure {
            subsystem: "plugin".to_string(),
            reason: e.to_string(),
        }
    }
}

impl From<crate::app_engine::errors::command_error::CommandError> for EngineError {
    fn from(e: crate::app_engine::errors::command_error::CommandError) -> Self {
        EngineError::SubsystemFailure {
            subsystem: "command".to_string(),
            reason: e.to_string(),
        }
    }
}

impl From<crate::app_engine::errors::task_error::TaskError> for EngineError {
    fn from(e: crate::app_engine::errors::task_error::TaskError) -> Self {
        EngineError::SubsystemFailure {
            subsystem: "task".to_string(),
            reason: e.to_string(),
        }
    }
}

impl From<crate::app_engine::errors::resource_error::ResourceError> for EngineError {
    fn from(e: crate::app_engine::errors::resource_error::ResourceError) -> Self {
        EngineError::SubsystemFailure {
            subsystem: "resource".to_string(),
            reason: e.to_string(),
        }
    }
}

impl From<crate::app_engine::errors::configuration_error::ConfigurationError> for EngineError {
    fn from(e: crate::app_engine::errors::configuration_error::ConfigurationError) -> Self {
        EngineError::SubsystemFailure {
            subsystem: "configuration".to_string(),
            reason: e.to_string(),
        }
    }
}

impl From<crate::app_engine::errors::scheduling_error::SchedulingError> for EngineError {
    fn from(e: crate::app_engine::errors::scheduling_error::SchedulingError) -> Self {
        EngineError::SubsystemFailure {
            subsystem: "scheduling".to_string(),
            reason: e.to_string(),
        }
    }
}

impl From<crate::app_engine::errors::history_error::HistoryError> for EngineError {
    fn from(e: crate::app_engine::errors::history_error::HistoryError) -> Self {
        EngineError::SubsystemFailure {
            subsystem: "history".to_string(),
            reason: e.to_string(),
        }
    }
}

impl From<crate::app_engine::errors::communication_error::CommunicationError> for EngineError {
    fn from(e: crate::app_engine::errors::communication_error::CommunicationError) -> Self {
        EngineError::SubsystemFailure {
            subsystem: "communication".to_string(),
            reason: e.to_string(),
        }
    }
}

impl From<crate::app_engine::errors::context_error::ContextError> for EngineError {
    fn from(e: crate::app_engine::errors::context_error::ContextError) -> Self {
        EngineError::SubsystemFailure {
            subsystem: "context".to_string(),
            reason: e.to_string(),
        }
    }
}