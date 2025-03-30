//! Configuration utilities for integration tests
//!
//! This module provides utilities for loading and managing test configurations
//! for integration tests. It supports loading configurations from files,
//! environment variables, and defaults.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::{TestError, TestResult};

/// Configuration for test resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResourceConfig {
    /// Base directory for test resources
    pub base_dir: Option<PathBuf>,
    /// Input files for the test
    pub input_files: HashMap<String, PathBuf>,
    /// Output directory for test artifacts
    pub output_dir: Option<PathBuf>,
    /// Whether to clean up test resources after the test
    pub cleanup: bool,
}

impl Default for TestResourceConfig {
    fn default() -> Self {
        Self {
            base_dir: None,
            input_files: HashMap::new(),
            output_dir: None,
            cleanup: true,
        }
    }
}

/// Mock configuration for integration tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockConfig {
    /// Whether to verify mock expectations after the test
    pub verify_expectations: bool,
    /// Default values for mock responses
    pub default_values: HashMap<String, serde_json::Value>,
    /// Error simulation configuration
    pub error_simulation: HashMap<String, bool>,
}

impl Default for MockConfig {
    fn default() -> Self {
        Self {
            verify_expectations: true,
            default_values: HashMap::new(),
            error_simulation: HashMap::new(),
        }
    }
}

/// Environment configuration for integration tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Environment variables to set for the test
    pub variables: HashMap<String, String>,
    /// Whether to restore original environment variables after the test
    pub restore_after_test: bool,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            variables: HashMap::new(),
            restore_after_test: true,
        }
    }
}

/// Timeout configuration for integration tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Overall test timeout in seconds
    pub test_timeout_secs: u64,
    /// Operation timeout in seconds
    pub operation_timeout_secs: u64,
    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            test_timeout_secs: 30,
            operation_timeout_secs: 5,
            connection_timeout_secs: 10,
        }
    }
}

/// Test configuration for integration tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    /// Name of the test
    pub name: String,
    /// Description of the test
    pub description: Option<String>,
    /// Resources configuration
    pub resources: TestResourceConfig,
    /// Mock configuration
    pub mocks: MockConfig,
    /// Environment configuration
    pub environment: EnvironmentConfig,
    /// Timeout configuration
    pub timeouts: TimeoutConfig,
    /// Additional configuration options
    pub options: HashMap<String, serde_json::Value>,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            name: "unnamed_test".to_string(),
            description: None,
            resources: TestResourceConfig::default(),
            mocks: MockConfig::default(),
            environment: EnvironmentConfig::default(),
            timeouts: TimeoutConfig::default(),
            options: HashMap::new(),
        }
    }
}

impl TestConfig {
    /// Create a new test configuration with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Load a test configuration from a JSON file
    pub fn from_json_file(path: impl AsRef<Path>) -> TestResult<Self> {
        let file_content = fs::read_to_string(path)
            .map_err(|e| TestError::IoError(format!("Failed to read config file: {}", e)))?;

        let config: TestConfig = serde_json::from_str(&file_content).map_err(|e| {
            TestError::ConfigurationError(format!("Failed to parse config file: {}", e))
        })?;

        Ok(config)
    }

    /// Load a test configuration from a TOML file
    pub fn from_toml_file(path: impl AsRef<Path>) -> TestResult<Self> {
        let file_content = fs::read_to_string(path)
            .map_err(|e| TestError::IoError(format!("Failed to read config file: {}", e)))?;

        let config: TestConfig = toml::from_str(&file_content).map_err(|e| {
            TestError::ConfigurationError(format!("Failed to parse config file: {}", e))
        })?;

        Ok(config)
    }

    /// Save the test configuration to a JSON file
    pub fn to_json_file(&self, path: impl AsRef<Path>) -> TestResult<()> {
        let json = serde_json::to_string_pretty(self).map_err(|e| {
            TestError::ConfigurationError(format!("Failed to serialize config: {}", e))
        })?;

        fs::write(path, json)
            .map_err(|e| TestError::IoError(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }

    /// Save the test configuration to a TOML file
    pub fn to_toml_file(&self, path: impl AsRef<Path>) -> TestResult<()> {
        let toml = toml::to_string_pretty(self).map_err(|e| {
            TestError::ConfigurationError(format!("Failed to serialize config: {}", e))
        })?;

        fs::write(path, toml)
            .map_err(|e| TestError::IoError(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }

    /// Get an option value as a typed value
    pub fn get_option<T: FromStr>(&self, key: &str) -> TestResult<Option<T>>
    where
        T::Err: std::fmt::Display,
    {
        if let Some(value) = self.options.get(key) {
            if let Some(string_value) = value.as_str() {
                match T::from_str(string_value) {
                    Ok(typed_value) => return Ok(Some(typed_value)),
                    Err(e) => {
                        return Err(TestError::ConfigurationError(format!(
                            "Failed to parse option {}: {}",
                            key, e
                        )));
                    }
                }
            }

            return Err(TestError::ConfigurationError(format!(
                "Option {} is not a string",
                key
            )));
        }

        Ok(None)
    }

    /// Set an option value
    pub fn set_option<T: ToString>(&mut self, key: impl Into<String>, value: T) {
        let string_value = value.to_string();
        self.options
            .insert(key.into(), serde_json::Value::String(string_value));
    }

    /// Apply environment variable overrides to the configuration
    pub fn apply_env_overrides(&mut self, prefix: &str) -> TestResult<()> {
        // Apply environment variables with the specified prefix
        for (key, value) in std::env::vars() {
            if key.starts_with(prefix) {
                let config_key = key[prefix.len()..].to_lowercase();
                self.set_option(config_key, value);
            }
        }

        Ok(())
    }
}

/// Builder for test configurations
pub struct TestConfigBuilder {
    config: TestConfig,
}

impl TestConfigBuilder {
    /// Create a new test configuration builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            config: TestConfig::new(name),
        }
    }

    /// Set the test description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.config.description = Some(description.into());
        self
    }

    /// Set the base directory for test resources
    pub fn with_resource_dir(mut self, base_dir: impl Into<PathBuf>) -> Self {
        self.config.resources.base_dir = Some(base_dir.into());
        self
    }

    /// Add an input file for the test
    pub fn with_input_file(mut self, name: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        self.config
            .resources
            .input_files
            .insert(name.into(), path.into());
        self
    }

    /// Set the output directory for test artifacts
    pub fn with_output_dir(mut self, output_dir: impl Into<PathBuf>) -> Self {
        self.config.resources.output_dir = Some(output_dir.into());
        self
    }

    /// Set whether to clean up test resources after the test
    pub fn with_cleanup(mut self, cleanup: bool) -> Self {
        self.config.resources.cleanup = cleanup;
        self
    }

    /// Set whether to verify mock expectations after the test
    pub fn with_verify_expectations(mut self, verify: bool) -> Self {
        self.config.mocks.verify_expectations = verify;
        self
    }

    /// Add a default value for a mock response
    pub fn with_mock_default(
        mut self,
        key: impl Into<String>,
        value: impl Serialize,
    ) -> TestResult<Self> {
        let json_value = serde_json::to_value(value).map_err(|e| {
            TestError::ConfigurationError(format!("Failed to serialize mock default: {}", e))
        })?;
        self.config
            .mocks
            .default_values
            .insert(key.into(), json_value);
        Ok(self)
    }

    /// Add an environment variable for the test
    pub fn with_env_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config
            .environment
            .variables
            .insert(key.into(), value.into());
        self
    }

    /// Set the test timeout
    pub fn with_test_timeout(mut self, seconds: u64) -> Self {
        self.config.timeouts.test_timeout_secs = seconds;
        self
    }

    /// Set the operation timeout
    pub fn with_operation_timeout(mut self, seconds: u64) -> Self {
        self.config.timeouts.operation_timeout_secs = seconds;
        self
    }

    /// Set the connection timeout
    pub fn with_connection_timeout(mut self, seconds: u64) -> Self {
        self.config.timeouts.connection_timeout_secs = seconds;
        self
    }

    /// Set a custom option
    pub fn with_option<T: ToString>(mut self, key: impl Into<String>, value: T) -> Self {
        self.config.set_option(key, value);
        self
    }

    /// Build the test configuration
    pub fn build(self) -> TestConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{
        TestResult, assert_eq, assert_false, assert_none, assert_some, assert_true,
    };
    use tempfile::tempdir;

    #[test]
    fn test_default_config() -> TestResult<()> {
        let config = TestConfig::default();
        assert_eq(
            config.name,
            "unnamed_test",
            "Default config should have the unnamed_test name",
        )?;
        assert_none(
            config.description.as_ref(),
            "Default config should have no description",
        )?;
        assert_none(
            config.resources.base_dir.as_ref(),
            "Default config should have no base directory",
        )?;
        assert_none(
            config.resources.output_dir.as_ref(),
            "Default config should have no output directory",
        )?;
        assert_true(
            config.resources.cleanup,
            "Default config should have cleanup enabled",
        )?;
        assert_true(
            config.mocks.verify_expectations,
            "Default config should verify expectations",
        )?;
        assert_true(
            config.environment.restore_after_test,
            "Default config should restore environment after test",
        )?;
        assert_eq(
            config.timeouts.test_timeout_secs,
            30,
            "Default config should have 30 second test timeout",
        )?;

        Ok(())
    }

    #[test]
    fn test_config_builder() -> TestResult<()> {
        let config = TestConfigBuilder::new("test_config")
            .with_description("Test configuration")
            .with_resource_dir("/tmp/test")
            .with_input_file("input", "input.txt")
            .with_output_dir("/tmp/test/output")
            .with_cleanup(false)
            .with_verify_expectations(true)
            .with_env_var("TEST_VAR", "test_value")
            .with_test_timeout(60)
            .with_option("custom_option", "custom_value")
            .build();

        assert_eq(
            config.name,
            "test_config",
            "Config should have the correct name",
        )?;
        assert_eq(
            config.description,
            Some("Test configuration".to_string()),
            "Config should have the correct description",
        )?;
        assert_eq(
            config.resources.base_dir,
            Some(PathBuf::from("/tmp/test")),
            "Config should have the correct base directory",
        )?;
        assert_eq(
            config.resources.input_files.get("input"),
            Some(&PathBuf::from("input.txt")),
            "Config should have the correct input file",
        )?;
        assert_eq(
            config.resources.output_dir,
            Some(PathBuf::from("/tmp/test/output")),
            "Config should have the correct output directory",
        )?;
        assert_false(
            config.resources.cleanup,
            "Config should have cleanup disabled",
        )?;
        assert_true(
            config.mocks.verify_expectations,
            "Config should verify expectations",
        )?;
        assert_eq(
            config.environment.variables.get("TEST_VAR"),
            Some(&"test_value".to_string()),
            "Config should have the correct environment variable",
        )?;
        assert_eq(
            config.timeouts.test_timeout_secs,
            60,
            "Config should have the correct test timeout",
        )?;
        assert_eq(
            config.get_option::<String>("custom_option")?,
            Some("custom_value".to_string()),
            "Config should have the correct custom option",
        )?;

        Ok(())
    }

    #[test]
    fn test_json_serialization() -> TestResult<()> {
        let temp_dir = tempdir()?;
        let config_path = temp_dir.path().join("config.json");

        let config = TestConfigBuilder::new("json_test")
            .with_description("JSON test")
            .with_resource_dir("/tmp/test")
            .with_env_var("TEST_VAR", "test_value")
            .build();

        config.to_json_file(&config_path)?;

        let loaded_config = TestConfig::from_json_file(&config_path)?;

        assert_eq(
            loaded_config.name,
            "json_test",
            "Loaded config should have the correct name",
        )?;
        assert_eq(
            loaded_config.description,
            Some("JSON test".to_string()),
            "Loaded config should have the correct description",
        )?;
        assert_eq(
            loaded_config.resources.base_dir,
            Some(PathBuf::from("/tmp/test")),
            "Loaded config should have the correct base directory",
        )?;
        assert_eq(
            loaded_config.environment.variables.get("TEST_VAR"),
            Some(&"test_value".to_string()),
            "Loaded config should have the correct environment variable",
        )?;

        Ok(())
    }

    #[test]
    fn test_toml_serialization() -> TestResult<()> {
        let temp_dir = tempdir()?;
        let config_path = temp_dir.path().join("config.toml");

        let config = TestConfigBuilder::new("toml_test")
            .with_description("TOML test")
            .with_resource_dir("/tmp/test")
            .with_env_var("TEST_VAR", "test_value")
            .build();

        config.to_toml_file(&config_path)?;

        let loaded_config = TestConfig::from_toml_file(&config_path)?;

        assert_eq(
            loaded_config.name,
            "toml_test",
            "Loaded config should have the correct name",
        )?;
        assert_eq(
            loaded_config.description,
            Some("TOML test".to_string()),
            "Loaded config should have the correct description",
        )?;
        assert_eq(
            loaded_config.resources.base_dir,
            Some(PathBuf::from("/tmp/test")),
            "Loaded config should have the correct base directory",
        )?;
        assert_eq(
            loaded_config.environment.variables.get("TEST_VAR"),
            Some(&"test_value".to_string()),
            "Loaded config should have the correct environment variable",
        )?;

        Ok(())
    }

    #[test]
    fn test_option_access() -> TestResult<()> {
        let mut config = TestConfig::default();
        config.set_option("string_option", "string_value");
        config.set_option("int_option", "42");
        config.set_option("bool_option", "true");

        assert_eq(
            config.get_option::<String>("string_option")?,
            Some("string_value".to_string()),
            "String option should have the correct value",
        )?;
        assert_eq(
            config.get_option::<i32>("int_option")?,
            Some(42),
            "Integer option should have the correct value",
        )?;
        assert_eq(
            config.get_option::<bool>("bool_option")?,
            Some(true),
            "Boolean option should have the correct value",
        )?;
        assert_eq(
            config.get_option::<String>("non_existent")?,
            None,
            "Non-existent option should return None",
        )?;

        Ok(())
    }
}
