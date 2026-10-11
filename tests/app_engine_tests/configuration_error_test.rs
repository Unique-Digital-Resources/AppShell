use app_shell::app_engine::ConfigurationError;

#[test]
fn validation_failed_display() {
    let e = ConfigurationError::ValidationFailed {
        key: "count".to_string(),
        reason: "below minimum".to_string(),
    };
    assert!(e.to_string().contains("count"));
    assert!(e.to_string().contains("below minimum"));
}

#[test]
fn not_found_display() {
    let e = ConfigurationError::NotFound {
        key: "missing".to_string(),
    };
    assert!(e.to_string().contains("missing"));
}

#[test]
fn invalid_type_display() {
    let e = ConfigurationError::InvalidType {
        key: "count".to_string(),
        expected: "integer".to_string(),
        actual: "string".to_string(),
    };
    assert!(e.to_string().contains("count"));
}

#[test]
fn load_failed_display() {
    let e = ConfigurationError::LoadFailed {
        reason: "file not found".to_string(),
    };
    assert!(e.to_string().contains("file not found"));
}

#[test]
fn configuration_error_implements_std_error() {
    fn assert_std_error<E: std::error::Error>(_: E) {}
    assert_std_error(ConfigurationError::NotFound {
        key: "x".to_string(),
    });
}