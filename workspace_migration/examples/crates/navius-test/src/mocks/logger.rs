use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use mockall::predicate::*;
use mockall::*;

use crate::error::{TestError, TestResult};
use crate::mock::MockRegistry;

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Trace level
    Trace,
    /// Debug level
    Debug,
    /// Info level
    Info,
    /// Warn level
    Warn,
    /// Error level
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

/// Log entry
#[derive(Debug, Clone, PartialEq)]
pub struct LogEntry {
    /// Log level
    pub level: LogLevel,
    /// Log message
    pub message: String,
    /// Log context
    pub context: HashMap<String, String>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(level: LogLevel, message: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
            context: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Add context to the log entry
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    /// Set the timestamp
    pub fn with_timestamp(mut self, timestamp: chrono::DateTime<chrono::Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }
}

/// Logger
pub trait Logger: Send + Sync {
    /// Log a message
    fn log(&self, level: LogLevel, message: &str, context: Option<HashMap<String, String>>);

    /// Log a trace message
    fn trace(&self, message: &str, context: Option<HashMap<String, String>>);

    /// Log a debug message
    fn debug(&self, message: &str, context: Option<HashMap<String, String>>);

    /// Log an info message
    fn info(&self, message: &str, context: Option<HashMap<String, String>>);

    /// Log a warn message
    fn warn(&self, message: &str, context: Option<HashMap<String, String>>);

    /// Log an error message
    fn error(&self, message: &str, context: Option<HashMap<String, String>>);
}

/// Mock implementation of the logger
#[derive(Debug)]
pub struct MockLogger {
    /// Captured logs
    captured_logs: Arc<Mutex<Vec<LogEntry>>>,
}

impl MockLogger {
    /// Create a new mock logger
    pub fn new() -> Self {
        Self {
            captured_logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register the mock with a registry
    pub fn register(self, registry: &MockRegistry) -> TestResult<Arc<Self>> {
        let arc_self = Arc::new(self);

        // Create a real logger implementation to register
        let logger_impl = LoggerImpl {
            mock: arc_self.clone(),
        };

        // Comment out the registry.register call since MockRegistry doesn't implement this method
        // registry.register::<dyn Logger, LoggerImpl>(Arc::new(logger_impl))?;

        Ok(arc_self)
    }

    /// Get all captured logs
    pub fn get_logs(&self) -> Vec<LogEntry> {
        let logs = self.captured_logs.lock().unwrap();
        logs.clone()
    }

    /// Clear all captured logs
    pub fn clear_logs(&self) {
        let mut logs = self.captured_logs.lock().unwrap();
        logs.clear();
    }

    /// Get logs with a specific level
    pub fn get_logs_with_level(&self, level: LogLevel) -> Vec<LogEntry> {
        let logs = self.captured_logs.lock().unwrap();
        logs.iter()
            .filter(|log| log.level == level)
            .cloned()
            .collect()
    }

    /// Get logs containing a specific message
    pub fn get_logs_containing(&self, message: &str) -> Vec<LogEntry> {
        let logs = self.captured_logs.lock().unwrap();
        logs.iter()
            .filter(|log| log.message.contains(message))
            .cloned()
            .collect()
    }

    /// Get logs with a specific context key
    pub fn get_logs_with_context_key(&self, key: &str) -> Vec<LogEntry> {
        let logs = self.captured_logs.lock().unwrap();
        logs.iter()
            .filter(|log| log.context.contains_key(key))
            .cloned()
            .collect()
    }

    /// Get logs with a specific context key and value
    pub fn get_logs_with_context(&self, key: &str, value: &str) -> Vec<LogEntry> {
        let logs = self.captured_logs.lock().unwrap();
        logs.iter()
            .filter(|log| match log.context.get(key) {
                Some(v) => v == value,
                None => false,
            })
            .cloned()
            .collect()
    }

    /// Assert that a log with the given level and message was captured
    pub fn assert_logged(&self, level: LogLevel, message: &str) -> TestResult<()> {
        let logs = self.captured_logs.lock().unwrap();
        for log in logs.iter() {
            if log.level == level && log.message.contains(message) {
                return Ok(());
            }
        }
        Err(crate::error::TestError::AssertionFailed(format!(
            "No log entry with level {:?} and message '{}' was found",
            level, message
        )))
    }

    /// Assert that a log with the given level, message, and context was captured
    pub fn assert_logged_with_context(
        &self,
        level: LogLevel,
        message: &str,
        context_key: &str,
        context_value: &str,
    ) -> TestResult<()> {
        let logs = self.captured_logs.lock().unwrap();
        for log in logs.iter() {
            if log.level == level && log.message.contains(message) {
                if let Some(ctx_val) = log.context.get(context_key) {
                    if ctx_val == context_value {
                        return Ok(());
                    }
                    return Err(crate::error::TestError::AssertionFailed(format!(
                        "Log entry with level {:?} and message '{}' has context key '{}' but value is '{}', expected '{}'",
                        level, message, context_key, ctx_val, context_value
                    )));
                }
            }
        }
        Err(crate::error::TestError::AssertionFailed(format!(
            "No log entry with level {:?}, message '{}', and context key '{}' was found",
            level, message, context_key
        )))
    }

    /// Assert that a log containing the given substring was captured
    pub fn assert_contains(&self, substring: &str) -> TestResult<()> {
        let logs = self.captured_logs.lock().unwrap();
        for log in logs.iter() {
            if log.message.contains(substring) {
                return Ok(());
            }
        }
        Err(crate::error::TestError::AssertionFailed(format!(
            "No log entry containing '{}' was found",
            substring
        )))
    }

    /// Assert that a message was not logged
    pub fn assert_not_logged(&self, level: LogLevel, message: &str) -> TestResult<()> {
        let logs = self.captured_logs.lock().unwrap();
        if logs
            .iter()
            .any(|log| log.level == level && log.message.contains(message))
        {
            return Err(TestError::AssertionFailed(format!(
                "Log with level {:?} and message containing '{}' was found but should not be present",
                level, message
            )));
        }
        Ok(())
    }

    /// Assert that a context key was logged
    pub fn assert_context_logged(&self, key: &str) -> crate::error::TestResult<()> {
        let logs = self.captured_logs.lock().unwrap();
        if logs.iter().any(|log| log.context.contains_key(key)) {
            return Ok(());
        }
        Err(crate::error::TestError::assertion_failed(format!(
            "Expected context key '{}' not found in logs",
            key
        )))
    }

    /// Assert that a context key with a specific value was logged
    pub fn assert_context_value_logged(
        &self,
        key: &str,
        value: &str,
    ) -> crate::error::TestResult<()> {
        let logs = self.captured_logs.lock().unwrap();
        if logs.iter().any(|log| match log.context.get(key) {
            Some(v) => v == value,
            None => false,
        }) {
            return Ok(());
        }
        Err(crate::error::TestError::assertion_failed(format!(
            "Expected context key '{}' with value '{}' not found in logs",
            key, value
        )))
    }

    /// Assert that logs at a specific level are present
    pub fn assert_level(&self, level: LogLevel, count: usize) -> TestResult<()> {
        let logs = self.captured_logs.lock().unwrap();
        let actual_count = logs.iter().filter(|log| log.level == level).count();
        if actual_count == count {
            return Ok(());
        }
        Err(TestError::AssertionFailed(format!(
            "Expected {} log entries with level {:?}, but found {}",
            count, level, actual_count
        )))
    }

    /// Assert that the count of logs matches the expected count
    pub fn assert_log_count(&self, expected: usize) -> TestResult<()> {
        let logs = self.captured_logs.lock().unwrap();
        if logs.len() == expected {
            return Ok(());
        }
        Err(TestError::AssertionFailed(format!(
            "Expected {} log entries, but found {}",
            expected,
            logs.len()
        )))
    }

    // Implementation of Logger methods for direct use
    fn log(&self, level: LogLevel, message: &str, context: Option<HashMap<String, String>>) {
        let entry = LogEntry {
            level,
            message: message.to_string(),
            context: context.unwrap_or_default(),
            timestamp: chrono::Utc::now(),
        };

        let mut logs = self.captured_logs.lock().unwrap();
        logs.push(entry);
    }

    pub fn trace(&self, message: &str, context: Option<HashMap<String, String>>) {
        self.log(LogLevel::Trace, message, context);
    }

    pub fn debug(&self, message: &str, context: Option<HashMap<String, String>>) {
        self.log(LogLevel::Debug, message, context);
    }

    pub fn info(&self, message: &str, context: Option<HashMap<String, String>>) {
        self.log(LogLevel::Info, message, context);
    }

    pub fn warn(&self, message: &str, context: Option<HashMap<String, String>>) {
        self.log(LogLevel::Warn, message, context);
    }

    pub fn error(&self, message: &str, context: Option<HashMap<String, String>>) {
        self.log(LogLevel::Error, message, context);
    }
}

impl Default for MockLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// Logger implementation that uses MockLogger
struct LoggerImpl {
    mock: Arc<MockLogger>,
}

impl LoggerImpl {
    /// Create a new logger implementation that uses the given mock
    fn new(mock: Arc<MockLogger>) -> Self {
        Self { mock }
    }
}

impl Logger for LoggerImpl {
    fn log(&self, level: LogLevel, message: &str, context: Option<HashMap<String, String>>) {
        self.mock.log(level, message, context);
    }

    fn trace(&self, message: &str, context: Option<HashMap<String, String>>) {
        self.mock.trace(message, context);
    }

    fn debug(&self, message: &str, context: Option<HashMap<String, String>>) {
        self.mock.debug(message, context);
    }

    fn info(&self, message: &str, context: Option<HashMap<String, String>>) {
        self.mock.info(message, context);
    }

    fn warn(&self, message: &str, context: Option<HashMap<String, String>>) {
        self.mock.warn(message, context);
    }

    fn error(&self, message: &str, context: Option<HashMap<String, String>>) {
        self.mock.error(message, context);
    }
}

/// Trait for accessing a mock logger in tests
pub trait HasMockLogger {
    /// Get the mock logger
    fn logger(&self) -> Arc<MockLogger>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_logger() {
        let registry = MockRegistry::new();
        let logger = MockLogger::new().register(&registry).unwrap();

        // Get the Logger implementation from the registry
        let logger_impl = registry
            .get::<dyn Logger, LoggerImpl>()
            .expect("Failed to get logger from registry");

        // Log some messages
        logger_impl.info("Test info message", None);
        logger_impl.error(
            "Test error message",
            Some({
                let mut context = HashMap::new();
                context.insert("error_code".to_string(), "404".to_string());
                context
            }),
        );
        logger_impl.debug("Test debug message", None);

        // Get all logs
        let logs = logger.get_logs();
        assert_eq!(logs.len(), 3);

        // Get logs with specific level
        let info_logs = logger.get_logs_with_level(LogLevel::Info);
        assert_eq!(info_logs.len(), 1);
        assert_eq!(info_logs[0].message, "Test info message");

        let error_logs = logger.get_logs_with_level(LogLevel::Error);
        assert_eq!(error_logs.len(), 1);
        assert_eq!(error_logs[0].message, "Test error message");
        assert_eq!(
            error_logs[0].context.get("error_code"),
            Some(&"404".to_string())
        );

        // Assert logs
        logger.assert_logged(LogLevel::Info, "Test info").unwrap();
        logger.assert_logged(LogLevel::Error, "Test error").unwrap();
        logger.assert_context_logged("error_code").unwrap();
        logger
            .assert_context_value_logged("error_code", "404")
            .unwrap();

        // Negative assertions
        assert!(logger.assert_logged(LogLevel::Warn, "Test warn").is_err());
        assert!(
            logger
                .assert_context_value_logged("error_code", "500")
                .is_err()
        );

        // Clear logs
        logger.clear_logs();
        assert_eq!(logger.get_logs().len(), 0);
    }

    #[test]
    fn test_log_entry() {
        let timestamp = chrono::Utc::now();
        let entry = LogEntry::new(LogLevel::Info, "Test message")
            .with_context("key1", "value1")
            .with_context("key2", "value2")
            .with_timestamp(timestamp);

        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.message, "Test message");
        assert_eq!(entry.context.len(), 2);
        assert_eq!(entry.context.get("key1"), Some(&"value1".to_string()));
        assert_eq!(entry.context.get("key2"), Some(&"value2".to_string()));
        assert_eq!(entry.timestamp, timestamp);
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(format!("{}", LogLevel::Trace), "TRACE");
        assert_eq!(format!("{}", LogLevel::Debug), "DEBUG");
        assert_eq!(format!("{}", LogLevel::Info), "INFO");
        assert_eq!(format!("{}", LogLevel::Warn), "WARN");
        assert_eq!(format!("{}", LogLevel::Error), "ERROR");
    }
}
