use app_shell::app_engine::Trigger;

#[test]
fn manual_is_manual() {
    assert!(Trigger::Manual.is_manual());
    assert!(!Trigger::OnEvent("x".to_string()).is_manual());
}

#[test]
fn trigger_name() {
    assert_eq!(Trigger::Manual.name(), "manual");
    assert_eq!(Trigger::OnEvent("ready".to_string()).name(), "ready");
    assert_eq!(Trigger::OnSignal("idle".to_string()).name(), "idle");
    assert_eq!(Trigger::OnCondition("ready".to_string()).name(), "ready");
}

#[test]
fn trigger_equality() {
    assert_eq!(Trigger::Manual, Trigger::Manual);
    assert_ne!(
        Trigger::OnEvent("a".to_string()),
        Trigger::OnEvent("b".to_string())
    );
}