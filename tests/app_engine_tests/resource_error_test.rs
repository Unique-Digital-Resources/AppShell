use app_shell::app_engine::ResourceError;

#[test]
fn not_found_display() {
    let e = ResourceError::NotFound { id: 42 };
    assert!(e.to_string().contains("42"));
}

#[test]
fn loader_not_found_display() {
    let e = ResourceError::LoaderNotFound {
        resource_type: "image".to_string(),
    };
    assert!(e.to_string().contains("image"));
}

#[test]
fn load_failed_display() {
    let e = ResourceError::LoadFailed {
        source: "file:///x".to_string(),
        reason: "disk full".to_string(),
    };
    assert!(e.to_string().contains("file:///x"));
    assert!(e.to_string().contains("disk full"));
}

#[test]
fn in_use_display() {
    let e = ResourceError::InUse { id: 5 };
    assert!(e.to_string().contains("in use"));
}

#[test]
fn resource_error_implements_std_error() {
    fn assert_std_error<E: std::error::Error>(_: E) {}
    assert_std_error(ResourceError::NotFound { id: 0 });
}