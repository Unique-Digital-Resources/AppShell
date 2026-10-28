//! `CommandDefinition` — the metadata describing a command capability.
//!
//! A definition is the *contract* describing what a command does and what it
//! accepts/returns. It is intentionally lightweight in Phase 3. Later phases
//! can add input/output schemas, capabilities, and availability conditions.

//! `CommandDefinition` — metadata describing a command capability.
//!
//! Phase 20: now supports version and input/output schemas.

use std::collections::HashMap;
use std::fmt;

use crate::app_engine::metadata::payload_schema::PayloadSchema;
use crate::app_engine::metadata::version::Version;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CommandDefinitionId(String);

impl CommandDefinitionId {
    pub fn new(id: impl Into<String>) -> Self { Self(id.into()) }
    pub fn as_str(&self) -> &str { &self.0 }
}

impl From<String> for CommandDefinitionId {
    fn from(s: String) -> Self { Self(s) }
}

impl From<&str> for CommandDefinitionId {
    fn from(s: &str) -> Self { Self(s.to_string()) }
}

impl fmt::Display for CommandDefinitionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(&self.0) }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandDefinition {
    id: CommandDefinitionId,
    name: String,
    description: String,
    metadata: HashMap<String, String>,
    version: Version,
    input_schema: Option<PayloadSchema>,
    output_schema: Option<PayloadSchema>,
}

impl CommandDefinition {
    pub fn new(
        id: impl Into<CommandDefinitionId>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            metadata: HashMap::new(),
            version: Version::default(),
            input_schema: None,
            output_schema: None,
        }
    }

    pub fn id(&self) -> &CommandDefinitionId { &self.id }
    pub fn name(&self) -> &str { &self.name }
    pub fn description(&self) -> &str { &self.description }
    pub fn metadata(&self) -> &HashMap<String, String> { &self.metadata }
    pub fn version(&self) -> &Version { &self.version }
    pub fn input_schema(&self) -> Option<&PayloadSchema> { self.input_schema.as_ref() }
    pub fn output_schema(&self) -> Option<&PayloadSchema> { self.output_schema.as_ref() }

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