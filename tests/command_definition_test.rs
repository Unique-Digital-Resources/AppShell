//! Tests for `CommandDefinition` and `CommandDefinitionId`.

use app_shell::app_engine::{CommandDefinition, CommandDefinitionId};

#[test]
fn definition_id_constructs_from_string_and_str() {
    let a = CommandDefinitionId::new("app.quit");
    let b: CommandDefinitionId = "app.quit".into();
    let c: CommandDefinitionId = String::from("app.quit").into();
    assert_eq!(a, b);
    assert_eq!(a, c);
    assert_eq!(a.as_str(), "app.quit");
}

#[test]
fn definition_id_displays_as_str() {
    let id = CommandDefinitionId::new("project.export");
    assert_eq!(id.to_string(), "project.export");
}

#[test]
fn definition_stores_id_name_and_description() {
    let def = CommandDefinition::new("app.quit", "Quit", "Quit the application");
    assert_eq!(def.id(), &CommandDefinitionId::new("app.quit"));
    assert_eq!(def.name(), "Quit");
    assert_eq!(def.description(), "Quit the application");
}

#[test]
fn definition_metadata_can_be_added() {
    let def = CommandDefinition::new("app.quit", "Quit", "Quit the application")
        .with_metadata("category", "file")
        .with_metadata("shortcut", "Ctrl+Q");

    assert_eq!(def.metadata().get("category"), Some(&"file".to_string()));
    assert_eq!(def.metadata().get("shortcut"), Some(&"Ctrl+Q".to_string()));
}

#[test]
fn definitions_with_same_id_are_equal() {
    let a = CommandDefinition::new("app.quit", "Quit", "Quit the app");
    let b = CommandDefinition::new("app.quit", "Quit", "Quit the app");
    assert_eq!(a, b);
}

#[test]
fn definitions_with_different_ids_are_distinct() {
    let a = CommandDefinition::new("app.quit", "Quit", "Quit");
    let b = CommandDefinition::new("app.save", "Save", "Save");
    assert_ne!(a, b);
}