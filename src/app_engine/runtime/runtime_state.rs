//! Runtime state — currently an alias of the generic `LifecycleState`.
//!
//! Kept as a distinct name so that, in the future, runtime-specific states
//! (e.g. `Paused`, `Suspended`) can be added without disturbing other
//! consumers of `LifecycleState`.

pub type RuntimeState = crate::app_engine::lifecycle::lifecycle_state::LifecycleState;