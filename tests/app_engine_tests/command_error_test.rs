//! Tests for `CommandError` variants and their Display impls.

use app_shell::app_engine::CommandError;

#[test]
fn unknown_command_displays_id() {
    let err = CommandError::UnknownCommand {
        id: "app.foo".to_string(),
    };
    assert_eq!(err.to_string(), "Unknown command: app.foo");
}

#[test]
fn invalid_arguments_displays_message() {
    let err = CommandError::InvalidArguments("expected String".to_string());
    assert!(err.to_string().contains("Invalid arguments"));
    assert!(err.to_string().contains("expected String"));
}

#[test]
fn execution_failed_displays_message() {
    let err = CommandError::ExecutionFailed("disk full".to_string());
    assert!(err.to_string().contains("disk full"));
}

#[test]
fn cancelled_displays_simple_message() {
    let err = CommandError::Cancelled;
    assert_eq!(err.to_string(), "Command cancelled");
}

#[test]
fn permission_denied_displays_reason() {
    let err = CommandError::PermissionDenied("not authorized".to_string());
    assert!(err.to_string().contains("Permission denied"));
    assert!(err.to_string().contains("not authorized"));
}

#[test]
fn already_registered_displays_id() {
    let err = CommandError::AlreadyRegistered {
        id: "app.echo".to_string(),
    };
    assert!(err.to_string().contains("app.echo"));
}

#[test]
fn no_handler_displays_id() {
    let err = CommandError::NoHandler {
        id: "app.foo".to_string(),
    };
    assert!(err.to_string().contains("app.foo"));
}

#[test]
fn command_error_implements_std_error() {
    fn assert_std_error<E: std::error::Error>(_: E) {}
    assert_std_error(CommandError::Cancelled);
}