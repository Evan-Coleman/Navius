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
    use navius_test::error::{TestResult, assert_contains, assert_eq, assert_false, assert_true};

    #[test]
    fn get_set_config() -> TestResult<()> {
        let mut config = Config::new();
        config.set("app.name", "test-app")?;
        config.set("app.port", "8080")?;

        let app_name = config.get::<String>("app.name")?;
        assert_eq(
            app_name,
            "test-app".to_string(),
            "Config should return the correct app name",
        )?;

        let app_port = config.get::<u16>("app.port")?;
        assert_eq(
            app_port,
            8080,
            "Config should return the correct port number",
        )?;

        Ok(())
    }

    #[test]
    fn key_not_found() -> TestResult<()> {
        let config = Config::new();
        let result = config.get::<String>("app.name");

        assert_true(
            result.is_err(),
            "Getting a non-existent key should return an error",
        )?;

        Ok(())
    }

    #[test]
    fn has_key() -> TestResult<()> {
        let mut config = Config::new();
        config.set("app.name", "test-app")?;

        assert_true(
            config.has("app.name"),
            "Config should have the key that was set",
        )?;

        assert_false(
            config.has("app.port"),
            "Config should not have keys that were not set",
        )?;

        Ok(())
    }

    #[test]
    fn config_keys() -> TestResult<()> {
        let mut config = Config::new();
        config.set("app.name", "test-app")?;
        config.set("app.port", "8080")?;

        let keys = config.keys();

        assert_eq(
            keys.len(),
            2,
            "Config should have the correct number of keys",
        )?;

        assert_contains(&keys, &"app.name", "Keys should contain app.name")?;

        assert_contains(&keys, &"app.port", "Keys should contain app.port")?;

        Ok(())
    }
}
