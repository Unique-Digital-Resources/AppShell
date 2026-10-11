use app_shell::app_engine::TaskState;

#[test]
fn pending_can_start_and_cancel() {
    let s = TaskState::Pending;
    assert!(s.can_start());
    assert!(s.can_cancel());
    assert!(!s.can_pause());
    assert!(!s.is_terminal());
}

#[test]
fn running_can_pause_complete_fail_cancel() {
    let s = TaskState::Running;
    assert!(s.can_pause());
    assert!(s.can_complete());
    assert!(s.can_fail());
    assert!(s.can_cancel());
    assert!(s.is_active());
}

#[test]
fn paused_can_resume_and_cancel() {
    let s = TaskState::Paused;
    assert!(s.can_resume());
    assert!(s.can_cancel());
    assert!(s.is_active());
}

#[test]
fn terminal_states_block_all_transitions() {
    for s in [TaskState::Completed, TaskState::Cancelled, TaskState::Failed] {
        assert!(!s.can_start());
        assert!(!s.can_pause());
        assert!(!s.can_resume());
        assert!(!s.can_cancel());
        assert!(!s.can_complete());
        assert!(!s.can_fail());
        assert!(s.is_terminal());
        assert!(!s.is_active());
    }
}

#[test]
fn as_str_and_display() {
    assert_eq!(TaskState::Pending.as_str(), "Pending");
    assert_eq!(TaskState::Running.to_string(), "Running");
}