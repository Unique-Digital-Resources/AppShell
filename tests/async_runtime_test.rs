#![cfg(feature = "async-runtime")]

use app_shell::app_engine::*;
use std::time::Duration;

// --- AsyncRuntime tests ---

#[test]
fn async_runtime_creates() {
    let runtime = AsyncRuntime::new().unwrap();
    // If it creates, we're good
    drop(runtime);
}

#[test]
fn async_runtime_block_on() {
    let runtime = AsyncRuntime::new_current_thread().unwrap();
    let result = runtime.block_on(async { 42 });
    assert_eq!(result, 42);
}

#[test]
fn async_runtime_spawn() {
    let runtime = AsyncRuntime::new().unwrap();
    let handle = runtime.spawn(async { 100 });
    // Block on the join handle
    let result = runtime.block_on(async { handle.await.unwrap() });
    assert_eq!(result, 100);
}

// --- AsyncTaskHandler tests ---

struct AsyncEchoHandler;

impl AsyncTaskHandler for AsyncEchoHandler {
    fn execute(
        &self,
        input: &TaskInput,
        _ctx: &TaskContext,
        _engine: &EngineRef,
    ) -> AsyncTaskFuture {
        let s = input.get::<String>().cloned().unwrap_or_default();
        Box::pin(async move {
            // Simulate async work
            tokio::time::sleep(Duration::from_millis(1)).await;
            Ok(TaskResult::completed_with(s))
        })
    }
}

#[test]
fn async_handler_executes() {
    let engine = Bootstrap::create().unwrap();

    engine.register_async_task(
        TaskDefinition::new("async.echo", "Async Echo", "Echoes asynchronously"),
        Box::new(AsyncEchoHandler),
    ).unwrap();

    let task_id = engine.task_manager().create(
        "async.echo",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new("hello async".to_string()),
    ).unwrap();

    let runtime = AsyncRuntime::new().unwrap();
    let result = engine.start_task_async(task_id, &runtime).unwrap();

    assert!(result.is_completed());
    assert_eq!(result.output::<String>(), Some(&"hello async".to_string()));
    assert_eq!(engine.task_manager().get_state(task_id), Some(TaskState::Completed));
}

#[test]
fn async_handler_can_sleep() {
    struct SleepHandler;
    impl AsyncTaskHandler for SleepHandler {
        fn execute(&self, _input: &TaskInput, _ctx: &TaskContext, _engine: &EngineRef) -> AsyncTaskFuture {
            Box::pin(async move {
                tokio::time::sleep(Duration::from_millis(10)).await;
                Ok(TaskResult::completed())
            })
        }
    }

    let engine = Bootstrap::create().unwrap();
    engine.register_async_task(
        TaskDefinition::new("sleep", "Sleep", "Sleeps then completes"),
        Box::new(SleepHandler),
    ).unwrap();

    let task_id = engine.task_manager().create(
        "sleep",
        TaskContext::new(ExecutionId::new()),
        TaskInput::empty(),
    ).unwrap();

    let runtime = AsyncRuntime::new().unwrap();
    let result = engine.start_task_async(task_id, &runtime).unwrap();
    assert!(result.is_completed());
}

#[test]
fn async_handler_can_fail() {
    struct FailHandler;
    impl AsyncTaskHandler for FailHandler {
        fn execute(&self, _input: &TaskInput, _ctx: &TaskContext, _engine: &EngineRef) -> AsyncTaskFuture {
            Box::pin(async {
                tokio::time::sleep(Duration::from_millis(1)).await;
                Err(TaskError::ExecutionFailed { id: 0, reason: "async failure".to_string() })
            })
        }
    }

    let engine = Bootstrap::create().unwrap();
    engine.register_async_task(
        TaskDefinition::new("fail", "Fail", "Fails asynchronously"),
        Box::new(FailHandler),
    ).unwrap();

    let task_id = engine.task_manager().create(
        "fail",
        TaskContext::new(ExecutionId::new()),
        TaskInput::empty(),
    ).unwrap();

    let runtime = AsyncRuntime::new().unwrap();
    let result = engine.start_task_async(task_id, &runtime);
    assert!(result.is_err());
    assert_eq!(engine.task_manager().get_state(task_id), Some(TaskState::Failed));
}

// --- Mixed sync + async tests ---

struct SyncEchoHandler;
impl TaskHandler for SyncEchoHandler {
    fn execute(&self, input: &TaskInput, _ctx: &TaskContext, _engine: &EngineRef) -> Result<TaskResult, TaskError> {
        let s = input.get::<String>().cloned().unwrap_or_default();
        Ok(TaskResult::completed_with(s))
    }
}

#[test]
fn sync_handler_works_with_async_runtime() {
    let engine = Bootstrap::create().unwrap();
    engine.task_manager().register(
        TaskDefinition::new("sync.echo", "Sync Echo", "Echoes synchronously"),
        Box::new(SyncEchoHandler),
    ).unwrap();

    let task_id = engine.task_manager().create(
        "sync.echo",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new("hello sync".to_string()),
    ).unwrap();

    // start_task_async delegates to start() for sync handlers
    let runtime = AsyncRuntime::new().unwrap();
    let result = engine.start_task_async(task_id, &runtime).unwrap();

    assert!(result.is_completed());
    assert_eq!(result.output::<String>(), Some(&"hello sync".to_string()));
}

#[test]
fn mixed_sync_and_async_handlers() {
    let engine = Bootstrap::create().unwrap();

    // Register sync handler
    engine.task_manager().register(
        TaskDefinition::new("sync", "Sync", "Sync task"),
        Box::new(SyncEchoHandler),
    ).unwrap();

    // Register async handler
    engine.register_async_task(
        TaskDefinition::new("async", "Async", "Async task"),
        Box::new(AsyncEchoHandler),
    ).unwrap();

    // Create both tasks
    let sync_id = engine.task_manager().create(
        "sync",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new("sync_input".to_string()),
    ).unwrap();

    let async_id = engine.task_manager().create(
        "async",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new("async_input".to_string()),
    ).unwrap();

    let runtime = AsyncRuntime::new().unwrap();

    // Execute sync task
    let sync_result = engine.start_task_async(sync_id, &runtime).unwrap();
    assert_eq!(sync_result.output::<String>(), Some(&"sync_input".to_string()));

    // Execute async task
    let async_result = engine.start_task_async(async_id, &runtime).unwrap();
    assert_eq!(async_result.output::<String>(), Some(&"async_input".to_string()));

    assert_eq!(engine.task_manager().get_state(sync_id), Some(TaskState::Completed));
    assert_eq!(engine.task_manager().get_state(async_id), Some(TaskState::Completed));
}

// --- HandlerKind tests ---

#[test]
fn handler_kind_detects_sync() {
    let engine = Bootstrap::create().unwrap();
    engine.task_manager().register(
        TaskDefinition::new("sync", "Sync", "Sync"),
        Box::new(SyncEchoHandler),
    ).unwrap();

    let task_id = engine.task_manager().create(
        "sync",
        TaskContext::new(ExecutionId::new()),
        TaskInput::empty(),
    ).unwrap();

    // Start to verify it works
    let result = engine.start_task(task_id).unwrap();
    assert!(result.is_completed());
}

#[test]
fn handler_kind_detects_async() {
    let engine = Bootstrap::create().unwrap();
    engine.register_async_task(
        TaskDefinition::new("async", "Async", "Async"),
        Box::new(AsyncEchoHandler),
    ).unwrap();

    let task_id = engine.task_manager().create(
        "async",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new("test".to_string()),
    ).unwrap();

    let runtime = AsyncRuntime::new().unwrap();
    let result = engine.start_task_async(task_id, &runtime).unwrap();
    assert!(result.is_completed());
}

// --- AsyncError tests ---

#[test]
fn async_error_display() {
    let e = AsyncError::RuntimeCreationFailed("out of memory".to_string());
    assert!(e.to_string().contains("out of memory"));
}

#[test]
fn async_error_implements_std_error() {
    fn assert_std_error<E: std::error::Error>(_: E) {}
    assert_std_error(AsyncError::RuntimeCreationFailed("test".to_string()));
}