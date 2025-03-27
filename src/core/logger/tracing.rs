use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{Level, debug, error, info, trace, warn};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{
    EnvFilter, Registry,
    fmt::{format::FmtSpan, time::UtcTime},
};

use super::config::LoggingConfig;
use super::error::LoggingError;
use super::interface::{LogInfo, LogLevel, LoggingOperations, StructuredLog};
use super::provider::LoggingProvider;

/// Tracing-based implementation of the LoggingOperations interface
pub struct TracingLogger {
    global_context: Mutex<HashMap<String, String>>,
    level: Mutex<LogLevel>,
}

impl TracingLogger {
    /// Create a new TracingLogger with the specified configuration
    pub fn new(config: &LoggingConfig) -> Self {
        Self {
            global_context: Mutex::new(config.global_fields.clone()),
            level: Mutex::new(config.parse_level()),
        }
    }

    /// Convert our LogLevel to tracing's Level
    fn to_tracing_level(level: LogLevel) -> Level {
        match level {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }
}

impl LoggingOperations for TracingLogger {
    fn log(&self, level: LogLevel, info: LogInfo) -> Result<(), LoggingError> {
        if !self.is_enabled(level) {
            return Ok(());
        }

        // Merge global context with log-specific fields
        let mut fields = info.additional_fields.clone();

        {
            let global_ctx = self.global_context.lock().unwrap();
            for (k, v) in global_ctx.iter() {
                fields.insert(k.clone(), v.clone());
            }
        }

        // Create a structured log for tracking
        let structured_log = StructuredLog::from((level, info.clone()));

        // Log using the appropriate tracing macro
        match level {
            LogLevel::Trace => {
                trace!(
                    message = %info.message,
                    context = info.context.as_deref().unwrap_or(""),
                    module = info.module.as_deref().unwrap_or(""),
                    request_id = info.request_id.as_deref().unwrap_or(""),
                    user_id = info.user_id.as_deref().unwrap_or(""),
                    fields = ?fields,
                );
            }
            LogLevel::Debug => {
                debug!(
                    message = %info.message,
                    context = info.context.as_deref().unwrap_or(""),
                    module = info.module.as_deref().unwrap_or(""),
                    request_id = info.request_id.as_deref().unwrap_or(""),
                    user_id = info.user_id.as_deref().unwrap_or(""),
                    fields = ?fields,
                );
            }
            LogLevel::Info => {
                info!(
                    message = %info.message,
                    context = info.context.as_deref().unwrap_or(""),
                    module = info.module.as_deref().unwrap_or(""),
                    request_id = info.request_id.as_deref().unwrap_or(""),
                    user_id = info.user_id.as_deref().unwrap_or(""),
                    fields = ?fields,
                );
            }
            LogLevel::Warn => {
                warn!(
                    message = %info.message,
                    context = info.context.as_deref().unwrap_or(""),
                    module = info.module.as_deref().unwrap_or(""),
                    request_id = info.request_id.as_deref().unwrap_or(""),
                    user_id = info.user_id.as_deref().unwrap_or(""),
                    fields = ?fields,
                );
            }
            LogLevel::Error => {
                error!(
                    message = %info.message,
                    context = info.context.as_deref().unwrap_or(""),
                    module = info.module.as_deref().unwrap_or(""),
                    request_id = info.request_id.as_deref().unwrap_or(""),
                    user_id = info.user_id.as_deref().unwrap_or(""),
                    fields = ?fields,
                );
            }
        }

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

        let info = LogInfo {
            message: record.message.clone(),
            context: record.context.clone(),
            module: record.module.clone(),
            request_id: record.request_id.clone(),
            user_id: record.user_id.clone(),
            timestamp: Some(
                chrono::DateTime::parse_from_rfc3339(&record.timestamp)
                    .map_err(|e| LoggingError::SerializationError(e.to_string()))?
                    .with_timezone(&chrono::Utc),
            ),
            additional_fields: record.additional_fields.clone(),
        };

        self.log(level, info)
    }

    fn with_global_context(&self, key: &str, value: &str) -> Result<(), LoggingError> {
        let mut global_ctx = self.global_context.lock().unwrap();
        global_ctx.insert(key.to_string(), value.to_string());
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
        // Tracing doesn't have an explicit flush mechanism
        // This is a no-op for this implementation
        Ok(())
    }

    fn child(&self, context: &str) -> Arc<dyn LoggingOperations> {
        let child_logger = TracingLogger {
            global_context: self.global_context.clone(),
            level: self.level.clone(),
        };

        // Add the context to the new logger
        if let Ok(mut global_ctx) = child_logger.global_context.lock() {
            global_ctx.insert("child_context".to_string(), context.to_string());
        }

        Arc::new(child_logger)
    }
}

/// Provider for tracing-based loggers
pub struct TracingLoggerProvider;

impl TracingLoggerProvider {
    pub fn new() -> Self {
        Self
    }

    /// Initialize the tracing subscriber based on configuration
    pub fn init_tracing(config: &LoggingConfig) -> Result<(), LoggingError> {
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(config.level.clone()));

        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_timer(UtcTime::rfc_3339())
            .with_span_events(FmtSpan::CLOSE);

        let subscriber = Registry::default().with(env_filter).with(fmt_layer);

        // Initialize the subscriber
        subscriber.try_init().map_err(|e| {
            LoggingError::InitializationError(format!("Failed to initialize tracing: {}", e))
        })
    }
}

#[async_trait]
impl LoggingProvider for TracingLoggerProvider {
    async fn create_logger(
        &self,
        config: &LoggingConfig,
    ) -> Result<Arc<dyn LoggingOperations>, LoggingError> {
        // Initialize tracing subscriber if needed
        if config.logger_type == "tracing" {
            // We'll skip initialization in tests and when we know it's already initialized
            // This is a bit of a hack but tracing subscriber can only be initialized once
            if std::env::var("RUST_LOG").is_err() {
                let _ = Self::init_tracing(config);
            }
        }

        // Create the logger
        let logger = TracingLogger::new(config);
        Ok(Arc::new(logger))
    }

    fn name(&self) -> &'static str {
        "tracing"
    }

    fn supports(&self, config: &LoggingConfig) -> bool {
        config.logger_type == "tracing"
    }
}
