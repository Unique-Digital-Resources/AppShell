//! `Serializer` — trait for serializing/deserializing type-erased payloads.
//!
//! The Domain Engine provides implementations of this trait for its payload
//! types. The App Engine uses the `SerializationRegistry` to look up the
//! correct serializer by type name.

use std::any::Any;

use crate::app_engine::errors::serialization_error::SerializationError;

/// A serializer that can serialize and deserialize a specific type.
///
/// The Domain Engine implements this for its payload types (e.g.,
/// `ExportProjectInput`, `DocumentData`). The type name is used to
/// look up the serializer in the `SerializationRegistry`.
pub trait Serializer: Send + Sync {
    /// The type name this serializer handles (e.g., "ExportProjectInput").
    fn type_name(&self) -> &str;

    /// Serialize a type-erased value to bytes.
    fn serialize(&self, data: &dyn Any) -> Result<Vec<u8>, SerializationError>;

    /// Deserialize bytes back to a type-erased value.
    fn deserialize(&self, bytes: &[u8]) -> Result<Box<dyn Any + Send>, SerializationError>;
}