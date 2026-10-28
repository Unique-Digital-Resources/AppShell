//! `TaskDefinition` — metadata describing a type of managed work.
//!
//! Phase 20: now supports version and input/output schemas.

use std::collections::HashMap;
use std::fmt;

use crate::app_engine::metadata::payload_schema::PayloadSchema;
use crate::app_engine::metadata::version::Version;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TaskDefinitionId(String);

impl TaskDefinitionId {
    pub fn new(id: impl Into<String>) -> Self { Self(id.into()) }
    pub fn as_str(&self) -> &str { &self.0 }
}

impl From<String> for TaskDefinitionId {
    fn from(s: String) -> Self { Self(s) }
}

impl From<&str> for TaskDefinitionId {
    fn from(s: &str) -> Self { Self(s.to_string()) }
}

impl fmt::Display for TaskDefinitionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(&self.0) }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskDefinition {
    id: TaskDefinitionId,
    name: String,
    description: String,
    supports_pause: bool,
    supports_cancel: bool,
    metadata: HashMap<String, String>,
    version: Version,
    input_schema: Option<PayloadSchema>,
    output_schema: Option<PayloadSchema>,
}

impl TaskDefinition {
    pub fn new(
        id: impl Into<TaskDefinitionId>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            supports_pause: false,
            supports_cancel: false,
            metadata: HashMap::new(),
            version: Version::default(),
            input_schema: None,
            output_schema: None,
        }
    }

    pub fn id(&self) -> &TaskDefinitionId { &self.id }
    pub fn name(&self) -> &str { &self.name }
    pub fn description(&self) -> &str { &self.description }
    pub fn supports_pause(&self) -> bool { self.supports_pause }
    pub fn supports_cancel(&self) -> bool { self.supports_cancel }
    pub fn metadata(&self) -> &HashMap<String, String> { &self.metadata }
    pub fn version(&self) -> &Version { &self.version }
    pub fn input_schema(&self) -> Option<&PayloadSchema> { self.input_schema.as_ref() }
    pub fn output_schema(&self) -> Option<&PayloadSchema> { self.output_schema.as_ref() }

    pub fn with_pause_support(mut self) -> Self {
        self.supports_pause = true;
        self
    }

    pub fn with_cancel_support(mut self) -> Self {
        self.supports_cancel = true;
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn with_version(mut self, version: Version) -> Self {
        self.version = version;
        self
    }

    pub fn with_input_schema(mut self, schema: PayloadSchema) -> Self {
        self.input_schema = Some(schema);
        self
    }

    pub fn with_output_schema(mut self, schema: PayloadSchema) -> Self {
        self.output_schema = Some(schema);
        self
    }

    pub fn is_compatible_with(&self, other: &Version) -> bool {
        self.version.is_compatible_with(other)
    }
}