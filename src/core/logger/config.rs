use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::interface::LogLevel;

/// Configuration for the logging system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// The type of logger to use (tracing, json, console, custom)
    #[serde(default = "default_logger_type")]
    pub logger_type: String,

    /// The minimum log level to record
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Output format (json, text, pretty)
    #[serde(default = "default_log_format")]
    pub format: String,

    /// Whether to include timestamps in logs
    #[serde(default = "default_true")]
    pub include_timestamps: bool,

    /// Whether to include file and line information
    #[serde(default = "default_true")]
    pub include_file_info: bool,

    /// Whether to colorize output (for console loggers)
    #[serde(default = "default_true")]
    pub colorize: bool,

    /// Path for file-based logging (if applicable)
    #[serde(default)]
    pub file_path: Option<String>,

    /// Maximum log file size in megabytes
    #[serde(default = "default_log_file_size_mb")]
    pub max_file_size_mb: u64,

    /// Maximum number of log files to keep
    #[serde(default = "default_log_file_count")]
    pub max_file_count: u32,

    /// Custom fields to include in all log records
    #[serde(default)]
    pub global_fields: HashMap<String, String>,

    /// Advanced provider-specific options
    #[serde(default)]
    pub provider_options: HashMap<String, String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            logger_type: default_logger_type(),
            level: default_log_level(),
            format: default_log_format(),
            include_timestamps: default_true(),
            include_file_info: default_true(),
            colorize: default_true(),
            file_path: None,
            max_file_size_mb: default_log_file_size_mb(),
            max_file_count: default_log_file_count(),
            global_fields: HashMap::new(),
            provider_options: HashMap::new(),
        }
    }
}

impl LoggingConfig {
    /// Parse the configured log level into the LogLevel enum
    pub fn parse_level(&self) -> LogLevel {
        match self.level.to_lowercase().as_str() {
            "trace" => LogLevel::Trace,
            "debug" => LogLevel::Debug,
            "info" => LogLevel::Info,
            "warn" | "warning" => LogLevel::Warn,
            "error" | "err" => LogLevel::Error,
            _ => LogLevel::Info, // Default to Info for unknown levels
        }
    }

    /// Check if the configuration is valid
    pub fn validate(&self) -> Result<(), String> {
        // Check if logger type is supported
        match self.logger_type.to_lowercase().as_str() {
            "tracing" | "json" | "console" | "file" | "custom" => {}
            _ => return Err(format!("Unsupported logger type: {}", self.logger_type)),
        }

        // Check if format is supported
        match self.format.to_lowercase().as_str() {
            "json" | "text" | "pretty" => {}
            _ => return Err(format!("Unsupported log format: {}", self.format)),
        }

        // Validate file path if file logging is enabled
        if self.logger_type == "file" && self.file_path.is_none() {
            return Err("File path must be specified for file logger".to_string());
        }

        // All checks passed
        Ok(())
    }
}

fn default_logger_type() -> String {
    "tracing".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "json".to_string()
}

fn default_true() -> bool {
    true
}

fn default_log_file_size_mb() -> u64 {
    10
}

fn default_log_file_count() -> u32 {
    5
}
