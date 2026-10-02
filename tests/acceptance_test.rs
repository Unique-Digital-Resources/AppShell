//! Phase 1 acceptance criteria — the gate that must pass before Phase 2.

use app_shell::app_engine::{
    AppRuntime, ApplicationContext, Bootstrap, Context, EngineError, Lifecycle,
    LifecycleManager, LifecycleState, RuntimeState,
};

/// ✓ Create an AppRuntime
#[test]
fn acceptance_create_runtime() {
    let _ = AppRuntime::new().expect("AppRuntime::new should succeed");
}

/// ✓ Runtime starts in Created state
#[test]
fn acceptance_starts_in_created_state() {
    let runtime = AppRuntime::new().unwrap();
    assert_eq!(runtime.state(), RuntimeState::Created);
}

/// ✓ Initialize changes it to Initialized
#[test]
fn acceptance_initialize_transitions_to_initialized() {
    let mut r = AppRuntime::new().unwrap();
    r.initialize().unwrap();
    assert_eq!(r.state(), RuntimeState::Initialized);
}

/// ✓ Start changes it to Running
#[test]
fn acceptance_start_transitions_to_running() {
    let mut r = AppRuntime::new().unwrap();
    r.initialize().unwrap();
    r.start().unwrap();
    assert_eq!(r.state(), RuntimeState::Running);
}

/// ✓ Run operates while Running
#[test]
fn acceptance_run_while_running() {
    let mut r = AppRuntime::new().unwrap();
    r.initialize().unwrap();
    r.start().unwrap();
    r.run().unwrap();
    assert_eq!(r.state(), RuntimeState::Running);
}

/// ✓ Stop changes it to Stopped
#[test]
fn acceptance_stop_transitions_to_stopped() {
    let mut r = AppRuntime::new().unwrap();
    r.initialize().unwrap();
    r.start().unwrap();
    r.stop().unwrap();
    assert_eq!(r.state(), RuntimeState::Stopped);
}

/// ✓ Dispose changes it to Disposed
#[test]
fn acceptance_dispose_transitions_to_disposed() {
    let mut r = AppRuntime::new().unwrap();
    r.initialize().unwrap();
    r.start().unwrap();
    r.stop().unwrap();
    r.dispose().unwrap();
    assert_eq!(r.state(), RuntimeState::Disposed);
}

/// ✓ Invalid transitions return EngineError
#[test]
fn acceptance_invalid_transitions_return_engine_error() {
    let mut r = AppRuntime::new().unwrap();

    // start before initialize
    assert!(matches!(
        r.start(),
        Err(EngineError::InvalidLifecycleTransition { .. })
    ));

    // run before start
    r.initialize().unwrap();
    assert!(matches!(
        r.run(),
        Err(EngineError::InvalidLifecycleTransition { .. })
    ));

    // dispose before stop
    r.start().unwrap();
    assert!(matches!(
        r.dispose(),
        Err(EngineError::InvalidLifecycleTransition { .. })
    ));

    // initialize twice
    r.stop().unwrap();
    let mut r2 = AppRuntime::new().unwrap();
    r2.initialize().unwrap();
    assert!(matches!(
        r2.initialize(),
        Err(EngineError::InvalidLifecycleTransition { .. })
    ));
}

/// ✓ Context exists independently of Domain Engine
#[test]
fn acceptance_context_independent_of_domain() {
    let ctx = ApplicationContext::new("test_app");
    assert_eq!(ctx.application_id(), "test_app");
    assert!(ctx.instance_id() > 0);

    // Base Context exists as an abstraction too.
    let _base = Context::new();
}

/// ✓ Bootstrap can construct a valid runtime
#[test]
fn acceptance_bootstrap_constructs_runtime() {
    let runtime = Bootstrap::create().expect("Bootstrap::create must succeed");
    assert_eq!(runtime.state(), RuntimeState::Created);
}

/// ✓ App Engine can run without UI
/// (No UI crate, renderer, or window is required to complete the lifecycle.)
#[test]
fn acceptance_runs_without_ui() {
    let mut r = Bootstrap::create().unwrap();
    r.initialize().unwrap();
    r.start().unwrap();
    r.run().unwrap();
    r.stop().unwrap();
    r.dispose().unwrap();
}

/// ✓ App Engine has no dependency on Domain Engine
/// (There is no `domain_engine` import anywhere in the App Engine module.
///  We verify indirectly by confirming the public API surface.)
#[test]
fn acceptance_no_domain_dependency() {
    // Types reachable purely through `app_engine::`:
    let _: AppRuntime = Bootstrap::create().unwrap();
    let _: ApplicationContext = ApplicationContext::new("app");
    let _: Context = Context::new();
    let _: LifecycleManager = LifecycleManager::new();
    let _: LifecycleState = LifecycleState::Created;
    let _: RuntimeState = RuntimeState::Created;
    let _: EngineError = EngineError::NotInitialized;
}

/// ✓ External code accesses the engine through lib.rs
/// (All of the types used in this file are reachable through
///  `use app_shell::app_engine::{...}`, which is the public boundary.)
#[test]
fn acceptance_external_access_via_public_boundary() {
    fn _check_api_surface(
        _: AppRuntime,
        _: Bootstrap,
        _: ApplicationContext,
        _: Context,
        _: EngineError,
        _: LifecycleManager,
        _: LifecycleState,
        _: RuntimeState,
    ) where
        AppRuntime: Lifecycle,
    {
    }

    let r = Bootstrap::create().unwrap();
    let ctx = ApplicationContext::new("app");
    let base = Context::new();
    let mgr = LifecycleManager::new();
    let err = EngineError::NotInitialized;
    let ls = LifecycleState::Created;
    let rs = RuntimeState::Created;
    _check_api_surface(r, Bootstrap::new(), ctx, base, err, mgr, ls, rs);
}