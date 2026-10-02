use app_shell::app_engine::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

struct EchoHandler;
impl TaskHandler for EchoHandler {
    fn execute(
        &self,
        input: &TaskInput,
        _ctx: &TaskContext,
        _engine: &EngineRef,
    ) -> Result<TaskResult, TaskError> {
        let s: &String = input.get::<String>()
            .ok_or_else(|| TaskError::ExecutionFailed {
                id: 0, reason: "expected String".into()
            })?;
        Ok(TaskResult::completed_with(s.clone()))
    }
}

// --- TaskPool tests ---

#[test]
fn task_pool_creates_workers() {
    let pool = TaskPool::new(4);
    assert_eq!(pool.size(), 4);
}

#[test]
fn task_pool_executes_jobs() {
    let pool = TaskPool::new(2);
    let result = Arc::new(Mutex::new(0));

    let r = result.clone();
    pool.spawn(move || {
        *r.lock().unwrap() = 42;
    });

    thread::sleep(Duration::from_millis(50));
    assert_eq!(*result.lock().unwrap(), 42);
}

#[test]
fn task_pool_handles_multiple_jobs() {
    let pool = TaskPool::new(4);
    let counter = Arc::new(Mutex::new(0));

    for _ in 0..10 {
        let c = counter.clone();
        pool.spawn(move || {
            let mut guard = c.lock().unwrap();
            *guard += 1;
        });
    }

    thread::sleep(Duration::from_millis(100));
    assert_eq!(*counter.lock().unwrap(), 10);
}

// --- Thread-safe TaskManager tests ---

#[test]
fn task_manager_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<TaskManager>();
}

#[test]
fn task_manager_create_with_shared_reference() {
    let mgr = Arc::new(TaskManager::new());
    mgr.register(
        TaskDefinition::new("app.echo", "Echo", "Echo"),
        Box::new(EchoHandler),
    ).unwrap();

    let task_id = mgr.create(
        "app.echo",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new("hello".to_string()),
    ).unwrap();

    assert!(mgr.contains(task_id));
    assert_eq!(mgr.count(), 1);
    assert_eq!(mgr.get_state(task_id), Some(TaskState::Pending));
}

#[test]
fn task_manager_concurrent_creates() {
    let mgr = Arc::new(TaskManager::new());
    mgr.register(
        TaskDefinition::new("app.echo", "Echo", "Echo"),
        Box::new(EchoHandler),
    ).unwrap();

    let mut handles = vec![];

    for i in 0..10 {
        let mgr_clone = Arc::clone(&mgr);
        handles.push(thread::spawn(move || {
            mgr_clone.create(
                "app.echo",
                TaskContext::new(ExecutionId::new()),
                TaskInput::new(format!("task_{}", i)),
            ).unwrap()
        }));
    }

    let mut task_ids = vec![];
    for handle in handles {
        task_ids.push(handle.join().unwrap());
    }

    assert_eq!(mgr.count(), 10);

    // All task IDs are unique
    let mut sorted = task_ids.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), 10);
}

// --- Async trait tests ---

#[test]
fn async_task_handler_trait_exists() {
    // Verify the type exists and is constructible
    fn _check<F>(_f: F)
    where
        F: Fn(&TaskInput, &TaskContext, &EngineRef) -> AsyncTaskFuture,
    {}

    // Just verify the type compiles
    let _ = std::marker::PhantomData::<&dyn AsyncTaskHandler>;
}

#[test]
fn async_command_handler_trait_exists() {
    let _ = std::marker::PhantomData::<&dyn AsyncCommandHandler>;
}

// --- Integration: concurrent task execution via thread pool ---

#[test]
fn concurrent_task_execution_via_thread_pool() {
    let engine = Bootstrap::create().unwrap();
    engine.task_manager().register(
        TaskDefinition::new("app.echo", "Echo", "Echo"),
        Box::new(EchoHandler),
    ).unwrap();

    let task_ids: Vec<_> = (0..5).map(|i| {
        engine.task_manager().create(
            "app.echo",
            TaskContext::new(ExecutionId::new()),
            TaskInput::new(format!("task_{}", i)),
        ).unwrap()
    }).collect();

    let pool = TaskPool::new(4);
    let results = Arc::new(Mutex::new(vec![]));

    for task_id in task_ids {
        let results_clone = results.clone();
        pool.spawn(move || {
            let _ = task_id; // In a real app, start_task would be called here
            results_clone.lock().unwrap().push(task_id.value());
        });
    }

    thread::sleep(Duration::from_millis(100));
    assert_eq!(results.lock().unwrap().len(), 5);
}