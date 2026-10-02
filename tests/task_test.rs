use app_shell::app_engine::{
    ExecutionId, Task, TaskContext, TaskId, TaskInput, TaskState,
};

#[test]
fn task_id_is_unique() {
    let a = TaskId::new();
    let b = TaskId::new();
    assert_ne!(a, b);
}

#[test]
fn task_input_holds_typed_data() {
    let input = TaskInput::new("hello".to_string());
    assert_eq!(input.get::<String>(), Some(&"hello".to_string()));
}

#[test]
fn task_input_empty_is_empty() {
    let input = TaskInput::empty();
    assert!(input.is_empty());
    assert!(!input.has_data());
}

#[test]
fn task_starts_in_pending_state() {
    let task = Task::new(
        "project.export",
        TaskContext::new(ExecutionId::new()),
        TaskInput::empty(),
        false,
        false,
    );
    assert_eq!(task.state(), TaskState::Pending);
    assert!(task.can_start());
    assert!(!task.supports_pause());
    assert!(!task.supports_cancel());
}

#[test]
fn task_with_capabilities() {
    let task = Task::new(
        "project.export",
        TaskContext::new(ExecutionId::new()),
        TaskInput::empty(),
        true,
        true,
    );
    assert!(task.supports_pause());
    assert!(task.supports_cancel());
}

#[test]
fn task_set_progress_clamps() {
    let mut task = Task::new(
        "x",
        TaskContext::new(ExecutionId::new()),
        TaskInput::empty(),
        false,
        false,
    );
    task.set_progress(-1.0);
    assert_eq!(task.progress(), 0.0);
    task.set_progress(2.0);
    assert_eq!(task.progress(), 1.0);
    task.set_progress(0.5);
    assert_eq!(task.progress(), 0.5);
}