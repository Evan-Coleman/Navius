//! Plugin System Services
//!
//! This module contains service implementations for the capability traits.

use navius_core::error::{Error, Result};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};
use tracing::{debug, error, info, warn};

use crate::capabilities::{ConfigCapability, LoggingCapability, StorageCapability};

/// Logger Service implementation
pub struct LoggerService {
    name: String,
    level: Mutex<String>,
}

impl LoggerService {
    pub fn new(name: &str, level: &str) -> Self {
        Self {
            name: name.to_string(),
            level: Mutex::new(level.to_string()),
        }
    }
}

impl LoggingCapability for LoggerService {
    fn info(&self, message: &str) {
        info!("[{}] {}", self.name, message);
    }

    fn warn(&self, message: &str) {
        warn!("[{}] {}", self.name, message);
    }

    fn error(&self, message: &str) {
        error!("[{}] {}", self.name, message);
    }

    fn debug(&self, message: &str) {
        debug!("[{}] {}", self.name, message);
    }

    fn set_level(&self, level: &str) -> Result<()> {
        let mut current_level = self.level.lock().unwrap();
        *current_level = level.to_string();
        Ok(())
    }
}

/// Config Service implementation
pub struct ConfigService {
    logger: Arc<dyn LoggingCapability>,
    config: RwLock<HashMap<String, String>>,
}

impl ConfigService {
    pub fn new(logger: Arc<dyn LoggingCapability>) -> Self {
        Self {
            logger,
            config: RwLock::new(HashMap::new()),
        }
    }
}

impl ConfigCapability for ConfigService {
    fn load(&self) -> Result<()> {
        self.logger.info("Loading configuration");

        // Load default configuration values
        let mut config = self.config.write().unwrap();
        config.insert("app.name".to_string(), "Plugin System Example".to_string());
        config.insert("app.version".to_string(), "0.1.0".to_string());
        config.insert("server.port".to_string(), "8080".to_string());
        config.insert(
            "db.url".to_string(),
            "postgres://localhost/navius".to_string(),
        );

        self.logger.info("Configuration loaded");
        Ok(())
    }

    fn get_string(&self, key: &str) -> Result<String> {
        let config = self.config.read().unwrap();
        config
            .get(key)
            .cloned()
            .ok_or_else(|| Error::new(&format!("Configuration key '{}' not found", key)))
    }

    fn get_int(&self, key: &str) -> Result<i64> {
        let value = self.get_string(key)?;
        value
            .parse::<i64>()
            .map_err(|e| Error::new(&format!("Failed to parse '{}' as integer: {}", value, e)))
    }

    fn get_bool(&self, key: &str) -> Result<bool> {
        let value = self.get_string(key)?;
        match value.to_lowercase().as_str() {
            "true" | "yes" | "1" => Ok(true),
            "false" | "no" | "0" => Ok(false),
            _ => Err(Error::new(&format!(
                "Failed to parse '{}' as boolean",
                value
            ))),
        }
    }

    fn get_map(&self, key: &str) -> Result<HashMap<String, String>> {
        let config = self.config.read().unwrap();
        let prefix = format!("{}.", key);

        let mut result = HashMap::new();
        for (k, v) in config.iter() {
            if k.starts_with(&prefix) {
                let key_suffix = k.strip_prefix(&prefix).unwrap();
                result.insert(key_suffix.to_string(), v.clone());
            }
        }

        if result.is_empty() {
            return Err(Error::new(&format!("No configuration found for '{}'", key)));
        }

        Ok(result)
    }

    fn set(&self, key: &str, value: &str) -> Result<()> {
        let mut config = self.config.write().unwrap();
        config.insert(key.to_string(), value.to_string());
        Ok(())
    }
}

/// Storage Service implementation
pub struct StorageService {
    logger: Arc<dyn LoggingCapability>,
    config: Arc<dyn ConfigCapability>,
    data: RwLock<HashMap<String, Vec<u8>>>,
    initialized: Mutex<bool>,
}

impl StorageService {
    pub fn new(logger: Arc<dyn LoggingCapability>, config: Arc<dyn ConfigCapability>) -> Self {
        Self {
            logger,
            config,
            data: RwLock::new(HashMap::new()),
            initialized: Mutex::new(false),
        }
    }
}

#[async_trait::async_trait]
impl StorageCapability for StorageService {
    async fn initialize(&self) -> Result<()> {
        self.logger.info("Initializing storage service");

        let mut initialized = self.initialized.lock().unwrap();
        if *initialized {
            self.logger.warn("Storage service already initialized");
            return Ok(());
        }

        // For a real storage service, we would connect to the database here
        // For this example, we just mark it as initialized
        *initialized = true;

        self.logger.info("Storage service initialized");
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        self.logger.info("Shutting down storage service");

        let mut initialized = self.initialized.lock().unwrap();
        if !*initialized {
            self.logger.warn("Storage service not initialized");
            return Ok(());
        }

        // For a real storage service, we would disconnect from the database here
        // For this example, we just mark it as uninitialized
        *initialized = false;

        self.logger.info("Storage service shut down");
        Ok(())
    }

    async fn store(&self, key: &str, value: &[u8]) -> Result<()> {
        self.logger
            .debug(&format!("Storing value for key '{}'", key));

        let mut data = self.data.write().unwrap();
        data.insert(key.to_string(), value.to_vec());

        Ok(())
    }

    async fn retrieve(&self, key: &str) -> Result<Vec<u8>> {
        self.logger
            .debug(&format!("Retrieving value for key '{}'", key));

        let data = self.data.read().unwrap();
        data.get(key)
            .cloned()
            .ok_or_else(|| Error::new(&format!("Key '{}' not found", key)))
    }

    async fn delete(&self, key: &str) -> Result<()> {
        self.logger
            .debug(&format!("Deleting value for key '{}'", key));

        let mut data = self.data.write().unwrap();
        if data.remove(key).is_none() {
            return Err(Error::new(&format!("Key '{}' not found", key)));
        }

        Ok(())
    }

    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>> {
        self.logger
            .debug(&format!("Listing keys with prefix '{}'", prefix));

        let data = self.data.read().unwrap();
        let keys = data
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();

        Ok(keys)
    }
}
