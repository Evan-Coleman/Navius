use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use mockall::predicate::*;
use mockall::*;
use serde_json::Value;

use crate::error::{TestError, TestResult};
use crate::mock::MockRegistry;

/// Error type for configuration operations
#[derive(Debug, thiserror::Error)]
pub enum MockConfigError {
    /// Configuration file not found
    #[error("Configuration file not found: {0}")]
    NotFound(String),

    /// Invalid configuration format
    #[error("Invalid configuration format: {0}")]
    InvalidFormat(String),

    /// Missing configuration key
    #[error("Missing configuration key: {0}")]
    MissingKey(String),

    /// Type conversion error
    #[error("Type conversion error: {0}")]
    TypeConversionError(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(String),

    /// Other errors
    #[error("Other error: {0}")]
    OtherError(String),

    /// Not implemented
    #[error("Not implemented")]
    NotImplemented,
}

impl From<std::io::Error> for MockConfigError {
    fn from(err: std::io::Error) -> Self {
        MockConfigError::IoError(err.to_string())
    }
}

/// Configuration value type
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigValue {
    /// String value
    String(String),
    /// Integer value
    Integer(i64),
    /// Float value
    Float(f64),
    /// Boolean value
    Boolean(bool),
    /// Array value
    Array(Vec<ConfigValue>),
    /// Object value
    Object(HashMap<String, ConfigValue>),
    /// Null value
    Null,
}

impl ConfigValue {
    /// Create a ConfigValue from a serde_json::Value
    pub fn from_json(value: Value) -> Self {
        match value {
            Value::String(s) => ConfigValue::String(s),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    ConfigValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    ConfigValue::Float(f)
                } else {
                    ConfigValue::Float(n.as_f64().unwrap_or_default())
                }
            }
            Value::Bool(b) => ConfigValue::Boolean(b),
            Value::Array(a) => {
                ConfigValue::Array(a.into_iter().map(ConfigValue::from_json).collect())
            }
            Value::Object(o) => {
                let mut map = HashMap::new();
                for (k, v) in o {
                    map.insert(k, ConfigValue::from_json(v));
                }
                ConfigValue::Object(map)
            }
            Value::Null => ConfigValue::Null,
        }
    }

    /// Convert to a string value
    pub fn as_string(&self) -> TestResult<String> {
        match self {
            ConfigValue::String(s) => Ok(s.clone()),
            ConfigValue::Integer(i) => Ok(i.to_string()),
            ConfigValue::Float(f) => Ok(f.to_string()),
            ConfigValue::Boolean(b) => Ok(b.to_string()),
            _ => Err(TestError::ConversionError(format!(
                "Cannot convert {:?} to string",
                self
            ))),
        }
    }

    /// Convert to an integer value
    pub fn as_integer(&self) -> TestResult<i64> {
        match self {
            ConfigValue::Integer(i) => Ok(*i),
            ConfigValue::Float(f) => Ok(*f as i64),
            ConfigValue::String(s) => s.parse::<i64>().map_err(|e| {
                TestError::ConversionError(format!("Failed to parse string as integer: {}", e))
            }),
            _ => Err(TestError::ConversionError(format!(
                "Cannot convert {:?} to integer",
                self
            ))),
        }
    }

    /// Convert to a float value
    pub fn as_float(&self) -> TestResult<f64> {
        match self {
            ConfigValue::Float(f) => Ok(*f),
            ConfigValue::Integer(i) => Ok(*i as f64),
            ConfigValue::String(s) => s.parse::<f64>().map_err(|e| {
                TestError::ConversionError(format!("Failed to parse string as float: {}", e))
            }),
            _ => Err(TestError::ConversionError(format!(
                "Cannot convert {:?} to float",
                self
            ))),
        }
    }

    /// Convert to a boolean value
    pub fn as_boolean(&self) -> TestResult<bool> {
        match self {
            ConfigValue::Boolean(b) => Ok(*b),
            ConfigValue::Integer(i) => Ok(*i != 0),
            ConfigValue::String(s) => match s.to_lowercase().as_str() {
                "true" | "yes" | "1" | "on" => Ok(true),
                "false" | "no" | "0" | "off" => Ok(false),
                _ => Err(TestError::ConversionError(format!(
                    "Failed to parse string as boolean: {}",
                    s
                ))),
            },
            _ => Err(TestError::ConversionError(format!(
                "Cannot convert {:?} to boolean",
                self
            ))),
        }
    }

    /// Convert to an array value
    pub fn as_array(&self) -> TestResult<Vec<ConfigValue>> {
        match self {
            ConfigValue::Array(a) => Ok(a.clone()),
            _ => Err(TestError::ConversionError(format!(
                "Cannot convert {:?} to array",
                self
            ))),
        }
    }

    /// Convert to an object value
    pub fn as_object(&self) -> TestResult<HashMap<String, ConfigValue>> {
        match self {
            ConfigValue::Object(o) => Ok(o.clone()),
            _ => Err(TestError::ConversionError(format!(
                "Cannot convert {:?} to object",
                self
            ))),
        }
    }
}

/// Configuration provider
pub trait ConfigurationProvider: Send + Sync {
    /// Load configuration from a file
    fn load_from_file(&self, path: &str) -> Result<(), MockConfigError>;

    /// Get a string value from the configuration
    fn get_string(&self, key: &str) -> Result<String, MockConfigError>;

    /// Get an integer value from the configuration
    fn get_integer(&self, key: &str) -> Result<i64, MockConfigError>;

    /// Get a float value from the configuration
    fn get_float(&self, key: &str) -> Result<f64, MockConfigError>;

    /// Get a boolean value from the configuration
    fn get_boolean(&self, key: &str) -> Result<bool, MockConfigError>;

    /// Get an array value from the configuration
    fn get_array(&self, key: &str) -> Result<Vec<ConfigValue>, MockConfigError>;

    /// Get an object value from the configuration
    fn get_object(&self, key: &str) -> Result<HashMap<String, ConfigValue>, MockConfigError>;

    /// Check if a key exists in the configuration
    fn has(&self, key: &str) -> Result<bool, MockConfigError>;

    /// Set a value in the configuration
    fn set(&self, key: &str, value: ConfigValue) -> Result<(), MockConfigError>;

    /// Save the configuration to a file
    fn save_to_file(&self, path: &str) -> Result<(), MockConfigError>;
}

#[derive(Debug, Default)]
pub struct MockConfigurationProvider {}

impl MockConfigurationProvider {
    /// Create a new mock configuration provider
    pub fn new() -> Self {
        let mock = Self::default();
        mock
    }

    /// Register the mock with the registry
    pub fn register(self, registry: &MockRegistry) -> TestResult<Arc<Self>> {
        let arc_self = Arc::new(self);
        registry.register_mock::<dyn ConfigurationProvider>(arc_self.clone());
        Ok(arc_self)
    }

    /// Create a context for get_string method
    pub fn get_string_context(
        &self,
    ) -> MockGuard<'_, dyn Fn(&str) -> Result<String, MockConfigError>> {
        self.expect_get_string()
    }

    /// Create a context for get_integer method
    pub fn get_integer_context(
        &self,
    ) -> MockGuard<'_, dyn Fn(&str) -> Result<i64, MockConfigError>> {
        self.expect_get_integer()
    }

    /// Create a context for get_float method
    pub fn get_float_context(&self) -> MockGuard<'_, dyn Fn(&str) -> Result<f64, MockConfigError>> {
        self.expect_get_float()
    }

    /// Create a context for get_boolean method
    pub fn get_boolean_context(
        &self,
    ) -> MockGuard<'_, dyn Fn(&str) -> Result<bool, MockConfigError>> {
        self.expect_get_boolean()
    }

    /// Create a context for get_array method
    pub fn get_array_context(
        &self,
    ) -> MockGuard<'_, dyn Fn(&str) -> Result<Vec<ConfigValue>, MockConfigError>> {
        self.expect_get_array()
    }

    /// Create a context for get_object method
    pub fn get_object_context(
        &self,
    ) -> MockGuard<'_, dyn Fn(&str) -> Result<HashMap<String, ConfigValue>, MockConfigError>> {
        self.expect_get_object()
    }

    /// Create a context for has method
    pub fn has_context(&self) -> MockGuard<'_, dyn Fn(&str) -> Result<bool, MockConfigError>> {
        self.expect_has()
    }

    /// Create a context for set method
    pub fn set_context(
        &self,
    ) -> MockGuard<'_, dyn Fn(&str, ConfigValue) -> Result<(), MockConfigError>> {
        self.expect_set()
    }

    /// Create a context for load_from_file method
    pub fn load_from_file_context(
        &self,
    ) -> MockGuard<'_, dyn Fn(&str) -> Result<(), MockConfigError>> {
        self.expect_load_from_file()
    }

    /// Create a context for save_to_file method
    pub fn save_to_file_context(
        &self,
    ) -> MockGuard<'_, dyn Fn(&str) -> Result<(), MockConfigError>> {
        self.expect_save_to_file()
    }

    /// Set up expectation for a get_string operation
    pub fn expect_get_string(&self, key: &str, result: Result<String, MockConfigError>) {
        let _key_clone = key.to_string();

        // No-op implementation for mock
    }

    /// Set up expectation for a get_integer operation
    pub fn expect_get_integer(&self, key: &str, result: Result<i64, MockConfigError>) {
        let _key_clone = key.to_string();

        // No-op implementation for mock
    }

    /// Set up expectation for a get_float operation
    pub fn expect_get_float(&self, key: &str, result: Result<f64, MockConfigError>) {
        let _key_clone = key.to_string();

        // No-op implementation for mock
    }

    /// Set up expectation for a get_boolean operation
    pub fn expect_get_boolean(&self, key: &str, result: Result<bool, MockConfigError>) {
        let _key_clone = key.to_string();

        // No-op implementation for mock
    }

    /// Set up expectation for a get_array operation
    pub fn expect_get_array(&self, key: &str, result: Result<Vec<ConfigValue>, MockConfigError>) {
        let _key_clone = key.to_string();

        // No-op implementation for mock
    }

    /// Set up expectation for a get_object operation
    pub fn expect_get_object(
        &self,
        key: &str,
        result: Result<HashMap<String, ConfigValue>, MockConfigError>,
    ) {
        let _key_clone = key.to_string();

        // No-op implementation for mock
    }

    /// Set up expectation for a has operation
    pub fn expect_has(&self, key: &str, result: Result<bool, MockConfigError>) {
        let _key_clone = key.to_string();

        // No-op implementation for mock
    }

    /// Set up expectation for a set operation
    pub fn expect_set(&self, key: &str, value: ConfigValue, result: Result<(), MockConfigError>) {
        let _key_clone = key.to_string();
        let _value_clone = value;

        // No-op implementation for mock
    }

    /// Set up expectation for a load_from_file operation
    pub fn expect_load_from_file(&self, path: &str, result: Result<(), MockConfigError>) {
        let _path_clone = path.to_string();

        // No-op implementation for mock
    }

    /// Set up expectation for a save_to_file operation
    pub fn expect_save_to_file(&self, path: &str, result: Result<(), MockConfigError>) {
        let _path_clone = path.to_string();

        // No-op implementation for mock
    }
}

impl ConfigurationProvider for MockConfigurationProvider {
    fn load_from_file(&self, _path: &str) -> Result<(), MockConfigError> {
        Err(MockConfigError::NotImplemented)
    }

    fn get_string(&self, _key: &str) -> Result<String, MockConfigError> {
        Err(MockConfigError::NotImplemented)
    }

    fn get_integer(&self, _key: &str) -> Result<i64, MockConfigError> {
        Err(MockConfigError::NotImplemented)
    }

    fn get_float(&self, _key: &str) -> Result<f64, MockConfigError> {
        Err(MockConfigError::NotImplemented)
    }

    fn get_boolean(&self, _key: &str) -> Result<bool, MockConfigError> {
        Err(MockConfigError::NotImplemented)
    }

    fn get_array(&self, _key: &str) -> Result<Vec<ConfigValue>, MockConfigError> {
        Err(MockConfigError::NotImplemented)
    }

    fn get_object(&self, _key: &str) -> Result<HashMap<String, ConfigValue>, MockConfigError> {
        Err(MockConfigError::NotImplemented)
    }

    fn has(&self, _key: &str) -> Result<bool, MockConfigError> {
        Err(MockConfigError::NotImplemented)
    }

    fn set(&self, _key: &str, _value: ConfigValue) -> Result<(), MockConfigError> {
        Err(MockConfigError::NotImplemented)
    }

    fn save_to_file(&self, _path: &str) -> Result<(), MockConfigError> {
        Err(MockConfigError::NotImplemented)
    }
}

/// Trait for accessing a mock configuration provider in tests
pub trait HasMockConfiguration {
    /// Get the mock configuration provider
    fn configuration(&self) -> Arc<MockConfigurationProvider>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_configuration_provider() {
        let registry = MockRegistry::new();
        let config = MockConfigurationProvider::new()
            .register(&registry)
            .unwrap();

        // Set up expectations
        config.expect_get_string("app.name", Ok("Test App".to_string()));
        config.expect_get_integer("app.max_connections", Ok(100));
        config.expect_get_boolean("app.debug", Ok(true));

        // Use the mock
        let app_name = config.get_string("app.name").unwrap();
        let max_connections = config.get_integer("app.max_connections").unwrap();
        let debug = config.get_boolean("app.debug").unwrap();

        assert_eq!(app_name, "Test App");
        assert_eq!(max_connections, 100);
        assert_eq!(debug, true);

        // Verify all expectations have been met
        registry.verify().unwrap();
    }

    #[test]
    fn test_config_value_conversions() {
        // String conversions
        let string_value = ConfigValue::String("test".to_string());
        assert_eq!(string_value.as_string().unwrap(), "test");
        assert!(string_value.as_array().is_err());

        // Integer conversions
        let int_value = ConfigValue::Integer(42);
        assert_eq!(int_value.as_integer().unwrap(), 42);
        assert_eq!(int_value.as_float().unwrap(), 42.0);
        assert_eq!(int_value.as_string().unwrap(), "42");

        // Boolean conversions
        let bool_value = ConfigValue::Boolean(true);
        assert_eq!(bool_value.as_boolean().unwrap(), true);
        assert_eq!(bool_value.as_string().unwrap(), "true");

        // Array conversions
        let array_value = ConfigValue::Array(vec![
            ConfigValue::Integer(1),
            ConfigValue::Integer(2),
            ConfigValue::Integer(3),
        ]);
        let array = array_value.as_array().unwrap();
        assert_eq!(array.len(), 3);
        assert_eq!(array[0].as_integer().unwrap(), 1);
        assert_eq!(array[1].as_integer().unwrap(), 2);
        assert_eq!(array[2].as_integer().unwrap(), 3);

        // Object conversions
        let mut map = HashMap::new();
        map.insert("name".to_string(), ConfigValue::String("test".to_string()));
        map.insert("value".to_string(), ConfigValue::Integer(42));
        let object_value = ConfigValue::Object(map);

        let object = object_value.as_object().unwrap();
        assert_eq!(object.len(), 2);
        assert_eq!(object["name"].as_string().unwrap(), "test");
        assert_eq!(object["value"].as_integer().unwrap(), 42);
    }
}
