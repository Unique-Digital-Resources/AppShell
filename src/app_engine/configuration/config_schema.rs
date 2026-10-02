//! `ConfigSchema` — rules describing valid configuration.
//!
//! Separates "actual values" (Config) from "rules for values" (ConfigSchema).

use std::collections::HashMap;

use crate::app_engine::configuration::config::Config;
use crate::app_engine::errors::configuration_error::ConfigurationError;

/// Defines a single configuration property.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigPropertyDefinition {
    name: String,
    description: String,
    default: Option<String>,
    required: bool,
    min_value: Option<String>,
    max_value: Option<String>,
}

impl ConfigPropertyDefinition {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            default: None,
            required: false,
            min_value: None,
            max_value: None,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn default(&self) -> Option<&str> {
        self.default.as_deref()
    }

    pub fn is_required(&self) -> bool {
        self.required
    }

    pub fn min_value(&self) -> Option<&str> {
        self.min_value.as_deref()
    }

    pub fn max_value(&self) -> Option<&str> {
        self.max_value.as_deref()
    }

    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn with_min(mut self, min: impl Into<String>) -> Self {
        self.min_value = Some(min.into());
        self
    }

    pub fn with_max(mut self, max: impl Into<String>) -> Self {
        self.max_value = Some(max.into());
        self
    }
}

/// Schema describing valid configuration properties.
#[derive(Debug, Clone, Default)]
pub struct ConfigSchema {
    properties: HashMap<String, ConfigPropertyDefinition>,
}

impl ConfigSchema {
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
        }
    }

    pub fn with_property(mut self, def: ConfigPropertyDefinition) -> Self {
        self.properties.insert(def.name.clone(), def);
        self
    }

    pub fn define(&mut self, def: ConfigPropertyDefinition) {
        self.properties.insert(def.name.clone(), def);
    }

    pub fn get(&self, name: &str) -> Option<&ConfigPropertyDefinition> {
        self.properties.get(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.properties.contains_key(name)
    }

    pub fn property_count(&self) -> usize {
        self.properties.len()
    }

    pub fn property_names(&self) -> Vec<&str> {
        self.properties.keys().map(|s| s.as_str()).collect()
    }

    /// Generate a Config with all default values.
    pub fn defaults(&self) -> Config {
        let mut config = Config::new();
        for (name, def) in &self.properties {
            if let Some(default) = &def.default {
                config.set(name.clone(), default.clone());
            }
        }
        config
    }

    /// Validate a Config against this schema.
    pub fn validate(&self, config: &Config) -> Result<(), ConfigurationError> {
        for (name, def) in &self.properties {
            match config.get(name) {
                Some(value) => {
                    // Check min/max for numeric values
                    if let Some(min) = &def.min_value {
                        if let (Ok(val), Ok(min_val)) =
                            (value.parse::<i64>(), min.parse::<i64>())
                        {
                            if val < min_val {
                                return Err(ConfigurationError::ValidationFailed {
                                    key: name.clone(),
                                    reason: format!("value {} is below minimum {}", val, min_val),
                                });
                            }
                        }
                    }
                    if let Some(max) = &def.max_value {
                        if let (Ok(val), Ok(max_val)) =
                            (value.parse::<i64>(), max.parse::<i64>())
                        {
                            if val > max_val {
                                return Err(ConfigurationError::ValidationFailed {
                                    key: name.clone(),
                                    reason: format!("value {} is above maximum {}", val, max_val),
                                });
                            }
                        }
                    }
                }
                None => {
                    if def.required {
                        return Err(ConfigurationError::ValidationFailed {
                            key: name.clone(),
                            reason: "required property is missing".to_string(),
                        });
                    }
                }
            }
        }
        Ok(())
    }
}