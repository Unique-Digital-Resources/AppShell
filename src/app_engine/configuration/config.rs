//! `Config` — application configuration data.
//!
//! Configuration is what the application is configured to do.
//! State is what the application is currently doing.
//! These should remain separate.
//!
//! The App Engine provides the generic key-value store; domain-specific
//! configuration belongs in the Domain Engine.

use std::collections::HashMap;
use std::fmt;

/// Generic application configuration stored as key-value pairs.
#[derive(Debug, Clone, Default)]
pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|s| s.as_str())
    }

    /// Get a value parsed as the given type.
    pub fn get_as<T>(&self, key: &str) -> Option<T>
    where
        T: std::str::FromStr,
    {
        self.values.get(key).and_then(|s| s.parse::<T>().ok())
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        self.values.get(key).cloned()
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.values.get(key).and_then(|s| match s.as_str() {
            "true" | "1" | "yes" => Some(true),
            "false" | "0" | "no" => Some(false),
            _ => None,
        })
    }

    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.get_as::<i64>(key)
    }

    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.get_as::<f64>(key)
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.values.insert(key.into(), value.into());
    }

    pub fn set_if_absent(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.values.entry(key.into()).or_insert(value.into());
    }

    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.values.remove(key)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.values.keys().map(|s| s.as_str())
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn merge(&mut self, other: &Config) {
        for (k, v) in &other.values {
            self.values.insert(k.clone(), v.clone());
        }
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Config({} keys)", self.values.len())
    }
}