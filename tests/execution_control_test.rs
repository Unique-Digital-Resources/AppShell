use app_shell::app_engine::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// --- Stub handler that checks cancellation ---

struct CancellableRenderHandler {
    cancel_check_count: Arc<Mutex<usize>>,
}

impl TaskHandler for CancellableRenderHandler {
    fn execute(
        &self,
        _input: &TaskInput,
        context: &TaskContext,
        _engine: &EngineRef,
    ) -> Result<TaskResult, TaskError> {
        for _ in 0..1000 {
            *self.cancel_check_count.lock().unwrap() += 1;

            // Check cancellation
            if context.is_cancelled() {
                return Ok(TaskResult::cancelled());
            }

            // Check deadline
            if context.is_expired() {
                return Err(TaskError::ExecutionFailed {
                    id: 0,
                    reason: "timeout".to_string(),
                });
            }

            // Report progress
            context.output().progress(0.5);

            // Small yield
            std::thread::sleep(Duration::from_millis(1));
        }

        Ok(TaskResult::completed())
    }
}

// --- Cancellation tests ---

#[test]
fn cancellation_token_starts_uncancelled() {
    let token = CancellationToken::new();
    assert!(!token.is_cancelled());
}

#[test]
fn cancellation_token_can_be_cancelled() {
    let token = CancellationToken::new();
    token.cancel();
    assert!(token.is_cancelled());
}

#[test]
fn cancellation_token_is_cloneable_and_shared() {
    let token = CancellationToken::new();
    let clone = token.clone();
    token.cancel();
    assert!(clone.is_cancelled()); // Shared state
}

#[test]
fn cancellation_token_can_be_reset() {
    let token = CancellationToken::new();
    token.cancel();
    assert!(token.is_cancelled());
    token.reset();
    assert!(!token.is_cancelled());
}

#[test]
fn task_context_has_cancellation_token() {
    let ctx = TaskContext::new(ExecutionId::new());
    assert!(!ctx.is_cancelled());
}

#[test]
fn task_context_cancellation_propagates() {
    let ctx = TaskContext::new(ExecutionId::new());
    let token = ctx.cancellation_token();
    token.cancel();
    assert!(ctx.is_cancelled()); // Shared state through the token
}

// --- Deadline tests ---

#[test]
fn deadline_none_never_expires() {
    let deadline = Deadline::none();
    assert!(!deadline.is_expired());
}

#[test]
fn deadline_after_expires() {
    let deadline = Deadline::after(Duration::from_millis(1));
    std::thread::sleep(Duration::from_millis(5));
    assert!(deadline.is_expired());
}

#[test]
fn deadline_not_yet_expired() {
    let deadline = Deadline::after(Duration::from_secs(60));
    assert!(!deadline.is_expired());
}

#[test]
fn deadline_remaining() {
    let deadline = Deadline::after(Duration::from_secs(10));
    let remaining = deadline.remaining().unwrap();
    assert!(remaining <= Duration::from_secs(10));
    assert!(remaining > Duration::from_secs(8));
}

#[test]
fn deadline_none_has_no_remaining() {
    let deadline = Deadline::none();
    assert!(deadline.remaining().is_none());
}

#[test]
fn task_context_has_deadline() {
    let ctx = TaskContext::new(ExecutionId::new());
    assert!(!ctx.is_expired());
}

#[test]
fn task_context_with_deadline() {
    let ctx = TaskContext::new(ExecutionId::new())
        .with_deadline(Deadline::after(Duration::from_millis(1)));
    std::thread::sleep(Duration::from_millis(5));
    assert!(ctx.is_expired());
}

#[test]
fn command_context_with_deadline() {
    let ctx = CommandContext::new(ExecutionId::new())
        .with_deadline(Deadline::after(Duration::from_millis(1)));
    std::thread::sleep(Duration::from_millis(5));
    assert!(ctx.is_expired());
}

// --- Output channel tests ---

#[test]
fn output_channel_reports_progress() {
    let channel = TaskOutputChannel::new(42);
    channel.progress(0.5);
    assert_eq!(channel.current_progress(), 0.5);
}

#[test]
fn output_channel_progress_clamped() {
    let channel = TaskOutputChannel::new(42);
    channel.progress(-1.0);
    assert_eq!(channel.current_progress(), 0.0);
    channel.progress(2.0);
    assert_eq!(channel.current_progress(), 1.0);
}

#[test]
fn output_channel_emits_signal_with_bus() {
    let signal_bus = Arc::new(SignalBus::new());
    let received = Arc::new(Mutex::new(0.0_f32));

    let r = received.clone();
    signal_bus.subscribe(
        "task.progress_changed",
        Box::new(move |signal: &Signal| {
            if let Some(p) = signal.payload::<f32>() {
                *r.lock().unwrap() = *p;
            }
        }),
    );

    let channel = TaskOutputChannel::with_signal_bus(signal_bus, 42);
    channel.progress(0.75);

    assert_eq!(*received.lock().unwrap(), 0.75);
}

#[test]
fn output_channel_push_emits_signal() {
    let signal_bus = Arc::new(SignalBus::new());
    let received = Arc::new(Mutex::new(None::<String>));

    let r = received.clone();
    signal_bus.subscribe(
        "task.output",
        Box::new(move |signal: &Signal| {
            if let Some(s) = signal.payload::<String>() {
                *r.lock().unwrap() = Some(s.clone());
            }
        }),
    );

    let channel = TaskOutputChannel::with_signal_bus(signal_bus, 42);
    channel.push("intermediate result".to_string());

    assert_eq!(*received.lock().unwrap(), Some("intermediate result".to_string()));
}

#[test]
fn output_channel_without_bus_silently_succeeds() {
    let channel = TaskOutputChannel::new(42);
    channel.progress(0.5); // No crash
    channel.push("data".to_string()); // No crash
    assert_eq!(channel.current_progress(), 0.5);
}

// --- Integration: TaskManager + cancellation ---

#[test]
fn cancel_propagates_to_running_handler() {
    let mut engine = Bootstrap::create().unwrap();
    let check_count = Arc::new(Mutex::new(0usize));

    engine.task_manager_mut().register(
        TaskDefinition::new("render", "Render", "Render")
            .with_cancel_support(),
        Box::new(CancellableRenderHandler {
            cancel_check_count: check_count.clone(),
        }),
    ).unwrap();

    let task_id = engine.task_manager_mut().create(
        "render",
        TaskContext::new(ExecutionId::new()),
        TaskInput::empty(),
    ).unwrap();

    // Cancel the task (it's still pending, so state transitions to Cancelled)
    engine.task_manager_mut().cancel(task_id).unwrap();
    assert_eq!(
        engine.task_manager().get_state(task_id).unwrap(),
        TaskState::Cancelled,
    );
}

#[test]
fn handler_can_check_cancellation_during_execution() {
    let mut engine = Bootstrap::create().unwrap();
    let check_count = Arc::new(Mutex::new(0usize));

    engine.task_manager_mut().register(
        TaskDefinition::new("render", "Render", "Render"),
        Box::new(CancellableRenderHandler {
            cancel_check_count: check_count.clone(),
        }),
    ).unwrap();

    let task_id = engine.task_manager_mut().create(
        "render",
        TaskContext::new(ExecutionId::new())
            .with_deadline(Deadline::after(Duration::from_millis(1))),
        TaskInput::empty(),
    ).unwrap();

    // Start the task — it should timeout (deadline is 1ms)
    let result = engine.start_task(task_id);
    assert!(result.is_err());
    assert_eq!(
        engine.task_manager().get_state(task_id).unwrap(),
        TaskState::Failed,
    );

    // The handler should have checked cancellation at least once
    assert!(*check_count.lock().unwrap() > 0);
}

#[test]
fn task_context_output_channel_available_in_handler() {
    struct ProgressHandler { progress: Arc<Mutex<Option<f32>>> }
    impl TaskHandler for ProgressHandler {
        fn execute(&self, _input: &TaskInput, ctx: &TaskContext, _engine: &EngineRef) -> Result<TaskResult, TaskError> {
            ctx.output().progress(0.42);
            *self.progress.lock().unwrap() = Some(ctx.output().current_progress());
            Ok(TaskResult::completed())
        }
    }

    let mut engine = Bootstrap::create().unwrap();
    let progress = Arc::new(Mutex::new(None));

    engine.task_manager_mut().register(
        TaskDefinition::new("progress", "Progress", "Progress"),
        Box::new(ProgressHandler { progress: progress.clone() }),
    ).unwrap();

    let task_id = engine.task_manager_mut().create(
        "progress",
        TaskContext::new(ExecutionId::new()),
        TaskInput::empty(),
    ).unwrap();

    engine.start_task(task_id).unwrap();

    assert_eq!(*progress.lock().unwrap(), Some(0.42));
}

#[test]
fn handler_reports_progress_via_signal_bus() {
    struct ProgressEmittingHandler;
    impl TaskHandler for ProgressEmittingHandler {
        fn execute(&self, _input: &TaskInput, ctx: &TaskContext, _engine: &EngineRef) -> Result<TaskResult, TaskError> {
            ctx.output().progress(0.5);
            ctx.output().progress(0.75);
            ctx.output().progress(1.0);
            Ok(TaskResult::completed())
        }
    }

    let mut engine = Bootstrap::create().unwrap();
    let received = Arc::new(Mutex::new(vec![]));

    let r = received.clone();
    engine.signal_bus().subscribe(
        "task.progress_changed",
        Box::new(move |signal: &Signal| {
            if let Some(p) = signal.payload::<f32>() {
                r.lock().unwrap().push(*p);
            }
        }),
    );

    engine.task_manager_mut().register(
        TaskDefinition::new("emit", "Emit", "Emit"),
        Box::new(ProgressEmittingHandler),
    ).unwrap();

    let task_id = engine.task_manager_mut().create(
        "emit",
        TaskContext::new(ExecutionId::new()),
        TaskInput::empty(),
    ).unwrap();

    engine.start_task(task_id).unwrap();

    let values = received.lock().unwrap();
    assert_eq!(*values, vec![0.5, 0.75, 1.0]);
}

#[test]
fn handler_streams_intermediate_output() {
    struct StreamingHandler;
    impl TaskHandler for StreamingHandler {
        fn execute(&self, _input: &TaskInput, ctx: &TaskContext, _engine: &EngineRef) -> Result<TaskResult, TaskError> {
            for i in 0..3 {
                ctx.output().push(format!("frame_{}", i));
            }
            Ok(TaskResult::completed())
        }
    }

    let mut engine = Bootstrap::create().unwrap();
    let received = Arc::new(Mutex::new(vec![]));

    let r = received.clone();
    engine.signal_bus().subscribe(
        "task.output",
        Box::new(move |signal: &Signal| {
            if let Some(s) = signal.payload::<String>() {
                r.lock().unwrap().push(s.clone());
            }
        }),
    );

    engine.task_manager_mut().register(
        TaskDefinition::new("stream", "Stream", "Stream"),
        Box::new(StreamingHandler),
    ).unwrap();

    let task_id = engine.task_manager_mut().create(
        "stream",
        TaskContext::new(ExecutionId::new()),
        TaskInput::empty(),
    ).unwrap();

    engine.start_task(task_id).unwrap();

    let values = received.lock().unwrap();
    assert_eq!(values.len(), 3);
    assert_eq!(values[0], "frame_0");
    assert_eq!(values[1], "frame_1");
    assert_eq!(values[2], "frame_2");
}