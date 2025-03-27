#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Arc;

    use super::config::LoggingConfig;
    use super::error::LoggingError;
    use super::interface::{LogInfo, LogLevel, LoggingOperations, StructuredLog};
    use super::provider::{LoggingProvider, LoggingProviderRegistry};
    use super::tracing::TracingLoggerProvider;

    // Mock logger for testing
    struct MockLogger {
        messages: std::sync::Mutex<Vec<(LogLevel, String)>>,
        level: std::sync::Mutex<LogLevel>,
    }

    impl MockLogger {
        fn new() -> Self {
            Self {
                messages: std::sync::Mutex::new(Vec::new()),
                level: std::sync::Mutex::new(LogLevel::Info),
            }
        }

        fn messages(&self) -> Vec<(LogLevel, String)> {
            self.messages.lock().unwrap().clone()
        }
    }

    impl LoggingOperations for MockLogger {
        fn log(&self, level: LogLevel, info: LogInfo) -> Result<(), LoggingError> {
            if !self.is_enabled(level) {
                return Ok(());
            }
            let mut messages = self.messages.lock().unwrap();
            messages.push((level, info.message));
            Ok(())
        }

        fn log_structured(&self, record: StructuredLog) -> Result<(), LoggingError> {
            let level = match record.level.as_str() {
                "TRACE" => LogLevel::Trace,
                "DEBUG" => LogLevel::Debug,
                "INFO" => LogLevel::Info,
                "WARN" => LogLevel::Warn,
                "ERROR" => LogLevel::Error,
                _ => LogLevel::Info,
            };
            let mut messages = self.messages.lock().unwrap();
            messages.push((level, record.message));
            Ok(())
        }

        fn with_global_context(&self, _key: &str, _value: &str) -> Result<(), LoggingError> {
            Ok(())
        }

        fn set_level(&self, level: LogLevel) -> Result<(), LoggingError> {
            let mut current_level = self.level.lock().unwrap();
            *current_level = level;
            Ok(())
        }

        fn get_level(&self) -> LogLevel {
            *self.level.lock().unwrap()
        }

        async fn flush(&self) -> Result<(), LoggingError> {
            Ok(())
        }

        fn child(&self, _context: &str) -> Arc<dyn LoggingOperations> {
            Arc::new(MockLogger::new())
        }
    }

    // Mock provider for testing
    struct MockLoggerProvider;

    impl MockLoggerProvider {
        fn new() -> Self {
            Self
        }
    }

    #[async_trait]
    impl LoggingProvider for MockLoggerProvider {
        async fn create_logger(
            &self,
            _config: &LoggingConfig,
        ) -> Result<Arc<dyn LoggingOperations>, LoggingError> {
            Ok(Arc::new(MockLogger::new()))
        }

        fn name(&self) -> &'static str {
            "mock"
        }

        fn supports(&self, config: &LoggingConfig) -> bool {
            config.logger_type == "mock"
        }
    }

    #[tokio::test]
    async fn test_logger_provider_registry() {
        // Create a registry
        let registry = LoggingProviderRegistry::new();

        // Register providers
        let mock_provider = Arc::new(MockLoggerProvider::new());
        let tracing_provider = Arc::new(TracingLoggerProvider::new());

        registry.register_provider(mock_provider).unwrap();
        registry.register_provider(tracing_provider).unwrap();

        // Get provider by name
        let provider = registry.get_provider("mock").unwrap();
        assert_eq!(provider.name(), "mock");

        // Set default provider
        registry.set_default_provider("mock").unwrap();
        let default_provider = registry.get_default_provider().unwrap();
        assert_eq!(default_provider.name(), "mock");

        // List providers
        let providers = registry.list_providers();
        assert!(providers.contains(&"mock".to_string()));
        assert!(providers.contains(&"tracing".to_string()));
    }

    #[tokio::test]
    async fn test_mock_logger() {
        // Create a mock logger
        let logger = Arc::new(MockLogger::new());

        // Log some messages
        logger.info(LogInfo::new("Info message")).unwrap();
        logger.warn(LogInfo::new("Warning message")).unwrap();
        logger.error(LogInfo::new("Error message")).unwrap();

        // Debug should be filtered out by default (Info level)
        logger.debug(LogInfo::new("Debug message")).unwrap();

        // Get logged messages
        let messages = logger.messages();
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].0, LogLevel::Info);
        assert_eq!(messages[0].1, "Info message");
        assert_eq!(messages[1].0, LogLevel::Warn);
        assert_eq!(messages[1].1, "Warning message");
        assert_eq!(messages[2].0, LogLevel::Error);
        assert_eq!(messages[2].1, "Error message");

        // Change log level and try again
        logger.set_level(LogLevel::Debug).unwrap();
        logger.debug(LogInfo::new("Now debug works")).unwrap();

        let messages = logger.messages();
        assert_eq!(messages.len(), 4);
        assert_eq!(messages[3].0, LogLevel::Debug);
        assert_eq!(messages[3].1, "Now debug works");
    }

    #[tokio::test]
    async fn test_config_validation() {
        // Valid config
        let config = LoggingConfig {
            logger_type: "console".to_string(),
            format: "json".to_string(),
            ..Default::default()
        };
        assert!(config.validate().is_ok());

        // Invalid logger type
        let config = LoggingConfig {
            logger_type: "invalid".to_string(),
            ..Default::default()
        };
        assert!(config.validate().is_err());

        // Invalid format
        let config = LoggingConfig {
            format: "invalid".to_string(),
            ..Default::default()
        };
        assert!(config.validate().is_err());

        // File logger needs path
        let config = LoggingConfig {
            logger_type: "file".to_string(),
            file_path: None,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[tokio::test]
    async fn test_structured_log() {
        // Create a structured log
        let mut fields = HashMap::new();
        fields.insert("custom_field".to_string(), "custom_value".to_string());

        let log_info = LogInfo {
            message: "Test message".to_string(),
            context: Some("test".to_string()),
            module: Some("logger".to_string()),
            request_id: Some("123".to_string()),
            user_id: Some("user1".to_string()),
            timestamp: Some(chrono::Utc::now()),
            additional_fields: fields,
        };

        let structured_log = StructuredLog::from((LogLevel::Info, log_info));

        // Verify structured log
        assert_eq!(structured_log.level, "INFO");
        assert_eq!(structured_log.message, "Test message");
        assert_eq!(structured_log.context, Some("test".to_string()));
        assert_eq!(structured_log.module, Some("logger".to_string()));
        assert_eq!(structured_log.request_id, Some("123".to_string()));
        assert_eq!(structured_log.user_id, Some("user1".to_string()));
        assert!(
            structured_log
                .additional_fields
                .contains_key("custom_field")
        );
        assert_eq!(
            structured_log.additional_fields.get("custom_field"),
            Some(&"custom_value".to_string())
        );
    }
}
