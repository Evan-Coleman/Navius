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

//! Configuration value types for the Navius configuration system.
//!
//! This module defines the core value types used to represent configuration values.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use super::errors::ConfigError;

/// A strongly-typed configuration value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    /// String value
    String(String),
    /// Integer value
    Integer(i64),
    /// Floating point value
    Float(f64),
    /// Boolean value
    Boolean(bool),
    /// Array of configuration values
    Array(Vec<ConfigValue>),
    /// Object with string keys and configuration values
    Object(HashMap<String, ConfigValue>),
    /// Null value
    Null,
}

impl ConfigValue {
    /// Check if this value is a string
    pub fn is_string(&self) -> bool {
        matches!(self, ConfigValue::String(_))
    }

    /// Check if this value is an integer
    pub fn is_integer(&self) -> bool {
        matches!(self, ConfigValue::Integer(_))
    }

    /// Check if this value is a float
    pub fn is_float(&self) -> bool {
        matches!(self, ConfigValue::Float(_))
    }

    /// Check if this value is a boolean
    pub fn is_boolean(&self) -> bool {
        matches!(self, ConfigValue::Boolean(_))
    }

    /// Check if this value is an array
    pub fn is_array(&self) -> bool {
        matches!(self, ConfigValue::Array(_))
    }

    /// Check if this value is an object
    pub fn is_object(&self) -> bool {
        matches!(self, ConfigValue::Object(_))
    }

    /// Check if this value is null
    pub fn is_null(&self) -> bool {
        matches!(self, ConfigValue::Null)
    }

    /// Try to get this value as a string
    pub fn as_string(&self) -> Result<&str, ConfigError> {
        match self {
            ConfigValue::String(s) => Ok(s),
            _ => Err(ConfigError::TypeError {
                expected: "string".to_string(),
                actual: self.type_name().to_string(),
            }),
        }
    }

    /// Try to get this value as an integer
    pub fn as_integer(&self) -> Result<i64, ConfigError> {
        match self {
            ConfigValue::Integer(i) => Ok(*i),
            ConfigValue::Float(f) => {
                if *f == (*f as i64) as f64 {
                    Ok(*f as i64)
                } else {
                    Err(ConfigError::TypeError {
                        expected: "integer".to_string(),
                        actual: "float (non-integer)".to_string(),
                    })
                }
            }
            ConfigValue::String(s) => s.parse::<i64>().map_err(|_| ConfigError::TypeError {
                expected: "integer".to_string(),
                actual: "string (non-numeric)".to_string(),
            }),
            _ => Err(ConfigError::TypeError {
                expected: "integer".to_string(),
                actual: self.type_name().to_string(),
            }),
        }
    }

    /// Try to get this value as a float
    pub fn as_float(&self) -> Result<f64, ConfigError> {
        match self {
            ConfigValue::Float(f) => Ok(*f),
            ConfigValue::Integer(i) => Ok(*i as f64),
            ConfigValue::String(s) => s.parse::<f64>().map_err(|_| ConfigError::TypeError {
                expected: "float".to_string(),
                actual: "string (non-numeric)".to_string(),
            }),
            _ => Err(ConfigError::TypeError {
                expected: "float".to_string(),
                actual: self.type_name().to_string(),
            }),
        }
    }

    /// Try to get this value as a boolean
    pub fn as_boolean(&self) -> Result<bool, ConfigError> {
        match self {
            ConfigValue::Boolean(b) => Ok(*b),
            ConfigValue::Integer(i) => {
                if *i == 0 {
                    Ok(false)
                } else if *i == 1 {
                    Ok(true)
                } else {
                    Err(ConfigError::TypeError {
                        expected: "boolean".to_string(),
                        actual: format!("integer ({})", i),
                    })
                }
            }
            ConfigValue::String(s) => {
                let lower = s.to_lowercase();
                match lower.as_str() {
                    "true" | "yes" | "y" | "1" => Ok(true),
                    "false" | "no" | "n" | "0" => Ok(false),
                    _ => Err(ConfigError::TypeError {
                        expected: "boolean".to_string(),
                        actual: format!("string ({})", s),
                    }),
                }
            }
            _ => Err(ConfigError::TypeError {
                expected: "boolean".to_string(),
                actual: self.type_name().to_string(),
            }),
        }
    }

    /// Try to get this value as an array
    pub fn as_array(&self) -> Result<&Vec<ConfigValue>, ConfigError> {
        match self {
            ConfigValue::Array(a) => Ok(a),
            _ => Err(ConfigError::TypeError {
                expected: "array".to_string(),
                actual: self.type_name().to_string(),
            }),
        }
    }

    /// Try to get this value as an object
    pub fn as_object(&self) -> Result<&HashMap<String, ConfigValue>, ConfigError> {
        match self {
            ConfigValue::Object(o) => Ok(o),
            _ => Err(ConfigError::TypeError {
                expected: "object".to_string(),
                actual: self.type_name().to_string(),
            }),
        }
    }

    /// Get the type name of this value
    pub fn type_name(&self) -> &'static str {
        match self {
            ConfigValue::String(_) => "string",
            ConfigValue::Integer(_) => "integer",
            ConfigValue::Float(_) => "float",
            ConfigValue::Boolean(_) => "boolean",
            ConfigValue::Array(_) => "array",
            ConfigValue::Object(_) => "object",
            ConfigValue::Null => "null",
        }
    }
}

impl From<String> for ConfigValue {
    fn from(s: String) -> Self {
        ConfigValue::String(s)
    }
}

impl From<&str> for ConfigValue {
    fn from(s: &str) -> Self {
        ConfigValue::String(s.to_string())
    }
}

impl From<i64> for ConfigValue {
    fn from(i: i64) -> Self {
        ConfigValue::Integer(i)
    }
}

impl From<i32> for ConfigValue {
    fn from(i: i32) -> Self {
        ConfigValue::Integer(i as i64)
    }
}

impl From<f64> for ConfigValue {
    fn from(f: f64) -> Self {
        ConfigValue::Float(f)
    }
}

impl From<bool> for ConfigValue {
    fn from(b: bool) -> Self {
        ConfigValue::Boolean(b)
    }
}

impl<T: Into<ConfigValue>> From<Vec<T>> for ConfigValue {
    fn from(v: Vec<T>) -> Self {
        ConfigValue::Array(v.into_iter().map(|item| item.into()).collect())
    }
}

impl<K: Into<String>, V: Into<ConfigValue>> From<HashMap<K, V>> for ConfigValue {
    fn from(m: HashMap<K, V>) -> Self {
        let map = m.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        ConfigValue::Object(map)
    }
}

impl fmt::Display for ConfigValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigValue::String(s) => write!(f, "{}", s),
            ConfigValue::Integer(i) => write!(f, "{}", i),
            ConfigValue::Float(fl) => write!(f, "{}", fl),
            ConfigValue::Boolean(b) => write!(f, "{}", b),
            ConfigValue::Array(_) => write!(f, "[...]"),
            ConfigValue::Object(_) => write!(f, "{{...}}"),
            ConfigValue::Null => write!(f, "null"),
        }
    }
}

/// A collection of configuration values
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigValues {
    values: HashMap<String, ConfigValue>,
}

impl ConfigValues {
    /// Create a new empty configuration values
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Get a value by key
    pub fn get(&self, key: &str) -> Option<&ConfigValue> {
        // Support nested keys with dot notation (e.g., "database.url")
        let parts: Vec<&str> = key.split('.').collect();

        if parts.len() == 1 {
            // Simple key lookup
            self.values.get(key)
        } else {
            // Nested key lookup
            let mut current = self;
            let mut current_value: Option<&ConfigValue> = None;

            for (i, part) in parts.iter().enumerate() {
                if i == 0 {
                    // First level lookup in the root values
                    current_value = current.values.get(*part);
                } else if let Some(ConfigValue::Object(obj)) = current_value {
                    // Nested lookup in an object
                    current_value = obj.get(*part);
                } else {
                    // Not found or not an object
                    return None;
                }

                if current_value.is_none() {
                    return None;
                }
            }

            current_value
        }
    }

    /// Set a value by key
    pub fn set<K: Into<String>, V: Into<ConfigValue>>(&mut self, key: K, value: V) {
        self.values.insert(key.into(), value.into());
    }

    /// Check if a key exists
    pub fn has(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    /// Get all keys
    pub fn keys(&self) -> Vec<&str> {
        self.values.keys().map(|k| k.as_str()).collect()
    }

    /// Get all key-value pairs
    pub fn entries(&self) -> &HashMap<String, ConfigValue> {
        &self.values
    }

    /// Merge another ConfigValues into this one
    pub fn merge(&mut self, other: ConfigValues) {
        // Simple implementation that overrides values with the same key
        for (key, value) in other.values {
            self.values.insert(key, value);
        }
    }

    /// Create a sub-configuration with keys that have a specific prefix
    pub fn subset(&self, prefix: &str) -> ConfigValues {
        let prefix_dot = format!("{}.", prefix);
        let mut result = ConfigValues::new();

        for (key, value) in self.values.iter() {
            if key == prefix {
                // Exact match - add all nested values to the root
                if let ConfigValue::Object(obj) = value {
                    for (subkey, subvalue) in obj {
                        result.values.insert(subkey.clone(), subvalue.clone());
                    }
                }
            } else if key.starts_with(&prefix_dot) {
                // Key has the prefix - remove the prefix and add
                let new_key = key[prefix_dot.len()..].to_string();
                result.values.insert(new_key, value.clone());
            }
        }

        result
    }
}

impl Default for ConfigValues {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_value_conversions() {
        // String conversions
        let string_value = ConfigValue::String("test".to_string());
        assert_eq!(string_value.as_string().unwrap(), "test");
        assert!(string_value.as_integer().is_err());

        // Integer conversions
        let int_value = ConfigValue::Integer(42);
        assert_eq!(int_value.as_integer().unwrap(), 42);
        assert_eq!(int_value.as_float().unwrap(), 42.0);
        assert!(int_value.as_string().is_err());

        // Boolean conversions
        let bool_value = ConfigValue::Boolean(true);
        assert_eq!(bool_value.as_boolean().unwrap(), true);
        assert!(bool_value.as_integer().is_err());

        // Complex types
        let array_value =
            ConfigValue::Array(vec![ConfigValue::Integer(1), ConfigValue::Integer(2)]);
        let array_result = array_value.as_array().unwrap();
        assert_eq!(array_result.len(), 2);
        assert_eq!(array_result[0], ConfigValue::Integer(1));

        // String to boolean conversion
        let string_true = ConfigValue::String("true".to_string());
        assert_eq!(string_true.as_boolean().unwrap(), true);

        let string_yes = ConfigValue::String("yes".to_string());
        assert_eq!(string_yes.as_boolean().unwrap(), true);

        let string_false = ConfigValue::String("false".to_string());
        assert_eq!(string_false.as_boolean().unwrap(), false);

        let string_no = ConfigValue::String("no".to_string());
        assert_eq!(string_no.as_boolean().unwrap(), false);
    }

    #[test]
    fn test_config_values() {
        let mut config = ConfigValues::new();

        // Test simple values
        config.set("name", "test-app");
        config.set("version", "1.0.0");
        config.set("port", 8080);

        assert_eq!(config.get("name").unwrap().as_string().unwrap(), "test-app");
        assert_eq!(config.get("port").unwrap().as_integer().unwrap(), 8080);
        assert!(config.has("name"));
        assert!(!config.has("missing"));

        // Test nested values
        let mut db_config = HashMap::new();
        db_config.insert("url".to_string(), "postgres://localhost".into());
        db_config.insert("port".to_string(), 5432.into());

        config.set("database", db_config);

        let db = config.get("database").unwrap().as_object().unwrap();
        assert_eq!(
            db.get("url").unwrap().as_string().unwrap(),
            "postgres://localhost"
        );

        // Test dot notation access
        assert_eq!(
            config.get("database.url").unwrap().as_string().unwrap(),
            "postgres://localhost"
        );
        assert_eq!(
            config.get("database.port").unwrap().as_integer().unwrap(),
            5432
        );

        // Test subset
        let db_subset = config.subset("database");
        assert_eq!(
            db_subset.get("url").unwrap().as_string().unwrap(),
            "postgres://localhost"
        );
        assert_eq!(db_subset.get("port").unwrap().as_integer().unwrap(), 5432);
    }
}
