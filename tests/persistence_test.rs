use app_shell::app_engine::{
    Config, ConfigPropertyDefinition, ConfigSchema, ConfigStore,
    FileStorageBackend, MemoryStorageBackend, PersistenceError, StorageBackend,
};
use std::collections::HashMap;

// --- MemoryStorageBackend tests ---

#[test]
fn memory_backend_starts_empty() {
    let backend = MemoryStorageBackend::new();
    assert_eq!(backend.keys().unwrap().len(), 0);
}

#[test]
fn memory_backend_save_and_load() {
    let backend = MemoryStorageBackend::new();
    backend.save("key1", "value1").unwrap();
    backend.save("key2", "value2").unwrap();

    assert_eq!(backend.load("key1").unwrap(), Some("value1".to_string()));
    assert_eq!(backend.load("key2").unwrap(), Some("value2".to_string()));
    assert_eq!(backend.load("missing").unwrap(), None);
}

#[test]
fn memory_backend_exists() {
    let backend = MemoryStorageBackend::new();
    backend.save("key", "value").unwrap();
    assert!(backend.exists("key"));
    assert!(!backend.exists("missing"));
}

#[test]
fn memory_backend_remove() {
    let backend = MemoryStorageBackend::new();
    backend.save("key", "value").unwrap();
    assert!(backend.exists("key"));

    backend.remove("key").unwrap();
    assert!(!backend.exists("key"));
    assert_eq!(backend.load("key").unwrap(), None);
}

#[test]
fn memory_backend_keys() {
    let backend = MemoryStorageBackend::new();
    backend.save("a", "1").unwrap();
    backend.save("b", "2").unwrap();
    backend.save("c", "3").unwrap();

    let mut keys = backend.keys().unwrap();
    keys.sort();
    assert_eq!(keys, vec!["a", "b", "c"]);
}

#[test]
fn memory_backend_with_data() {
    let mut data = HashMap::new();
    data.insert("preloaded".to_string(), "value".to_string());
    let backend = MemoryStorageBackend::with_data(data);

    assert_eq!(backend.load("preloaded").unwrap(), Some("value".to_string()));
}

// --- FileStorageBackend tests ---

#[test]
fn file_backend_save_and_load() {
    let temp_dir = std::env::temp_dir().join("app_engine_test_file_backend_1");
    let _ = std::fs::remove_dir_all(&temp_dir);

    {
        let backend = FileStorageBackend::new(&temp_dir).unwrap();
        backend.save("key1", "value1").unwrap();
        backend.save("key2", "value2").unwrap();

        assert_eq!(backend.load("key1").unwrap(), Some("value1".to_string()));
        assert_eq!(backend.load("key2").unwrap(), Some("value2".to_string()));
    }

    // Create a new backend from the same directory — should load from files
    let backend2 = FileStorageBackend::new(&temp_dir).unwrap();
    assert_eq!(backend2.load("key1").unwrap(), Some("value1".to_string()));
    assert_eq!(backend2.load("key2").unwrap(), Some("value2".to_string()));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn file_backend_exists() {
    let temp_dir = std::env::temp_dir().join("app_engine_test_file_backend_2");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let backend = FileStorageBackend::new(&temp_dir).unwrap();
    backend.save("key", "value").unwrap();
    assert!(backend.exists("key"));
    assert!(!backend.exists("missing"));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn file_backend_remove() {
    let temp_dir = std::env::temp_dir().join("app_engine_test_file_backend_3");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let backend = FileStorageBackend::new(&temp_dir).unwrap();
    backend.save("key", "value").unwrap();
    assert!(backend.exists("key"));

    backend.remove("key").unwrap();
    assert!(!backend.exists("key"));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn file_backend_keys() {
    let temp_dir = std::env::temp_dir().join("app_engine_test_file_backend_4");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let backend = FileStorageBackend::new(&temp_dir).unwrap();
    backend.save("a", "1").unwrap();
    backend.save("b", "2").unwrap();

    let keys = backend.keys().unwrap();
    assert_eq!(keys.len(), 2);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn file_backend_survives_recreation() {
    let temp_dir = std::env::temp_dir().join("app_engine_test_file_backend_5");
    let _ = std::fs::remove_dir_all(&temp_dir);

    // First backend writes
    {
        let backend = FileStorageBackend::new(&temp_dir).unwrap();
        backend.save("persistent", "data").unwrap();
    }

    // Second backend reads from the same directory
    {
        let backend = FileStorageBackend::new(&temp_dir).unwrap();
        assert_eq!(backend.load("persistent").unwrap(), Some("data".to_string()));
    }

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn file_backend_sanitizes_keys() {
    let temp_dir = std::env::temp_dir().join("app_engine_test_file_backend_6");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let backend = FileStorageBackend::new(&temp_dir).unwrap();
    backend.save("app/name", "MyApp").unwrap();

    // The key with slashes is sanitized internally
    assert!(backend.exists("app/name"));
    assert_eq!(backend.load("app/name").unwrap(), Some("MyApp".to_string()));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

// --- ConfigStore with backend tests ---

#[test]
fn config_store_with_memory_backend() {
    let schema = ConfigSchema::new()
        .with_property(
            ConfigPropertyDefinition::new("app.name", "App name")
                .with_default("Default"),
        )
        .with_property(
            ConfigPropertyDefinition::new("worker_count", "Workers")
                .with_default("4"),
        );

    let backend = Box::new(MemoryStorageBackend::new());
    let store = ConfigStore::with_backend(schema, backend);

    // Defaults loaded
    assert_eq!(store.get("app.name").as_deref(), Some("Default"));
    assert_eq!(store.get("worker_count").as_deref(), Some("4"));

    // Set persists to backend
    store.set("app.name", "MyApp").unwrap();
    assert_eq!(store.get("app.name").as_deref(), Some("MyApp"));

    // Reset reloads from backend (which has the persisted value)
    store.reset();
    assert_eq!(store.get("app.name").as_deref(), Some("MyApp")); // from backend
}

#[test]
fn config_store_with_file_backend_survives_recreation() {
    let temp_dir = std::env::temp_dir().join("app_engine_test_config_1");
    let _ = std::fs::remove_dir_all(&temp_dir);

    let schema = ConfigSchema::new()
        .with_property(
            ConfigPropertyDefinition::new("app.name", "App name")
                .with_default("Default"),
        );

    // First store writes to file
    {
        let backend = Box::new(FileStorageBackend::new(&temp_dir).unwrap());
        let store = ConfigStore::with_backend(schema.clone(), backend);
        store.set("app.name", "PersistedApp").unwrap();
    }

    // Second store reads from the same files
    {
        let backend = Box::new(FileStorageBackend::new(&temp_dir).unwrap());
        let store = ConfigStore::with_backend(schema, backend);
        assert_eq!(store.get("app.name").as_deref(), Some("PersistedApp"));
    }

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn config_store_save_persists_all_keys() {
    let schema = ConfigSchema::new()
        .with_property(
            ConfigPropertyDefinition::new("app.name", "Name")
                .with_default("Default")
                .required(),
        )
        .with_property(
            ConfigPropertyDefinition::new("worker_count", "Workers")
                .with_default("4"),
        );

    let backend = Box::new(MemoryStorageBackend::new());
    let store = ConfigStore::with_backend(schema, backend);

    let mut new_config = Config::new();
    new_config.set("app.name", "NewApp");
    new_config.set("worker_count", "8");
    store.save(new_config).unwrap();

    assert_eq!(store.get("app.name").as_deref(), Some("NewApp"));
    assert_eq!(store.get("worker_count").as_deref(), Some("8"));
}

#[test]
fn config_store_remove_persists() {
    let schema = ConfigSchema::new()
        .with_property(
            ConfigPropertyDefinition::new("app.name", "Name"),
            // NO .with_default() — so reset() doesn't bring it back
        );

    let backend = Box::new(MemoryStorageBackend::new());
    let store = ConfigStore::with_backend(schema, backend);

    // Set a value (persists to backend)
    store.set("app.name", "Changed").unwrap();
    assert!(store.contains("app.name"));

    // Remove (also removes from backend)
    store.remove("app.name");
    assert!(!store.contains("app.name"));

    // Reset reloads from backend — key should still be gone (no default to restore)
    store.reset();
    assert!(!store.contains("app.name"));
}

#[test]
fn config_store_without_backend_works() {
    // Backward compatibility — no backend, in-memory only
    let schema = ConfigSchema::new()
        .with_property(
            ConfigPropertyDefinition::new("key", "Key").with_default("default"),
        );

    let store = ConfigStore::new(schema);
    assert!(!store.has_backend());
    assert_eq!(store.get("key").as_deref(), Some("default"));

    store.set("key", "new").unwrap();
    assert_eq!(store.get("key").as_deref(), Some("new"));

    store.reset();
    assert_eq!(store.get("key").as_deref(), Some("default")); // from schema defaults
}

#[test]
fn config_store_has_backend_flag() {
    let schema = ConfigSchema::new();

    let store1 = ConfigStore::new(schema.clone());
    assert!(!store1.has_backend());

    let store2 = ConfigStore::with_memory_backend(schema);
    assert!(store2.has_backend());
}

// --- PersistenceError tests ---

#[test]
fn persistence_error_display() {
    let e = PersistenceError::IoError {
        reason: "disk full".to_string(),
    };
    assert!(e.to_string().contains("disk full"));

    let e = PersistenceError::BackendNotAvailable;
    assert!(e.to_string().contains("backend"));
}

#[test]
fn persistence_error_implements_std_error() {
    fn assert_std_error<E: std::error::Error>(_: E) {}
    assert_std_error(PersistenceError::BackendNotAvailable);
}

#[test]
fn persistence_error_from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
    let p_err: PersistenceError = io_err.into();
    assert!(matches!(p_err, PersistenceError::IoError { .. }));
}