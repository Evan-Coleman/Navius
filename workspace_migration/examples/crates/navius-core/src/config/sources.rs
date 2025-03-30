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

//! Configuration sources for the Navius configuration system.
//!
//! This module defines the sources from which configuration can be loaded.

use std::fmt;
use std::path::PathBuf;

/// Configuration source types
#[derive(Debug, Clone)]
pub enum ConfigSource {
    /// Configuration from a file
    File {
        /// Path to the file
        path: PathBuf,
        /// Format of the file (e.g., json, yaml, toml)
        format: FileFormat,
    },

    /// Configuration from environment variables
    Environment {
        /// Prefix for environment variables
        prefix: String,
        /// Separator for nested keys
        separator: String,
    },

    /// Configuration from command line arguments
    CommandLine {
        /// Prefix for command line arguments
        prefix: String,
    },

    /// Configuration from a remote source
    Remote {
        /// URL of the remote source
        url: String,
        /// Authentication token (if required)
        auth_token: Option<String>,
    },

    /// Configuration from in-memory values
    Memory,

    /// Configuration from a database
    Database {
        /// Connection string for the database
        connection_string: String,
        /// Table or collection name
        table: String,
    },
}

/// File formats supported by the configuration system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    /// JSON format
    Json,
    /// YAML format
    Yaml,
    /// TOML format
    Toml,
    /// INI format
    Ini,
    /// Properties format
    Properties,
    /// Auto-detect format from file extension
    Auto,
}

impl fmt::Display for FileFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileFormat::Json => write!(f, "JSON"),
            FileFormat::Yaml => write!(f, "YAML"),
            FileFormat::Toml => write!(f, "TOML"),
            FileFormat::Ini => write!(f, "INI"),
            FileFormat::Properties => write!(f, "Properties"),
            FileFormat::Auto => write!(f, "Auto"),
        }
    }
}

impl FileFormat {
    /// Detect file format from file extension
    pub fn from_extension(path: &std::path::Path) -> Self {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "json" => FileFormat::Json,
            "yaml" | "yml" => FileFormat::Yaml,
            "toml" => FileFormat::Toml,
            "ini" => FileFormat::Ini,
            "properties" | "props" => FileFormat::Properties,
            _ => FileFormat::Json, // Default to JSON if unknown
        }
    }

    /// Check if this format is enabled by feature flags
    pub fn is_enabled(&self) -> bool {
        match self {
            FileFormat::Json => true, // JSON is always enabled
            #[cfg(feature = "yaml")]
            FileFormat::Yaml => true,
            #[cfg(not(feature = "yaml"))]
            FileFormat::Yaml => false,
            #[cfg(feature = "toml")]
            FileFormat::Toml => true,
            #[cfg(not(feature = "toml"))]
            FileFormat::Toml => false,
            #[cfg(feature = "ini")]
            FileFormat::Ini => true,
            #[cfg(not(feature = "ini"))]
            FileFormat::Ini => false,
            #[cfg(feature = "properties")]
            FileFormat::Properties => true,
            #[cfg(not(feature = "properties"))]
            FileFormat::Properties => false,
            FileFormat::Auto => true, // Auto is always enabled
        }
    }
}

impl ConfigSource {
    /// Create a new file source
    pub fn file<P: Into<PathBuf>>(path: P, format: FileFormat) -> Self {
        ConfigSource::File {
            path: path.into(),
            format,
        }
    }

    /// Create a new environment source
    pub fn environment(prefix: &str, separator: &str) -> Self {
        ConfigSource::Environment {
            prefix: prefix.to_string(),
            separator: separator.to_string(),
        }
    }

    /// Create a new command line source
    pub fn command_line(prefix: &str) -> Self {
        ConfigSource::CommandLine {
            prefix: prefix.to_string(),
        }
    }

    /// Create a new remote source
    pub fn remote(url: &str, auth_token: Option<&str>) -> Self {
        ConfigSource::Remote {
            url: url.to_string(),
            auth_token: auth_token.map(|s| s.to_string()),
        }
    }

    /// Create a new memory source
    pub fn memory() -> Self {
        ConfigSource::Memory
    }

    /// Create a new database source
    pub fn database(connection_string: &str, table: &str) -> Self {
        ConfigSource::Database {
            connection_string: connection_string.to_string(),
            table: table.to_string(),
        }
    }

    /// Get the name of this source
    pub fn name(&self) -> &'static str {
        match self {
            ConfigSource::File { .. } => "file",
            ConfigSource::Environment { .. } => "environment",
            ConfigSource::CommandLine { .. } => "command_line",
            ConfigSource::Remote { .. } => "remote",
            ConfigSource::Memory => "memory",
            ConfigSource::Database { .. } => "database",
        }
    }

    /// Get a descriptive string for this source
    pub fn description(&self) -> String {
        match self {
            ConfigSource::File { path, format } => {
                format!("file: {} ({})", path.display(), format)
            }
            ConfigSource::Environment { prefix, separator } => {
                format!("environment: prefix={}, separator={}", prefix, separator)
            }
            ConfigSource::CommandLine { prefix } => {
                format!("command line: prefix={}", prefix)
            }
            ConfigSource::Remote { url, .. } => {
                format!("remote: {}", url)
            }
            ConfigSource::Memory => "memory".to_string(),
            ConfigSource::Database { table, .. } => {
                format!("database: table={}", table)
            }
        }
    }
}

impl fmt::Display for ConfigSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_file_format_from_extension() {
        assert_eq!(
            FileFormat::from_extension(Path::new("config.json")),
            FileFormat::Json
        );
        assert_eq!(
            FileFormat::from_extension(Path::new("config.yaml")),
            FileFormat::Yaml
        );
        assert_eq!(
            FileFormat::from_extension(Path::new("config.yml")),
            FileFormat::Yaml
        );
        assert_eq!(
            FileFormat::from_extension(Path::new("config.toml")),
            FileFormat::Toml
        );
        assert_eq!(
            FileFormat::from_extension(Path::new("config.ini")),
            FileFormat::Ini
        );
        assert_eq!(
            FileFormat::from_extension(Path::new("config.properties")),
            FileFormat::Properties
        );
        assert_eq!(
            FileFormat::from_extension(Path::new("config")),
            FileFormat::Json
        );
    }

    #[test]
    fn test_config_source_descriptions() {
        let file_source = ConfigSource::file("config.json", FileFormat::Json);
        assert!(file_source.description().contains("config.json"));

        let env_source = ConfigSource::environment("APP_", "__");
        assert!(env_source.description().contains("APP_"));
        assert!(env_source.description().contains("__"));

        let memory_source = ConfigSource::memory();
        assert_eq!(memory_source.description(), "memory");
    }
}
