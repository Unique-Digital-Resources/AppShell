//! Phase 5 acceptance criteria.

use app_shell::app_engine::{
    AppEngine, Bootstrap, ExecutionId, JobState, Schedule, TaskContext,
    TaskDefinition, TaskHandler, TaskInput, TaskResult, TaskState,
    EngineRef,
};
use std::time::{Duration, Instant};

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

/// 1. Submit a Task to the scheduler
/// 2. Create a Job
#[test]
fn acceptance_submit_creates_job() {
    let mut engine = setup_engine();
    let task_id = engine.task_manager_mut().create(
        "app.echo",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new("hello".to_string()),
    ).unwrap();
    let job_id = engine.scheduler_mut().submit(task_id, Schedule::Immediate, 0);
    assert!(engine.scheduler().get_job(job_id).is_some());
}

/// 3. Execute a task immediately
#[test]
fn acceptance_execute_immediately() {
    let mut engine = setup_engine();
    let task_id = engine.task_manager_mut().create(
        "app.echo",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new("now".to_string()),
    ).unwrap();
    let _job_id = engine.scheduler_mut().submit(task_id, Schedule::Immediate, 0);
    let job = engine.scheduler_mut().dispatch().unwrap();
    let result = engine.start_task(job.task_id()).unwrap();
    assert!(result.is_completed());
}

/// 4. Execute after a delay
#[test]
fn acceptance_execute_after_delay() {
    let mut engine = setup_engine();
    let task_id = engine.task_manager_mut().create(
        "app.echo",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new("delayed".to_string()),
    ).unwrap();
    let _job_id = engine.scheduler_mut().submit(task_id, Schedule::Delayed(Duration::from_millis(1)), 0);
    assert_eq!(engine.scheduler().queue_len(), 0);
    std::thread::sleep(Duration::from_millis(5));
    engine.scheduler_mut().tick();
    assert_eq!(engine.scheduler().queue_len(), 1);
    let job = engine.scheduler_mut().dispatch().unwrap();
    let result = engine.start_task(job.task_id()).unwrap();
    assert!(result.is_completed());
}

/// 5. Queue multiple jobs
/// 6. Dequeue jobs for execution
#[test]
fn acceptance_queue_and_dequeue_multiple() {
    let mut engine = setup_engine();
    for _ in 0..5 {
        let task_id = engine.task_manager_mut().create(
            "app.echo",
            TaskContext::new(ExecutionId::new()),
            TaskInput::new("job".to_string()),
        ).unwrap();
        engine.scheduler_mut().submit(task_id, Schedule::Immediate, 0);
    }
    assert_eq!(engine.scheduler().queue_len(), 5);
    for _ in 0..5 {
        let job = engine.scheduler_mut().dispatch().unwrap();
        let result = engine.start_task(job.task_id()).unwrap();
        assert!(result.is_completed());
    }
    assert_eq!(engine.scheduler().queue_len(), 0);
}

/// 7. Apply priority to queued jobs
#[test]
fn acceptance_priority_ordering() {
    let mut engine = setup_engine();
    let low = engine.task_manager_mut().create("app.echo", TaskContext::new(ExecutionId::new()), TaskInput::new("low".to_string())).unwrap();
    let high = engine.task_manager_mut().create("app.echo", TaskContext::new(ExecutionId::new()), TaskInput::new("high".to_string())).unwrap();
    let mid = engine.task_manager_mut().create("app.echo", TaskContext::new(ExecutionId::new()), TaskInput::new("mid".to_string())).unwrap();

    engine.scheduler_mut().submit(low, Schedule::Immediate, 1);
    engine.scheduler_mut().submit(high, Schedule::Immediate, 10);
    engine.scheduler_mut().submit(mid, Schedule::Immediate, 5);

    assert_eq!(engine.scheduler_mut().dispatch().unwrap().priority(), 10);
    assert_eq!(engine.scheduler_mut().dispatch().unwrap().priority(), 5);
    assert_eq!(engine.scheduler_mut().dispatch().unwrap().priority(), 1);
}

/// 8. Represent periodic execution
#[test]
fn acceptance_periodic_representation() {
    let interval = Schedule::Interval(Duration::from_secs(60));
    assert!(interval.is_recurring());
    assert!(interval.next_occurrence(Instant::now()).is_some());
}

/// 9. Represent trigger-based activation
#[test]
fn acceptance_trigger_based_activation() {
    use app_shell::app_engine::Trigger;
    let mut engine = setup_engine();
    let task_id = engine.task_manager_mut().create(
        "app.echo",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new("triggered".to_string()),
    ).unwrap();
    let trigger = Trigger::OnEvent("startup".to_string());
    let _job_id = engine.scheduler_mut().submit_with_trigger(task_id, Schedule::Immediate, trigger, 0);
    assert_eq!(engine.scheduler().queue_len(), 0);
    engine.scheduler_mut().satisfy_trigger("startup");
    engine.scheduler_mut().tick();
    assert_eq!(engine.scheduler().queue_len(), 1);
    let job = engine.scheduler_mut().dispatch().unwrap();
    let result = engine.start_task(job.task_id()).unwrap();
    assert!(result.is_completed());
}

/// 10. Cancel a scheduled job
#[test]
fn acceptance_cancel_before_execution() {
    let mut engine = setup_engine();
    let task_id = engine.task_manager_mut().create(
        "app.echo",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new("cancelled".to_string()),
    ).unwrap();
    let job_id = engine.scheduler_mut().submit(task_id, Schedule::Delayed(Duration::from_secs(60)), 0);
    assert_eq!(engine.scheduler().scheduled_count(), 1);
    engine.scheduler_mut().cancel(job_id).unwrap();
    assert_eq!(engine.scheduler().scheduled_count(), 0);
}

/// 11. Keep scheduling separate from execution
#[test]
fn acceptance_scheduling_separate_from_execution() {
    let mut engine = setup_engine();
    let task_id = engine.task_manager_mut().create(
        "app.echo",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new("separate".to_string()),
    ).unwrap();
    let _job_id = engine.scheduler_mut().submit(task_id, Schedule::Immediate, 0);
    let job = engine.scheduler_mut().dispatch().unwrap();
    assert_eq!(job.task_id(), task_id);
}

/// 12. Keep task state separate from job state
#[test]
fn acceptance_task_state_separate_from_job_state() {
    let mut engine = setup_engine();
    let task_id = engine.task_manager_mut().create(
        "app.echo",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new("state".to_string()),
    ).unwrap();
    let job_id = engine.scheduler_mut().submit(task_id, Schedule::Delayed(Duration::from_secs(60)), 0);
    assert_eq!(engine.task_manager().get_state(task_id).unwrap(), TaskState::Pending);
    assert_eq!(engine.scheduler().get_job(job_id).unwrap().state(), JobState::Scheduled);
}

/// 13. Pass executable jobs to TaskExecutor
#[test]
fn acceptance_pass_jobs_to_executor() {
    let mut engine = setup_engine();
    let task_id = engine.task_manager_mut().create(
        "app.echo",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new("exec".to_string()),
    ).unwrap();
    engine.scheduler_mut().submit(task_id, Schedule::Immediate, 0);
    engine.scheduler_mut().tick();
    let job = engine.scheduler_mut().dispatch().unwrap();
    assert_eq!(job.state(), JobState::Dispatched);
    let result = engine.start_task(job.task_id()).unwrap();
    assert!(result.is_completed());
}

/// 14. Execute domain work without scheduler knowing
#[test]
fn acceptance_domain_work_without_scheduler_knowing() {
    let mut engine = Bootstrap::create().unwrap();
    engine.task_manager_mut().register(
        TaskDefinition::new("project.export", "Export", "Export project"),
        Box::new(EchoHandler),
    ).unwrap();
    let task_id = engine.task_manager_mut().create(
        "project.export",
        TaskContext::new(ExecutionId::new()),
        TaskInput::new("/output.pdf".to_string()),
    ).unwrap();
    engine.scheduler_mut().submit(task_id, Schedule::Immediate, 5);
    let job = engine.scheduler_mut().dispatch().unwrap();
    let result = engine.start_task(job.task_id()).unwrap();
    assert_eq!(result.output::<String>(), Some(&"/output.pdf".to_string()));
}