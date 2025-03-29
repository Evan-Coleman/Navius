//! Configuration management for the Navius framework.
//!
//! This module provides configuration loading and management functionality.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Core configuration structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Application settings
    #[serde(default)]
    pub app: AppConfig,

    /// Server settings
    #[serde(default)]
    pub server: ServerConfig,

    /// Logging settings
    #[serde(default)]
    pub logging: LoggingConfig,
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Application name
    #[serde(default = "default_app_name")]
    pub name: String,

    /// Application version
    #[serde(default = "default_app_version")]
    pub version: String,

    /// Environment name (dev, test, prod)
    #[serde(default = "default_environment")]
    pub environment: String,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Host to bind to
    #[serde(default = "default_host")]
    pub host: String,

    /// Port to listen on
    #[serde(default = "default_port")]
    pub port: u16,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Whether to log to file
    #[serde(default)]
    pub file: bool,

    /// Log file path (if file logging is enabled)
    #[serde(default = "default_log_file")]
    pub file_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app: AppConfig::default(),
            server: ServerConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: default_app_name(),
            version: default_app_version(),
            environment: default_environment(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            file: false,
            file_path: default_log_file(),
        }
    }
}

// Default values
fn default_app_name() -> String {
    "navius-app".to_string()
}

fn default_app_version() -> String {
    "0.1.0".to_string()
}

fn default_environment() -> String {
    "development".to_string()
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    3000
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_file() -> String {
    "logs/navius.log".to_string()
}

impl Config {
    /// Load configuration from the default location.
    pub fn load() -> Result<Self> {
        Self::load_from("config")
    }

    /// Load configuration from the specified directory.
    pub fn load_from<P: AsRef<Path>>(config_dir: P) -> Result<Self> {
        // Start with default config
        let mut builder = ::config::Config::builder().add_source(
            ::config::File::with_name(&format!("{}/default", config_dir.as_ref().display()))
                .required(false),
        );

        // Load environment-specific config
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
        builder = builder.add_source(
            ::config::File::with_name(&format!("{}/{}", config_dir.as_ref().display(), env))
                .required(false),
        );

        // Add environment variables with prefix NAVIUS_
        builder = builder.add_source(
            ::config::Environment::with_prefix("NAVIUS")
                .separator("__")
                .try_parsing(true),
        );

        // Build the config
        let config = builder
            .build()
            .map_err(|e| Error::configuration(format!("Failed to load config: {}", e)))?;

        // Convert to our Config type
        let config = config
            .try_deserialize()
            .map_err(|e| Error::configuration(format!("Failed to deserialize config: {}", e)))?;

        Ok(config)
    }

    /// Load configuration from environment variables only.
    pub fn from_env() -> Result<Self> {
        let builder = ::config::Config::builder().add_source(
            ::config::Environment::with_prefix("NAVIUS")
                .separator("__")
                .try_parsing(true),
        );

        let config = builder
            .build()
            .map_err(|e| Error::configuration(format!("Failed to load config from env: {}", e)))?;

        let mut config: Self = config
            .try_deserialize()
            .map_err(|e| Error::configuration(format!("Failed to deserialize config: {}", e)))?;

        // If any required values are not provided, use defaults
        if config.app.name.is_empty() {
            config.app.name = default_app_name();
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.app.name, "navius-app");
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.logging.level, "info");
    }
}
