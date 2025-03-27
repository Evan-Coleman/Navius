use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;

use super::error::LoggingError;

/// Log levels that match standard logging conventions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Standard log information structure
#[derive(Debug, Clone)]
pub struct LogInfo {
    pub message: String,
    pub context: Option<String>,
    pub module: Option<String>,
    pub request_id: Option<String>,
    pub user_id: Option<String>,
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub additional_fields: HashMap<String, String>,
}

impl LogInfo {
    pub fn new<M: Display>(message: M) -> Self {
        Self {
            message: message.to_string(),
            context: None,
            module: None,
            request_id: None,
            user_id: None,
            timestamp: Some(chrono::Utc::now()),
            additional_fields: HashMap::new(),
        }
    }

    pub fn with_context<C: Display>(mut self, context: C) -> Self {
        self.context = Some(context.to_string());
        self
    }

    pub fn with_module<M: Display>(mut self, module: M) -> Self {
        self.module = Some(module.to_string());
        self
    }

    pub fn with_request_id<R: Display>(mut self, request_id: R) -> Self {
        self.request_id = Some(request_id.to_string());
        self
    }

    pub fn with_user_id<U: Display>(mut self, user_id: U) -> Self {
        self.user_id = Some(user_id.to_string());
        self
    }

    pub fn with_timestamp(mut self, timestamp: chrono::DateTime<chrono::Utc>) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    pub fn with_field<K: Display, V: Display>(mut self, key: K, value: V) -> Self {
        self.additional_fields
            .insert(key.to_string(), value.to_string());
        self
    }
}

/// Structured log format for easy serialization
#[derive(Debug, Clone, serde::Serialize)]
pub struct StructuredLog {
    pub level: String,
    pub message: String,
    pub context: Option<String>,
    pub module: Option<String>,
    pub request_id: Option<String>,
    pub user_id: Option<String>,
    pub timestamp: String,
    #[serde(flatten)]
    pub additional_fields: HashMap<String, String>,
}

impl From<(LogLevel, LogInfo)> for StructuredLog {
    fn from((level, info): (LogLevel, LogInfo)) -> Self {
        let level_str = match level {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        };

        let timestamp = info.timestamp.unwrap_or_else(chrono::Utc::now).to_rfc3339();

        Self {
            level: level_str.to_string(),
            message: info.message,
            context: info.context,
            module: info.module,
            request_id: info.request_id,
            user_id: info.user_id,
            timestamp,
            additional_fields: info.additional_fields,
        }
    }
}

/// Core logging operations interface
#[async_trait]
pub trait LoggingOperations: Send + Sync + 'static {
    /// Log a message at the specified level
    fn log(&self, level: LogLevel, info: LogInfo) -> Result<(), LoggingError>;

    /// Log at trace level
    fn trace(&self, info: LogInfo) -> Result<(), LoggingError> {
        self.log(LogLevel::Trace, info)
    }

    /// Log at debug level
    fn debug(&self, info: LogInfo) -> Result<(), LoggingError> {
        self.log(LogLevel::Debug, info)
    }

    /// Log at info level
    fn info(&self, info: LogInfo) -> Result<(), LoggingError> {
        self.log(LogLevel::Info, info)
    }

    /// Log at warn level
    fn warn(&self, info: LogInfo) -> Result<(), LoggingError> {
        self.log(LogLevel::Warn, info)
    }

    /// Log at error level
    fn error(&self, info: LogInfo) -> Result<(), LoggingError> {
        self.log(LogLevel::Error, info)
    }

    /// Log a structured record directly
    fn log_structured(&self, record: StructuredLog) -> Result<(), LoggingError>;

    /// Add a global context field to all subsequent logs
    fn with_global_context(&self, key: &str, value: &str) -> Result<(), LoggingError>;

    /// Set the minimum log level
    fn set_level(&self, level: LogLevel) -> Result<(), LoggingError>;

    /// Get the current minimum log level
    fn get_level(&self) -> LogLevel;

    /// Check if the given level is enabled
    fn is_enabled(&self, level: LogLevel) -> bool {
        level >= self.get_level()
    }

    /// Flush any buffered logs
    async fn flush(&self) -> Result<(), LoggingError>;

    /// Create a child logger with additional context
    fn child(&self, context: &str) -> Arc<dyn LoggingOperations>;
}
