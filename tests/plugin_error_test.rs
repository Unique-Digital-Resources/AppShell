use app_shell::app_engine::PluginError;

#[test]
fn not_found_display() {
    let e = PluginError::NotFound { id: 42 };
    assert!(e.to_string().contains("42"));
}

#[test]
fn already_registered_display() {
    let e = PluginError::AlreadyRegistered {
        id: "image".to_string(),
    };
    assert!(e.to_string().contains("image"));
}

#[test]
fn load_failed_display() {
    let e = PluginError::LoadFailed {
        source: "static:x".to_string(),
        reason: "bad format".to_string(),
    };
    assert!(e.to_string().contains("static:x"));
    assert!(e.to_string().contains("bad format"));
}

#[test]
fn dependency_not_met_display() {
    let e = PluginError::DependencyNotMet {
        id: "a".to_string(),
        dependency: "b".to_string(),
    };
    assert!(e.to_string().contains("'a'"));
    assert!(e.to_string().contains("'b'"));
}

#[test]
fn plugin_error_implements_std_error() {
    fn assert_std_error<E: std::error::Error>(_: E) {}
    assert_std_error(PluginError::NotFound { id: 0 });
}