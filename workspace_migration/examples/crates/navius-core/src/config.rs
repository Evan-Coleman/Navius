//! Configuration management for the Navius framework.
//!
//! This module provides configuration utilities for the Navius framework.

use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use crate::error::{Error, Result};

/// Configuration for the Navius framework
#[derive(Debug, Clone)]
pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    /// Create a new empty configuration
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Set a configuration value
    pub fn set<K: Into<String>, V: ToString>(&mut self, key: K, value: V) -> Result<()> {
        self.values.insert(key.into(), value.to_string());
        Ok(())
    }

    /// Get a configuration value
    pub fn get<T: FromStr>(&self, key: &str) -> Result<T>
    where
        T::Err: fmt::Display,
    {
        let value = self
            .values
            .get(key)
            .ok_or_else(|| Error::new(&format!("Configuration key not found: {}", key)))?;

        value
            .parse::<T>()
            .map_err(|e| Error::new(&format!("Failed to parse configuration value: {}", e)))
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
