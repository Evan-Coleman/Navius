//! Configuration management for the Navius framework.
//!
//! This module provides configuration utilities for the Navius framework.

use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::str::FromStr;

use crate::error::{Error, Result};
use serde::de::DeserializeOwned;

/// Configuration for the Navius framework
#[derive(Debug, Clone)]
pub struct Config {
    /// Values loaded from configuration files
    values: HashMap<String, String>,
}

impl Config {
    /// Create a new empty configuration
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Create a default configuration
    pub fn default() -> Self {
        Self::new()
    }

    /// Load configuration from a file
    pub fn from_file(_path: &Path) -> Result<Self> {
        // This is a simplified implementation
        Ok(Self::new())
    }

    /// Get a configuration value by key with type conversion
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<T> {
        let value = self.values.get(key).ok_or_else(|| {
            Error::configuration(&format!("Configuration key not found: {}", key))
        })?;

        // For simple implementation, we'll just parse the string
        // This would normally use serde to deserialize
        match serde_json::from_str::<T>(value) {
            Ok(value) => Ok(value),
            Err(e) => Err(Error::configuration(&format!(
                "Failed to parse configuration value: {}",
                e
            ))),
        }
    }

    /// Set a configuration value
    pub fn set<T: serde::Serialize>(&mut self, key: &str, value: T) -> Result<()> {
        let value_str = serde_json::to_string(&value)
            .map_err(|e| Error::configuration(&format!("Failed to serialize value: {}", e)))?;

        self.values.insert(key.to_string(), value_str);
        Ok(())
    }

    /// Check if the configuration is empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Check if a configuration key exists
    pub fn has(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    /// Get all configuration keys
    pub fn keys(&self) -> Vec<&str> {
        self.values.keys().map(|k| k.as_str()).collect()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_set_config() {
        let mut config = Config::new();
        config.set("app.name", "test-app").unwrap();
        config.set("app.port", "8080").unwrap();

        assert_eq!(config.get::<String>("app.name").unwrap(), "test-app");
        assert_eq!(config.get::<u16>("app.port").unwrap(), 8080);
    }

    #[test]
    fn key_not_found() {
        let config = Config::new();
        let result = config.get::<String>("app.name");
        assert!(result.is_err());
    }

    #[test]
    fn has_key() {
        let mut config = Config::new();
        config.set("app.name", "test-app").unwrap();

        assert!(config.has("app.name"));
        assert!(!config.has("app.port"));
    }

    #[test]
    fn config_keys() {
        let mut config = Config::new();
        config.set("app.name", "test-app").unwrap();
        config.set("app.port", "8080").unwrap();

        let keys = config.keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"app.name"));
        assert!(keys.contains(&"app.port"));
    }
}
