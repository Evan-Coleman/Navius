//! Plugin System Configuration
//!
//! This module contains configuration utilities for the Plugin System example.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

use navius_core::error::{Error, Result};
use tracing::info;

/// ConfigSource represents a source of configuration values
pub trait ConfigSource {
    /// Get a configuration value by key
    fn get(&self, key: &str) -> Option<String>;

    /// Set a configuration value
    fn set(&mut self, key: &str, value: &str) -> Result<()>;

    /// Get all keys with a given prefix
    fn get_keys_with_prefix(&self, prefix: &str) -> Vec<String>;
}

/// EnvironmentConfigSource provides configuration from environment variables
pub struct EnvironmentConfigSource {
    prefix: String,
}

impl EnvironmentConfigSource {
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
        }
    }

    fn env_key(&self, key: &str) -> String {
        format!("{}_{}", self.prefix, key.replace('.', "_").to_uppercase())
    }
}

impl ConfigSource for EnvironmentConfigSource {
    fn get(&self, key: &str) -> Option<String> {
        let env_key = self.env_key(key);
        env::var(&env_key).ok()
    }

    fn set(&mut self, key: &str, value: &str) -> Result<()> {
        let env_key = self.env_key(key);
        env::set_var(env_key, value);
        Ok(())
    }

    fn get_keys_with_prefix(&self, prefix: &str) -> Vec<String> {
        let env_prefix = self.env_key(prefix);
        env::vars()
            .filter_map(|(k, _)| {
                if k.starts_with(&env_prefix) {
                    let raw_key = k
                        .strip_prefix(&self.prefix)
                        .unwrap_or(&k)
                        .trim_start_matches('_')
                        .to_lowercase()
                        .replace('_', ".");
                    Some(raw_key)
                } else {
                    None
                }
            })
            .collect()
    }
}

/// FileConfigSource provides configuration from a file
pub struct FileConfigSource {
    values: HashMap<String, String>,
    path: String,
}

impl FileConfigSource {
    pub fn new(path: &str) -> Result<Self> {
        let mut source = Self {
            values: HashMap::new(),
            path: path.to_string(),
        };

        source.load()?;
        Ok(source)
    }

    pub fn load(&mut self) -> Result<()> {
        info!("Loading configuration from file: {}", self.path);

        if !Path::new(&self.path).exists() {
            return Err(Error::new(&format!(
                "Configuration file not found: {}",
                self.path
            )));
        }

        let content = fs::read_to_string(&self.path)
            .map_err(|e| Error::new(&format!("Failed to read configuration file: {}", e)))?;

        // Very simple parsing - just key=value pairs
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                self.values
                    .insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        Ok(())
    }
}

impl ConfigSource for FileConfigSource {
    fn get(&self, key: &str) -> Option<String> {
        self.values.get(key).cloned()
    }

    fn set(&mut self, key: &str, value: &str) -> Result<()> {
        self.values.insert(key.to_string(), value.to_string());

        // In a real implementation, we might save back to the file
        // For this example, we'll just update the in-memory values

        Ok(())
    }

    fn get_keys_with_prefix(&self, prefix: &str) -> Vec<String> {
        self.values
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect()
    }
}

/// MemoryConfigSource provides configuration from memory
pub struct MemoryConfigSource {
    values: HashMap<String, String>,
}

impl MemoryConfigSource {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn with_defaults() -> Self {
        let mut source = Self::new();
        source.set("app.name", "Plugin System Example").unwrap();
        source.set("app.version", "0.1.0").unwrap();
        source.set("server.port", "8080").unwrap();
        source
    }
}

impl ConfigSource for MemoryConfigSource {
    fn get(&self, key: &str) -> Option<String> {
        self.values.get(key).cloned()
    }

    fn set(&mut self, key: &str, value: &str) -> Result<()> {
        self.values.insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn get_keys_with_prefix(&self, prefix: &str) -> Vec<String> {
        self.values
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect()
    }
}

/// CompositeConfigSource combines multiple configuration sources
pub struct CompositeConfigSource {
    sources: Vec<Box<dyn ConfigSource>>,
}

impl CompositeConfigSource {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    pub fn add_source(&mut self, source: Box<dyn ConfigSource>) {
        self.sources.push(source);
    }
}

impl ConfigSource for CompositeConfigSource {
    fn get(&self, key: &str) -> Option<String> {
        // Check sources in reverse order (later sources override earlier ones)
        for source in self.sources.iter().rev() {
            if let Some(value) = source.get(key) {
                return Some(value);
            }
        }
        None
    }

    fn set(&mut self, key: &str, value: &str) -> Result<()> {
        // Set in the first source (if any)
        if let Some(source) = self.sources.first_mut() {
            source.set(key, value)
        } else {
            Err(Error::new("No configuration sources available"))
        }
    }

    fn get_keys_with_prefix(&self, prefix: &str) -> Vec<String> {
        // Collect keys from all sources, removing duplicates
        let mut keys = Vec::new();
        let mut seen = HashMap::new();

        for source in self.sources.iter() {
            for key in source.get_keys_with_prefix(prefix) {
                if !seen.contains_key(&key) {
                    seen.insert(key.clone(), true);
                    keys.push(key);
                }
            }
        }

        keys
    }
}
