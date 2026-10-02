use app_shell::app_engine::*;
use std::any::Any;

// --- Domain type for testing ---

#[derive(Debug, Clone, PartialEq)]
struct ExportInput {
    project_id: String,
    format: String,
}

// --- Domain serializer for ExportInput ---

struct ExportInputSerializer;

impl Serializer for ExportInputSerializer {
    fn type_name(&self) -> &str {
        "serialization_test::ExportInput"
    }

    fn serialize(&self, data: &dyn Any) -> Result<Vec<u8>, SerializationError> {
        let input = data
            .downcast_ref::<ExportInput>()
            .ok_or_else(|| SerializationError::TypeMismatch {
                expected: self.type_name().to_string(),
                actual: "unknown".to_string(),
            })?;
        // Simple serialization: "project_id|format"
        let serialized = format!("{}|{}", input.project_id, input.format);
        Ok(serialized.into_bytes())
    }

    fn deserialize(&self, bytes: &[u8]) -> Result<Box<dyn Any + Send>, SerializationError> {
        let s = std::str::from_utf8(bytes)
            .map_err(|e| SerializationError::DeserializationFailed {
                type_name: self.type_name().to_string(),
                reason: e.to_string(),
            })?;
        let parts: Vec<&str> = s.split('|').collect();
        if parts.len() != 2 {
            return Err(SerializationError::DeserializationFailed {
                type_name: self.type_name().to_string(),
                reason: "expected 'project_id|format'".to_string(),
            });
        }
        Ok(Box::new(ExportInput {
            project_id: parts[0].to_string(),
            format: parts[1].to_string(),
        }))
    }
}

// --- SerializationRegistry tests ---

#[test]
fn registry_starts_empty() {
    let registry = SerializationRegistry::new();
    assert!(registry.is_empty());
}

#[test]
fn registry_register_and_check() {
    let registry = SerializationRegistry::new();
    registry.register(Box::new(ExportInputSerializer));
    assert!(registry.has_serializer("serialization_test::ExportInput"));
    assert_eq!(registry.len(), 1);
}

#[test]
fn registry_serialize_and_deserialize() {
    let registry = SerializationRegistry::new();
    registry.register(Box::new(ExportInputSerializer));

    let input = ExportInput {
        project_id: "proj_42".to_string(),
        format: "pdf".to_string(),
    };

    let bytes = registry.serialize(&input, "serialization_test::ExportInput").unwrap();
    assert!(!bytes.is_empty());

    let deserialized = registry.deserialize(&bytes, "serialization_test::ExportInput").unwrap();
    let restored = deserialized.downcast_ref::<ExportInput>().unwrap();
    assert_eq!(restored, &input);
}

#[test]
fn registry_no_serializer_error() {
    let registry = SerializationRegistry::new();
    let result = registry.serialize(&42_i32, "i32");
    assert!(matches!(result, Err(SerializationError::NoSerializer { .. })));
}

#[test]
fn registry_registered_types() {
    let registry = SerializationRegistry::new();
    registry.register(Box::new(ExportInputSerializer));
    let types = registry.registered_types();
    assert!(types.contains(&"serialization_test::ExportInput".to_string()));
}

// --- CommandInput serialization tests ---

#[test]
fn command_input_with_serialized() {
    let input = CommandInput::with_serialized(
        ExportInput { project_id: "p1".to_string(), format: "pdf".to_string() },
        b"p1|pdf".to_vec(),
    );
    assert!(input.has_data());
    assert!(input.has_serialized());
    assert_eq!(input.serialized(), Some(&b"p1|pdf"[..]));
}

#[test]
fn command_input_from_bytes() {
    let input = CommandInput::from_bytes(
        "serialization_test::ExportInput",
        b"proj_99|png".to_vec(),
    );
    assert!(!input.has_data());
    assert!(input.has_serialized());
    assert_eq!(input.type_name(), "serialization_test::ExportInput");
}

#[test]
fn command_input_ensure_serialized() {
    let registry = SerializationRegistry::new();
    registry.register(Box::new(ExportInputSerializer));

    let mut input = CommandInput::new(ExportInput {
        project_id: "p2".to_string(),
        format: "docx".to_string(),
    });
    assert!(!input.has_serialized());

    input.ensure_serialized(&registry).unwrap();
    assert!(input.has_serialized());
    assert_eq!(input.serialized(), Some(&b"p2|docx"[..]));
}

#[test]
fn command_input_ensure_deserialized() {
    let registry = SerializationRegistry::new();
    registry.register(Box::new(ExportInputSerializer));

    let mut input = CommandInput::from_bytes(
        "serialization_test::ExportInput",
        b"p3|xlsx".to_vec(),
    );
    assert!(!input.has_data());

    input.ensure_deserialized(&registry).unwrap();
    assert!(input.has_data());
    let data = input.get::<ExportInput>().unwrap();
    assert_eq!(data.project_id, "p3");
    assert_eq!(data.format, "xlsx");
}

#[test]
fn command_input_ensure_serialized_is_idempotent() {
    let registry = SerializationRegistry::new();
    registry.register(Box::new(ExportInputSerializer));

    let mut input = CommandInput::with_serialized(
        ExportInput { project_id: "p4".to_string(), format: "csv".to_string() },
        b"existing".to_vec(),
    );

    // Already has serialized — should not re-serialize
    input.ensure_serialized(&registry).unwrap();
    assert_eq!(input.serialized(), Some(&b"existing"[..]));
}

// --- TaskInput serialization tests ---

#[test]
fn task_input_with_serialized() {
    let input = TaskInput::with_serialized(
        ExportInput { project_id: "t1".to_string(), format: "pdf".to_string() },
        b"t1|pdf".to_vec(),
    );
    assert!(input.has_data());
    assert!(input.has_serialized());
}

#[test]
fn task_input_from_bytes() {
    let input = TaskInput::from_bytes(
        "serialization_test::ExportInput",
        b"t2|png".to_vec(),
    );
    assert!(!input.has_data());
    assert!(input.has_serialized());
}

#[test]
fn task_input_ensure_serialized() {
    let registry = SerializationRegistry::new();
    registry.register(Box::new(ExportInputSerializer));

    let mut input = TaskInput::new(ExportInput {
        project_id: "t3".to_string(),
        format: "docx".to_string(),
    });
    input.ensure_serialized(&registry).unwrap();
    assert!(input.has_serialized());
}

#[test]
fn task_input_ensure_deserialized() {
    let registry = SerializationRegistry::new();
    registry.register(Box::new(ExportInputSerializer));

    let mut input = TaskInput::from_bytes(
        "serialization_test::ExportInput",
        b"t4|xlsx".to_vec(),
    );
    input.ensure_deserialized(&registry).unwrap();
    assert!(input.has_data());
    let data = input.get::<ExportInput>().unwrap();
    assert_eq!(data.project_id, "t4");
}

// --- Event serialization tests ---

#[test]
fn event_with_serialized_payload() {
    let event = Event::new("test.event")
        .with_payload_and_serialized(
            ExportInput { project_id: "e1".to_string(), format: "pdf".to_string() },
            b"e1|pdf".to_vec(),
        );
    assert!(event.has_payload());
    assert!(event.has_serialized_payload());
    assert_eq!(event.serialized_payload(), Some(&b"e1|pdf"[..]));
}

#[test]
fn event_from_serialized() {
    let event = Event::from_serialized(
        "test.event",
        "serialization_test::ExportInput",
        b"e2|png".to_vec(),
    );
    assert!(!event.has_payload());
    assert!(event.has_serialized_payload());
    assert_eq!(event.payload_type_name(), "serialization_test::ExportInput");
}

#[test]
fn event_ensure_serialized() {
    let registry = SerializationRegistry::new();
    registry.register(Box::new(ExportInputSerializer));

    let mut event = Event::new("test.event").with_payload(ExportInput {
        project_id: "e3".to_string(),
        format: "docx".to_string(),
    });
    assert!(!event.has_serialized_payload());

    event.ensure_serialized(&registry).unwrap();
    assert!(event.has_serialized_payload());
}

#[test]
fn event_ensure_deserialized() {
    let registry = SerializationRegistry::new();
    registry.register(Box::new(ExportInputSerializer));

    let mut event = Event::from_serialized(
        "test.event",
        "serialization_test::ExportInput",
        b"e4|xlsx".to_vec(),
    );
    assert!(!event.has_payload());

    event.ensure_deserialized(&registry).unwrap();
    assert!(event.has_payload());
    let data = event.payload::<ExportInput>().unwrap();
    assert_eq!(data.project_id, "e4");
}

// --- HistoryStore with backend tests ---

#[test]
fn history_store_with_backend() {
    use app_shell::app_engine::MemoryStorageBackend;

    let backend = Box::new(MemoryStorageBackend::new());
    let store = HistoryStore::with_backend(backend);
    assert!(store.has_backend());
    assert!(store.is_empty());
}

#[test]
fn history_store_without_backend() {
    let store = HistoryStore::new();
    assert!(!store.has_backend());
}

#[test]
fn history_store_append_with_backend() {
    use app_shell::app_engine::MemoryStorageBackend;

    let backend = Box::new(MemoryStorageBackend::new());
    let store = HistoryStore::with_backend(backend);

    let _id = store.append(
        HistoryEntry::new("test.operation").undoable(),
    );
    assert_eq!(store.entry_count(), 1);
    assert_eq!(store.active_count(), 1);
    assert_eq!(store.position(), 1);

    let current = store.current().unwrap();
    assert_eq!(current.1, "test.operation");
    assert!(current.2); // undoable
}

#[test]
fn history_store_undo_redo_with_backend() {
    use app_shell::app_engine::MemoryStorageBackend;

    let backend = Box::new(MemoryStorageBackend::new());
    let store = HistoryStore::with_backend(backend);

    store.append(HistoryEntry::new("op.a").undoable());
    store.append(HistoryEntry::new("op.b").undoable());

    assert_eq!(store.position(), 2);

    let result = store.undo();
    assert_eq!(result.count(), 1);
    assert_eq!(store.position(), 1);

    let result = store.redo();
    assert_eq!(result.count(), 1);
    assert_eq!(store.position(), 2);
}

// --- SerializationError tests ---

#[test]
fn serialization_error_display() {
    let e = SerializationError::NoSerializer { type_name: "Foo".to_string() };
    assert!(e.to_string().contains("Foo"));
}

#[test]
fn serialization_error_implements_std_error() {
    fn assert_std_error<E: std::error::Error>(_: E) {}
    assert_std_error(SerializationError::NoSerializer { type_name: "x".to_string() });
}

// --- Integration: serialize for IPC ---

#[test]
fn command_input_roundtrip_for_ipc() {
    let registry = SerializationRegistry::new();
    registry.register(Box::new(ExportInputSerializer));

    // Sender creates input with typed data
    let mut input = CommandInput::new(ExportInput {
        project_id: "ipc_test".to_string(),
        format: "json".to_string(),
    });

    // Serialize for IPC
    input.ensure_serialized(&registry).unwrap();
    let type_name = input.type_name();
    let bytes = input.serialized().unwrap().to_vec();

    // Receiver reconstructs from bytes
    let mut received = CommandInput::from_bytes(type_name, bytes);
    assert!(!received.has_data());

    // Deserialize back to typed data
    received.ensure_deserialized(&registry).unwrap();
    let data = received.get::<ExportInput>().unwrap();
    assert_eq!(data.project_id, "ipc_test");
    assert_eq!(data.format, "json");
}

// --- Integration: event for IPC ---

#[test]
fn event_roundtrip_for_ipc() {
    let registry = SerializationRegistry::new();
    registry.register(Box::new(ExportInputSerializer));

    // Sender creates event with typed payload
    let mut event = Event::new("document.exported").with_payload(ExportInput {
        project_id: "doc_42".to_string(),
        format: "pdf".to_string(),
    });

    // Serialize for IPC
    event.ensure_serialized(&registry).unwrap();
    let type_name = event.payload_type_name();
    let bytes = event.serialized_payload().unwrap().to_vec();

    // Receiver reconstructs from bytes
    let mut received = Event::from_serialized("document.exported", type_name, bytes);
    assert!(!received.has_payload());

    // Deserialize
    received.ensure_deserialized(&registry).unwrap();
    let payload = received.payload::<ExportInput>().unwrap();
    assert_eq!(payload.project_id, "doc_42");
    assert_eq!(payload.format, "pdf");
}