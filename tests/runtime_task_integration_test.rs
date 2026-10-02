use app_shell::app_engine::{
    Bootstrap, ExecutionId, Lifecycle, TaskContext, TaskDefinition, TaskHandler,
    TaskInput, TaskManager, TaskResult, TaskState, EngineRef,
};

struct DoubleHandler;
impl TaskHandler for DoubleHandler {
    fn execute(
        &self,
        input: &TaskInput,
        _ctx: &TaskContext,
        _engine: &EngineRef,
    ) -> Result<TaskResult, app_shell::app_engine::TaskError> {
        let n: &i32 = input.get::<i32>()
            .ok_or_else(|| app_shell::app_engine::TaskError::ExecutionFailed {
                id: 0, reason: "expected i32".to_string()
            })?;
        Ok(TaskResult::completed_with(n * 2))
    }
}

#[test]
fn runtime_owns_task_manager() {
    let runtime = Bootstrap::create().unwrap();
    let _: &TaskManager = runtime.task_manager();
}

#[test]
fn runtime_task_manager_starts_empty() {
    let runtime = Bootstrap::create().unwrap();
    assert_eq!(runtime.task_manager().count(), 0);
}

#[test]
fn runtime_can_register_and_execute_task() {
    let mut runtime = Bootstrap::create().unwrap();

    runtime.task_manager_mut().register(
        TaskDefinition::new("math.double", "Double", "Double a number"),
        Box::new(DoubleHandler),
    ).unwrap();

    let task_id = runtime.task_manager_mut().create(
        "math.double",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new(21_i32),
    ).unwrap();

    let result = runtime.start_task(task_id).unwrap();
    assert!(result.is_completed());
    assert_eq!(result.output::<i32>(), Some(&42));
    assert_eq!(
        runtime.task_manager().get_state(task_id).unwrap(),
        TaskState::Completed
    );
}

#[test]
fn task_manager_independent_of_lifecycle() {
    let mut runtime = Bootstrap::create().unwrap();
    runtime.initialize().unwrap();
    runtime.start().unwrap();
    assert_eq!(runtime.task_manager().count(), 0);
    runtime.stop().unwrap();
    runtime.dispose().unwrap();
}