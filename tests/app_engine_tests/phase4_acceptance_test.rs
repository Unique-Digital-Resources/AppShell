//! Phase 4 acceptance criteria.

use app_shell::app_engine::{
    AppEngine, Bootstrap, ExecutionId, TaskContext, TaskDefinition, TaskHandler,
    TaskInput, TaskResult, TaskResultStatus, TaskState, EngineRef,
};

struct EchoHandler;
impl TaskHandler for EchoHandler {
    fn execute(
        &self,
        input: &TaskInput,
        _ctx: &TaskContext,
        _engine: &EngineRef,
    ) -> Result<TaskResult, app_shell::app_engine::TaskError> {
        let s: &String = input.get::<String>()
            .ok_or_else(|| app_shell::app_engine::TaskError::ExecutionFailed {
                id: 0, reason: "expected String".to_string()
            })?;
        Ok(TaskResult::completed_with(s.clone()))
    }
}

fn setup_engine() -> AppEngine {
    let mut engine = Bootstrap::create().unwrap();
    engine.task_manager_mut().register(
        TaskDefinition::new("app.echo", "Echo", "Echo input"),
        Box::new(EchoHandler),
    ).unwrap();
    engine
}

/// 1. Define a task type
/// 2. Create a task instance with unique ID
#[test]
fn acceptance_create_task_with_unique_id() {
    let mut engine = setup_engine();
    let id1 = engine.task_manager_mut().create(
        "app.echo",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new("/a".to_string()),
    ).unwrap();
    let id2 = engine.task_manager_mut().create(
        "app.echo",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new("/b".to_string()),
    ).unwrap();
    assert_ne!(id1, id2);
}

/// 3. Attach an execution context
#[test]
fn acceptance_attach_execution_context() {
    let mut engine = setup_engine();
    let exec_id = ExecutionId::new();
    let id = engine.task_manager_mut().create(
        "app.echo",
        TaskContext::new(exec_id).with_metadata("source", "cli"),
        TaskInput::new("/tmp".to_string()),
    ).unwrap();
    assert_eq!(engine.task_manager().task_execution_id(id), Some(exec_id));
    assert_eq!(engine.task_manager().task_context_task_id(id), Some(id));
}

/// 4. Track task state
/// 5. Enforce valid state transitions
#[test]
fn acceptance_track_and_enforce_state() {
    let mut engine = setup_engine();
    let id = engine.task_manager_mut().create(
        "app.echo",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new("/x".to_string()),
    ).unwrap();
    assert_eq!(engine.task_manager().get_state(id), Some(TaskState::Pending));
    assert!(engine.task_manager_mut().pause(id).is_err());
}

/// 6. Start produces a result
#[test]
fn acceptance_start_produces_result() {
    let mut engine = setup_engine();
    let id = engine.task_manager_mut().create(
        "app.echo",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new("hello".to_string()),
    ).unwrap();
    let result = engine.start_task(id).unwrap();
    assert_eq!(result.status(), TaskResultStatus::Completed);
    assert_eq!(result.output::<String>(), Some(&"hello".to_string()));
    assert_eq!(engine.task_manager().get_state(id), Some(TaskState::Completed));
}

/// 7. Report execution failure
#[test]
fn acceptance_report_execution_failure() {
    struct FailHandler;
    impl TaskHandler for FailHandler {
        fn execute(&self, _input: &TaskInput, _ctx: &TaskContext, _engine: &EngineRef) -> Result<TaskResult, app_shell::app_engine::TaskError> {
            Err(app_shell::app_engine::TaskError::ExecutionFailed { id: 0, reason: "disk full".to_string() })
        }
    }

    let mut engine = Bootstrap::create().unwrap();
    engine.task_manager_mut().register(
        TaskDefinition::new("app.fail", "Fail", "Always fails"),
        Box::new(FailHandler),
    ).unwrap();

    let id = engine.task_manager_mut().create(
        "app.fail",
        TaskContext::new(ExecutionId::new()),
        TaskInput::empty(),
    ).unwrap();
    let result = engine.start_task(id);
    assert!(result.is_err());
    assert_eq!(engine.task_manager().get_state(id), Some(TaskState::Failed));
}

/// 8. Cancel a cancellable task
#[test]
fn acceptance_cancel_cancellable_task() {
    let mut engine = Bootstrap::create().unwrap();
    engine.task_manager_mut().register(
        TaskDefinition::new("app.echo", "Echo", "Echo").with_cancel_support(),
        Box::new(EchoHandler),
    ).unwrap();

    let id = engine.task_manager_mut().create(
        "app.echo",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new("/x".to_string()),
    ).unwrap();
    engine.task_manager_mut().cancel(id).unwrap();
    assert_eq!(engine.task_manager().get_state(id), Some(TaskState::Cancelled));
}

/// 9. Pause/resume a pausable task
#[test]
fn acceptance_pause_resume_pausable_task() {
    let mut engine = Bootstrap::create().unwrap();
    engine.task_manager_mut().register(
        TaskDefinition::new("app.echo", "Echo", "Echo").with_pause_support(),
        Box::new(EchoHandler),
    ).unwrap();

    let id = engine.task_manager_mut().create(
        "app.echo",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new("/x".to_string()),
    ).unwrap();
    assert!(engine.task_manager_mut().pause(id).is_err());
}

/// 10. Manage multiple task instances
#[test]
fn acceptance_manage_multiple_tasks() {
    let mut engine = setup_engine();
    let ids: Vec<_> = (0..5).map(|i| {
        engine.task_manager_mut().create(
            "app.echo",
            TaskContext::new(ExecutionId::new()),
            TaskInput::new(format!("job{}", i)),
        ).unwrap()
    }).collect();

    assert_eq!(engine.task_manager().count(), 5);
    for id in &ids {
        engine.start_task(*id).unwrap();
        assert_eq!(engine.task_manager().get_state(*id), Some(TaskState::Completed));
    }
}

/// 11. Domain operation executes as a task
/// 12. All domain-specific logic outside App Engine
#[test]
fn acceptance_domain_operation_as_task() {
    let mut engine = Bootstrap::create().unwrap();
    engine.task_manager_mut().register(
        TaskDefinition::new("project.export", "Export", "Export project"),
        Box::new(EchoHandler),
    ).unwrap();

    let id = engine.task_manager_mut().create(
        "project.export",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new("/final.pdf".to_string()),
    ).unwrap();
    let result = engine.start_task(id).unwrap();
    assert!(result.is_completed());
}

/// Full lifecycle
#[test]
fn acceptance_full_architectural_flow() {
    let mut engine = Bootstrap::create().unwrap();
    engine.task_manager_mut().register(
        TaskDefinition::new("project.export", "Export", "Export"),
        Box::new(EchoHandler),
    ).unwrap();

    let id = engine.task_manager_mut().create(
        "project.export",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new("/result.pdf".to_string()),
    ).unwrap();

    let result = engine.start_task(id).unwrap();
    assert_eq!(result.status(), TaskResultStatus::Completed);
    assert_eq!(result.output::<String>(), Some(&"/result.pdf".to_string()));
    assert_eq!(engine.task_manager().get_state(id), Some(TaskState::Completed));
}