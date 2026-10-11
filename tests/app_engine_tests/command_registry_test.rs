use app_shell::app_engine::{
    CommandContext, CommandDefinition, CommandDefinitionId, CommandError, CommandHandler,
    CommandInput, CommandRegistry, CommandResult, EngineRef,
};

struct StubHandler;

impl CommandHandler for StubHandler {
    fn execute(
        &self,
        _input: &CommandInput,
        _context: &CommandContext,
        _engine: &EngineRef,
    ) -> Result<CommandResult, CommandError> {
        Ok(CommandResult::success())
    }
}

fn make_def(id: &str) -> CommandDefinition {
    CommandDefinition::new(id, id, id)
}

#[test]
fn new_registry_is_empty() {
    let reg = CommandRegistry::new();
    assert!(reg.is_empty());
    assert_eq!(reg.len(), 0);
}

#[test]
fn register_adds_to_registry() {
    let mut reg = CommandRegistry::new();
    reg.register(make_def("app.quit"), Box::new(StubHandler)).unwrap();
    assert_eq!(reg.len(), 1);
    assert!(reg.contains(&CommandDefinitionId::new("app.quit")));
}

#[test]
fn register_duplicate_returns_error() {
    let mut reg = CommandRegistry::new();
    reg.register(make_def("app.quit"), Box::new(StubHandler)).unwrap();
    let result = reg.register(make_def("app.quit"), Box::new(StubHandler));
    assert!(matches!(result, Err(CommandError::AlreadyRegistered { .. })));
}

#[test]
fn get_returns_definition() {
    let mut reg = CommandRegistry::new();
    reg.register(make_def("app.quit"), Box::new(StubHandler)).unwrap();
    let def = reg.get(&CommandDefinitionId::new("app.quit")).expect("definition must exist");
    assert_eq!(def.name(), "app.quit");
}

#[test]
fn get_missing_returns_none() {
    let reg = CommandRegistry::new();
    assert!(reg.get(&CommandDefinitionId::new("missing")).is_none());
}

#[test]
fn unregister_removes_entry() {
    let mut reg = CommandRegistry::new();
    reg.register(make_def("app.quit"), Box::new(StubHandler)).unwrap();
    let removed = reg.unregister(&CommandDefinitionId::new("app.quit"));
    assert!(removed.is_some());
    assert!(!reg.contains(&CommandDefinitionId::new("app.quit")));
    assert_eq!(reg.len(), 0);
}

#[test]
fn unregister_missing_returns_none() {
    let mut reg = CommandRegistry::new();
    assert!(reg.unregister(&CommandDefinitionId::new("missing")).is_none());
}

#[test]
fn list_returns_all_definitions() {
    let mut reg = CommandRegistry::new();
    reg.register(make_def("app.quit"), Box::new(StubHandler)).unwrap();
    reg.register(make_def("app.save"), Box::new(StubHandler)).unwrap();
    reg.register(make_def("app.open"), Box::new(StubHandler)).unwrap();
    let list = reg.list();
    assert_eq!(list.len(), 3);
}

#[test]
fn clear_removes_all() {
    let mut reg = CommandRegistry::new();
    reg.register(make_def("app.quit"), Box::new(StubHandler)).unwrap();
    reg.register(make_def("app.save"), Box::new(StubHandler)).unwrap();
    reg.clear();
    assert!(reg.is_empty());
}