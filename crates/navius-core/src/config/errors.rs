// Copyright (c) 2025 Navius Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Error types for the configuration system.
//!
//! This module defines error types specific to configuration operations.

use std::error::Error;
use std::fmt;
use std::io;

/// Errors that can occur in the configuration system
#[derive(Debug)]
pub enum ConfigError {
    /// Key not found in configuration
    KeyNotFound {
        /// The key that was not found
        key: String,
    },

    /// Type conversion error
    TypeError {
        /// The expected type
        expected: String,
        /// The actual type
        actual: String,
    },

    /// Error loading a configuration file
    LoadError {
        /// The file path or source identifier
        source: String,
        /// The reason for the failure
        reason: String,
    },

    /// Error parsing a configuration file
    ParseError {
        /// The file path or source identifier
        source: String,
        /// The reason for the failure
        reason: String,
    },

    /// Error accessing a configuration source
    SourceError {
        /// The source identifier
        source: String,
        /// The reason for the failure
        reason: String,
    },

    /// Environment variable error
    EnvError {
        /// The environment variable
        var: String,
        /// The reason for the failure
        reason: String,
    },

    /// General error
    General(String),

    /// I/O error
    Io(io::Error),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::KeyNotFound { key } => {
                write!(f, "Configuration key not found: {}", key)
            }
            ConfigError::TypeError { expected, actual } => {
                write!(
                    f,
                    "Configuration type error: expected {}, got {}",
                    expected, actual
                )
            }
            ConfigError::LoadError { source, reason } => {
                write!(
                    f,
                    "Failed to load configuration from {}: {}",
                    source, reason
                )
            }
            ConfigError::ParseError { source, reason } => {
                write!(
                    f,
                    "Failed to parse configuration from {}: {}",
                    source, reason
                )
            }
            ConfigError::SourceError { source, reason } => {
                write!(f, "Configuration source error from {}: {}", source, reason)
            }
            ConfigError::EnvError { var, reason } => {
                write!(f, "Environment variable error for {}: {}", var, reason)
            }
            ConfigError::General(msg) => {
                write!(f, "Configuration error: {}", msg)
            }
            ConfigError::Io(err) => {
                write!(f, "Configuration I/O error: {}", err)
            }
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConfigError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> Self {
        ConfigError::Io(err)
    }
}

impl From<std::string::FromUtf8Error> for ConfigError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        ConfigError::General(format!("UTF-8 conversion error: {}", err))
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        ConfigError::ParseError {
            source: "JSON".to_string(),
            reason: err.to_string(),
        }
    }
}

#[cfg(feature = "toml")]
impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> Self {
        ConfigError::ParseError {
            source: "TOML".to_string(),
            reason: err.to_string(),
        }
    }
}

impl From<&str> for ConfigError {
    fn from(msg: &str) -> Self {
        ConfigError::General(msg.to_string())
    }
}

impl From<String> for ConfigError {
    fn from(msg: String) -> Self {
        ConfigError::General(msg)
    }
}

// Implement conversion from ConfigError to our application-wide error type
impl From<ConfigError> for crate::error::Error {
    fn from(err: ConfigError) -> Self {
        // Map configuration errors to appropriate application error categories
        match &err {
            ConfigError::KeyNotFound { .. } => {
                crate::error::Error::new_with_code("CONFIG_KEY_NOT_FOUND", &err.to_string())
            }
            ConfigError::TypeError { .. } => {
                crate::error::Error::new_with_code("CONFIG_TYPE_ERROR", &err.to_string())
            }
            ConfigError::LoadError { .. } | ConfigError::ParseError { .. } => {
                crate::error::Error::new_with_code("CONFIG_LOAD_ERROR", &err.to_string())
            }
            ConfigError::SourceError { .. } => {
                crate::error::Error::new_with_code("CONFIG_SOURCE_ERROR", &err.to_string())
            }
            ConfigError::EnvError { .. } => {
                crate::error::Error::new_with_code("CONFIG_ENV_ERROR", &err.to_string())
            }
            ConfigError::General(_) | ConfigError::Io(_) => {
                crate::error::Error::new_with_code("CONFIG_ERROR", &err.to_string())
            }
        }
    }
}

// Helper methods for creating common error types
impl ConfigError {
    /// Create a new key not found error
    pub fn key_not_found(key: &str) -> Self {
        ConfigError::KeyNotFound {
            key: key.to_string(),
        }
    }

    /// Create a new type error
    pub fn type_error(expected: &str, actual: &str) -> Self {
        ConfigError::TypeError {
            expected: expected.to_string(),
            actual: actual.to_string(),
        }
    }

    /// Create a new load error
    pub fn load_error(source: &str, reason: &str) -> Self {
        ConfigError::LoadError {
            source: source.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Create a new parse error
    pub fn parse_error(source: &str, reason: &str) -> Self {
        ConfigError::ParseError {
            source: source.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Create a new source error
    pub fn source_error(source: &str, reason: &str) -> Self {
        ConfigError::SourceError {
            source: source.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Create a new environment variable error
    pub fn env_error(var: &str, reason: &str) -> Self {
        ConfigError::EnvError {
            var: var.to_string(),
            reason: reason.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use navius_test::error::{TestResult, assert_eq, assert_true};

    #[test]
    fn test_error_display() -> TestResult<()> {
        let error = ConfigError::key_not_found("database.url");
        assert_eq(
            error.to_string(),
            "Configuration key not found: database.url",
            "Key not found error should have the correct message",
        )?;

        let error = ConfigError::type_error("string", "integer");
        assert_eq(
            error.to_string(),
            "Configuration type error: expected string, got integer",
            "Type error should have the correct message",
        )?;

        let error = ConfigError::load_error("config.json", "file not found");
        assert_eq(
            error.to_string(),
            "Failed to load configuration from config.json: file not found",
            "Load error should have the correct message",
        )?;

        Ok(())
    }

    #[test]
    fn test_error_conversions() -> TestResult<()> {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let config_error: ConfigError = io_error.into();

        match config_error {
            ConfigError::Io(_) => {
                // The expected case
                assert_true(
                    true,
                    "IO error should be converted to ConfigError::Io variant",
                )?;
            }
            _ => {
                return Err("Expected Io error variant".into());
            }
        }

        let app_error: crate::error::Error = ConfigError::key_not_found("test").into();
        assert_eq(
            app_error.code(),
            "CONFIG_KEY_NOT_FOUND",
            "Application error should have the correct error code",
        )?;

        Ok(())
    }
}
