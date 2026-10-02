use app_shell::app_engine::{TaskDefinition, TaskDefinitionId};

#[test]
fn definition_stores_basic_fields() {
    let def = TaskDefinition::new("project.export", "Export", "Export the project");
    assert_eq!(def.id(), &TaskDefinitionId::new("project.export"));
    assert_eq!(def.name(), "Export");
    assert_eq!(def.description(), "Export the project");
    assert!(!def.supports_pause());
    assert!(!def.supports_cancel());
}

#[test]
fn definition_with_capabilities() {
    let def = TaskDefinition::new("project.export", "Export", "Export")
        .with_pause_support()
        .with_cancel_support();
    assert!(def.supports_pause());
    assert!(def.supports_cancel());
}

#[test]
fn definition_id_from_str_and_string() {
    let a = TaskDefinitionId::new("x");
    let b: TaskDefinitionId = "x".into();
    let c: TaskDefinitionId = String::from("x").into();
    assert_eq!(a, b);
    assert_eq!(a, c);
}