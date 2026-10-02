//! `PayloadSchema` — describes the expected type of a payload.
//!
//! Used by command/task definitions to declare input/output types.

use std::any::TypeId;
use std::fmt;

#[derive(Debug, Clone)]
pub struct PayloadSchema {
    type_name: &'static str,
    type_id: TypeId,
    description: String,
}

impl PayloadSchema {
    /// Create a schema for a specific type.
    pub fn of<T: 'static>() -> Self {
        Self {
            type_name: std::any::type_name::<T>(),
            type_id: TypeId::of::<T>(),
            description: String::new(),
        }
    }

    /// Create a schema with a description.
    pub fn of_with_description<T: 'static>(description: impl Into<String>) -> Self {
        Self {
            type_name: std::any::type_name::<T>(),
            type_id: TypeId::of::<T>(),
            description: description.into(),
        }
    }

    pub fn type_name(&self) -> &'static str {
        self.type_name
    }

    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    /// Check if a value matches this schema's type.
    pub fn matches<T: 'static>(&self) -> bool {
        TypeId::of::<T>() == self.type_id
    }
}

impl PartialEq for PayloadSchema {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id
    }
}

impl Eq for PayloadSchema {}

impl fmt::Display for PayloadSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PayloadSchema({})", self.type_name)
    }
}