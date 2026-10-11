//! Test DomainEngine with the full AppEngine lifecycle.

use app_shell::app_engine::{AppEngine, Lifecycle, LifecycleState};

#[test]
fn app_engine_lifecycle_works() {
    let mut engine = AppEngine::new().unwrap();
    assert_eq!(engine.state(), LifecycleState::Created);

    engine.initialize().unwrap();
    assert_eq!(engine.state(), LifecycleState::Initialized);

    engine.start().unwrap();
    assert_eq!(engine.state(), LifecycleState::Running);

    // Verify subsystems are accessible while running.
    assert!(engine.history_store().is_empty());

    engine.stop().unwrap();
    assert_eq!(engine.state(), LifecycleState::Stopped);

    engine.dispose().unwrap();
    assert_eq!(engine.state(), LifecycleState::Disposed);
}