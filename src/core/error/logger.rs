use crate::core::error::error_types::{AppError, ErrorSeverity};
use std::fmt::Display;
use tracing::{Level, debug, error, info, trace, warn};

/// Log levels matching tracing levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<ErrorSeverity> for LogLevel {
    fn from(severity: ErrorSeverity) -> Self {
        match severity {
            ErrorSeverity::Low => LogLevel::Info,
            ErrorSeverity::Medium => LogLevel::Warn,
            ErrorSeverity::High => LogLevel::Error,
            ErrorSeverity::Critical => LogLevel::Error,
        }
    }
}

impl From<LogLevel> for Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }
}

/// Standard log information
#[derive(Debug)]
pub struct LogInfo {
    pub message: String,
    pub context: Option<String>,
    pub module: Option<String>,
    pub request_id: Option<String>,
    pub user_id: Option<String>,
}

impl LogInfo {
    pub fn new<M: Display>(message: M) -> Self {
        Self {
            message: message.to_string(),
            context: None,
            module: None,
            request_id: None,
            user_id: None,
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
}

/// Standardized logging functions
pub fn log(level: LogLevel, info: LogInfo) {
    match level {
        LogLevel::Trace => trace!(
            message = %info.message,
            context = info.context.as_deref().unwrap_or(""),
            module = info.module.as_deref().unwrap_or(""),
            request_id = info.request_id.as_deref().unwrap_or(""),
            user_id = info.user_id.as_deref().unwrap_or(""),
        ),
        LogLevel::Debug => debug!(
            message = %info.message,
            context = info.context.as_deref().unwrap_or(""),
            module = info.module.as_deref().unwrap_or(""),
            request_id = info.request_id.as_deref().unwrap_or(""),
            user_id = info.user_id.as_deref().unwrap_or(""),
        ),
        LogLevel::Info => info!(
            message = %info.message,
            context = info.context.as_deref().unwrap_or(""),
            module = info.module.as_deref().unwrap_or(""),
            request_id = info.request_id.as_deref().unwrap_or(""),
            user_id = info.user_id.as_deref().unwrap_or(""),
        ),
        LogLevel::Warn => warn!(
            message = %info.message,
            context = info.context.as_deref().unwrap_or(""),
            module = info.module.as_deref().unwrap_or(""),
            request_id = info.request_id.as_deref().unwrap_or(""),
            user_id = info.user_id.as_deref().unwrap_or(""),
        ),
        LogLevel::Error => error!(
            message = %info.message,
            context = info.context.as_deref().unwrap_or(""),
            module = info.module.as_deref().unwrap_or(""),
            request_id = info.request_id.as_deref().unwrap_or(""),
            user_id = info.user_id.as_deref().unwrap_or(""),
        ),
    }
}

/// Log an error with standardized format
pub fn log_error(error: &AppError, context: Option<String>, request_id: Option<String>) {
    let level = LogLevel::from(error.severity());
    let error_type = error.error_type();
    let error_message = error.to_string();

    match level {
        LogLevel::Error => {
            error!(
                error_type = %error_type,
                error = %error_message,
                context = context.as_deref().unwrap_or(""),
                request_id = request_id.as_deref().unwrap_or(""),
                "Error occurred"
            );
        }
        LogLevel::Warn => {
            warn!(
                error_type = %error_type,
                error = %error_message,
                context = context.as_deref().unwrap_or(""),
                request_id = request_id.as_deref().unwrap_or(""),
                "Error occurred"
            );
        }
        _ => {
            info!(
                error_type = %error_type,
                error = %error_message,
                context = context.as_deref().unwrap_or(""),
                request_id = request_id.as_deref().unwrap_or(""),
                "Error occurred"
            );
        }
    }
}

/// Macro for logging at different levels with consistent structure
#[macro_export]
macro_rules! log_at_level {
    ($level:ident, $info:expr) => {
        $level!(
            message = %$info.message,
            context = $info.context.as_deref().unwrap_or(""),
            module = $info.module.as_deref().unwrap_or(""),
            request_id = $info.request_id.as_deref().unwrap_or(""),
            user_id = $info.user_id.as_deref().unwrap_or(""),
        );
    };
}
