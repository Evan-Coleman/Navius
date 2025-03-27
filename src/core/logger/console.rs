use async_trait::async_trait;
use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use super::config::LoggingConfig;
use super::error::LoggingError;
use super::interface::{LogInfo, LogLevel, LoggingOperations, StructuredLog};
use super::provider::LoggingProvider;

/// Simple console-based logger that writes directly to stdout/stderr
pub struct ConsoleLogger {
    global_context: Mutex<HashMap<String, String>>,
    level: Mutex<LogLevel>,
    use_colors: bool,
}

impl ConsoleLogger {
    /// Create a new console logger
    pub fn new(config: &LoggingConfig) -> Self {
        Self {
            global_context: Mutex::new(config.global_fields.clone()),
            level: Mutex::new(config.parse_level()),
            use_colors: config.colorize,
        }
    }

    /// Get color for a log level
    fn level_color(&self, level: LogLevel) -> Color {
        match level {
            LogLevel::Trace => Color::Magenta,
            LogLevel::Debug => Color::Cyan,
            LogLevel::Info => Color::Green,
            LogLevel::Warn => Color::Yellow,
            LogLevel::Error => Color::Red,
        }
    }

    /// Format level as a string
    fn format_level(&self, level: LogLevel) -> &'static str {
        match level {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => " INFO",
            LogLevel::Warn => " WARN",
            LogLevel::Error => "ERROR",
        }
    }

    /// Write a log message to the console
    fn write_console_log(&self, level: LogLevel, info: &LogInfo) -> Result<(), LoggingError> {
        let timestamp = info
            .timestamp
            .unwrap_or_else(chrono::Utc::now)
            .format("%Y-%m-%d %H:%M:%S%.3f")
            .to_string();

        let level_str = self.format_level(level);

        // Get output stream based on level
        let mut output = if level == LogLevel::Error {
            StandardStream::stderr(if self.use_colors {
                ColorChoice::Auto
            } else {
                ColorChoice::Never
            })
        } else {
            StandardStream::stdout(if self.use_colors {
                ColorChoice::Auto
            } else {
                ColorChoice::Never
            })
        };

        // Set color for level
        if self.use_colors {
            output
                .set_color(ColorSpec::new().set_fg(Some(self.level_color(level))))
                .map_err(|e| LoggingError::IoError(e))?;
        }

        // Write log level
        write!(&mut output, "[{}]", level_str).map_err(|e| LoggingError::IoError(e))?;

        // Reset color
        if self.use_colors {
            output.reset().map_err(|e| LoggingError::IoError(e))?;
        }

        // Write timestamp and message
        write!(&mut output, " [{}] {}", timestamp, info.message)
            .map_err(|e| LoggingError::IoError(e))?;

        // Add context if available
        if let Some(ctx) = &info.context {
            write!(&mut output, " (context: {})", ctx).map_err(|e| LoggingError::IoError(e))?;
        }

        // Add module if available
        if let Some(module) = &info.module {
            write!(&mut output, " [module: {}]", module).map_err(|e| LoggingError::IoError(e))?;
        }

        // Add request ID if available
        if let Some(req_id) = &info.request_id {
            write!(&mut output, " [request: {}]", req_id).map_err(|e| LoggingError::IoError(e))?;
        }

        // Add user ID if available
        if let Some(user_id) = &info.user_id {
            write!(&mut output, " [user: {}]", user_id).map_err(|e| LoggingError::IoError(e))?;
        }

        // Add any additional fields
        if !info.additional_fields.is_empty() {
            write!(&mut output, " {{")?;
            let mut first = true;
            for (key, value) in &info.additional_fields {
                if !first {
                    write!(&mut output, ", ")?;
                }
                write!(&mut output, "{}: {}", key, value)?;
                first = false;
            }
            write!(&mut output, "}}")?;
        }

        // Add newline
        writeln!(&mut output).map_err(|e| LoggingError::IoError(e))?;

        Ok(())
    }
}

impl LoggingOperations for ConsoleLogger {
    fn log(&self, level: LogLevel, info: LogInfo) -> Result<(), LoggingError> {
        if !self.is_enabled(level) {
            return Ok(());
        }

        // Merge global context with log info
        let mut log_info = info.clone();
        {
            let global_ctx = self.global_context.lock().unwrap();
            for (k, v) in global_ctx.iter() {
                log_info.additional_fields.insert(k.clone(), v.clone());
            }
        }

        self.write_console_log(level, &log_info)
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
        // Console output is automatically flushed
        Ok(())
    }

    fn child(&self, context: &str) -> Arc<dyn LoggingOperations> {
        let child_logger = ConsoleLogger {
            global_context: self.global_context.clone(),
            level: self.level.clone(),
            use_colors: self.use_colors,
        };

        // Add context to child logger
        if let Ok(mut global_ctx) = child_logger.global_context.lock() {
            global_ctx.insert("child_context".to_string(), context.to_string());
        }

        Arc::new(child_logger)
    }
}

/// Provider for console loggers
pub struct ConsoleLoggerProvider;

impl ConsoleLoggerProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LoggingProvider for ConsoleLoggerProvider {
    async fn create_logger(
        &self,
        config: &LoggingConfig,
    ) -> Result<Arc<dyn LoggingOperations>, LoggingError> {
        Ok(Arc::new(ConsoleLogger::new(config)))
    }

    fn name(&self) -> &'static str {
        "console"
    }

    fn supports(&self, config: &LoggingConfig) -> bool {
        config.logger_type == "console"
    }
}
