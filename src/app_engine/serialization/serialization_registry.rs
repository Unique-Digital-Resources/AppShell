//! `SerializationRegistry` — stores serializers by type name.
//!
//! The Domain Engine registers serializers for its payload types.
//! The App Engine uses the registry to serialize/deserialize type-erased
//! payloads in `CommandInput`, `TaskInput`, `Event`, and `HistoryEntry`.

use std::any::Any;
use std::collections::HashMap;
use std::sync::RwLock;

use crate::app_engine::errors::serialization_error::SerializationError;
use crate::app_engine::serialization::serializer::Serializer;

pub struct SerializationRegistry {
    serializers: RwLock<HashMap<String, Box<dyn Serializer>>>,
}

impl SerializationRegistry {
    pub fn new() -> Self {
        Self {
            serializers: RwLock::new(HashMap::new()),
        }
    }

    /// Register a serializer for a type.
    pub fn register(&self, serializer: Box<dyn Serializer>) {
        let type_name = serializer.type_name().to_string();
        self.serializers.write().unwrap().insert(type_name, serializer);
    }

    /// Check if a serializer is registered for the given type name.
    pub fn has_serializer(&self, type_name: &str) -> bool {
        self.serializers.read().unwrap().contains_key(type_name)
    }

    /// Serialize a value using the registered serializer for its type name.
    pub fn serialize(
        &self,
        data: &dyn Any,
        type_name: &str,
    ) -> Result<Vec<u8>, SerializationError> {
        let serializers = self.serializers.read().unwrap();
        let serializer = serializers.get(type_name).ok_or_else(|| {
            SerializationError::NoSerializer {
                type_name: type_name.to_string(),
            }
        })?;
        serializer.serialize(data)
    }

    /// Deserialize bytes using the registered serializer for the type name.
    pub fn deserialize(
        &self,
        bytes: &[u8],
        type_name: &str,
    ) -> Result<Box<dyn Any + Send>, SerializationError> {
        let serializers = self.serializers.read().unwrap();
        let serializer = serializers.get(type_name).ok_or_else(|| {
            SerializationError::NoSerializer {
                type_name: type_name.to_string(),
            }
        })?;
        serializer.deserialize(bytes)
    }

    /// List all registered type names.
    pub fn registered_types(&self) -> Vec<String> {
        self.serializers.read().unwrap().keys().cloned().collect()
    }

    /// Number of registered serializers.
    pub fn len(&self) -> usize {
        self.serializers.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.serializers.read().unwrap().is_empty()
    }
}

impl Default for SerializationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for SerializationRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SerializationRegistry")
            .field("count", &self.serializers.read().unwrap().len())
            .finish()
    }
}