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
#[automock]
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

/// Logger with captured logs
#[derive(Debug)]
pub struct MockLogger {
    /// The mock logger
    mock: MockMockLogger,
    /// Captured logs
    captured_logs: Arc<Mutex<Vec<LogEntry>>>,
}

impl MockLogger {
    /// Create a new mock logger
    pub fn new() -> Self {
        let captured_logs = Arc::new(Mutex::new(Vec::new()));
        let mut mock = MockMockLogger::default();

        // Set up default implementations
        let captured_logs_clone = captured_logs.clone();
        mock.expect_log()
            .times(..)
            .returning(move |level, message, context| {
                let entry = LogEntry {
                    level,
                    message: message.to_string(),
                    context: context.unwrap_or_default(),
                    timestamp: chrono::Utc::now(),
                };

                captured_logs_clone.lock().unwrap().push(entry);
            });

        let captured_logs_clone = captured_logs.clone();
        mock.expect_trace()
            .times(..)
            .returning(move |message, context| {
                let entry = LogEntry {
                    level: LogLevel::Trace,
                    message: message.to_string(),
                    context: context.unwrap_or_default(),
                    timestamp: chrono::Utc::now(),
                };

                captured_logs_clone.lock().unwrap().push(entry);
            });

        let captured_logs_clone = captured_logs.clone();
        mock.expect_debug()
            .times(..)
            .returning(move |message, context| {
                let entry = LogEntry {
                    level: LogLevel::Debug,
                    message: message.to_string(),
                    context: context.unwrap_or_default(),
                    timestamp: chrono::Utc::now(),
                };

                captured_logs_clone.lock().unwrap().push(entry);
            });

        let captured_logs_clone = captured_logs.clone();
        mock.expect_info()
            .times(..)
            .returning(move |message, context| {
                let entry = LogEntry {
                    level: LogLevel::Info,
                    message: message.to_string(),
                    context: context.unwrap_or_default(),
                    timestamp: chrono::Utc::now(),
                };

                captured_logs_clone.lock().unwrap().push(entry);
            });

        let captured_logs_clone = captured_logs.clone();
        mock.expect_warn()
            .times(..)
            .returning(move |message, context| {
                let entry = LogEntry {
                    level: LogLevel::Warn,
                    message: message.to_string(),
                    context: context.unwrap_or_default(),
                    timestamp: chrono::Utc::now(),
                };

                captured_logs_clone.lock().unwrap().push(entry);
            });

        let captured_logs_clone = captured_logs.clone();
        mock.expect_error()
            .times(..)
            .returning(move |message, context| {
                let entry = LogEntry {
                    level: LogLevel::Error,
                    message: message.to_string(),
                    context: context.unwrap_or_default(),
                    timestamp: chrono::Utc::now(),
                };

                captured_logs_clone.lock().unwrap().push(entry);
            });

        Self {
            mock,
            captured_logs,
        }
    }

    /// Register the mock with the registry
    pub fn register(self, registry: &MockRegistry) -> TestResult<Arc<Self>> {
        let arc_self = Arc::new(self);
        let logger_impl = LoggerImpl::new(arc_self.clone());
        registry.register::<dyn Logger, LoggerImpl>(Arc::new(logger_impl))?;
        Ok(arc_self)
    }

    /// Get captured logs
    pub fn get_logs(&self) -> Vec<LogEntry> {
        self.captured_logs.lock().unwrap().clone()
    }

    /// Clear captured logs
    pub fn clear_logs(&self) {
        self.captured_logs.lock().unwrap().clear();
    }

    /// Get logs with a specific level
    pub fn get_logs_with_level(&self, level: LogLevel) -> Vec<LogEntry> {
        self.captured_logs
            .lock()
            .unwrap()
            .iter()
            .filter(|log| log.level == level)
            .cloned()
            .collect()
    }

    /// Get logs containing a specific message
    pub fn get_logs_containing(&self, message: &str) -> Vec<LogEntry> {
        self.captured_logs
            .lock()
            .unwrap()
            .iter()
            .filter(|log| log.message.contains(message))
            .cloned()
            .collect()
    }

    /// Get logs with a specific context key
    pub fn get_logs_with_context_key(&self, key: &str) -> Vec<LogEntry> {
        self.captured_logs
            .lock()
            .unwrap()
            .iter()
            .filter(|log| log.context.contains_key(key))
            .cloned()
            .collect()
    }

    /// Get logs with a specific context key and value
    pub fn get_logs_with_context(&self, key: &str, value: &str) -> Vec<LogEntry> {
        self.captured_logs
            .lock()
            .unwrap()
            .iter()
            .filter(|log| log.context.get(key).map(|v| v == value).unwrap_or(false))
            .cloned()
            .collect()
    }

    /// Assert that a specific message was logged at a specific level
    pub fn assert_logged(&self, level: LogLevel, message: &str) -> TestResult<()> {
        let logs = self.get_logs_with_level(level);
        if logs.iter().any(|log| log.message.contains(message)) {
            Ok(())
        } else {
            Err(TestError::AssertionError(format!(
                "No log entry found with level {:?} containing message: {}",
                level, message
            )))
        }
    }

    /// Assert that a specific message was not logged
    pub fn assert_not_logged(&self, message: &str) -> TestResult<()> {
        let logs = self.get_logs_containing(message);
        if logs.is_empty() {
            Ok(())
        } else {
            Err(TestError::AssertionError(format!(
                "Log entry found containing message: {}",
                message
            )))
        }
    }

    /// Assert that a specific context key was logged
    pub fn assert_context_logged(&self, key: &str) -> TestResult<()> {
        let logs = self.get_logs_with_context_key(key);
        if !logs.is_empty() {
            Ok(())
        } else {
            Err(TestError::AssertionError(format!(
                "No log entry found with context key: {}",
                key
            )))
        }
    }

    /// Assert that a specific context key and value was logged
    pub fn assert_context_value_logged(&self, key: &str, value: &str) -> TestResult<()> {
        let logs = self.get_logs_with_context(key, value);
        if !logs.is_empty() {
            Ok(())
        } else {
            Err(TestError::AssertionError(format!(
                "No log entry found with context key: {} and value: {}",
                key, value
            )))
        }
    }
}

/// Default implementation
impl Default for MockLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// Logger implementation that delegates to the mock
struct LoggerImpl {
    mock_logger: Arc<MockLogger>,
}

impl LoggerImpl {
    /// Create a new logger implementation
    fn new(mock_logger: Arc<MockLogger>) -> Self {
        Self { mock_logger }
    }
}

impl Logger for LoggerImpl {
    fn log(&self, level: LogLevel, message: &str, context: Option<HashMap<String, String>>) {
        self.mock_logger.mock.log(level, message, context);
    }

    fn trace(&self, message: &str, context: Option<HashMap<String, String>>) {
        self.mock_logger.mock.trace(message, context);
    }

    fn debug(&self, message: &str, context: Option<HashMap<String, String>>) {
        self.mock_logger.mock.debug(message, context);
    }

    fn info(&self, message: &str, context: Option<HashMap<String, String>>) {
        self.mock_logger.mock.info(message, context);
    }

    fn warn(&self, message: &str, context: Option<HashMap<String, String>>) {
        self.mock_logger.mock.warn(message, context);
    }

    fn error(&self, message: &str, context: Option<HashMap<String, String>>) {
        self.mock_logger.mock.error(message, context);
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
